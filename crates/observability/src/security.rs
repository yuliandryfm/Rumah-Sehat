//! Security log (ADR-010, KEPUTUSAN-124, coding-standards §4.3).
//!
//! Kejadian dipancarkan lewat satu fungsi per jenis. Parameter hanya ID, pola rute, dan alasan
//! dari daftar tertutup (`&'static str`), tidak ada teks bebas, sehingga isinya tunduk pada daftar tolak
//! yang sama dengan application log. Layer ([`crate::log`]) merutekan target [`SECURITY_TARGET`] ke
//! sink security dengan `log_stream = "security"`; field `log_stream` dari kode lain dibuang.
//!
//! Jenis yang belum punya fungsi (hasil verifikasi hash berantai audit, percobaan ubah/hapus audit,
//! penguncian akun, kegagalan MFA, log audit penyedia cloud) ditambahkan oleh task pemilik fiturnya
//! dengan pola yang sama.

use foundation::Uuid;
use tracing::warn;

/// Target tracing khusus security log. Dipakai layer untuk memilih aliran.
pub const SECURITY_TARGET: &str = "ruang_sehat::security";

/// Alasan login gagal (daftar tertutup).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoginFailure {
    /// Kredensial salah.
    BadCredentials,
    /// Akun terkunci.
    AccountLocked,
    /// Kode MFA salah.
    MfaFailed,
}

impl LoginFailure {
    fn as_str(self) -> &'static str {
        match self {
            Self::BadCredentials => "bad_credentials",
            Self::AccountLocked => "account_locked",
            Self::MfaFailed => "mfa_failed",
        }
    }
}

/// Alasan akses ditolak (daftar tertutup).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessDenial {
    /// Peran tidak berhak.
    RoleNotAllowed,
    /// Percobaan mengganti ID ke milik pihak lain.
    ForeignResourceId,
}

impl AccessDenial {
    fn as_str(self) -> &'static str {
        match self {
            Self::RoleNotAllowed => "role_not_allowed",
            Self::ForeignResourceId => "foreign_resource_id",
        }
    }
}

fn opt(id: Option<Uuid>) -> String {
    id.map(|u| u.hyphenated().to_string()).unwrap_or_default()
}

/// Login gagal. `account_id` kosong bila akun tidak dikenal.
pub fn login_failed(account_id: Option<Uuid>, reason: LoginFailure) {
    warn!(target: SECURITY_TARGET, event_type = "login_failed", account_id = %opt(account_id), reason = reason.as_str(), "security_event");
}

/// Akses ditolak, termasuk percobaan ganti ID. `route` harus pola rute, bukan URL berisi ID.
pub fn access_denied(account_id: Option<Uuid>, route: &str, reason: AccessDenial) {
    warn!(target: SECURITY_TARGET, event_type = "access_denied", account_id = %opt(account_id), route = route, reason = reason.as_str(), "security_event");
}

/// Batas laju terlampaui.
pub fn rate_limited(account_id: Option<Uuid>, route: &str) {
    warn!(target: SECURITY_TARGET, event_type = "rate_limited", account_id = %opt(account_id), route = route, "security_event");
}

/// Tanda tangan webhook rekanan tidak valid.
pub fn webhook_signature_invalid(partner_id: Option<Uuid>, route: &str) {
    warn!(target: SECURITY_TARGET, event_type = "webhook_signature_invalid", partner_id = %opt(partner_id), route = route, "security_event");
}

/// Kredensial klien rekanan tidak valid.
pub fn partner_credential_invalid(partner_id: Option<Uuid>, route: &str) {
    warn!(target: SECURITY_TARGET, event_type = "partner_credential_invalid", partner_id = %opt(partner_id), route = route, "security_event");
}

/// Relay audit terlambat atau gagal.
pub fn audit_relay_delayed(lag_seconds: u64) {
    warn!(target: SECURITY_TARGET, event_type = "audit_relay_delayed", lag_seconds = lag_seconds, "security_event");
}

/// Tahap kegagalan penulisan event audit (daftar tertutup).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditWriteFailure {
    /// Event ditolak pustaka audit (field wajib, daftar izin, masking).
    InvalidEvent,
    /// Penulisan ke `audit_outbox` gagal di database.
    StoreError,
    /// Aksi bisnis mengakhiri transaksi teraudit (SQL kendali transaksi mentah).
    TransactionEscaped,
}

impl AuditWriteFailure {
    fn as_str(self) -> &'static str {
        match self {
            Self::InvalidEvent => "invalid_event",
            Self::StoreError => "store_error",
            Self::TransactionEscaped => "transaction_escaped",
        }
    }
}

/// Penulisan event audit gagal; aksi pemanggil dibatalkan (ADR-009 butir 2, klarifikasi T-21 US-1501 AC3).
/// Hanya kode event dan tahap kegagalan; isi event dan pesan galat database tidak dicatat.
pub fn audit_write_failed(event_code: &'static str, stage: AuditWriteFailure) {
    warn!(target: SECURITY_TARGET, event_type = "audit_write_failed", event_code = event_code, stage = stage.as_str(), "security_event");
}

/// Proyeksi status tidak cocok dengan ledger status (ADR-008).
pub fn status_projection_mismatch(entity: &'static str, entity_id: Uuid) {
    warn!(target: SECURITY_TARGET, event_type = "status_projection_mismatch", entity = entity, entity_id = %entity_id.hyphenated(), "security_event");
}

/// Hak akses seseorang berubah.
pub fn access_rights_changed(actor_id: Uuid, subject_id: Uuid) {
    warn!(target: SECURITY_TARGET, event_type = "access_rights_changed", actor_id = %actor_id.hyphenated(), subject_id = %subject_id.hyphenated(), "security_event");
}
