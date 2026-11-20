//! Jangkar kepala rantai (ADR-009 butir 5, DD B4; F1-T25).
//!
//! Kepala rantai (`seq`, `event_hash`) ditulis berkala oleh relay ke penyimpanan tulis-sekali (WORM), ke
//! tabel `audit_chain_anchor` (hak `INSERT` `*_audit_writer`), dan ke security log. Verifikasi harian
//! (`audit-verify`) membaca jangkar **langsung dari penyimpanan WORM** (KEPUTUSAN-141), tidak hanya dari
//! tabel, sehingga penyerang yang menulis ulang rantai di database tetap terdeteksi.
//!
//! [`AnchorStore`] adalah antarmukanya. Implementasi di crate ini: [`FileAnchorStore`] (lokal: satu berkas
//! per jangkar, dibuat dengan `create_new`, tidak pernah ditimpa). Implementasi staging/production di bucket
//! NEO Object Storage dengan Object Lock mode Compliance (SRC-99, KEPUTUSAN-78) belum ada di task ini.

use crate::chain::{ChainHash, hex, parse_hash_hex};
use crate::error::{StoreError, StoreErrorKind};
use crate::writer::ChainHead;
use foundation::{Clock, Uuid, new_id};
use serde_json::{Value as Json, json};
use sqlx::PgConnection;
use std::future::Future;
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::pin::Pin;
use time::OffsetDateTime;

/// Format isi objek jangkar versi 1.
pub const ANCHOR_FORMAT_V1: &str = "ruang-sehat/audit-chain-head/v1";
const FILE_PREFIX: &str = "chain-head-";
const FILE_SUFFIX: &str = ".json";
/// Nama constraint unik `seq` jangkar: pelanggarannya = kepala yang sama sudah dijangkarkan.
pub const ANCHOR_SEQ_CONSTRAINT: &str = "uq_audit_chain_anchor_seq";

/// Satu jangkar kepala rantai.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChainAnchor {
    /// `seq` kepala rantai.
    pub seq: i64,
    /// `event_hash` kepala rantai.
    pub event_hash: ChainHash,
    /// Waktu penjangkaran (jam server relay).
    pub anchored_at: OffsetDateTime,
}

/// Future hasil [`AnchorStore`].
pub type AnchorFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, StoreError>> + Send + 'a>>;

/// Penyimpanan jangkar tulis-sekali.
pub trait AnchorStore: Send + Sync {
    /// Menulis jangkar; mengembalikan kunci objeknya (`worm_object_key`). Menulis ulang jangkar dengan
    /// `seq` dan `event_hash` yang sama sah (idempoten); `seq` sama dengan hash berbeda = galat.
    fn put<'a>(&'a self, anchor: &'a ChainAnchor) -> AnchorFuture<'a, String>;
    /// Semua jangkar, urut `seq`.
    fn list(&self) -> AnchorFuture<'_, Vec<ChainAnchor>>;
}

/// Jangkar sebagai berkas di satu direktori lokal (pengembangan dan test).
#[derive(Debug, Clone)]
pub struct FileAnchorStore {
    dir: PathBuf,
}

impl FileAnchorStore {
    /// Direktori jangkar; dibuat bila belum ada.
    pub fn new(dir: impl Into<PathBuf>) -> Result<Self, StoreError> {
        let dir = dir.into();
        std::fs::create_dir_all(&dir).map_err(|_| anchor_err("anchor_dir"))?;
        Ok(Self { dir })
    }

    /// Direktori jangkar.
    pub fn dir(&self) -> &Path {
        &self.dir
    }

    fn file_name(seq: i64) -> String {
        format!("{FILE_PREFIX}{seq:020}{FILE_SUFFIX}")
    }

    fn put_sync(&self, anchor: &ChainAnchor) -> Result<String, StoreError> {
        if anchor.seq < 1 {
            return Err(anchor_err("anchor_seq"));
        }
        let name = Self::file_name(anchor.seq);
        let path = self.dir.join(&name);
        let body = encode(anchor)?;
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
        {
            Ok(mut f) => {
                f.write_all(body.as_bytes())
                    .and_then(|()| f.sync_all())
                    .map_err(|_| anchor_err("anchor_write"))?;
            }
            Err(e) if e.kind() == ErrorKind::AlreadyExists => {
                let existing = read_anchor(&path)?;
                if existing.seq != anchor.seq || existing.event_hash != anchor.event_hash {
                    return Err(anchor_err("anchor_conflict"));
                }
            }
            Err(_) => return Err(anchor_err("anchor_write")),
        }
        Ok(name)
    }

    fn list_sync(&self) -> Result<Vec<ChainAnchor>, StoreError> {
        let mut out = Vec::new();
        let entries = std::fs::read_dir(&self.dir).map_err(|_| anchor_err("anchor_list"))?;
        for entry in entries {
            let entry = entry.map_err(|_| anchor_err("anchor_list"))?;
            let name = entry.file_name();
            let Some(name) = name.to_str() else {
                continue;
            };
            let Some(seq) = name
                .strip_prefix(FILE_PREFIX)
                .and_then(|r| r.strip_suffix(FILE_SUFFIX))
            else {
                continue;
            };
            let anchor = read_anchor(&entry.path())?;
            if seq.parse::<i64>().ok() != Some(anchor.seq) {
                return Err(anchor_err("anchor_name_mismatch"));
            }
            out.push(anchor);
        }
        out.sort_by_key(|a| a.seq);
        Ok(out)
    }
}

impl AnchorStore for FileAnchorStore {
    fn put<'a>(&'a self, anchor: &'a ChainAnchor) -> AnchorFuture<'a, String> {
        // Berkas kecil; operasi sinkron singkat (pengembangan/test saja).
        Box::pin(async move { self.put_sync(anchor) })
    }

    fn list(&self) -> AnchorFuture<'_, Vec<ChainAnchor>> {
        Box::pin(async move { self.list_sync() })
    }
}

fn anchor_err(what: &'static str) -> StoreError {
    StoreError::new(StoreErrorKind::Anchor, what)
}

fn encode(anchor: &ChainAnchor) -> Result<String, StoreError> {
    let at = anchor
        .anchored_at
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|_| anchor_err("anchored_at"))?;
    let micros = i64::try_from(anchor.anchored_at.unix_timestamp_nanos() / 1_000)
        .map_err(|_| anchor_err("anchored_at"))?;
    Ok(json!({
        "format": ANCHOR_FORMAT_V1,
        "seq": anchor.seq,
        "event_hash": hex(&anchor.event_hash),
        "anchored_at": at,
        "anchored_at_unix_micros": micros,
    })
    .to_string())
}

fn read_anchor(path: &Path) -> Result<ChainAnchor, StoreError> {
    let text = std::fs::read_to_string(path).map_err(|_| anchor_err("anchor_read"))?;
    let v: Json = serde_json::from_str(&text).map_err(|_| anchor_err("anchor_format"))?;
    if v.get("format").and_then(Json::as_str) != Some(ANCHOR_FORMAT_V1) {
        return Err(anchor_err("anchor_format"));
    }
    let seq = v
        .get("seq")
        .and_then(Json::as_i64)
        .ok_or(anchor_err("anchor_format"))?;
    let event_hash = v
        .get("event_hash")
        .and_then(Json::as_str)
        .and_then(parse_hash_hex)
        .ok_or(anchor_err("anchor_format"))?;
    let anchored_at = v
        .get("anchored_at_unix_micros")
        .and_then(Json::as_i64)
        .and_then(|m| OffsetDateTime::from_unix_timestamp_nanos(i128::from(m) * 1_000).ok())
        .ok_or(anchor_err("anchor_format"))?;
    Ok(ChainAnchor {
        seq,
        event_hash,
        anchored_at,
    })
}

/// Hasil [`anchor_head`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnchorWritten {
    /// Jangkar yang ditulis.
    pub anchor: ChainAnchor,
    /// Kunci objek di penyimpanan WORM.
    pub object_key: String,
    /// `audit_chain_anchor.id` baru, atau None bila `seq` itu sudah tercatat sebelumnya.
    pub row_id: Option<Uuid>,
}

/// Menjangkarkan kepala rantai: penyimpanan WORM dahulu, lalu baris `audit_chain_anchor` (writer), lalu
/// security log. Bila baris untuk `seq` itu sudah ada (relay dimulai ulang), cukup objek WORM-nya.
pub async fn anchor_head(
    writer: &mut PgConnection,
    store: &dyn AnchorStore,
    head: ChainHead,
    clock: &dyn Clock,
) -> Result<AnchorWritten, StoreError> {
    let anchor = ChainAnchor {
        seq: head.seq,
        event_hash: head.event_hash,
        anchored_at: crate::chain::truncate_to_micros(clock.now()),
    };
    let object_key = store.put(&anchor).await?;
    let id = new_id();
    let inserted = sqlx::query!(
        r#"INSERT INTO audit.audit_chain_anchor (id, seq, event_hash, anchored_at, worm_object_key)
           VALUES ($1, $2, $3, $4, $5)"#,
        id,
        anchor.seq,
        anchor.event_hash.as_slice(),
        anchor.anchored_at,
        object_key,
    )
    .execute(&mut *writer)
    .await;
    let row_id = match inserted {
        Ok(_) => Some(id),
        Err(e)
            if e.as_database_error().is_some_and(|d| {
                d.code().as_deref() == Some("23505")
                    && d.constraint() == Some(ANCHOR_SEQ_CONSTRAINT)
            }) =>
        {
            None
        }
        Err(e) => return Err(StoreError::db("insert_chain_anchor", &e)),
    };
    observability::security::audit_chain_anchored(anchor.seq, &hex(&anchor.event_hash));
    Ok(AnchorWritten {
        anchor,
        object_key,
        row_id,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::datetime;

    fn dir() -> PathBuf {
        std::env::temp_dir().join(format!("rs-anchor-unit-{}", new_id()))
    }

    fn anchor(seq: i64, b: u8) -> ChainAnchor {
        ChainAnchor {
            seq,
            event_hash: [b; 32],
            anchored_at: datetime!(2027-03-01 10:00:00 UTC),
        }
    }

    #[tokio::test]
    async fn file_store_is_write_once_and_lists_in_seq_order() {
        let d = dir();
        let s = FileAnchorStore::new(&d).unwrap();
        assert_eq!(
            s.put(&anchor(10, 1)).await.unwrap(),
            "chain-head-00000000000000000010.json"
        );
        s.put(&anchor(2, 2)).await.unwrap();
        // Tulis ulang identik (relay dimulai ulang, waktu berbeda) sah.
        let mut again = anchor(10, 1);
        again.anchored_at = datetime!(2027-03-01 11:00:00 UTC);
        s.put(&again).await.unwrap();
        // seq sama, hash berbeda: ditolak, berkas lama tidak berubah.
        let e = s.put(&anchor(10, 9)).await.unwrap_err();
        assert_eq!(
            (e.kind(), e.what()),
            (StoreErrorKind::Anchor, "anchor_conflict")
        );
        let listed = s.list().await.unwrap();
        assert_eq!(listed, vec![anchor(2, 2), anchor(10, 1)]);
        assert!(s.put(&anchor(0, 1)).await.is_err());
        std::fs::remove_dir_all(&d).unwrap();
    }

    #[tokio::test]
    async fn corrupt_anchor_file_fails_listing() {
        let d = dir();
        let s = FileAnchorStore::new(&d).unwrap();
        s.put(&anchor(3, 3)).await.unwrap();
        std::fs::write(d.join("chain-head-00000000000000000004.json"), b"{}").unwrap();
        assert_eq!(s.list().await.unwrap_err().what(), "anchor_format");
        std::fs::remove_dir_all(&d).unwrap();
    }
}
