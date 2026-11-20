//! Blind index NIK (ADR-013 §4, skema fisik §7.2, KEPUTUSAN-105, [SRC-46]).
//!
//! `nik_bidx = HMAC-SHA256(K_nik[v], "rs.nik.v1" ‖ 0x00 ‖ NIK_normal)`, disimpan utuh 32 byte
//! (CHECK `octet_length(nik_bidx) = 32`) bersama `nik_bidx_key_version` = `v` (CHECK `>= 1`).
//! `NIK_normal` = 16 digit ASCII setelah spasi dan tanda baca dibuang.
//!
//! Kunci `K_nik` Ruang Sehat disimpan di OpenBao KV v2 sebagai peta `{versi: kunci}` dengan nomor versi
//! eksplisit di isi rahasia (skema fisik §7.2), tidak bergantung pada versi bawaan KV. Format isi
//! rahasia yang dibaca [`load_nik_blind_index_keyring`]: setiap kunci berupa nomor versi desimal
//! (1–32767, tanpa nol di depan) dan nilainya 32 byte acak dalam base64 standar. Versi tertinggi
//! dipakai untuk menulis.
//!
//! Selama rotasi kunci (TD-10, UP-40), pemeriksaan keunikan NIK wajib mencocokkan **semua** versi kunci
//! yang aktif ([`NikBlindIndexKeyring::lookup_candidates`]), bukan hanya versi terbaru, karena baris
//! yang belum ditulis ulang masih memakai versi lama (prosedur rotasi DD A2.8).

use crate::envelope::SecretBytes;
use crate::error::CryptoError;
use crate::openbao::{OpenBaoClient, OpenBaoConfig, valid_path};
use crate::primitives::{HmacSha256Key, SHA256_LEN, constant_time_eq};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use foundation::Sensitive;
use http::Method;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fmt;
use zeroize::{Zeroize, Zeroizing};

/// Label domain HMAC blind index NIK versi 1.
pub const NIK_BIDX_LABEL_V1: &[u8] = b"rs.nik.v1";
/// Panjang blind index (byte), sama dengan CHECK di skema.
pub const BLIND_INDEX_LEN: usize = SHA256_LEN;
/// Panjang kunci blind index (byte).
pub const BLIND_INDEX_KEY_LEN: usize = 32;
/// Jumlah digit NIK (ADR-013 §4, [SRC-63]).
pub const NIK_DIGITS: usize = 16;

/// Format NIK tidak valid (bukan 16 digit setelah normalisasi). Tidak membawa nilai masukan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidNik;

impl fmt::Display for InvalidNik {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("NIK harus 16 digit angka")
    }
}

impl std::error::Error for InvalidNik {}

/// NIK yang sudah dinormalkan (16 digit ASCII). Tersamar di `Debug`/`Display`, dinolkan saat dibuang.
pub struct Nik(Sensitive<Zeroizing<String>>);

impl Nik {
    /// Normalisasi: buang spasi (termasuk spasi Unicode) dan tanda baca ASCII, lalu wajib tepat
    /// 16 digit ASCII. Digit non-ASCII dan huruf ditolak.
    pub fn parse(input: &str) -> Result<Self, InvalidNik> {
        let normal: Zeroizing<String> = Zeroizing::new(
            input
                .chars()
                .filter(|c| !c.is_whitespace() && !c.is_ascii_punctuation())
                .collect(),
        );
        if normal.len() != NIK_DIGITS || !normal.bytes().all(|b| b.is_ascii_digit()) {
            return Err(InvalidNik);
        }
        Ok(Self(Sensitive::new(normal)))
    }

    /// NIK normal (16 digit), misalnya untuk dienkripsi ke `nik_enc`.
    pub fn expose(&self) -> &str {
        self.0.expose()
    }

    /// Empat digit terakhir untuk kolom `nik_last4` (tampilan termasking, ADR-013 §4).
    pub fn last4(&self) -> String {
        let s: &str = self.0.expose();
        s.get(NIK_DIGITS - 4..).unwrap_or_default().to_string()
    }
}

impl fmt::Debug for Nik {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Nik([REDACTED])")
    }
}

/// Blind index beserta versi kuncinya (kolom `nik_bidx`, `nik_bidx_key_version`).
#[derive(Clone, PartialEq, Eq)]
pub struct BlindIndex {
    /// 32 byte HMAC-SHA256.
    pub value: [u8; BLIND_INDEX_LEN],
    /// Versi kunci (`>= 1`, muat di `smallint`).
    pub key_version: i16,
}

impl fmt::Debug for BlindIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BlindIndex")
            .field("value", &"[REDACTED]")
            .field("key_version", &self.key_version)
            .finish()
    }
}

/// Kunci blind index NIK per versi.
pub struct NikBlindIndexKeyring {
    keys: BTreeMap<i16, HmacSha256Key>,
    current: i16,
}

impl fmt::Debug for NikBlindIndexKeyring {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NikBlindIndexKeyring")
            .field("versions", &self.keys.keys().collect::<Vec<_>>())
            .field("current", &self.current)
            .finish()
    }
}

impl NikBlindIndexKeyring {
    /// Membuat keyring dari pasangan (versi, kunci 32 byte). Versi `>= 1` dan unik; minimal satu kunci.
    pub fn new(keys: Vec<(i16, SecretBytes)>) -> Result<Self, CryptoError> {
        let mut map = BTreeMap::new();
        for (version, key) in keys {
            if version < 1 || key.expose().len() != BLIND_INDEX_KEY_LEN {
                return Err(CryptoError::InvalidConfig);
            }
            if map
                .insert(version, HmacSha256Key::new(key.expose())?)
                .is_some()
            {
                return Err(CryptoError::InvalidConfig);
            }
        }
        let current = *map.keys().next_back().ok_or(CryptoError::InvalidConfig)?;
        Ok(Self { keys: map, current })
    }

    /// Versi kunci untuk menulis (tertinggi).
    pub fn current_version(&self) -> i16 {
        self.current
    }

    /// Semua versi kunci aktif, terbaru dulu.
    pub fn versions(&self) -> Vec<i16> {
        self.keys.keys().rev().copied().collect()
    }

    fn compute_with(key: &HmacSha256Key, version: i16, nik: &Nik) -> BlindIndex {
        let mut msg = Zeroizing::new(Vec::with_capacity(NIK_BIDX_LABEL_V1.len() + 1 + NIK_DIGITS));
        msg.extend_from_slice(NIK_BIDX_LABEL_V1);
        msg.push(0x00);
        msg.extend_from_slice(nik.expose().as_bytes());
        BlindIndex {
            value: key.sign(&msg),
            key_version: version,
        }
    }

    /// Blind index dengan kunci versi terbaru (untuk baris baru dan penulisan ulang saat rotasi).
    pub fn compute(&self, nik: &Nik) -> Result<BlindIndex, CryptoError> {
        self.compute_with_version(nik, self.current)
    }

    /// Blind index dengan kunci versi tertentu.
    pub fn compute_with_version(&self, nik: &Nik, version: i16) -> Result<BlindIndex, CryptoError> {
        let key = self
            .keys
            .get(&version)
            .ok_or(CryptoError::UnknownKeyVersion)?;
        Ok(Self::compute_with(key, version, nik))
    }

    /// Blind index NIK untuk **setiap** versi kunci aktif (terbaru dulu). Pencarian keunikan (UP-40)
    /// mencocokkan pasangan (`nik_bidx`, `nik_bidx_key_version`) mana pun dari daftar ini (TD-10).
    pub fn lookup_candidates(&self, nik: &Nik) -> Vec<BlindIndex> {
        self.keys
            .iter()
            .rev()
            .map(|(v, k)| Self::compute_with(k, *v, nik))
            .collect()
    }

    /// Memeriksa apakah nilai tersimpan cocok dengan NIK (perbandingan waktu-konstan). Versi yang tidak
    /// dikenal (misalnya kunci lama sudah dicabut) menghasilkan `false`.
    pub fn matches(&self, nik: &Nik, stored_value: &[u8], stored_version: i16) -> bool {
        match self.compute_with_version(nik, stored_version) {
            Ok(b) => constant_time_eq(&b.value, stored_value),
            Err(_) => false,
        }
    }
}

#[derive(Deserialize)]
struct KvEnvelope {
    data: KvData,
}

#[derive(Deserialize)]
struct KvData {
    data: BTreeMap<String, String>,
}

/// Mengurai nomor versi desimal tanpa nol di depan, 1–32767.
fn parse_version(s: &str) -> Option<i16> {
    if s.is_empty() || s.starts_with('0') || !s.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    s.parse::<i16>().ok().filter(|v| *v >= 1)
}

fn keyring_from_kv(
    mut data: BTreeMap<String, String>,
) -> Result<NikBlindIndexKeyring, CryptoError> {
    let mut keys = Vec::with_capacity(data.len());
    let mut result = Ok(());
    for (name, value) in data.iter_mut() {
        let decoded = STANDARD.decode(value.as_bytes()).map(Zeroizing::new);
        value.zeroize();
        match (parse_version(name), decoded) {
            (Some(v), Ok(bytes)) if bytes.len() == BLIND_INDEX_KEY_LEN => {
                keys.push((v, Sensitive::new(bytes)));
            }
            _ => result = Err(CryptoError::InvalidConfig),
        }
    }
    result?;
    NikBlindIndexKeyring::new(keys)
}

/// Memuat kunci blind index NIK dari OpenBao KV v2: `GET /v1/<mount>/data/<path>` [SRC-488].
/// Mount dan path hanya `[a-z0-9_-]` (path boleh ber-`/`). Dipanggil saat start dan saat rotasi.
pub async fn load_nik_blind_index_keyring(
    openbao: &OpenBaoConfig,
    mount: &str,
    path: &str,
) -> Result<NikBlindIndexKeyring, CryptoError> {
    if !valid_path(mount, false) || !valid_path(path, true) {
        return Err(CryptoError::InvalidConfig);
    }
    let client = OpenBaoClient::new(openbao)?;
    let raw = client
        .call(Method::GET, &format!("/v1/{mount}/data/{path}"), None)
        .await?;
    let parsed: KvEnvelope =
        serde_json::from_slice(&raw).map_err(|_| CryptoError::KmsInvalidResponse)?;
    keyring_from_kv(parsed.data.data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::envelope::secret_bytes;

    // NIK sintetis (pola berulang, bukan milik orang nyata).
    const NIK: &str = "3200000000000001";

    fn hex(b: &[u8]) -> String {
        b.iter().map(|x| format!("{x:02x}")).collect()
    }

    fn ring(entries: &[(i16, u8)]) -> NikBlindIndexKeyring {
        NikBlindIndexKeyring::new(
            entries
                .iter()
                .map(|(v, b)| (*v, secret_bytes(vec![*b; 32])))
                .collect(),
        )
        .unwrap()
    }

    #[test]
    fn blind_index_matches_independent_hmac_computation() {
        // Nilai harapan dihitung terpisah dengan modul hmac/hashlib Python:
        // HMAC-SHA256(0x0b * 32, b"rs.nik.v1" + b"\x00" + b"3200000000000001").
        let r = ring(&[(1, 0x0b)]);
        let b = r.compute(&Nik::parse(NIK).unwrap()).unwrap();
        assert_eq!(
            hex(&b.value),
            "1ba799f899645b60737916440c7bc440e6153ec674568431729bf1d2a946b231"
        );
        assert_eq!(b.key_version, 1);
        assert_eq!(b.value.len(), 32); // CHECK octet_length(nik_bidx) = 32

        let seq: Vec<u8> = (0u8..32).collect();
        let r2 = NikBlindIndexKeyring::new(vec![(3, secret_bytes(seq))]).unwrap();
        let b2 = r2.compute(&Nik::parse(NIK).unwrap()).unwrap();
        assert_eq!(
            hex(&b2.value),
            "aeac5031306b77543588817382a300b1828d80750919114c3fe916e956c48acd"
        );
        assert_eq!(b2.key_version, 3);
    }

    #[test]
    fn nik_normalization_strips_spaces_and_punctuation_only() {
        let r = ring(&[(1, 0x0b)]);
        let base = r.compute(&Nik::parse(NIK).unwrap()).unwrap();
        for variant in [
            "3200 0000 0000 0001",
            " 3200-0000-0000-0001 ",
            "3200.0000.0000.0001",
            "3200\u{a0}000000000001",
        ] {
            let n = Nik::parse(variant).unwrap();
            assert_eq!(n.expose(), NIK);
            assert_eq!(r.compute(&n).unwrap(), base, "{variant:?}");
        }
    }

    #[test]
    fn invalid_nik_is_rejected() {
        for bad in [
            "",
            "320000000000000",   // 15 digit
            "32000000000000011", // 17 digit
            "320000000000000A",
            "३२००००००००००००००", // digit Devanagari
            "3200–0000–0000–0001x",
        ] {
            assert_eq!(Nik::parse(bad).err(), Some(InvalidNik), "{bad:?}");
        }
    }

    #[test]
    fn rotation_lookup_covers_all_active_versions() {
        let old = ring(&[(1, 0x11)]);
        let both = ring(&[(1, 0x11), (2, 0x22)]);
        let nik = Nik::parse(NIK).unwrap();
        let stored_old = old.compute(&nik).unwrap(); // baris yang belum ditulis ulang
        assert_eq!(both.current_version(), 2);
        assert_eq!(both.versions(), vec![2, 1]);
        let new = both.compute(&nik).unwrap();
        assert_eq!(new.key_version, 2);
        assert_ne!(
            new.value, stored_old.value,
            "versi berbeda menghasilkan nilai berbeda"
        );
        let candidates = both.lookup_candidates(&nik);
        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0], new);
        assert!(
            candidates.contains(&stored_old),
            "TD-10: versi lama ikut dicocokkan"
        );
        assert!(both.matches(&nik, &stored_old.value, 1));
        assert!(both.matches(&nik, &new.value, 2));
        assert!(!both.matches(&nik, &stored_old.value, 2));
        assert!(!both.matches(&nik, &new.value, 9));
        let other = Nik::parse("3200000000000002").unwrap();
        assert!(!both.matches(&other, &new.value, 2));
        assert_eq!(
            both.compute_with_version(&nik, 3).err(),
            Some(CryptoError::UnknownKeyVersion)
        );
    }

    #[test]
    fn keyring_rejects_bad_versions_and_key_lengths() {
        let k = |n| secret_bytes(vec![1u8; n]);
        for keys in [
            vec![],
            vec![(0, k(32))],
            vec![(-1, k(32))],
            vec![(1, k(31))],
            vec![(1, k(33))],
            vec![(1, k(32)), (1, k(32))],
        ] {
            assert_eq!(
                NikBlindIndexKeyring::new(keys).err(),
                Some(CryptoError::InvalidConfig)
            );
        }
    }

    #[test]
    fn kv_secret_format_is_validated() {
        let b64 = |b: u8| STANDARD.encode([b; 32]);
        let ok: BTreeMap<String, String> =
            [("1".to_string(), b64(1)), ("2".to_string(), b64(2))].into();
        let r = keyring_from_kv(ok).unwrap();
        assert_eq!(r.versions(), vec![2, 1]);
        for (name, value) in [
            ("v1", b64(1)),
            ("01", b64(1)),
            ("0", b64(1)),
            ("40000", b64(1)),
            ("1", STANDARD.encode([1u8; 16])),
            ("1", "bukan base64".to_string()),
        ] {
            let m: BTreeMap<String, String> = [(name.to_string(), value)].into();
            assert_eq!(
                keyring_from_kv(m).err(),
                Some(CryptoError::InvalidConfig),
                "{name}"
            );
        }
        assert_eq!(
            keyring_from_kv(BTreeMap::new()).err(),
            Some(CryptoError::InvalidConfig)
        );
    }

    #[test]
    fn debug_output_does_not_leak_nik_keys_or_index() {
        let r = ring(&[(1, 0x0b)]);
        let nik = Nik::parse(NIK).unwrap();
        let b = r.compute(&nik).unwrap();
        let out = format!("{nik:?} {b:?} {r:?}");
        assert!(!out.contains(NIK));
        assert!(!out.contains("1ba799f8"));
        assert!(!out.contains("11, 11") && !out.contains("0b0b"));
        assert_eq!(nik.last4(), "0001");
    }
}
