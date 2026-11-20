//! Persetujuan orang kedua lewat Control Group OpenBao (F1-T65, US-208 AC4, ADR-025, `[SRC-253]`).
//!
//! Alur (API `sys/control-group/*` dan `sys/wrapping/unwrap` OpenBao 2.7.2, SRC-670):
//! 1. Operator membaca path KV yang dilindungi *factor* Control Group. OpenBao tidak menjalankan
//!    permintaan itu, tetapi menjawab dengan token pembungkus sekali pakai beserta `accessor`-nya.
//! 2. Approver (identitas lain, anggota grup approver) memanggil `sys/control-group/authorize` dengan
//!    `accessor` itu.
//! 3. Perintah yang membutuhkan persetujuan memanggil [`verify_control_group_approval`]:
//!    - `auth/token/lookup-self` dengan token operator → `entity_id` operator;
//!    - `sys/control-group/request` dengan `accessor` → `approved`, `request_path`, `request_entity.id`,
//!      `authorizations[].entity_id`;
//!    - syarat: disetujui, path sama dengan yang diharapkan, peminta = operator yang menjalankan perintah,
//!      minimal satu approver dan tidak ada approver yang sama dengan peminta;
//!    - `sys/wrapping/unwrap` dengan token pembungkus → permintaan asli baru dijalankan saat ini, lalu
//!      token pembungkus dicabut (`auth/token/revoke-self`) karena OpenBao 2.7.2 mengizinkan unwrap
//!      berulang selama TTL Control Group (SRC-671); isi KV wajib memuat `purpose` = nilai yang
//!      diharapkan. Satu persetujuan = satu eksekusi.
//!
//! Token operator dan token pembungkus tidak pernah masuk galat, log, atau `Debug`. Yang dikembalikan hanya
//! ID entitas OpenBao (UUID) operator dan approver untuk dicatat di audit trail.

use crate::error::CryptoError;
use crate::openbao::{OpenBaoClient, OpenBaoConfig, valid_path};
use foundation::{Sensitive, Uuid};
use http::Method;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fmt;
use std::time::Duration;

/// Galat verifikasi persetujuan. Tidak membawa token, accessor, maupun isi respons.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApprovalError {
    /// OpenBao tidak dapat dipakai (koneksi, batas waktu, ditolak, respons tidak valid, konfigurasi).
    OpenBao(CryptoError),
    /// Permintaan belum disetujui.
    NotApproved,
    /// Permintaan yang disetujui bukan untuk path yang diharapkan.
    WrongPath,
    /// Token operator tidak terikat entitas, atau peminta bukan operator yang menjalankan perintah.
    RequesterMismatch,
    /// Tidak ada approver, atau approver sama dengan peminta (bukan orang kedua).
    NoSecondParty,
    /// Isi rahasia yang dibuka tidak memuat `purpose` yang diharapkan.
    WrongPurpose,
    /// Persetujuan lebih dari [`MAX_APPROVERS`] (tidak dapat dicatat seluruhnya; gagal-tertutup).
    TooManyApprovers,
}

impl fmt::Display for ApprovalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OpenBao(e) => write!(f, "OpenBao: {e}"),
            Self::NotApproved => f.write_str("permintaan Control Group belum disetujui"),
            Self::WrongPath => f.write_str("persetujuan bukan untuk path yang diharapkan"),
            Self::RequesterMismatch => {
                f.write_str("peminta persetujuan bukan operator yang menjalankan perintah")
            }
            Self::NoSecondParty => f.write_str("tidak ada persetujuan dari orang kedua"),
            Self::WrongPurpose => f.write_str("rahasia yang disetujui bukan untuk perintah ini"),
            Self::TooManyApprovers => f.write_str("jumlah persetujuan melebihi batas yang dicatat"),
        }
    }
}

impl std::error::Error for ApprovalError {}

impl From<CryptoError> for ApprovalError {
    fn from(e: CryptoError) -> Self {
        Self::OpenBao(e)
    }
}

/// Masukan verifikasi.
pub struct ControlGroupRequest {
    /// Alamat OpenBao (`http://<ip-loopback>:<port>`, batasan sama dengan [`OpenBaoConfig`]).
    pub address: String,
    /// Token operator yang menjalankan perintah (login OpenBao operator, bukan token proses).
    pub operator_token: Sensitive<String>,
    /// `accessor` token pembungkus Control Group.
    pub accessor: String,
    /// Token pembungkus Control Group (sekali pakai).
    pub wrapping_token: Sensitive<String>,
    /// Mount KV v2 yang dilindungi, misalnya `rs-admin`.
    pub kv_mount: String,
    /// Path rahasia di mount itu, misalnya `bootstrap/super-admin`.
    pub kv_path: String,
    /// Nilai field `purpose` yang wajib ada di rahasia.
    pub purpose: String,
    /// Batas waktu per permintaan.
    pub timeout: Duration,
}

impl fmt::Debug for ControlGroupRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ControlGroupRequest")
            .field("address", &self.address)
            .field("kv_mount", &self.kv_mount)
            .field("kv_path", &self.kv_path)
            .finish_non_exhaustive()
    }
}

/// Hasil persetujuan yang sudah diverifikasi dan dipakai (token pembungkus sudah dibuka).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedApproval {
    /// Entitas OpenBao operator (peminta).
    pub operator_entity_id: Uuid,
    /// Entitas OpenBao semua penyetuju, urut sesuai jawaban OpenBao (1..=[`MAX_APPROVERS`]). OpenBao hanya
    /// mencatat anggota grup factor (panggilan `authorize` oleh pemegang hak di luar grup tidak menambah
    /// entri, diamati SRC-671). Policy production dapat meminta lebih dari satu persetujuan, jadi semuanya
    /// dikembalikan untuk dicatat.
    pub approver_entity_ids: Vec<Uuid>,
}

/// Batas jumlah persetujuan yang diterima; lebih dari ini ditolak ([`ApprovalError::TooManyApprovers`]).
pub const MAX_APPROVERS: usize = 3;

#[derive(Deserialize)]
struct LookupSelf {
    data: LookupSelfData,
}

#[derive(Deserialize)]
struct LookupSelfData {
    entity_id: Option<String>,
}

#[derive(Deserialize)]
struct RequestStatus {
    data: RequestStatusData,
}

#[derive(Deserialize)]
struct RequestStatusData {
    approved: bool,
    request_path: Option<String>,
    request_entity: Option<Entity>,
    authorizations: Option<Vec<Authorization>>,
}

/// OpenBao 2.7.2 menjawab `request_entity.ID` (huruf besar, diamati SRC-671), dokumentasi API menulis `id`
/// (SRC-670): keduanya diterima.
#[derive(Deserialize)]
struct Entity {
    #[serde(alias = "ID")]
    id: Option<String>,
}

#[derive(Deserialize)]
struct Authorization {
    entity_id: Option<String>,
}

#[derive(Deserialize)]
struct Unwrapped {
    data: UnwrappedKv,
}

#[derive(Deserialize)]
struct UnwrappedKv {
    data: BTreeMap<String, serde_json::Value>,
}

fn parse_entity(id: Option<&str>) -> Option<Uuid> {
    id.filter(|s| !s.is_empty())
        .and_then(|s| Uuid::parse_str(s).ok())
}

/// Accessor token OpenBao: alfanumerik, `-`, `_`, `.`; 1–128 karakter.
fn valid_accessor(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 128
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
}

/// Memverifikasi persetujuan Control Group lalu membuka token pembungkusnya (lihat dokumentasi modul).
pub async fn verify_control_group_approval(
    req: &ControlGroupRequest,
) -> Result<VerifiedApproval, ApprovalError> {
    if !valid_path(&req.kv_mount, false)
        || !valid_path(&req.kv_path, true)
        || !valid_accessor(&req.accessor)
        || req.purpose.is_empty()
    {
        return Err(CryptoError::InvalidConfig.into());
    }
    let operator = OpenBaoClient::new(&OpenBaoConfig {
        address: req.address.clone(),
        token: req.operator_token.clone(),
        timeout: req.timeout,
    })?;
    let raw = operator
        .call(Method::GET, "/v1/auth/token/lookup-self", None)
        .await?;
    let me: LookupSelf =
        serde_json::from_slice(&raw).map_err(|_| CryptoError::KmsInvalidResponse)?;
    let operator_entity =
        parse_entity(me.data.entity_id.as_deref()).ok_or(ApprovalError::RequesterMismatch)?;

    let body = serde_json::to_vec(&serde_json::json!({ "accessor": req.accessor }))
        .map_err(|_| CryptoError::InvalidConfig)?;
    let raw = operator
        .call(Method::POST, "/v1/sys/control-group/request", Some(body))
        .await?;
    let status: RequestStatus =
        serde_json::from_slice(&raw).map_err(|_| CryptoError::KmsInvalidResponse)?;
    let s = status.data;
    if !s.approved {
        return Err(ApprovalError::NotApproved);
    }
    let expected_path = format!("{}/data/{}", req.kv_mount, req.kv_path);
    if s.request_path.as_deref().map(|p| p.trim_start_matches('/')) != Some(expected_path.as_str())
    {
        return Err(ApprovalError::WrongPath);
    }
    let requester = parse_entity(s.request_entity.as_ref().and_then(|e| e.id.as_deref()))
        .ok_or(ApprovalError::RequesterMismatch)?;
    if requester != operator_entity {
        return Err(ApprovalError::RequesterMismatch);
    }
    let mut approvers = Vec::new();
    for a in s.authorizations.unwrap_or_default() {
        let id = parse_entity(a.entity_id.as_deref()).ok_or(ApprovalError::NoSecondParty)?;
        if id == requester {
            return Err(ApprovalError::NoSecondParty);
        }
        approvers.push(id);
    }
    if approvers.is_empty() {
        return Err(ApprovalError::NoSecondParty);
    }
    if approvers.len() > MAX_APPROVERS {
        return Err(ApprovalError::TooManyApprovers);
    }

    // Membuka token pembungkus: permintaan baca asli baru dijalankan sekarang, sekali saja.
    let wrapper = OpenBaoClient::new(&OpenBaoConfig {
        address: req.address.clone(),
        token: req.wrapping_token.clone(),
        timeout: req.timeout,
    })?;
    let raw = wrapper
        .call(Method::POST, "/v1/sys/wrapping/unwrap", None)
        .await?;
    // Token pembungkus Control Group TIDAK sekali pakai di OpenBao 2.7.2: unwrap kedua dengan token yang
    // sama menjalankan ulang permintaan asli selama TTL (diamati, SRC-671). Token dicabut segera supaya
    // satu persetujuan = satu eksekusi; pencabutan gagal = perintah gagal sebelum mengubah apa pun.
    wrapper
        .call(Method::POST, "/v1/auth/token/revoke-self", None)
        .await?;
    let unwrapped: Unwrapped =
        serde_json::from_slice(&raw).map_err(|_| CryptoError::KmsInvalidResponse)?;
    match unwrapped.data.data.get("purpose") {
        Some(serde_json::Value::String(p)) if *p == req.purpose => {}
        _ => return Err(ApprovalError::WrongPurpose),
    }
    Ok(VerifiedApproval {
        operator_entity_id: operator_entity,
        approver_entity_ids: approvers,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn req() -> ControlGroupRequest {
        ControlGroupRequest {
            address: "http://127.0.0.1:1".into(),
            operator_token: Sensitive::new("token-operator-uji".into()),
            accessor: "abc-123".into(),
            wrapping_token: Sensitive::new("token-pembungkus-uji".into()),
            kv_mount: "rs-admin".into(),
            kv_path: "bootstrap/super-admin".into(),
            purpose: "rs_super_admin_bootstrap".into(),
            timeout: Duration::from_secs(1),
        }
    }

    #[test]
    fn debug_hides_tokens_and_accessor() {
        let out = format!("{:?}", req());
        assert!(!out.contains("token-operator-uji"));
        assert!(!out.contains("token-pembungkus-uji"));
        assert!(!out.contains("abc-123"));
    }

    #[tokio::test]
    async fn invalid_inputs_are_rejected_before_any_request() {
        type Patch = fn(&mut ControlGroupRequest);
        let cases: [Patch; 6] = [
            |r| r.kv_mount = "a/b".into(),
            |r| r.kv_path = "../x".into(),
            |r| r.accessor = String::new(),
            |r| r.accessor = "a b".into(),
            |r| r.accessor = "x".repeat(129),
            |r| r.purpose = String::new(),
        ];
        for patch in cases {
            let mut r = req();
            patch(&mut r);
            assert_eq!(
                verify_control_group_approval(&r).await.err(),
                Some(ApprovalError::OpenBao(CryptoError::InvalidConfig))
            );
        }
    }

    #[test]
    fn entity_ids_must_be_uuids() {
        assert!(parse_entity(Some("c8b6e404-de4b-50a4-2917-715ff8beec8e")).is_some());
        assert!(parse_entity(Some("")).is_none());
        assert!(parse_entity(Some("Bob")).is_none());
        assert!(parse_entity(None).is_none());
    }
}
