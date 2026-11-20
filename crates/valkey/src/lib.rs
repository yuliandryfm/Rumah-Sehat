//! Klien Valkey Ruang Sehat dan RME (F1-T33, ADR-007, ADR-011 butir 8, RT-18).
//!
//! Pembungkus tipis crate `redis` 1.8.2 (docs/05-stack.md §1.1, `[SRC-171]`, API diverifikasi: SRC-571)
//! dengan perilaku yang dibutuhkan pemanggil keamanan:
//! - **Tidak pernah menggantung**: setiap operasi dibatasi waktu ([`ValkeyConfig::op_timeout`]) di lapisan
//!   ini, di atas batas waktu koneksi dan respons milik `ConnectionManager`. Habis waktu = galat, sehingga
//!   pemanggil beralih ke sumber kebenaran (PostgreSQL, RT-18 mitigasi 2).
//! - **Tidak gagal saat start**: koneksi dibuat malas (`ConnectionManager::new_lazy_with_config`); proses
//!   tetap berjalan bila Valkey belum tersedia, dan tersambung ulang otomatis.
//! - **Tanpa isi di galat**: [`ValkeyError`] hanya membawa jenis kegagalan, tanpa pesan server, URL, atau
//!   password (ADR-010 butir 5).
//! - TTL dalam milidetik dan **dibulatkan ke atas** (minimal 1 ms), supaya entri pencabutan tidak pernah
//!   kedaluwarsa lebih awal dari yang diminta.
//!
//! Kompatibilitas dengan Valkey 9.1.3 (image `infra/local/compose.yaml`) dibuktikan test integrasi
//! `tests/valkey_compat.rs` (SRC-572). TLS antar-host belum diaktifkan di sini (fitur TLS crate tidak
//! dinyalakan); konfigurasinya menyusul bersama butir 9a docs/05-stack.md §13.

use foundation::Sensitive;
use redis::aio::{ConnectionManager, ConnectionManagerConfig};
use std::fmt;
use std::future::Future;
use std::time::Duration;

/// Bawaan batas waktu satu operasi (milidetik). Usulan teknis F1-T33: cukup untuk satu putaran di jaringan
/// privat, cukup pendek supaya jalur cadangan database tidak menambah latensi permintaan secara berarti.
pub const DEFAULT_OP_TIMEOUT_MS: u64 = 250;

/// Konfigurasi klien.
pub struct ValkeyConfig {
    /// URL `redis://` atau `valkey://` beserta password (rahasia).
    pub url: Sensitive<String>,
    /// Batas waktu satu operasi, termasuk menunggu koneksi.
    pub op_timeout: Duration,
}

impl fmt::Debug for ValkeyConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ValkeyConfig")
            .field("url", &"[REDACTED]")
            .field("op_timeout", &self.op_timeout)
            .finish()
    }
}

/// Jenis kegagalan operasi Valkey.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValkeyErrorKind {
    /// URL tidak valid (saat pembuatan klien).
    Config,
    /// Operasi melewati batas waktu.
    Timeout,
    /// Koneksi gagal atau putus.
    Unavailable,
    /// Server menolak perintah atau balasan tidak sesuai jenis yang diharapkan.
    Protocol,
}

/// Galat operasi Valkey; hanya jenisnya (tanpa pesan server, URL, atau password).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValkeyError {
    kind: ValkeyErrorKind,
}

impl ValkeyError {
    /// Jenis kegagalan.
    pub fn kind(self) -> ValkeyErrorKind {
        self.kind
    }

    /// Kode tetap untuk log.
    pub fn code(self) -> &'static str {
        match self.kind {
            ValkeyErrorKind::Config => "config",
            ValkeyErrorKind::Timeout => "timeout",
            ValkeyErrorKind::Unavailable => "unavailable",
            ValkeyErrorKind::Protocol => "protocol",
        }
    }

    fn new(kind: ValkeyErrorKind) -> Self {
        Self { kind }
    }
}

impl fmt::Display for ValkeyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "valkey: {}", self.code())
    }
}

impl std::error::Error for ValkeyError {}

fn map_err(e: &redis::RedisError) -> ValkeyError {
    if e.is_timeout() {
        ValkeyError::new(ValkeyErrorKind::Timeout)
    } else if e.is_io_error() || e.is_connection_dropped() || e.is_connection_refusal() {
        ValkeyError::new(ValkeyErrorKind::Unavailable)
    } else {
        ValkeyError::new(ValkeyErrorKind::Protocol)
    }
}

/// TTL milidetik dibulatkan ke atas, minimal 1.
fn ttl_ms(ttl: Duration) -> u64 {
    let ms = ttl.as_millis();
    let rounded = if Duration::from_millis(u64::try_from(ms).unwrap_or(u64::MAX)) < ttl {
        ms.saturating_add(1)
    } else {
        ms
    };
    u64::try_from(rounded).unwrap_or(u64::MAX).max(1)
}

/// Klien Valkey. Murah di-clone (koneksi multipleks bersama).
#[derive(Clone)]
pub struct Valkey {
    conn: ConnectionManager,
    op_timeout: Duration,
}

impl fmt::Debug for Valkey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Valkey")
            .field("op_timeout", &self.op_timeout)
            .finish_non_exhaustive()
    }
}

impl Valkey {
    /// Membuat klien tanpa membuka koneksi (koneksi dibuka saat operasi pertama). Wajib dipanggil di dalam
    /// runtime tokio. Galat hanya bila URL tidak valid.
    pub fn connect_lazy(config: &ValkeyConfig) -> Result<Self, ValkeyError> {
        let client = redis::Client::open(config.url.expose().as_str())
            .map_err(|_| ValkeyError::new(ValkeyErrorKind::Config))?;
        let manager_config = ConnectionManagerConfig::new()
            .set_connection_timeout(Some(config.op_timeout))
            .set_response_timeout(Some(config.op_timeout))
            .set_number_of_retries(1)
            .set_max_delay(config.op_timeout);
        let conn = ConnectionManager::new_lazy_with_config(client, manager_config)
            .map_err(|_| ValkeyError::new(ValkeyErrorKind::Config))?;
        Ok(Self {
            conn,
            op_timeout: config.op_timeout,
        })
    }

    async fn bounded<T, F>(&self, op: F) -> Result<T, ValkeyError>
    where
        F: Future<Output = redis::RedisResult<T>>,
    {
        match tokio::time::timeout(self.op_timeout, op).await {
            Ok(Ok(v)) => Ok(v),
            Ok(Err(e)) => Err(map_err(&e)),
            Err(_) => Err(ValkeyError::new(ValkeyErrorKind::Timeout)),
        }
    }

    /// `PING`.
    pub async fn ping(&self) -> Result<(), ValkeyError> {
        let mut c = self.conn.clone();
        let pong: String = self
            .bounded(async move { redis::cmd("PING").query_async(&mut c).await })
            .await?;
        if pong == "PONG" {
            Ok(())
        } else {
            Err(ValkeyError::new(ValkeyErrorKind::Protocol))
        }
    }

    /// Versi server dari `INFO server` (`valkey_version`), untuk log start dan test kompatibilitas.
    pub async fn server_version(&self) -> Result<String, ValkeyError> {
        let mut c = self.conn.clone();
        let mut cmd = redis::cmd("INFO");
        cmd.arg("server");
        let info: String = self
            .bounded(async move { cmd.query_async(&mut c).await })
            .await?;
        info.lines()
            .find_map(|l| l.trim().strip_prefix("valkey_version:"))
            .map(str::to_string)
            .ok_or(ValkeyError::new(ValkeyErrorKind::Protocol))
    }

    /// `GET key`.
    pub async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, ValkeyError> {
        let mut c = self.conn.clone();
        let cmd = {
            let mut cmd = redis::cmd("GET");
            cmd.arg(key);
            cmd
        };
        self.bounded(async move { cmd.query_async(&mut c).await })
            .await
    }

    /// `MGET key...`: nilai per kunci sesuai urutan.
    pub async fn get_many(&self, keys: &[&str]) -> Result<Vec<Option<Vec<u8>>>, ValkeyError> {
        if keys.is_empty() {
            return Ok(Vec::new());
        }
        let mut c = self.conn.clone();
        let mut cmd = redis::cmd("MGET");
        for k in keys {
            cmd.arg(*k);
        }
        let values: Vec<Option<Vec<u8>>> = self
            .bounded(async move { cmd.query_async(&mut c).await })
            .await?;
        if values.len() == keys.len() {
            Ok(values)
        } else {
            Err(ValkeyError::new(ValkeyErrorKind::Protocol))
        }
    }

    /// `SET key value PX ttl` (menimpa).
    pub async fn set(&self, key: &str, value: &[u8], ttl: Duration) -> Result<(), ValkeyError> {
        let mut c = self.conn.clone();
        let mut cmd = redis::cmd("SET");
        cmd.arg(key).arg(value).arg("PX").arg(ttl_ms(ttl));
        let reply: Option<String> = self
            .bounded(async move { cmd.query_async(&mut c).await })
            .await?;
        match reply.as_deref() {
            Some("OK") => Ok(()),
            _ => Err(ValkeyError::new(ValkeyErrorKind::Protocol)),
        }
    }

    /// `SET key value NX PX ttl`: `true` bila tersimpan, `false` bila kunci sudah ada.
    pub async fn set_if_absent(
        &self,
        key: &str,
        value: &[u8],
        ttl: Duration,
    ) -> Result<bool, ValkeyError> {
        let mut c = self.conn.clone();
        let mut cmd = redis::cmd("SET");
        cmd.arg(key).arg(value).arg("NX").arg("PX").arg(ttl_ms(ttl));
        let reply: Option<String> = self
            .bounded(async move { cmd.query_async(&mut c).await })
            .await?;
        match reply.as_deref() {
            Some("OK") => Ok(true),
            None => Ok(false),
            Some(_) => Err(ValkeyError::new(ValkeyErrorKind::Protocol)),
        }
    }

    /// `SET key value` tanpa TTL (penanda yang hidup selama isi Valkey tidak hilang).
    pub async fn set_persistent(&self, key: &str, value: &[u8]) -> Result<(), ValkeyError> {
        let mut c = self.conn.clone();
        let mut cmd = redis::cmd("SET");
        cmd.arg(key).arg(value);
        let reply: Option<String> = self
            .bounded(async move { cmd.query_async(&mut c).await })
            .await?;
        match reply.as_deref() {
            Some("OK") => Ok(()),
            _ => Err(ValkeyError::new(ValkeyErrorKind::Protocol)),
        }
    }

    /// Beberapa `SET key value PX ttl` dalam satu pipeline atomik (`MULTI`/`EXEC`).
    pub async fn set_many(&self, items: &[(String, Vec<u8>, Duration)]) -> Result<(), ValkeyError> {
        if items.is_empty() {
            return Ok(());
        }
        let mut c = self.conn.clone();
        let mut pipe = redis::pipe();
        pipe.atomic();
        for (k, v, ttl) in items {
            pipe.cmd("SET")
                .arg(k.as_str())
                .arg(v.as_slice())
                .arg("PX")
                .arg(ttl_ms(*ttl))
                .ignore();
        }
        self.bounded(async move { pipe.query_async::<()>(&mut c).await })
            .await
    }

    /// `DEL key`.
    pub async fn delete(&self, key: &str) -> Result<(), ValkeyError> {
        let mut c = self.conn.clone();
        let mut cmd = redis::cmd("DEL");
        cmd.arg(key);
        let _removed: i64 = self
            .bounded(async move { cmd.query_async(&mut c).await })
            .await?;
        Ok(())
    }

    /// `INCR key`: nilai setelah dinaikkan.
    pub async fn incr(&self, key: &str) -> Result<i64, ValkeyError> {
        let mut c = self.conn.clone();
        let mut cmd = redis::cmd("INCR");
        cmd.arg(key);
        self.bounded(async move { cmd.query_async(&mut c).await })
            .await
    }

    /// `SET key value` tanpa TTL **hanya bila** nilai `guard_key` masih `expected` (`None` = kunci tidak
    /// ada), atomik di server lewat skrip Lua (`EVAL`). `true` bila tersimpan.
    pub async fn set_persistent_if_unchanged(
        &self,
        key: &str,
        value: &[u8],
        guard_key: &str,
        expected: Option<&[u8]>,
    ) -> Result<bool, ValkeyError> {
        const SCRIPT: &str = "local cur = redis.call('GET', KEYS[2]) \
            if ARGV[3] == '0' then if cur then return 0 end \
            elseif cur ~= ARGV[2] then return 0 end \
            redis.call('SET', KEYS[1], ARGV[1]) return 1";
        let mut c = self.conn.clone();
        let mut cmd = redis::cmd("EVAL");
        cmd.arg(SCRIPT)
            .arg(2)
            .arg(key)
            .arg(guard_key)
            .arg(value)
            .arg(expected.unwrap_or_default())
            .arg(if expected.is_some() { "1" } else { "0" });
        let stored: i64 = self
            .bounded(async move { cmd.query_async(&mut c).await })
            .await?;
        Ok(stored == 1)
    }

    /// `PTTL key` dalam milidetik: `None` bila kunci tidak ada atau tanpa TTL.
    pub async fn ttl(&self, key: &str) -> Result<Option<Duration>, ValkeyError> {
        let mut c = self.conn.clone();
        let mut cmd = redis::cmd("PTTL");
        cmd.arg(key);
        let ms: i64 = self
            .bounded(async move { cmd.query_async(&mut c).await })
            .await?;
        Ok(u64::try_from(ms).ok().map(Duration::from_millis))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ttl_rounds_up_and_is_at_least_one_millisecond() {
        assert_eq!(ttl_ms(Duration::from_millis(1500)), 1500);
        assert_eq!(ttl_ms(Duration::from_micros(1_500_001)), 1501);
        assert_eq!(ttl_ms(Duration::from_nanos(1)), 1);
        assert_eq!(ttl_ms(Duration::ZERO), 1);
    }

    #[test]
    fn errors_and_config_do_not_leak_secrets() {
        let cfg = ValkeyConfig {
            url: Sensitive::new("redis://:rahasia-sekali@127.0.0.1:6379/".to_string()),
            op_timeout: Duration::from_millis(10),
        };
        assert!(!format!("{cfg:?}").contains("rahasia"));
        let e = ValkeyError::new(ValkeyErrorKind::Unavailable);
        assert_eq!(e.to_string(), "valkey: unavailable");
    }
}
