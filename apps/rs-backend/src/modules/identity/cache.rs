//! Bentuk kunci dan nilai cache Valkey modul `identity` (ADR-011 butir 8). Hanya ID internal dan kode;
//! tanpa email, nama, atau token (ADR-010 butir 5).
//!
//! - `rs:account:{realm}:{subject}` → akun (JSON kecil, ber-TTL pendek).
//! - `rs:session-revoked:sid:{sid}` → `1`, TTL sampai pencabutan berakhir.
//! - `rs:session-revoked:loaded` → penanda bahwa semua pencabutan aktif di PostgreSQL sudah disalin ke
//!   Valkey. Valkey berjalan tanpa persistensi (`infra/local/compose.yaml`), jadi isi yang hilang karena
//!   restart ikut menghapus penanda; tanpa penanda, pemeriksaan pencabutan membaca PostgreSQL.

use authn::Realm;
use authz::{AccountRecord, AccountStatus, AccountType, AdminSubRole};
use foundation::Uuid;
use serde::{Deserialize, Serialize};

/// Penanda salinan lengkap pencabutan.
pub(super) const REVOCATIONS_LOADED: &str = "rs:session-revoked:loaded";
/// Penghitung kegagalan penyalinan: dinaikkan setiap kali satu pencabutan gagal disalin. Pembangunan ulang
/// hanya memasang penanda bila penghitung tidak berubah sejak snapshot PostgreSQL diambil, sehingga
/// pencabutan yang gagal disalin selama pembangunan ulang tidak tertutup penanda baru.
pub(super) const REVOCATIONS_EPOCH: &str = "rs:session-revoked:epoch";

pub(super) fn account_key(realm: Realm, subject: &str) -> String {
    format!("rs:account:{}:{subject}", realm.as_str())
}

pub(super) fn revoked_session_key(idp_session_id: &str) -> String {
    format!("rs:session-revoked:sid:{idp_session_id}")
}

/// Versi bentuk nilai akun; nilai versi lain dianggap tidak ada (dibaca ulang dari database).
const ACCOUNT_VALUE_VERSION: u8 = 1;

#[derive(Serialize, Deserialize)]
struct CachedAccount {
    v: u8,
    id: String,
    account_type: String,
    status: String,
    sub_roles: Vec<String>,
}

pub(super) fn encode_account(rec: &AccountRecord) -> Vec<u8> {
    let value = CachedAccount {
        v: ACCOUNT_VALUE_VERSION,
        id: rec.account_id.hyphenated().to_string(),
        account_type: rec.account_type.as_str().to_string(),
        status: rec.status.as_str().to_string(),
        sub_roles: rec
            .sub_roles
            .iter()
            .map(|r| r.as_str().to_string())
            .collect(),
    };
    serde_json::to_vec(&value).unwrap_or_default()
}

/// Nilai rusak, versi lain, atau kode tidak dikenal = `None` (dibaca ulang dari database, tidak pernah
/// memberi hak).
pub(super) fn decode_account(bytes: &[u8]) -> Option<AccountRecord> {
    let v: CachedAccount = serde_json::from_slice(bytes).ok()?;
    if v.v != ACCOUNT_VALUE_VERSION {
        return None;
    }
    let mut sub_roles = Vec::with_capacity(v.sub_roles.len());
    for r in &v.sub_roles {
        sub_roles.push(AdminSubRole::parse(r)?);
    }
    Some(AccountRecord {
        account_id: Uuid::parse_str(&v.id).ok()?,
        account_type: AccountType::parse(&v.account_type)?,
        status: AccountStatus::parse(&v.status)?,
        sub_roles,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rec() -> AccountRecord {
        AccountRecord {
            account_id: Uuid::from_u128(0x0193_6f8e_9c01_7e02_8f03_1a2b_3c4d_5e01),
            account_type: AccountType::RsAdmin,
            status: AccountStatus::Inactive,
            sub_roles: vec![AdminSubRole::Auditor, AdminSubRole::Verifier],
        }
    }

    #[test]
    fn account_value_round_trips() {
        assert_eq!(decode_account(&encode_account(&rec())), Some(rec()));
    }

    #[test]
    fn corrupt_or_unknown_values_are_misses() {
        assert_eq!(decode_account(b"bukan json"), None);
        let other_version = String::from_utf8(encode_account(&rec()))
            .unwrap()
            .replace("\"v\":1", "\"v\":2");
        assert_eq!(decode_account(other_version.as_bytes()), None);
        let bad_role = String::from_utf8(encode_account(&rec()))
            .unwrap()
            .replace("auditor", "super_user");
        assert_eq!(decode_account(bad_role.as_bytes()), None);
        let bad_status = String::from_utf8(encode_account(&rec()))
            .unwrap()
            .replace("inactive", "aktif");
        assert_eq!(decode_account(bad_status.as_bytes()), None);
    }

    #[test]
    fn keys_hold_only_internal_identifiers() {
        assert_eq!(account_key(Realm::Pasien, "s-1"), "rs:account:pasien:s-1");
        assert_eq!(revoked_session_key("sid-1"), "rs:session-revoked:sid:sid-1");
        assert_ne!(revoked_session_key("loaded"), REVOCATIONS_LOADED);
        assert_ne!(revoked_session_key("epoch"), REVOCATIONS_EPOCH);
    }
}
