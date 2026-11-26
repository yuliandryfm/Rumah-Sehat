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
    /// Ketiga jenis dokumen wajib ada (F1-T102, Q8).
    pub(super) documents_complete: bool,
    /// NIB yang sama dipakai perusahaan lain yang belum terhapus, status apa pun (CR-035, Q10 = B2).
    pub(super) nib_used_by_other_company: bool,
}

/// US-1902 AC1: pengajuan "Menunggu verifikasi" urut dari yang terlama (`submitted_at`, lalu `id`),
/// keyset setelah perusahaan `after`. `complete_only`: `Some(true)` hanya yang dokumennya lengkap ("siap
/// diverifikasi"), `Some(false)` hanya yang belum lengkap, `None` semuanya (F1-T102, Q8). Index parsial
/// `ix_corporate_client_submitted_at_pending` melayani filter status dan urutan; hitungan dokumen memakai
/// `uqp_corp_client_doc_corporate_client_id_document_kind` per baris; penanda NIB dipakai memakai
/// `ix_corporate_client_nib` (CR-035).
pub(super) async fn pending_queue(
    pool: &PgPool,
    after: Option<Uuid>,
    complete_only: Option<bool>,
    limit: i64,
) -> Result<Vec<QueueRow>, sqlx::Error> {
    sqlx::query_as!(
        QueueRow,
        r#"SELECT q.id AS "id!", q.legal_name AS "legal_name!", q.nib AS "nib!", q.origin AS "origin!",
                  q.submission_round AS "submission_round!", q.submitted_at AS "submitted_at!",
                  q.documents_complete AS "documents_complete!",
                  q.nib_used_by_other_company AS "nib_used_by_other_company!"
           FROM (
             SELECT c.id, c.legal_name, c.nib, c.origin, c.submission_round, c.submitted_at,
                    (SELECT count(*) FILTER (WHERE d.document_kind IN ('nib', 'npwp', 'authority_proof'))
                       FROM app.corporate_client_document d
                      WHERE d.corporate_client_id = c.id AND d.deleted_at IS NULL) = 3 AS documents_complete,
                    EXISTS (SELECT 1 FROM app.corporate_client o
                             WHERE o.nib = c.nib AND o.id <> c.id AND o.deleted_at IS NULL)
                      AS nib_used_by_other_company
             FROM app.corporate_client c
             WHERE c.deleted_at IS NULL AND c.status = 'pending_verification'
           ) q
           WHERE ($2::boolean IS NULL OR q.documents_complete = $2)
             AND ($1::uuid IS NULL OR (q.submitted_at, q.id) >
                  (SELECT a.submitted_at, a.id FROM app.corporate_client a WHERE a.id = $1))
           ORDER BY q.submitted_at, q.id
           LIMIT $3"#,
        after,
        complete_only,
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
    /// NIB yang sama dipakai perusahaan lain yang belum terhapus, status apa pun (CR-035, Q10 = B2).
    pub(super) nib_used_by_other_company: bool,
}

/// Perusahaan yang belum dihapus lunak (ADR-003); `None` bila tidak ada.
pub(super) async fn client_detail(
    pool: &PgPool,
    corporate_client_id: Uuid,
) -> Result<Option<DetailRow>, sqlx::Error> {
    sqlx::query_as!(
        DetailRow,
        r#"SELECT c.id, c.legal_name, c.nib, c.npwp, c.office_address, c.office_phone, c.status, c.origin,
                  c.submission_round, c.submitted_at,
                  EXISTS (SELECT 1 FROM app.corporate_client o
                           WHERE o.nib = c.nib AND o.id <> c.id AND o.deleted_at IS NULL)
                    AS "nib_used_by_other_company!"
           FROM app.corporate_client c WHERE c.id = $1 AND c.deleted_at IS NULL"#,
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
pub(super) async fn round_decision<'e, E>(
    executor: E,
    corporate_client_id: Uuid,
    submission_round: i32,
) -> Result<Option<DecisionRow>, sqlx::Error>
where
    E: sqlx::PgExecutor<'e>,
{
    sqlx::query_as!(
        DecisionRow,
        "SELECT decision, reason_code, created_at FROM app.corporate_verification_decision \
         WHERE corporate_client_id = $1 AND submission_round = $2",
        corporate_client_id,
        submission_round,
    )
    .fetch_optional(executor)
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
    /// NIB (persetujuan memeriksa perusahaan Aktif lain dengan NIB sama, CR-035).
    pub(super) nib: String,
}

/// Mengunci baris perusahaan (`FOR UPDATE`) di transaksi keputusan: dua keputusan atas perusahaan yang sama
/// berurutan, dan yang kedua membaca status hasil yang pertama (US-1902 AC4).
pub(super) async fn lock_client(
    conn: &mut PgConnection,
    corporate_client_id: Uuid,
) -> Result<Option<LockedClient>, sqlx::Error> {
    sqlx::query_as!(
        LockedClient,
        "SELECT status, submission_round, nib FROM app.corporate_client \
         WHERE id = $1 AND deleted_at IS NULL FOR UPDATE",
        corporate_client_id,
    )
    .fetch_optional(conn)
    .await
}

/// Ada perusahaan Aktif lain (belum terhapus) dengan NIB `nib` selain `corporate_client_id` (CR-035: persetujuan
/// ditolak `4090116`). Penjaga terakhir untuk balapan dua persetujuan = `uqp_corporate_client_nib_active`.
pub(super) async fn other_active_client_with_nib(
    conn: &mut PgConnection,
    nib: &str,
    corporate_client_id: Uuid,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar!(
        r#"SELECT EXISTS (SELECT 1 FROM app.corporate_client
                           WHERE nib = $1 AND id <> $2 AND status = 'active' AND deleted_at IS NULL) AS "e!""#,
        nib,
        corporate_client_id,
    )
    .fetch_one(conn)
    .await
}

/// Nama index unik "satu perusahaan Aktif per NIB" (migrasi 20261126400000, CR-035).
pub(super) const NIB_ACTIVE_UNIQUE_INDEX: &str = "uqp_corporate_client_nib_active";

/// Galat sqlx berasal dari [`NIB_ACTIVE_UNIQUE_INDEX`] (dua persetujuan bersamaan atas NIB yang sama).
pub(super) fn is_active_nib_conflict(e: &sqlx::Error) -> bool {
    e.as_database_error()
        .and_then(|d| d.constraint())
        .is_some_and(|c| c == NIB_ACTIVE_UNIQUE_INDEX)
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

// ----- F1-T105 / F1-T107: perusahaan dibuatkan Verifikator dan undangan HR -----

/// Perusahaan baru yang dibuat Verifikator (langsung Aktif, US-1903 AC1).
pub(super) struct NewAdminClient<'a> {
    pub(super) id: Uuid,
    pub(super) created_by: Uuid,
    pub(super) legal_name: &'a str,
    pub(super) nib: &'a str,
    pub(super) npwp: &'a str,
    pub(super) office_address: &'a str,
    pub(super) office_phone: &'a str,
    pub(super) now: OffsetDateTime,
}

/// ID perusahaan **Aktif** belum terhapus dengan NIB yang sama (`uqp_corporate_client_nib_active`, CR-035: NIB
/// unik hanya di antara perusahaan Aktif); pemeriksaan awal supaya penolakan NIB ganda tidak bergantung pada galat
/// constraint.
pub(super) async fn active_client_with_nib(
    conn: &mut PgConnection,
    nib: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    sqlx::query_scalar!(
        "SELECT id FROM app.corporate_client WHERE nib = $1 AND status = 'active' AND deleted_at IS NULL",
        nib,
    )
    .fetch_optional(conn)
    .await
}

/// Menyisipkan perusahaan Aktif berasal `created_by_admin`. `false` bila NIB bentrok dengan perusahaan Aktif lain
/// yang belum terhapus (`uqp_corporate_client_nib_active`, balapan dengan pembuatan atau persetujuan lain).
/// `ON CONFLICT DO NOTHING` supaya transaksi pemanggil tetap dapat dipakai.
pub(super) async fn insert_admin_client(
    conn: &mut PgConnection,
    c: &NewAdminClient<'_>,
) -> Result<bool, sqlx::Error> {
    let r = sqlx::query!(
        "INSERT INTO app.corporate_client \
           (id, created_at, created_by, updated_at, updated_by, legal_name, nib, npwp, office_address, \
            office_phone, status, origin, submission_round, submitted_at) \
         VALUES ($1, $8, $2, $8, $2, $3, $4, $5, $6, $7, 'active', 'created_by_admin', 1, $8) \
         ON CONFLICT (nib) WHERE deleted_at IS NULL AND status = 'active' DO NOTHING",
        c.id,
        c.created_by,
        c.legal_name,
        c.nib,
        c.npwp,
        c.office_address,
        c.office_phone,
        c.now,
    )
    .execute(conn)
    .await?;
    Ok(r.rows_affected() == 1)
}

/// Menyisipkan keanggotaan HR (satu keanggotaan aktif per akun, `uqp_corporate_member_account_id`).
pub(super) async fn insert_member(
    conn: &mut PgConnection,
    id: Uuid,
    corporate_client_id: Uuid,
    account_id: Uuid,
    by: Uuid,
    now: OffsetDateTime,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO app.corporate_member \
           (id, created_at, created_by, updated_at, updated_by, account_id, corporate_client_id) \
         VALUES ($1, $5, $4, $5, $4, $3, $2)",
        id,
        corporate_client_id,
        account_id,
        by,
        now,
    )
    .execute(conn)
    .await?;
    Ok(())
}

// ----- F1-T101: pendaftaran perusahaan mandiri oleh HR (US-1901) -----

/// Perusahaan baru dari pendaftaran mandiri (status Menunggu verifikasi, putaran 1).
pub(super) struct NewSelfRegisteredClient<'a> {
    pub(super) id: Uuid,
    /// Akun HR pendaftar (dibuat di transaksi yang sama).
    pub(super) created_by: Uuid,
    pub(super) legal_name: &'a str,
    pub(super) nib: &'a str,
    pub(super) npwp: &'a str,
    pub(super) office_address: &'a str,
    pub(super) office_phone: &'a str,
    pub(super) now: OffsetDateTime,
}

/// Menyisipkan perusahaan `pending_verification` berasal `self_registered`. NIB boleh sama dengan perusahaan lain
/// (CR-035, Q10 = B2): keunikan NIB hanya di antara perusahaan Aktif, ditegakkan saat persetujuan.
pub(super) async fn insert_self_registered_client(
    conn: &mut PgConnection,
    c: &NewSelfRegisteredClient<'_>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO app.corporate_client \
           (id, created_at, created_by, updated_at, updated_by, legal_name, nib, npwp, office_address, \
            office_phone, status, origin, submission_round, submitted_at) \
         VALUES ($1, $8, $2, $8, $2, $3, $4, $5, $6, $7, 'pending_verification', 'self_registered', 1, $8)",
        c.id,
        c.created_by,
        c.legal_name,
        c.nib,
        c.npwp,
        c.office_address,
        c.office_phone,
        c.now,
    )
    .execute(conn)
    .await?;
    Ok(())
}

// ----- F1-T102: dokumen NIB, NPWP, dan bukti kewenangan oleh HR (US-1901 AC2) -----

/// Profil perusahaan untuk halaman status HR (US-1901 AC6, TD-47 b). Sengaja tanpa `Debug` (memuat NPWP,
/// Terbatas); service membungkus NPWP dengan `foundation::Sensitive` segera setelah dibaca.
pub(super) struct ClientProfileRow {
    pub(super) legal_name: String,
    pub(super) nib: String,
    pub(super) npwp: String,
    pub(super) office_address: String,
    pub(super) office_phone: String,
    pub(super) status: String,
    pub(super) submission_round: i32,
    pub(super) submitted_at: OffsetDateTime,
}

/// Profil perusahaan `corporate_client_id` yang belum dihapus lunak (PK). Pemanggil selalu memberi ID dari
/// cakupan sesi HR (`CorporateScope`), tidak pernah dari isi permintaan. `executor` boleh pool atau koneksi
/// transaksi yang sudah mengunci baris.
pub(super) async fn client_profile<'e, E>(
    executor: E,
    corporate_client_id: Uuid,
) -> Result<Option<ClientProfileRow>, sqlx::Error>
where
    E: sqlx::PgExecutor<'e>,
{
    sqlx::query_as!(
        ClientProfileRow,
        "SELECT legal_name, nib, npwp, office_address, office_phone, status, submission_round, submitted_at \
         FROM app.corporate_client WHERE id = $1 AND deleted_at IS NULL",
        corporate_client_id,
    )
    .fetch_optional(executor)
    .await
}

/// Pengajuan ulang (US-1902 AC3, F1-T102b): "Ditolak" putaran `submission_round` kembali "Menunggu verifikasi"
/// putaran berikutnya dengan `submitted_at` baru (antrean urut pengajuan terakhir). Jumlah baris berubah (0 =
/// baris tidak lagi berada di keadaan yang dikunci pemanggil).
pub(super) async fn resubmit_client(
    conn: &mut PgConnection,
    corporate_client_id: Uuid,
    submission_round: i32,
    actor_id: Uuid,
    now: OffsetDateTime,
) -> Result<u64, sqlx::Error> {
    let r = sqlx::query!(
        "UPDATE app.corporate_client \
         SET status = 'pending_verification', submission_round = submission_round + 1, submitted_at = $4, \
             updated_at = $4, updated_by = $3 \
         WHERE id = $1 AND submission_round = $2 AND status = 'rejected' AND deleted_at IS NULL",
        corporate_client_id,
        submission_round,
        actor_id,
        now,
    )
    .execute(conn)
    .await?;
    Ok(r.rows_affected())
}

/// Dokumen aktif perusahaan.
pub(super) struct DocumentRow {
    pub(super) id: Uuid,
    pub(super) document_kind: String,
    pub(super) stored_file_id: Uuid,
    pub(super) created_at: OffsetDateTime,
}

/// Dokumen aktif perusahaan, urut jenis (`uqp_corp_client_doc_corporate_client_id_document_kind`).
pub(super) async fn active_documents<'e, E>(
    executor: E,
    corporate_client_id: Uuid,
) -> Result<Vec<DocumentRow>, sqlx::Error>
where
    E: sqlx::PgExecutor<'e>,
{
    sqlx::query_as!(
        DocumentRow,
        "SELECT id, document_kind, stored_file_id, created_at FROM app.corporate_client_document \
         WHERE corporate_client_id = $1 AND deleted_at IS NULL ORDER BY document_kind",
        corporate_client_id,
    )
    .fetch_all(executor)
    .await
}

/// Menghapus lunak satu dokumen aktif karena diganti (ADR-003): selalu difilter perusahaan pemanggil.
/// `false` bila tidak ada baris aktif yang cocok.
pub(super) async fn soft_delete_document(
    conn: &mut PgConnection,
    document_id: Uuid,
    corporate_client_id: Uuid,
    by: Uuid,
    reason: &str,
    now: OffsetDateTime,
) -> Result<bool, sqlx::Error> {
    let r = sqlx::query!(
        "UPDATE app.corporate_client_document \
         SET deleted_at = $4, deleted_by = $3, delete_reason = $5, updated_at = $4, updated_by = $3 \
         WHERE id = $1 AND corporate_client_id = $2 AND deleted_at IS NULL",
        document_id,
        corporate_client_id,
        by,
        now,
        reason,
    )
    .execute(conn)
    .await?;
    Ok(r.rows_affected() == 1)
}

/// Dokumen baru.
pub(super) struct NewDocument<'a> {
    pub(super) id: Uuid,
    pub(super) corporate_client_id: Uuid,
    pub(super) document_kind: &'a str,
    pub(super) stored_file_id: Uuid,
    pub(super) by: Uuid,
    pub(super) now: OffsetDateTime,
}

/// Menyisipkan dokumen aktif. Satu dokumen aktif per jenis per perusahaan dijamin
/// `uqp_corp_client_doc_corporate_client_id_document_kind`; pemanggil menghapus lunak yang lama lebih dulu.
pub(super) async fn insert_document(
    conn: &mut PgConnection,
    d: &NewDocument<'_>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO app.corporate_client_document \
           (id, created_at, created_by, updated_at, updated_by, corporate_client_id, document_kind, stored_file_id) \
         VALUES ($1, $6, $5, $6, $5, $2, $3, $4)",
        d.id,
        d.corporate_client_id,
        d.document_kind,
        d.stored_file_id,
        d.by,
        d.now,
    )
    .execute(conn)
    .await?;
    Ok(())
}

/// Menyisipkan keanggotaan HR pendaftar beserta jabatannya (US-1901 AC1).
pub(super) async fn insert_member_with_title(
    conn: &mut PgConnection,
    id: Uuid,
    corporate_client_id: Uuid,
    account_id: Uuid,
    job_title: &str,
    now: OffsetDateTime,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO app.corporate_member \
           (id, created_at, created_by, updated_at, updated_by, account_id, corporate_client_id, job_title) \
         VALUES ($1, $5, $3, $5, $3, $3, $2, $4)",
        id,
        corporate_client_id,
        account_id,
        job_title,
        now,
    )
    .execute(conn)
    .await?;
    Ok(())
}
