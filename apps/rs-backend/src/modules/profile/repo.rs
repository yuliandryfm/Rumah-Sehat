//! Query sqlx atas tabel milik modul `profile` (DD A2): `patient` (A2.1, M), `care_recipient` (A2.2, M),
//! `identity_document` (A2.4, M), `identity_verification_request` (A2.5, T) beserta riwayat (A2.6, R) dan
//! proyeksinya (A2.7, P), dan `account_owner_nik_registry` (A2.8, M). Query Master memfilter baris
//! terhapus (ADR-003). Tabel T/R hanya `INSERT` (ADR-004, ADR-008); status lewat [`IvrStatusStore`] yang
//! dijalankan pustaka `status-flow` (ADR-008 butir 2-3).

use super::domain::{DocumentType, TriggerType, VerificationStatus};
use foundation::{CorrelationId, Uuid};
use sqlx::{PgConnection, PgPool};
use status_flow::{Appended, Current, StatusStore, Step};
use time::OffsetDateTime;

/// Galat repository. Pesan galat database tidak dibawa (dapat memuat isi baris di DETAIL, TD-03);
/// pelanggaran constraint unik yang bermakna bisnis dipisahkan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum RepoError {
    /// NIK sama dengan NIK penerima layanan lain di akun yang sama (`uqp_care_recipient_patient_id_nik_bidx`,
    /// US-108 AC4-AC5).
    DuplicateNikInAccount,
    /// NIK sudah tercatat di registri NIK pemilik akun (`uqp_account_owner_nik_registry_nik_bidx`, UP-40).
    RegistryConflict,
    /// Data tersimpan tidak sesuai kode yang dikenal (CHECK seharusnya mencegah).
    Corrupt,
    /// Galat database lain.
    Database,
}

fn map_db(e: sqlx::Error) -> RepoError {
    match e
        .as_database_error()
        .and_then(|d| d.constraint().map(str::to_owned))
        .as_deref()
    {
        Some("uqp_care_recipient_patient_id_nik_bidx") => RepoError::DuplicateNikInAccount,
        Some("uqp_account_owner_nik_registry_nik_bidx") => RepoError::RegistryConflict,
        _ => RepoError::Database,
    }
}

/// Pemilik satu penerima layanan.
pub(super) struct OwnerRow {
    pub(super) account_id: Uuid,
    pub(super) is_account_owner: bool,
}

pub(super) async fn care_recipient_owner(
    pool: &PgPool,
    care_recipient_id: Uuid,
) -> Result<Option<OwnerRow>, sqlx::Error> {
    sqlx::query_as!(
        OwnerRow,
        "SELECT p.account_id, cr.is_account_owner \
         FROM app.care_recipient cr \
         JOIN app.patient p ON p.id = cr.patient_id \
         WHERE cr.id = $1 AND cr.deleted_at IS NULL AND p.deleted_at IS NULL",
        care_recipient_id,
    )
    .fetch_optional(pool)
    .await
}

/// Penerima layanan yang dikunci untuk satu use case identitas (`FOR UPDATE`): pengajuan, pembatalan, dan
/// keputusan atas penerima layanan yang sama berjalan berurutan.
pub(super) struct LockedRecipient {
    pub(super) latest_verification_request_id: Option<Uuid>,
}

pub(super) async fn lock_care_recipient(
    conn: &mut PgConnection,
    care_recipient_id: Uuid,
) -> Result<Option<LockedRecipient>, RepoError> {
    sqlx::query_as!(
        LockedRecipient,
        "SELECT latest_verification_request_id \
         FROM app.care_recipient WHERE id = $1 AND deleted_at IS NULL FOR UPDATE",
        care_recipient_id,
    )
    .fetch_optional(conn)
    .await
    .map_err(map_db)
}

/// Permintaan verifikasi beserta status proyeksinya.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RequestState {
    pub(super) id: Uuid,
    pub(super) trigger: TriggerType,
    pub(super) status: VerificationStatus,
}

struct RequestStateRaw {
    id: Uuid,
    trigger_type: String,
    status: String,
}

impl TryFrom<RequestStateRaw> for RequestState {
    type Error = RepoError;
    fn try_from(r: RequestStateRaw) -> Result<Self, RepoError> {
        Ok(Self {
            id: r.id,
            trigger: TriggerType::parse(&r.trigger_type).ok_or(RepoError::Corrupt)?,
            status: VerificationStatus::parse(&r.status).ok_or(RepoError::Corrupt)?,
        })
    }
}

pub(super) async fn request_state(
    conn: &mut PgConnection,
    request_id: Uuid,
) -> Result<Option<RequestState>, RepoError> {
    let raw = sqlx::query_as!(
        RequestStateRaw,
        "SELECT r.id, r.trigger_type, s.status \
         FROM app.identity_verification_request r \
         JOIN app.identity_verification_request_state s ON s.identity_verification_request_id = r.id \
         WHERE r.id = $1",
        request_id,
    )
    .fetch_optional(conn)
    .await
    .map_err(map_db)?;
    raw.map(RequestState::try_from).transpose()
}

/// NIK (blind index) yang tercatat di dokumen permintaan efektif, untuk mengembalikan
/// `care_recipient.nik_*` saat pengajuan dibatalkan.
pub(super) struct PreviousEffective {
    pub(super) request_id: Uuid,
    pub(super) nik_bidx: Vec<u8>,
    pub(super) nik_bidx_key_version: i16,
    pub(super) nik_last4: String,
}

/// Permintaan efektif sebelumnya: permintaan terbaru penerima layanan selain `excluding` yang tidak
/// dibatalkan (US-106 AC5, AB-05a b; temuan S-04).
pub(super) async fn previous_effective_request(
    conn: &mut PgConnection,
    care_recipient_id: Uuid,
    excluding: Uuid,
) -> Result<Option<PreviousEffective>, RepoError> {
    sqlx::query_as!(
        PreviousEffective,
        "SELECT r.id AS request_id, d.nik_bidx, d.nik_bidx_key_version, d.nik_last4 \
         FROM app.identity_verification_request r \
         JOIN app.identity_verification_request_state s ON s.identity_verification_request_id = r.id \
         JOIN app.identity_document d ON d.id = r.identity_document_id \
         WHERE r.care_recipient_id = $1 AND r.id <> $2 AND s.status <> 'cancelled' \
         ORDER BY r.created_at DESC, r.id DESC LIMIT 1",
        care_recipient_id,
        excluding,
    )
    .fetch_optional(conn)
    .await
    .map_err(map_db)
}

/// Penerima layanan pemegang baris registri NIK pemilik akun yang aktif dan cocok dengan salah satu
/// pasangan (`nik_bidx`, `nik_bidx_key_version`) kandidat (semua versi kunci aktif, TD-10).
pub(super) async fn registry_holder(
    conn: &mut PgConnection,
    candidates: &[(Vec<u8>, i16)],
) -> Result<Option<Uuid>, RepoError> {
    let (values, versions): (Vec<Vec<u8>>, Vec<i16>) = candidates.iter().cloned().unzip();
    sqlx::query_scalar!(
        "SELECT care_recipient_id FROM app.account_owner_nik_registry \
         WHERE deleted_at IS NULL \
           AND (nik_bidx, nik_bidx_key_version) IN \
               (SELECT v, k FROM UNNEST($1::bytea[], $2::smallint[]) AS c(v, k)) \
         ORDER BY created_at LIMIT 1",
        &values,
        &versions,
    )
    .fetch_optional(conn)
    .await
    .map_err(map_db)
}

/// Baris `identity_document` baru.
pub(super) struct NewIdentityDocument<'a> {
    pub(super) id: Uuid,
    pub(super) created_by: Uuid,
    pub(super) care_recipient_id: Uuid,
    pub(super) document_type: DocumentType,
    pub(super) nik_enc: &'a [u8],
    pub(super) nik_bidx: &'a [u8],
    pub(super) nik_bidx_key_version: i16,
    pub(super) nik_last4: &'a str,
    pub(super) data_key_id: Uuid,
    pub(super) file_id: Uuid,
}

pub(super) async fn insert_identity_document(
    conn: &mut PgConnection,
    d: &NewIdentityDocument<'_>,
) -> Result<(), RepoError> {
    sqlx::query!(
        "INSERT INTO app.identity_document (id, created_by, updated_by, care_recipient_id, document_type, \
                                            nik_enc, nik_bidx, nik_bidx_key_version, nik_last4, \
                                            data_key_id, file_id) \
         VALUES ($1, $2, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
        d.id,
        d.created_by,
        d.care_recipient_id,
        d.document_type.as_str(),
        d.nik_enc,
        d.nik_bidx,
        d.nik_bidx_key_version,
        d.nik_last4,
        d.data_key_id,
        d.file_id,
    )
    .execute(conn)
    .await
    .map_err(map_db)?;
    Ok(())
}

pub(super) async fn insert_request(
    conn: &mut PgConnection,
    id: Uuid,
    created_by: Uuid,
    care_recipient_id: Uuid,
    identity_document_id: Uuid,
    trigger: TriggerType,
) -> Result<(), RepoError> {
    sqlx::query!(
        "INSERT INTO app.identity_verification_request \
             (id, created_by, care_recipient_id, identity_document_id, trigger_type) \
         VALUES ($1, $2, $3, $4, $5)",
        id,
        created_by,
        care_recipient_id,
        identity_document_id,
        trigger.as_str(),
    )
    .execute(conn)
    .await
    .map_err(map_db)?;
    Ok(())
}

/// NIK yang terakhir diajukan di `care_recipient` (DD A2.2): blind index versi kunci terbaru + 4 digit.
pub(super) struct RecipientNik<'a> {
    pub(super) bidx: &'a [u8],
    pub(super) key_version: i16,
    pub(super) last4: &'a str,
}

/// Memindahkan penunjuk permintaan efektif dan NIK terakhir penerima layanan (`None` = "Belum diunggah").
pub(super) async fn set_effective_request(
    conn: &mut PgConnection,
    care_recipient_id: Uuid,
    request_id: Option<Uuid>,
    nik: Option<RecipientNik<'_>>,
    updated_by: Uuid,
) -> Result<(), RepoError> {
    let n = sqlx::query!(
        "UPDATE app.care_recipient \
         SET latest_verification_request_id = $2, nik_bidx = $3, nik_bidx_key_version = $4, \
             nik_last4 = $5, updated_at = now(), updated_by = $6 \
         WHERE id = $1 AND deleted_at IS NULL",
        care_recipient_id,
        request_id,
        nik.as_ref().map(|n| n.bidx),
        nik.as_ref().map(|n| n.key_version),
        nik.as_ref().map(|n| n.last4),
        updated_by,
    )
    .execute(conn)
    .await
    .map_err(map_db)?
    .rows_affected();
    if n == 1 {
        Ok(())
    } else {
        Err(RepoError::Database)
    }
}

/// Isi event riwayat status selain status (DD STD-R).
#[derive(Debug, Clone)]
pub(super) struct EventDetails {
    /// ID baris event (dipakai juga sebagai AAD keterangan terenkripsi, F1-T62).
    pub(super) event_id: Uuid,
    pub(super) actor_type: &'static str,
    pub(super) actor_account_id: Uuid,
    pub(super) reason_code: Option<&'static str>,
    pub(super) note_enc: Option<Vec<u8>>,
    pub(super) note_data_key_id: Option<Uuid>,
    pub(super) correlation_id: CorrelationId,
}

/// [`StatusStore`] satu permintaan verifikasi di transaksi pemanggil (jalur tulis ADR-008 butir 2).
pub(super) struct IvrStatusStore<'c> {
    pub(super) conn: &'c mut PgConnection,
    pub(super) request_id: Uuid,
    pub(super) details: EventDetails,
}

impl StatusStore<VerificationStatus> for IvrStatusStore<'_> {
    type Error = RepoError;

    async fn current(&mut self) -> Result<Option<Current<VerificationStatus>>, RepoError> {
        let row = sqlx::query!(
            "SELECT status, version FROM app.identity_verification_request_state \
             WHERE identity_verification_request_id = $1",
            self.request_id,
        )
        .fetch_optional(&mut *self.conn)
        .await
        .map_err(map_db)?;
        row.map(|r| {
            Ok(Current {
                status: VerificationStatus::parse(&r.status).ok_or(RepoError::Corrupt)?,
                version: r.version,
            })
        })
        .transpose()
    }

    async fn append(&mut self, step: Step<VerificationStatus>) -> Result<Appended, RepoError> {
        // AB-36: versi yang sudah dipakai transaksi lain tidak disisipkan (tanpa galat yang membatalkan
        // transaksi); pustaka status-flow lalu mengevaluasi ulang dari status baru.
        let n = sqlx::query!(
            "INSERT INTO app.identity_verification_request_status_event \
                 (id, identity_verification_request_id, version, from_status, to_status, reason_code, \
                  note_enc, note_data_key_id, actor_type, actor_account_id, correlation_id) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) \
             ON CONFLICT ON CONSTRAINT uq_ivr_status_event_identity_verification_request_id_version \
             DO NOTHING",
            self.details.event_id,
            self.request_id,
            step.version,
            step.from.map(VerificationStatus::as_str),
            step.to.as_str(),
            self.details.reason_code,
            self.details.note_enc.as_deref(),
            self.details.note_data_key_id,
            self.details.actor_type,
            self.details.actor_account_id,
            self.details.correlation_id.as_uuid(),
        )
        .execute(&mut *self.conn)
        .await
        .map_err(map_db)?
        .rows_affected();
        Ok(if n == 1 {
            Appended::Inserted
        } else {
            Appended::Lost
        })
    }

    async fn project(&mut self, step: Step<VerificationStatus>) -> Result<bool, RepoError> {
        let n = if step.version == 1 {
            sqlx::query!(
                "INSERT INTO app.identity_verification_request_state \
                     (identity_verification_request_id, status, version) \
                 VALUES ($1, $2, 1)",
                self.request_id,
                step.to.as_str(),
            )
            .execute(&mut *self.conn)
            .await
        } else {
            sqlx::query!(
                "UPDATE app.identity_verification_request_state \
                 SET status = $2, version = $3, updated_at = now() \
                 WHERE identity_verification_request_id = $1 AND version = $3 - 1",
                self.request_id,
                step.to.as_str(),
                step.version,
            )
            .execute(&mut *self.conn)
            .await
        }
        .map_err(map_db)?
        .rows_affected();
        Ok(n == 1)
    }
}

/// Status identitas yang tampil ke pemilik (AB-05): permintaan efektif dan alasan keputusan terakhirnya.
pub(super) struct StatusRow {
    pub(super) request_id: Option<Uuid>,
    pub(super) trigger_type: Option<String>,
    pub(super) status: Option<String>,
    pub(super) reason_code: Option<String>,
    pub(super) submitted_at: Option<OffsetDateTime>,
    pub(super) nik_last4: Option<String>,
    /// Event status terakhir beserta keterangan terenkripsinya (US-201 AC3a).
    pub(super) event_id: Option<Uuid>,
    pub(super) note_enc: Option<Vec<u8>>,
    pub(super) note_data_key_id: Option<Uuid>,
}

pub(super) async fn identity_status(
    pool: &PgPool,
    care_recipient_id: Uuid,
) -> Result<Option<StatusRow>, RepoError> {
    sqlx::query_as!(
        StatusRow,
        r#"SELECT r.id AS "request_id?", r.trigger_type AS "trigger_type?", s.status AS "status?",
                  e.reason_code AS "reason_code?", r.created_at AS "submitted_at?",
                  cr.nik_last4 AS "nik_last4?", e.id AS "event_id?", e.note_enc AS "note_enc?",
                  e.note_data_key_id AS "note_data_key_id?"
           FROM app.care_recipient cr
           LEFT JOIN app.identity_verification_request r ON r.id = cr.latest_verification_request_id
           LEFT JOIN app.identity_verification_request_state s ON s.identity_verification_request_id = r.id
           LEFT JOIN app.identity_verification_request_status_event e
                  ON e.identity_verification_request_id = r.id AND e.version = s.version
           WHERE cr.id = $1 AND cr.deleted_at IS NULL"#,
        care_recipient_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(map_db)
}

// ----------------------------------------------------------------------------------------------------
// Verifikator (F1-T62, US-201).

/// Satu baris antrean pengajuan menunggu (US-201 AC1).
pub(super) struct QueueRow {
    pub(super) request_id: Uuid,
    pub(super) care_recipient_id: Uuid,
    pub(super) trigger_type: String,
    pub(super) submitted_at: OffsetDateTime,
    pub(super) document_type: String,
    pub(super) nik_last4: String,
    pub(super) full_name: String,
    pub(super) birth_date: time::Date,
    pub(super) sex: String,
    pub(super) is_account_owner: bool,
}

/// Pengajuan berstatus `pending` urut dari yang terlama (`created_at`, lalu `id`), setelah `after`
/// (ID permintaan terakhir halaman sebelumnya). Cursor yang tidak dikenal menghasilkan halaman kosong.
pub(super) async fn pending_queue(
    pool: &PgPool,
    after: Option<Uuid>,
    limit: i64,
) -> Result<Vec<QueueRow>, RepoError> {
    sqlx::query_as!(
        QueueRow,
        r#"SELECT r.id AS request_id, r.care_recipient_id, r.trigger_type, r.created_at AS submitted_at,
                  d.document_type, d.nik_last4 AS "nik_last4!", cr.full_name, cr.birth_date, cr.sex,
                  cr.is_account_owner
           FROM app.identity_verification_request_state s
           JOIN app.identity_verification_request r ON r.id = s.identity_verification_request_id
           JOIN app.identity_document d ON d.id = r.identity_document_id
           JOIN app.care_recipient cr ON cr.id = r.care_recipient_id
           WHERE s.status = 'pending' AND cr.deleted_at IS NULL AND d.deleted_at IS NULL
             AND ($1::uuid IS NULL OR (r.created_at, r.id) >
                  (SELECT c.created_at, c.id FROM app.identity_verification_request c WHERE c.id = $1))
           ORDER BY r.created_at, r.id
           LIMIT $2"#,
        after,
        limit,
    )
    .fetch_all(pool)
    .await
    .map_err(map_db)
}

/// Pengajuan yang akan diputuskan atau fotonya dibuka: penerima layanan, pemilik, dan dokumen.
pub(super) struct DecisionTarget {
    pub(super) care_recipient_id: Uuid,
    pub(super) is_account_owner: bool,
    pub(super) owner_account_id: Uuid,
    pub(super) document_id: Uuid,
    pub(super) nik_enc: Vec<u8>,
    pub(super) data_key_id: Uuid,
    pub(super) file_id: Uuid,
}

pub(super) async fn decision_target(
    pool: &PgPool,
    request_id: Uuid,
) -> Result<Option<DecisionTarget>, RepoError> {
    sqlx::query_as!(
        DecisionTarget,
        "SELECT r.care_recipient_id, cr.is_account_owner, p.account_id AS owner_account_id, \
                d.id AS document_id, d.nik_enc, d.data_key_id, d.file_id \
         FROM app.identity_verification_request r \
         JOIN app.identity_document d ON d.id = r.identity_document_id \
         JOIN app.care_recipient cr ON cr.id = r.care_recipient_id \
         JOIN app.patient p ON p.id = cr.patient_id \
         WHERE r.id = $1 AND cr.deleted_at IS NULL AND p.deleted_at IS NULL AND d.deleted_at IS NULL",
        request_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(map_db)
}

/// Detail pengajuan untuk Verifikator (US-201 AC1): data diri, dokumen, status, dan keputusan terakhir.
pub(super) struct DetailRow {
    pub(super) request_id: Uuid,
    pub(super) care_recipient_id: Uuid,
    pub(super) trigger_type: String,
    pub(super) submitted_at: OffsetDateTime,
    pub(super) document_id: Uuid,
    pub(super) document_type: String,
    pub(super) nik_enc: Vec<u8>,
    pub(super) data_key_id: Uuid,
    pub(super) full_name: String,
    pub(super) birth_date: time::Date,
    pub(super) sex: String,
    pub(super) is_account_owner: bool,
    pub(super) owner_account_id: Uuid,
    pub(super) status: String,
    pub(super) reason_code: Option<String>,
    pub(super) status_changed_at: OffsetDateTime,
}

pub(super) async fn verification_detail(
    pool: &PgPool,
    request_id: Uuid,
) -> Result<Option<DetailRow>, RepoError> {
    sqlx::query_as!(
        DetailRow,
        "SELECT r.id AS request_id, r.care_recipient_id, r.trigger_type, r.created_at AS submitted_at, \
                d.id AS document_id, d.document_type, d.nik_enc, d.data_key_id, \
                cr.full_name, cr.birth_date, cr.sex, cr.is_account_owner, p.account_id AS owner_account_id, \
                s.status, e.reason_code, e.created_at AS status_changed_at \
         FROM app.identity_verification_request r \
         JOIN app.identity_verification_request_state s ON s.identity_verification_request_id = r.id \
         JOIN app.identity_verification_request_status_event e \
              ON e.identity_verification_request_id = r.id AND e.version = s.version \
         JOIN app.identity_document d ON d.id = r.identity_document_id \
         JOIN app.care_recipient cr ON cr.id = r.care_recipient_id \
         JOIN app.patient p ON p.id = cr.patient_id \
         WHERE r.id = $1 AND cr.deleted_at IS NULL AND p.deleted_at IS NULL AND d.deleted_at IS NULL",
        request_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(map_db)
}

/// Penerima layanan **lain** pemegang baris registri NIK pemilik akun aktif yang cocok dengan salah satu
/// kandidat blind index (semua versi kunci aktif, TD-10; UP-40, US-201 AC7).
pub(super) async fn other_registry_holder(
    conn: &mut PgConnection,
    candidates: &[(Vec<u8>, i16)],
    care_recipient_id: Uuid,
) -> Result<Option<Uuid>, RepoError> {
    let (values, versions): (Vec<Vec<u8>>, Vec<i16>) = candidates.iter().cloned().unzip();
    sqlx::query_scalar!(
        "SELECT care_recipient_id FROM app.account_owner_nik_registry \
         WHERE deleted_at IS NULL AND care_recipient_id <> $3 \
           AND (nik_bidx, nik_bidx_key_version) IN \
               (SELECT v, k FROM UNNEST($1::bytea[], $2::smallint[]) AS c(v, k)) \
         ORDER BY created_at LIMIT 1",
        &values,
        &versions,
        care_recipient_id,
    )
    .fetch_optional(conn)
    .await
    .map_err(map_db)
}

/// Soft delete baris registri aktif penerima layanan (NIK lama) sebelum NIK baru disetujui (ADR-003).
pub(super) async fn retire_registry(
    conn: &mut PgConnection,
    care_recipient_id: Uuid,
    actor_account_id: Uuid,
    reason: &str,
) -> Result<(), RepoError> {
    sqlx::query!(
        "UPDATE app.account_owner_nik_registry \
         SET deleted_at = now(), deleted_by = $2, delete_reason = $3, updated_at = now(), updated_by = $2 \
         WHERE care_recipient_id = $1 AND deleted_at IS NULL",
        care_recipient_id,
        actor_account_id,
        reason,
    )
    .execute(conn)
    .await
    .map_err(map_db)?;
    Ok(())
}

/// Baris registri NIK pemilik akun baru (A2.8).
pub(super) struct NewRegistry<'a> {
    pub(super) id: Uuid,
    pub(super) created_by: Uuid,
    pub(super) nik_bidx: &'a [u8],
    pub(super) nik_bidx_key_version: i16,
    pub(super) care_recipient_id: Uuid,
    pub(super) identity_verification_request_id: Uuid,
}

/// Menyisipkan registri; NIK yang sudah tercatat aktif = [`RepoError::RegistryConflict`].
pub(super) async fn insert_registry(
    conn: &mut PgConnection,
    r: &NewRegistry<'_>,
) -> Result<(), RepoError> {
    sqlx::query!(
        "INSERT INTO app.account_owner_nik_registry \
             (id, created_by, updated_by, nik_bidx, nik_bidx_key_version, care_recipient_id, \
              identity_verification_request_id) \
         VALUES ($1, $2, $2, $3, $4, $5, $6)",
        r.id,
        r.created_by,
        r.nik_bidx,
        r.nik_bidx_key_version,
        r.care_recipient_id,
        r.identity_verification_request_id,
    )
    .execute(conn)
    .await
    .map_err(map_db)?;
    Ok(())
}

/// Dokumen NIK pemilik akun yang sudah disetujui untuk akun pasien `owner_account_id` (baris registri aktif
/// penerima layanan pemilik akun; US-201 AC8, KEPUTUSAN-294).
pub(super) struct OwnerDocument {
    pub(super) document_id: Uuid,
    pub(super) data_key_id: Uuid,
    pub(super) nik_enc: Vec<u8>,
}

pub(super) async fn approved_owner_document(
    pool: &PgPool,
    owner_account_id: Uuid,
) -> Result<Option<OwnerDocument>, RepoError> {
    sqlx::query_as!(
        OwnerDocument,
        "SELECT d.id AS document_id, d.data_key_id, d.nik_enc \
         FROM app.account_owner_nik_registry g \
         JOIN app.care_recipient cr ON cr.id = g.care_recipient_id \
         JOIN app.patient p ON p.id = cr.patient_id \
         JOIN app.identity_verification_request r ON r.id = g.identity_verification_request_id \
         JOIN app.identity_document d ON d.id = r.identity_document_id \
         WHERE p.account_id = $1 AND cr.is_account_owner \
           AND g.deleted_at IS NULL AND cr.deleted_at IS NULL AND p.deleted_at IS NULL \
         ORDER BY g.created_at DESC LIMIT 1",
        owner_account_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(map_db)
}
