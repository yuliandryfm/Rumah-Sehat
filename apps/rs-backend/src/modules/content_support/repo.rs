//! Query sqlx atas tabel milik modul `content_support`: `cms_content` (A15.4, kelas M). Hak `rs_app`
//! S, I, U tanpa DELETE (ADR-003). Query default memfilter `deleted_at IS NULL`; hanya jalur pulihkan dan
//! daftar "Terhapus" yang membaca baris terhapus.

use foundation::Uuid;
use sqlx::{PgConnection, PgPool};
use time::OffsetDateTime;

/// Baris `cms_content` (kolom yang dipakai aplikasi).
pub(super) struct ContentRow {
    pub(super) id: Uuid,
    pub(super) created_at: OffsetDateTime,
    pub(super) updated_at: OffsetDateTime,
    pub(super) deleted_at: Option<OffsetDateTime>,
    pub(super) delete_reason: Option<String>,
    pub(super) content_type: String,
    pub(super) title: String,
    pub(super) body: Option<String>,
    pub(super) category: Option<String>,
    pub(super) image_file_id: Option<Uuid>,
    pub(super) image_alt_text: Option<String>,
    pub(super) link_target_type: Option<String>,
    pub(super) link_target_ref: Option<String>,
    pub(super) publish_start_at: Option<OffsetDateTime>,
    pub(super) publish_end_at: Option<OffsetDateTime>,
    pub(super) editorial_status: String,
    pub(super) channel: String,
    pub(super) site_page: Option<String>,
}

/// Isian tersimpan (sudah dinormalkan service).
pub(super) struct Fields<'a> {
    pub(super) content_type: &'a str,
    pub(super) title: &'a str,
    pub(super) body: Option<&'a str>,
    pub(super) category: Option<&'a str>,
    pub(super) image_alt_text: Option<&'a str>,
    pub(super) link_target_type: Option<&'a str>,
    pub(super) link_target_ref: Option<&'a str>,
    pub(super) channel: &'a str,
    pub(super) site_page: Option<&'a str>,
}

/// Draf baru.
pub(super) async fn insert_draft(
    pool: &PgPool,
    id: Uuid,
    actor: Uuid,
    f: &Fields<'_>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO app.cms_content \
           (id, created_by, updated_by, content_type, title, body, category, image_alt_text, \
            link_target_type, link_target_ref, editorial_status, channel, site_page) \
         VALUES ($1, $2, $2, $3, $4, $5, $6, $7, $8, $9, 'draft', $10, $11)",
        id,
        actor,
        f.content_type,
        f.title,
        f.body,
        f.category,
        f.image_alt_text,
        f.link_target_type,
        f.link_target_ref,
        f.channel,
        f.site_page,
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Konten yang tidak terhapus.
pub(super) async fn find(pool: &PgPool, id: Uuid) -> Result<Option<ContentRow>, sqlx::Error> {
    sqlx::query_as!(
        ContentRow,
        "SELECT id, created_at, updated_at, deleted_at, delete_reason, content_type, title, body, category, \
                image_file_id, image_alt_text, link_target_type, link_target_ref, publish_start_at, \
                publish_end_at, editorial_status, channel, site_page \
         FROM app.cms_content WHERE id = $1 AND deleted_at IS NULL",
        id,
    )
    .fetch_optional(pool)
    .await
}

/// Mengunci baris (termasuk yang terhapus) sampai transaksi selesai.
pub(super) async fn lock(
    conn: &mut PgConnection,
    id: Uuid,
) -> Result<Option<ContentRow>, sqlx::Error> {
    sqlx::query_as!(
        ContentRow,
        "SELECT id, created_at, updated_at, deleted_at, delete_reason, content_type, title, body, category, \
                image_file_id, image_alt_text, link_target_type, link_target_ref, publish_start_at, \
                publish_end_at, editorial_status, channel, site_page \
         FROM app.cms_content WHERE id = $1 FOR UPDATE",
        id,
    )
    .fetch_optional(&mut *conn)
    .await
}

/// Mengubah isian draf. `false` bila baris bukan draf aktif (dicegah service lewat `lock`).
pub(super) async fn update_draft(
    conn: &mut PgConnection,
    id: Uuid,
    actor: Uuid,
    f: &Fields<'_>,
) -> Result<bool, sqlx::Error> {
    let r = sqlx::query!(
        "UPDATE app.cms_content SET content_type = $3, title = $4, body = $5, category = $6, \
                image_alt_text = $7, link_target_type = $8, link_target_ref = $9, channel = $10, \
                site_page = $11, updated_at = now(), updated_by = $2 \
         WHERE id = $1 AND deleted_at IS NULL AND editorial_status = 'draft'",
        id,
        actor,
        f.content_type,
        f.title,
        f.body,
        f.category,
        f.image_alt_text,
        f.link_target_type,
        f.link_target_ref,
        f.channel,
        f.site_page,
    )
    .execute(&mut *conn)
    .await?;
    Ok(r.rows_affected() == 1)
}

/// Draf → terjadwal dengan jendela tayang.
pub(super) async fn schedule(
    conn: &mut PgConnection,
    id: Uuid,
    actor: Uuid,
    start: OffsetDateTime,
    end: Option<OffsetDateTime>,
) -> Result<bool, sqlx::Error> {
    let r = sqlx::query!(
        "UPDATE app.cms_content SET editorial_status = 'scheduled', publish_start_at = $3, \
                publish_end_at = $4, updated_at = now(), updated_by = $2 \
         WHERE id = $1 AND deleted_at IS NULL AND editorial_status = 'draft'",
        id,
        actor,
        start,
        end,
    )
    .execute(&mut *conn)
    .await?;
    Ok(r.rows_affected() == 1)
}

/// Draf atau terjadwal → diarsipkan.
pub(super) async fn archive(
    conn: &mut PgConnection,
    id: Uuid,
    actor: Uuid,
) -> Result<bool, sqlx::Error> {
    let r = sqlx::query!(
        "UPDATE app.cms_content SET editorial_status = 'archived', updated_at = now(), updated_by = $2 \
         WHERE id = $1 AND deleted_at IS NULL AND editorial_status IN ('draft', 'scheduled')",
        id,
        actor,
    )
    .execute(&mut *conn)
    .await?;
    Ok(r.rows_affected() == 1)
}

/// Kunci transaksi per halaman tetap (dua kunci int4: namespace modul 11 + nomor halaman), supaya dua
/// penayangan halaman yang sama berjalan berurutan (US-1802 AC2). Dilepas otomatis saat transaksi selesai.
pub(super) async fn lock_fixed_page(
    conn: &mut PgConnection,
    page_no: i32,
) -> Result<(), sqlx::Error> {
    sqlx::query_scalar!(
        r#"SELECT 1::int4 AS "locked!" FROM pg_advisory_xact_lock(1101, $1::int4)"#,
        page_no,
    )
    .fetch_one(&mut *conn)
    .await?;
    Ok(())
}

/// Versi halaman tetap lain yang terjadwal untuk halaman yang sama (dikunci).
pub(super) struct VersionRow {
    pub(super) id: Uuid,
    pub(super) publish_start_at: Option<OffsetDateTime>,
    pub(super) publish_end_at: Option<OffsetDateTime>,
}

pub(super) async fn scheduled_fixed_versions(
    conn: &mut PgConnection,
    site_page: &str,
    exclude: Uuid,
) -> Result<Vec<VersionRow>, sqlx::Error> {
    sqlx::query_as!(
        VersionRow,
        "SELECT id, publish_start_at, publish_end_at FROM app.cms_content \
         WHERE site_page = $1 AND content_type = 'info_page' AND editorial_status = 'scheduled' \
           AND deleted_at IS NULL AND id <> $2 \
         FOR UPDATE",
        site_page,
        exclude,
    )
    .fetch_all(&mut *conn)
    .await
}

/// Memberi batas akhir tayang ke versi lama (US-1802 AC2).
pub(super) async fn end_versions(
    conn: &mut PgConnection,
    ids: &[Uuid],
    actor: Uuid,
    end: OffsetDateTime,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE app.cms_content SET publish_end_at = $3, updated_at = now(), updated_by = $2 \
         WHERE id = ANY($1) AND deleted_at IS NULL AND editorial_status = 'scheduled'",
        ids,
        actor,
        end,
    )
    .execute(&mut *conn)
    .await?;
    Ok(())
}

/// Soft delete (ADR-003). Mengembalikan waktu hapus (jam server); `None` bila sudah terhapus atau tidak ada.
pub(super) async fn soft_delete(
    conn: &mut PgConnection,
    id: Uuid,
    actor: Uuid,
    reason: &str,
) -> Result<Option<OffsetDateTime>, sqlx::Error> {
    sqlx::query_scalar!(
        r#"UPDATE app.cms_content SET deleted_at = now(), deleted_by = $2, delete_reason = $3,
                updated_at = now(), updated_by = $2
         WHERE id = $1 AND deleted_at IS NULL
         RETURNING deleted_at AS "deleted_at!""#,
        id,
        actor,
        reason,
    )
    .fetch_optional(&mut *conn)
    .await
}

/// Memulihkan konten terhapus (ADR-003). `false` bila tidak terhapus atau tidak ada.
pub(super) async fn restore(
    conn: &mut PgConnection,
    id: Uuid,
    actor: Uuid,
) -> Result<bool, sqlx::Error> {
    let r = sqlx::query!(
        "UPDATE app.cms_content SET deleted_at = NULL, deleted_by = NULL, delete_reason = NULL, \
                updated_at = now(), updated_by = $2 \
         WHERE id = $1 AND deleted_at IS NOT NULL",
        id,
        actor,
    )
    .execute(&mut *conn)
    .await?;
    Ok(r.rows_affected() == 1)
}

/// Saringan daftar backoffice. `status` = status tampilan (`deleted` membaca baris terhapus).
pub(super) struct ListFilter<'a> {
    pub(super) status: Option<&'a str>,
    pub(super) channel: Option<&'a str>,
    pub(super) site_page: Option<&'a str>,
    pub(super) before_id: Option<Uuid>,
    pub(super) limit: i64,
    pub(super) now: OffsetDateTime,
}

/// Daftar konten urut terbaru (ID UUIDv7 menurun), dengan cursor ID.
pub(super) async fn list(
    pool: &PgPool,
    f: &ListFilter<'_>,
) -> Result<Vec<ContentRow>, sqlx::Error> {
    sqlx::query_as!(
        ContentRow,
        "SELECT id, created_at, updated_at, deleted_at, delete_reason, content_type, title, body, category, \
                image_file_id, image_alt_text, link_target_type, link_target_ref, publish_start_at, \
                publish_end_at, editorial_status, channel, site_page \
         FROM app.cms_content \
         WHERE (CASE WHEN $1::text = 'deleted' THEN deleted_at IS NOT NULL \
                ELSE deleted_at IS NULL AND ( \
                  $1::text IS NULL \
                  OR ($1::text = 'draft' AND editorial_status = 'draft') \
                  OR ($1::text = 'archived' AND editorial_status = 'archived') \
                  OR ($1::text = 'scheduled' AND editorial_status = 'scheduled' AND publish_start_at > $2) \
                  OR ($1::text = 'live' AND editorial_status = 'scheduled' AND publish_start_at <= $2 \
                      AND (publish_end_at IS NULL OR publish_end_at > $2)) \
                  OR ($1::text = 'ended' AND editorial_status = 'scheduled' AND publish_end_at <= $2)) \
                END) \
           AND ($3::text IS NULL OR channel = $3) \
           AND ($4::text IS NULL OR site_page = $4) \
           AND ($5::uuid IS NULL OR id < $5) \
         ORDER BY id DESC \
         LIMIT $6",
        f.status,
        f.now,
        f.channel,
        f.site_page,
        f.before_id,
        f.limit,
    )
    .fetch_all(pool)
    .await
}

// ---------------------------------------------------------------- jalur publik (F1-T94)
//
// Satu definisi "Tayang di situs" dipakai semua query publik (ADR-027 butir 5, US-1801 AC3): kanal memuat
// situs, status `scheduled`, tidak terhapus, dan `$now` di dalam [publish_start_at, publish_end_at).
// Draf, terjadwal yang belum mulai, selesai, diarsipkan, dan terhapus tidak pernah terbaca lewat sini.

/// Halaman tetap yang sedang tayang untuk `site_page` (versi dengan waktu mulai terbaru).
pub(super) async fn live_site_page(
    pool: &PgPool,
    site_page: &str,
    now: OffsetDateTime,
) -> Result<Option<ContentRow>, sqlx::Error> {
    sqlx::query_as!(
        ContentRow,
        "SELECT id, created_at, updated_at, deleted_at, delete_reason, content_type, title, body, category, \
                image_file_id, image_alt_text, link_target_type, link_target_ref, publish_start_at, \
                publish_end_at, editorial_status, channel, site_page \
         FROM app.cms_content \
         WHERE site_page = $1 AND content_type = 'info_page' \
           AND channel IN ('site', 'app_and_site') AND editorial_status = 'scheduled' \
           AND deleted_at IS NULL AND publish_start_at <= $2 \
           AND (publish_end_at IS NULL OR publish_end_at > $2) \
         ORDER BY publish_start_at DESC, id DESC \
         LIMIT 1",
        site_page,
        now,
    )
    .fetch_optional(pool)
    .await
}

/// Satu konten situs yang sedang tayang.
pub(super) async fn live_site_item(
    pool: &PgPool,
    id: Uuid,
    now: OffsetDateTime,
) -> Result<Option<ContentRow>, sqlx::Error> {
    sqlx::query_as!(
        ContentRow,
        "SELECT id, created_at, updated_at, deleted_at, delete_reason, content_type, title, body, category, \
                image_file_id, image_alt_text, link_target_type, link_target_ref, publish_start_at, \
                publish_end_at, editorial_status, channel, site_page \
         FROM app.cms_content \
         WHERE id = $1 AND channel IN ('site', 'app_and_site') AND editorial_status = 'scheduled' \
           AND deleted_at IS NULL AND publish_start_at <= $2 \
           AND (publish_end_at IS NULL OR publish_end_at > $2)",
        id,
        now,
    )
    .fetch_optional(pool)
    .await
}

/// Saringan daftar publik.
pub(super) struct PublicFilter<'a> {
    pub(super) site_page: Option<&'a str>,
    pub(super) content_type: Option<&'a str>,
    pub(super) after_id: Option<Uuid>,
    pub(super) limit: i64,
    pub(super) now: OffsetDateTime,
}

/// Konten situs yang sedang tayang, urut jadwal tayang terbaru, cursor = ID konten terakhir halaman
/// sebelumnya. Cursor dinilai dengan syarat tayang yang sama, sehingga ID draf/terjadwal/terhapus sebagai
/// cursor menghasilkan daftar kosong yang sama dengan cursor basi (tidak membocorkan jadwal, US-1801 AC3).
pub(super) async fn live_site_items(
    pool: &PgPool,
    f: &PublicFilter<'_>,
) -> Result<Vec<ContentRow>, sqlx::Error> {
    sqlx::query_as!(
        ContentRow,
        "SELECT c.id, c.created_at, c.updated_at, c.deleted_at, c.delete_reason, c.content_type, c.title, \
                c.body, c.category, c.image_file_id, c.image_alt_text, c.link_target_type, \
                c.link_target_ref, c.publish_start_at, c.publish_end_at, c.editorial_status, c.channel, \
                c.site_page \
         FROM app.cms_content c \
         WHERE c.channel IN ('site', 'app_and_site') AND c.editorial_status = 'scheduled' \
           AND c.deleted_at IS NULL AND c.publish_start_at <= $1 \
           AND (c.publish_end_at IS NULL OR c.publish_end_at > $1) \
           AND ($2::text IS NULL OR c.site_page = $2) \
           AND ($3::text IS NULL OR c.content_type = $3) \
           AND ($4::uuid IS NULL OR (c.publish_start_at, c.id) < \
                (SELECT p.publish_start_at, p.id FROM app.cms_content p \
                  WHERE p.id = $4 AND p.channel IN ('site', 'app_and_site') \
                    AND p.editorial_status = 'scheduled' AND p.deleted_at IS NULL \
                    AND p.publish_start_at <= $1 \
                    AND (p.publish_end_at IS NULL OR p.publish_end_at > $1))) \
         ORDER BY c.publish_start_at DESC, c.id DESC \
         LIMIT $5",
        f.now,
        f.site_page,
        f.content_type,
        f.after_id,
        f.limit,
    )
    .fetch_all(pool)
    .await
}

// ---------------------------------------------------------------- gambar konten (F1-T93)

/// Memasang gambar dan teks alternatif ke draf aktif.
pub(super) async fn set_image(
    conn: &mut PgConnection,
    id: Uuid,
    actor: Uuid,
    image_file_id: Uuid,
    image_alt_text: &str,
) -> Result<bool, sqlx::Error> {
    let r = sqlx::query!(
        "UPDATE app.cms_content SET image_file_id = $3, image_alt_text = $4, updated_at = now(), \
                updated_by = $2 \
         WHERE id = $1 AND deleted_at IS NULL AND editorial_status = 'draft'",
        id,
        actor,
        image_file_id,
        image_alt_text,
    )
    .execute(&mut *conn)
    .await?;
    Ok(r.rows_affected() == 1)
}

/// Apakah berkas dipakai konten yang sedang tayang di situs (syarat tayang sama dengan jalur publik).
pub(super) async fn image_is_live_on_site(
    pool: &PgPool,
    image_file_id: Uuid,
    now: OffsetDateTime,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar!(
        r#"SELECT EXISTS (
             SELECT 1 FROM app.cms_content
              WHERE image_file_id = $1 AND channel IN ('site', 'app_and_site')
                AND editorial_status = 'scheduled' AND deleted_at IS NULL AND publish_start_at <= $2
                AND (publish_end_at IS NULL OR publish_end_at > $2)) AS "live!""#,
        image_file_id,
        now,
    )
    .fetch_one(pool)
    .await
}
