//! Query sqlx atas tabel milik modul `privacy` (DD A3; `apps/rs-backend/module-tables.toml`):
//! `consent_text_version` (A3.1, M), `consent_item` (A3.2, M), `consent_record` (A3.3, L). Query default
//! memfilter baris terhapus (ADR-003). Hak `rs_app`: versi dan butir S/I/U tanpa DELETE; bukti persetujuan
//! S/I saja (ADR-004).

use foundation::Uuid;
use sqlx::PgConnection;
use time::OffsetDateTime;

/// Versi teks yang sedang berlaku.
pub(super) struct VersionRow {
    pub(super) id: Uuid,
    pub(super) consent_type: String,
    pub(super) version_label: String,
    pub(super) content: String,
    pub(super) effective_from: OffsetDateTime,
}

/// Versi teks jenis `consent_type` yang berlaku pada `now`: sudah terbit (`published_at` terisi), belum
/// terhapus, dan `effective_from <= now`; bila lebih dari satu, yang mulai berlakunya paling akhir
/// (US-104 AC3: teks baru tampil mulai `effective_from`).
pub(super) async fn active_version(
    conn: &mut PgConnection,
    consent_type: &str,
    now: OffsetDateTime,
) -> Result<Option<VersionRow>, sqlx::Error> {
    sqlx::query_as!(
        VersionRow,
        "SELECT id, consent_type, version_label, content, effective_from \
         FROM app.consent_text_version \
         WHERE consent_type = $1 AND deleted_at IS NULL AND published_at IS NOT NULL \
           AND effective_from <= $2 \
         ORDER BY effective_from DESC, id DESC \
         LIMIT 1",
        consent_type,
        now,
    )
    .fetch_optional(conn)
    .await
}

/// Butir aktif satu versi, urut tampilan.
pub(super) struct ItemRow {
    pub(super) id: Uuid,
    pub(super) item_code: String,
    pub(super) label: String,
    pub(super) is_primary: bool,
    pub(super) is_required: bool,
    pub(super) sort_order: i16,
}

pub(super) async fn items(
    conn: &mut PgConnection,
    consent_text_version_id: Uuid,
) -> Result<Vec<ItemRow>, sqlx::Error> {
    sqlx::query_as!(
        ItemRow,
        "SELECT id, item_code, label, is_primary, is_required, sort_order \
         FROM app.consent_item \
         WHERE consent_text_version_id = $1 AND deleted_at IS NULL \
         ORDER BY sort_order, item_code",
        consent_text_version_id,
    )
    .fetch_all(conn)
    .await
}

/// Jumlah butir aktif milik versi `version_id` di antara `ids` (butir asing tidak dihitung), hanya bila versi itu
/// belum terhapus dan berjenis salah satu `allowed_types` (kesesuaian jenis teks dengan subjek, KEPUTUSAN-321).
pub(super) async fn count_items_of_version(
    conn: &mut PgConnection,
    version_id: Uuid,
    ids: &[Uuid],
    allowed_types: &[String],
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar!(
        "SELECT count(*) FROM app.consent_item i \
         JOIN app.consent_text_version v ON v.id = i.consent_text_version_id \
         WHERE i.consent_text_version_id = $1 AND i.deleted_at IS NULL AND i.id = ANY($2) \
           AND v.deleted_at IS NULL AND v.consent_type = ANY($3)",
        version_id,
        ids,
        allowed_types,
    )
    .fetch_one(conn)
    .await
    .map(|n| n.unwrap_or(0))
}

/// Satu bukti persetujuan (A3.3). `created_at` diisi jam server oleh trigger
/// (`tg_consent_record_server_time`).
pub(super) struct NewConsentRecord {
    pub(super) id: Uuid,
    /// Subjek pasien; tepat salah satu dari `patient_id` dan `account_id` terisi (CHECK `ck_consent_record_subject`).
    pub(super) patient_id: Option<Uuid>,
    /// Subjek akun HR perusahaan (F1-T101; kanal `corporate_portal`).
    pub(super) account_id: Option<Uuid>,
    pub(super) consent_text_version_id: Uuid,
    pub(super) consent_item_id: Uuid,
    pub(super) action: &'static str,
    pub(super) channel: &'static str,
    pub(super) correlation_id: Uuid,
}

/// Menulis bukti persetujuan di transaksi milik pemanggil (KEPUTUSAN-114). Hanya INSERT (ADR-004).
pub(super) async fn insert_record(
    conn: &mut PgConnection,
    r: &NewConsentRecord,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO app.consent_record \
           (id, patient_id, account_id, care_recipient_id, consent_text_version_id, consent_item_id, action, \
            channel, correlation_id) \
         VALUES ($1, $2, $8, NULL, $3, $4, $5, $6, $7)",
        r.id,
        r.patient_id,
        r.consent_text_version_id,
        r.consent_item_id,
        r.action,
        r.channel,
        r.correlation_id,
        r.account_id,
    )
    .execute(conn)
    .await?;
    Ok(())
}
