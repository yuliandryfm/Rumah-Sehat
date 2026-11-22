//! Akses data modul Korporat (F1-T100 keanggotaan HR; F1-T104 verifikasi perusahaan; F1-T107 anggota HR).
//! Satu-satunya tempat SQL modul; tabel A20 milik modul ini (`module-tables.toml`).
//! `corporate_verification_decision` (Transaksi) hanya `INSERT` (ADR-004).

use foundation::Uuid;
use sqlx::{PgConnection, PgPool};
use time::OffsetDateTime;

/// Keanggotaan aktif satu akun HR beserta status perusahaannya.
pub(super) struct MembershipRow {
    pub(super) corporate_member_id: Uuid,
    pub(super) corporate_client_id: Uuid,
    pub(super) client_status: String,
}

/// Keanggotaan aktif akun `account_id`: baris `corporate_member` yang belum dihapus lunak (paling banyak
/// satu, `uqp_corporate_member_account_id`) dengan perusahaan yang belum dihapus lunak (ADR-003). Index
/// `uqp_corporate_member_account_id` dan PK `corporate_client` melayani query ini.
pub(super) async fn active_membership(
    pool: &PgPool,
    account_id: Uuid,
) -> Result<Option<MembershipRow>, sqlx::Error> {
    sqlx::query_as!(
        MembershipRow,
        "SELECT m.id AS corporate_member_id, m.corporate_client_id, c.status AS client_status \
         FROM app.corporate_member m \
         JOIN app.corporate_client c ON c.id = m.corporate_client_id \
         WHERE m.account_id = $1 AND m.deleted_at IS NULL AND c.deleted_at IS NULL",
        account_id,
    )
    .fetch_optional(pool)
    .await
}

// ----- verifikasi perusahaan (F1-T104, US-1902) -----

/// Satu pengajuan di antrean (tanpa NPWP: NPWP hanya di detail, klasifikasi Terbatas DD A20).
pub(super) struct QueueRow {
    pub(super) id: Uuid,
    pub(super) legal_name: String,
    pub(super) nib: String,
    pub(super) origin: String,
    pub(super) submission_round: i32,
    pub(super) submitted_at: OffsetDateTime,
}

/// US-1902 AC1: pengajuan "Menunggu verifikasi" urut dari yang terlama (`submitted_at`, lalu `id`),
/// keyset setelah perusahaan `after`. Index parsial `ix_corporate_client_submitted_at_pending` melayani
/// filter dan urutan.
pub(super) async fn pending_queue(
    pool: &PgPool,
    after: Option<Uuid>,
    limit: i64,
) -> Result<Vec<QueueRow>, sqlx::Error> {
    sqlx::query_as!(
        QueueRow,
        "SELECT c.id, c.legal_name, c.nib, c.origin, c.submission_round, c.submitted_at \
         FROM app.corporate_client c \
         WHERE c.deleted_at IS NULL AND c.status = 'pending_verification' \
           AND ($1::uuid IS NULL OR (c.submitted_at, c.id) > \
                (SELECT a.submitted_at, a.id FROM app.corporate_client a WHERE a.id = $1)) \
         ORDER BY c.submitted_at, c.id \
         LIMIT $2",
        after,
        limit,
    )
    .fetch_all(pool)
    .await
}

/// Data perusahaan untuk detail Verifikator. Sengaja tanpa `Debug` (memuat NPWP, Terbatas); service
/// membungkus NPWP dengan `foundation::Sensitive` segera setelah dibaca.
pub(super) struct DetailRow {
    pub(super) id: Uuid,
    pub(super) legal_name: String,
    pub(super) nib: String,
    pub(super) npwp: String,
    pub(super) office_address: String,
    pub(super) office_phone: String,
    pub(super) status: String,
    pub(super) origin: String,
    pub(super) submission_round: i32,
    pub(super) submitted_at: OffsetDateTime,
}

/// Perusahaan yang belum dihapus lunak (ADR-003); `None` bila tidak ada.
pub(super) async fn client_detail(
    pool: &PgPool,
    corporate_client_id: Uuid,
) -> Result<Option<DetailRow>, sqlx::Error> {
    sqlx::query_as!(
        DetailRow,
        "SELECT id, legal_name, nib, npwp, office_address, office_phone, status, origin, \
                submission_round, submitted_at \
         FROM app.corporate_client WHERE id = $1 AND deleted_at IS NULL",
        corporate_client_id,
    )
    .fetch_optional(pool)
    .await
}

/// Keputusan atas satu putaran pengajuan.
pub(super) struct DecisionRow {
    pub(super) decision: String,
    pub(super) reason_code: Option<String>,
    pub(super) created_at: OffsetDateTime,
}

/// Keputusan putaran `submission_round` (paling banyak satu, `uq_corp_verif_decision_*`).
pub(super) async fn round_decision(
    pool: &PgPool,
    corporate_client_id: Uuid,
    submission_round: i32,
) -> Result<Option<DecisionRow>, sqlx::Error> {
    sqlx::query_as!(
        DecisionRow,
        "SELECT decision, reason_code, created_at FROM app.corporate_verification_decision \
         WHERE corporate_client_id = $1 AND submission_round = $2",
        corporate_client_id,
        submission_round,
    )
    .fetch_optional(pool)
    .await
}

/// Jenis dokumen aktif perusahaan (urut jenis).
pub(super) async fn active_document_kinds(
    pool: &PgPool,
    corporate_client_id: Uuid,
) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar!(
        "SELECT document_kind FROM app.corporate_client_document \
         WHERE corporate_client_id = $1 AND deleted_at IS NULL ORDER BY document_kind",
        corporate_client_id,
    )
    .fetch_all(pool)
    .await
}

/// Berkas dokumen aktif berjenis `document_kind` milik perusahaan yang belum dihapus lunak. Index
/// `uqp_corp_client_doc_corporate_client_id_document_kind` melayani query ini.
pub(super) async fn document_file(
    pool: &PgPool,
    corporate_client_id: Uuid,
    document_kind: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    sqlx::query_scalar!(
        "SELECT d.stored_file_id FROM app.corporate_client_document d \
         JOIN app.corporate_client c ON c.id = d.corporate_client_id \
         WHERE d.corporate_client_id = $1 AND d.document_kind = $2 \
           AND d.deleted_at IS NULL AND c.deleted_at IS NULL",
        corporate_client_id,
        document_kind,
    )
    .fetch_optional(pool)
    .await
}

/// Perusahaan terkunci untuk keputusan.
pub(super) struct LockedClient {
    pub(super) status: String,
    pub(super) submission_round: i32,
}

/// Mengunci baris perusahaan (`FOR UPDATE`) di transaksi keputusan: dua keputusan atas perusahaan yang sama
/// berurutan, dan yang kedua membaca status hasil yang pertama (US-1902 AC4).
pub(super) async fn lock_client(
    conn: &mut PgConnection,
    corporate_client_id: Uuid,
) -> Result<Option<LockedClient>, sqlx::Error> {
    sqlx::query_as!(
        LockedClient,
        "SELECT status, submission_round FROM app.corporate_client \
         WHERE id = $1 AND deleted_at IS NULL FOR UPDATE",
        corporate_client_id,
    )
    .fetch_optional(conn)
    .await
}

/// Keputusan baru.
pub(super) struct NewDecision<'a> {
    pub(super) id: Uuid,
    pub(super) corporate_client_id: Uuid,
    pub(super) submission_round: i32,
    pub(super) decision: &'a str,
    pub(super) reason_code: Option<&'a str>,
    pub(super) decided_by: Uuid,
}

/// Menyisipkan keputusan; `false` bila putaran itu sudah punya keputusan (keputusan pertama menang,
/// `uq_corp_verif_decision_corporate_client_id_submission_round`). `ON CONFLICT ... DO NOTHING` supaya
/// transaksi pemanggil tetap dapat dipakai (pola `status-flow` langkah c).
pub(super) async fn insert_decision(
    conn: &mut PgConnection,
    d: &NewDecision<'_>,
) -> Result<bool, sqlx::Error> {
    let r = sqlx::query!(
        "INSERT INTO app.corporate_verification_decision \
           (id, created_by, corporate_client_id, submission_round, decision, reason_code, decided_by) \
         VALUES ($1, $2, $3, $4, $5, $6, $2) \
         ON CONFLICT ON CONSTRAINT uq_corp_verif_decision_corporate_client_id_submission_round DO NOTHING",
        d.id,
        d.decided_by,
        d.corporate_client_id,
        d.submission_round,
        d.decision,
        d.reason_code,
    )
    .execute(conn)
    .await?;
    Ok(r.rows_affected() == 1)
}

/// Memindahkan status perusahaan dari "Menunggu verifikasi" putaran `submission_round` ke `to_status`.
/// Jumlah baris berubah (0 = baris tidak lagi berada di keadaan yang dikunci).
pub(super) async fn set_decided_status(
    conn: &mut PgConnection,
    corporate_client_id: Uuid,
    submission_round: i32,
    to_status: &str,
    actor_id: Uuid,
) -> Result<u64, sqlx::Error> {
    let r = sqlx::query!(
        "UPDATE app.corporate_client SET status = $3, updated_at = now(), updated_by = $4 \
         WHERE id = $1 AND submission_round = $2 AND status = 'pending_verification' \
           AND deleted_at IS NULL",
        corporate_client_id,
        submission_round,
        to_status,
        actor_id,
    )
    .execute(conn)
    .await?;
    Ok(r.rows_affected())
}

// ----- F1-T107: kelola anggota HR (US-1904 AC2, AC5) -----

/// Keanggotaan aktif (belum dihapus lunak) satu perusahaan.
pub(super) struct MemberRow {
    pub(super) id: Uuid,
    pub(super) account_id: Uuid,
    pub(super) job_title: Option<String>,
}

/// Mengunci tabel `corporate_member` mode `SHARE ROW EXCLUSIVE` sampai akhir transaksi (pola
/// KEPUTUSAN-290): bentrok dengan dirinya sendiri dan dengan penulisan baris (`ROW EXCLUSIVE`), tidak dengan
/// pembacaan pembaca keanggotaan (`ACCESS SHARE`). Hak cukup dengan `UPDATE` milik `rs_app` (SRC-673).
pub(super) async fn lock_members(conn: &mut PgConnection) -> Result<(), sqlx::Error> {
    sqlx::query!("LOCK TABLE app.corporate_member IN SHARE ROW EXCLUSIVE MODE")
        .execute(conn)
        .await?;
    Ok(())
}

/// Semua keanggotaan aktif perusahaan `client_id`, urut ID (index `ix_corporate_member_corporate_client_id`).
pub(super) async fn active_members(
    conn: &mut PgConnection,
    client_id: Uuid,
) -> Result<Vec<MemberRow>, sqlx::Error> {
    sqlx::query_as!(
        MemberRow,
        "SELECT id, account_id, job_title FROM app.corporate_member \
         WHERE corporate_client_id = $1 AND deleted_at IS NULL ORDER BY id",
        client_id,
    )
    .fetch_all(conn)
    .await
}

/// Perusahaan pemilik keanggotaan aktif `member_id` (`None` bila tidak ada atau sudah dicabut).
pub(super) async fn member_owner(
    pool: &PgPool,
    member_id: Uuid,
) -> Result<Option<Uuid>, sqlx::Error> {
    sqlx::query_scalar!(
        "SELECT corporate_client_id FROM app.corporate_member WHERE id = $1 AND deleted_at IS NULL",
        member_id,
    )
    .fetch_optional(pool)
    .await
}

/// Mencabut keanggotaan (hapus lunak, ADR-003). Selalu difilter perusahaan pemanggil (pertahanan
/// berlapis). `false` bila tidak ada baris aktif yang cocok.
pub(super) async fn revoke_member(
    conn: &mut PgConnection,
    member_id: Uuid,
    client_id: Uuid,
    by: Uuid,
    reason: &str,
    now: OffsetDateTime,
) -> Result<bool, sqlx::Error> {
    let done = sqlx::query!(
        "UPDATE app.corporate_member SET deleted_at = $4, deleted_by = $3, delete_reason = $5, \
           updated_at = $4, updated_by = $3 \
         WHERE id = $1 AND corporate_client_id = $2 AND deleted_at IS NULL",
        member_id,
        client_id,
        by,
        now,
        reason,
    )
    .execute(conn)
    .await?;
    Ok(done.rows_affected() == 1)
}
