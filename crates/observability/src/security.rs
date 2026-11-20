//! Security log (ADR-010, KEPUTUSAN-124, coding-standards §4.3).
//!
//! Kejadian dipancarkan lewat satu fungsi per jenis. Parameter hanya ID, pola rute, dan alasan
//! dari daftar tertutup (`&'static str`), tidak ada teks bebas, sehingga isinya tunduk pada daftar tolak
//! yang sama dengan application log. Layer ([`crate::log`]) merutekan target [`SECURITY_TARGET`] ke
//! sink security dengan `log_stream = "security"`; field `log_stream` dari kode lain dibuang.
//!
//! Jenis yang belum punya fungsi (percobaan ubah/hapus audit, penguncian akun, kegagalan MFA, log audit
//! penyedia cloud) ditambahkan oleh task pemilik fiturnya dengan pola yang sama. Relay dan verifikasi
//! rantai audit: F1-T24/F1-T25. Event Keycloak yang diambil worker dan kegagalan/keterlambatan
//! pengambilannya: F1-T34.

use foundation::Uuid;
use tracing::{info, warn};

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

/// Tahap kegagalan relay audit (daftar tertutup; F1-T24).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditRelayFailure {
    /// Membaca atau menghapus `audit_outbox` gagal.
    Outbox,
    /// Payload outbox tidak dapat dipetakan ke kolom `audit_event` (relay berhenti di event ini).
    InvalidPayload,
    /// `seq` sudah dipakai: relay lain menulis rantai bersamaan; batch dibatalkan (KEPUTUSAN-284).
    SeqConflict,
    /// Penulisan ke database audit gagal; batch dibatalkan, event tetap di outbox.
    AuditStore,
    /// Koneksi atau kunci relay hilang.
    Connection,
    /// Jangkar kepala rantai tidak dapat ditulis (F1-T25).
    Anchor,
}

impl AuditRelayFailure {
    fn as_str(self) -> &'static str {
        match self {
            Self::Outbox => "outbox",
            Self::InvalidPayload => "invalid_payload",
            Self::SeqConflict => "seq_conflict",
            Self::AuditStore => "audit_store",
            Self::Connection => "connection",
            Self::Anchor => "anchor",
        }
    }
}

/// Relay audit gagal memproses batch; event tetap di outbox dan dicoba ulang (ADR-009 butir 3).
/// `event_id` hanya untuk payload tidak valid (ID event, bukan data pribadi).
pub fn audit_relay_failed(stage: AuditRelayFailure, event_id: Option<Uuid>) {
    warn!(target: SECURITY_TARGET, event_type = "audit_relay_failed", stage = stage.as_str(), event_id = %opt(event_id), "security_event");
}

/// Kepala rantai audit dijangkarkan (ADR-009 butir 5: "ke WORM dan ke security log"). `event_hash_hex`
/// = SHA-256 kepala rantai dalam heksadesimal (bukan data pribadi).
pub fn audit_chain_anchored(seq: i64, event_hash_hex: &str) {
    info!(target: SECURITY_TARGET, event_type = "audit_chain_anchored", seq = seq, event_hash = event_hash_hex, "security_event");
}

/// Alasan ketidakcocokan rantai audit (daftar tertutup; F1-T25, klarifikasi T-22).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditChainMismatch {
    /// `seq` yang diharapkan tidak ada (event dihapus dari tengah rantai).
    SeqGap,
    /// `seq` muncul lebih dari sekali (event disisipkan).
    SeqDuplicate,
    /// `prev_hash` tidak sama dengan hash event sebelumnya.
    PrevHash,
    /// `event_hash` tidak sama dengan hasil hitung ulang (isi event diubah).
    EventHash,
    /// `canonical_version` tidak dikenal atau bentuk kanonis tidak dapat dibentuk.
    CanonicalVersion,
    /// Hasil hitung ulang di `seq` terjangkar berbeda dengan jangkar (rantai ditulis ulang).
    AnchorHash,
    /// Jangkar menunjuk `seq` di atas ujung rantai (ujung rantai dipotong).
    AnchorBeyondChain,
    /// `audit_event_key` memuat `seq` di atas ujung rantai (event ujung dihapus).
    KeyBeyondChain,
    /// Baris `audit_event_subject` milik event ini dengan `recorded_at` berbeda (subjek dipindah/disisipkan).
    SubjectRecordedAt,
    /// Baris `audit_event_subject` tanpa event (yatim).
    SubjectOrphan,
}

impl AuditChainMismatch {
    fn as_str(self) -> &'static str {
        match self {
            Self::SeqGap => "seq_gap",
            Self::SeqDuplicate => "seq_duplicate",
            Self::PrevHash => "prev_hash",
            Self::EventHash => "event_hash",
            Self::CanonicalVersion => "canonical_version",
            Self::AnchorHash => "anchor_hash",
            Self::AnchorBeyondChain => "anchor_beyond_chain",
            Self::KeyBeyondChain => "key_beyond_chain",
            Self::SubjectRecordedAt => "subject_recorded_at",
            Self::SubjectOrphan => "subject_orphan",
        }
    }
}

/// Verifikasi rantai audit menemukan ketidakcocokan (ADR-009 butir 5, US-1504 AC3).
pub fn audit_chain_mismatch(first_mismatch_seq: i64, reason: AuditChainMismatch) {
    warn!(target: SECURITY_TARGET, event_type = "audit_chain_mismatch", first_mismatch_seq = first_mismatch_seq, reason = reason.as_str(), "security_event");
}

/// Tahap kegagalan verifikasi rantai (daftar tertutup).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditVerifyFailure {
    /// Jangkar WORM tidak dapat dibaca.
    AnchorStore,
    /// Database audit tidak dapat dibaca (koneksi, hak, kunci).
    Database,
    /// Hasil verifikasi tidak dapat ditulis ke `audit_chain_verification`.
    RecordResult,
    /// Katalog partisi tidak dapat dibaca.
    PartitionCheck,
}

impl AuditVerifyFailure {
    fn as_str(self) -> &'static str {
        match self {
            Self::AnchorStore => "anchor_store",
            Self::Database => "database",
            Self::RecordResult => "record_result",
            Self::PartitionCheck => "partition_check",
        }
    }
}

/// Verifikasi rantai audit tidak dapat diselesaikan.
pub fn audit_chain_verification_failed(stage: AuditVerifyFailure) {
    warn!(target: SECURITY_TARGET, event_type = "audit_chain_verification_failed", stage = stage.as_str(), "security_event");
}

/// Partisi `audit_event` masa depan kurang dari 6 bulan, atau bulan berjalan tidak tercakup (skema fisik
/// §6.2 butir 2, TD-12). Tanpa partisi, relay berhenti dan event tertahan di outbox.
pub fn audit_partition_runway_low(future_partitions: u32, current_covered: bool) {
    warn!(target: SECURITY_TARGET, event_type = "audit_partition_runway_low", future_partitions = future_partitions, current_covered = current_covered, "security_event");
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

/// Relay email membuang pesan karena batas laju per penerima tercapai (ADR-026 butir 4; review keamanan F1-T36):
/// dasar peringatan atas kemungkinan penahanan email korban. Hanya ID korelasi relay; tanpa alamat.
pub fn email_recipient_rate_dropped(correlation_id: Uuid) {
    warn!(target: SECURITY_TARGET, event_type = "email_recipient_rate_dropped", relay_correlation_id = %correlation_id.hyphenated(), "security_event");
}

/// Penulisan event audit gagal; aksi pemanggil dibatalkan (ADR-009 butir 2, klarifikasi T-21 US-1501 AC3).
/// Hanya kode event dan tahap kegagalan; isi event dan pesan galat database tidak dicatat.
pub fn audit_write_failed(event_code: &'static str, stage: AuditWriteFailure) {
    warn!(target: SECURITY_TARGET, event_type = "audit_write_failed", event_code = event_code, stage = stage.as_str(), "security_event");
}

/// Event autentikasi Keycloak (E-01–E-04) yang sudah ditulis ke `audit_outbox`, diteruskan juga ke
/// security log (ADR-011 butir 10; F1-T34). Semua parameter dari daftar tertutup pemetaan; email,
/// username, alamat IP, dan token tidak pernah masuk ke sini. `failure_reason` = kode alasan bila gagal.
pub fn idp_event(
    realm: &'static str,
    event_code: &'static str,
    action: &'static str,
    account_id: Option<Uuid>,
    failure_reason: Option<&'static str>,
) {
    match failure_reason {
        None => {
            info!(target: SECURITY_TARGET, event_type = "idp_event", realm = realm, event_code = event_code, action = action, account_id = %opt(account_id), outcome = "success", "security_event");
        }
        Some(reason) => {
            warn!(target: SECURITY_TARGET, event_type = "idp_event", realm = realm, event_code = event_code, action = action, account_id = %opt(account_id), outcome = "failure", reason = reason, "security_event");
        }
    }
}

/// Tahap kegagalan pengambil event Keycloak (daftar tertutup; F1-T34).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdpEventPullFailure {
    /// Admin REST Keycloak tidak tersedia, melewati batas waktu, atau menjawab status tak terduga.
    KeycloakUnavailable,
    /// Kredensial klien layanan pembaca event ditolak endpoint token.
    KeycloakAuth,
    /// Event dari Keycloak tidak valid; pengambilan realm itu berhenti di event ini (posisi tidak maju).
    InvalidEvent,
    /// Event ditolak pustaka audit atau `audit_outbox` gagal ditulis.
    AuditWrite,
    /// Database (posisi baca, tanda terima) atau koneksi kunci gagal.
    Database,
    /// Terlalu banyak event dalam satu milidetik untuk diselesaikan.
    TooManyEvents,
}

impl IdpEventPullFailure {
    fn as_str(self) -> &'static str {
        match self {
            Self::KeycloakUnavailable => "keycloak_unavailable",
            Self::KeycloakAuth => "keycloak_auth",
            Self::InvalidEvent => "invalid_event",
            Self::AuditWrite => "audit_write",
            Self::Database => "database",
            Self::TooManyEvents => "too_many_events",
        }
    }
}

/// Pengambilan event Keycloak satu realm gagal; posisi baca tidak maju dan dicoba lagi putaran berikutnya.
/// `realm` = nama realm, atau `-` untuk kegagalan sesi pengambil.
pub fn idp_event_pull_failed(realm: &'static str, stage: IdpEventPullFailure) {
    warn!(target: SECURITY_TARGET, event_type = "idp_event_pull_failed", realm = realm, stage = stage.as_str(), "security_event");
}

/// Pengambilan event Keycloak satu realm terlambat melewati ambang (ADR-011 butir 10: lebih dari 5 menit).
pub fn idp_event_pull_delayed(realm: &'static str, lag_seconds: u64) {
    warn!(target: SECURITY_TARGET, event_type = "idp_event_pull_delayed", realm = realm, lag_seconds = lag_seconds, "security_event");
}

/// Proyeksi status tidak cocok dengan ledger status (ADR-008).
pub fn status_projection_mismatch(entity: &'static str, entity_id: Uuid) {
    warn!(target: SECURITY_TARGET, event_type = "status_projection_mismatch", entity = entity, entity_id = %entity_id.hyphenated(), "security_event");
}

/// Pengajuan nomor identitas ditahan karena ambang penolakan tercapai (US-106 AC4: "setiap penahanan
/// tercatat sebagai kejadian keamanan"; F1-T60). `scope` dari daftar tertutup: `account`,
/// `identity_number`, `ip`. Nomor identitas dan alamat IP tidak dicatat.
pub fn identity_submission_held(account_id: Option<Uuid>, scope: &'static str) {
    warn!(target: SECURITY_TARGET, event_type = "identity_submission_held", account_id = %opt(account_id), scope = scope, "security_event");
}

/// Penolakan pengajuan nomor identitas di seluruh sistem melewati ambang per jam (US-106 AC4: lebih dari 20
/// per jam memicu peringatan; F1-T60). Dipancarkan sekali per jam saat ambang terlewati.
pub fn identity_rejection_surge(rejections_this_hour: u64) {
    warn!(target: SECURITY_TARGET, event_type = "identity_rejection_surge", rejections_this_hour = rejections_this_hour, "security_event");
}

/// Hak akses seseorang berubah.
pub fn access_rights_changed(actor_id: Uuid, subject_id: Uuid) {
    warn!(target: SECURITY_TARGET, event_type = "access_rights_changed", actor_id = %actor_id.hyphenated(), subject_id = %subject_id.hyphenated(), "security_event");
}
