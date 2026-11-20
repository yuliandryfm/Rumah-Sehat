//! Use case CMS backoffice (US-1101 AC1-AC4, US-1802 AC1-AC5): draf, pratinjau (baca konten apa pun yang
//! tidak terhapus), jadwal, arsip, soft delete dan pulihkan (E-16, ADR-003).
//!
//! - Satu transaksi per use case (coding-standards §5.2); hapus dan pulihkan lewat `audit::in_audited_tx`
//!   sehingga event E-16 ter-commit bersama perubahannya (ADR-009 butir 2).
//! - Waktu hanya dari `Clock` (KEPUTUSAN-131); status "Tayang" dihitung terhadap jam server.
//! - Isi konten tidak pernah ditulis ke log (ADR-010); yang dicatat hanya ID konten.

use super::domain::{
    self, Channel, ContentFields, ContentType, DisplayStatus, EditorialStatus, FixedPagePlan,
    LinkTargetType, OtherVersion, PublishWindow, SitePage,
};
use super::error::ContentError;
use super::repo::{self, ContentRow, Fields};
use audit::{
    Actor, AuditContext, AuditEvent, AuditedTxError, Code, EventCode, FieldChange, FieldRule,
    ObjectPolicy, ObjectRef, Outcome, Value,
};
use authz::Principal;
use envelope::Meta;
use foundation::{Clock, Uuid, new_id};
use sqlx::PgPool;
use time::OffsetDateTime;

/// Daftar izin before/after objek `cms_content` di audit (ADR-009 butir 7): waktu hapus dicatat nilainya,
/// alasan hapus hanya namanya (teks bebas admin tidak masuk audit).
pub(super) static CMS_CONTENT_OBJECT: ObjectPolicy = ObjectPolicy::non_clinical(
    "cms_content",
    &[
        FieldRule::value("deleted_at"),
        FieldRule::name_only("delete_reason"),
    ],
);

/// Batas bawaan dan maksimum jumlah item per halaman daftar.
pub(super) const DEFAULT_PAGE_SIZE: i64 = 20;
pub(super) const MAX_PAGE_SIZE: i64 = 100;

/// Konten untuk backoffice (termasuk isi, untuk pratinjau).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ContentView {
    pub(super) id: Uuid,
    pub(super) content_type: ContentType,
    pub(super) title: String,
    pub(super) body: Option<String>,
    pub(super) category: Option<String>,
    pub(super) channel: Channel,
    pub(super) site_page: Option<SitePage>,
    pub(super) image_file_id: Option<Uuid>,
    pub(super) image_alt_text: Option<String>,
    pub(super) link_target_type: Option<LinkTargetType>,
    pub(super) link_target_ref: Option<String>,
    pub(super) publish_start_at: Option<OffsetDateTime>,
    pub(super) publish_end_at: Option<OffsetDateTime>,
    pub(super) editorial_status: EditorialStatus,
    pub(super) display_status: DisplayStatus,
    pub(super) created_at: OffsetDateTime,
    pub(super) updated_at: OffsetDateTime,
    pub(super) deleted_at: Option<OffsetDateTime>,
    pub(super) delete_reason: Option<String>,
}

fn parse<T>(v: Option<T>, what: &'static str) -> Result<T, ContentError> {
    v.ok_or(ContentError::Internal(what))
}

fn parse_opt<T>(
    v: Option<&str>,
    f: fn(&str) -> Option<T>,
    what: &'static str,
) -> Result<Option<T>, ContentError> {
    v.map(|s| parse(f(s), what)).transpose()
}

impl ContentView {
    pub(super) fn from_row(r: ContentRow, now: OffsetDateTime) -> Result<Self, ContentError> {
        let editorial_status = parse(
            EditorialStatus::parse(&r.editorial_status),
            "editorial_status",
        )?;
        let display_status = domain::display_status(
            editorial_status,
            r.publish_start_at,
            r.publish_end_at,
            r.deleted_at.is_some(),
            now,
        );
        Ok(Self {
            id: r.id,
            content_type: parse(ContentType::parse(&r.content_type), "content_type")?,
            title: r.title,
            body: r.body,
            category: r.category,
            channel: parse(Channel::parse(&r.channel), "channel")?,
            site_page: parse_opt(r.site_page.as_deref(), SitePage::parse, "site_page")?,
            image_file_id: r.image_file_id,
            image_alt_text: r.image_alt_text,
            link_target_type: parse_opt(
                r.link_target_type.as_deref(),
                LinkTargetType::parse,
                "link_target_type",
            )?,
            link_target_ref: r.link_target_ref,
            publish_start_at: r.publish_start_at,
            publish_end_at: r.publish_end_at,
            editorial_status,
            display_status,
            created_at: r.created_at,
            updated_at: r.updated_at,
            deleted_at: r.deleted_at,
            delete_reason: r.delete_reason,
        })
    }
}

fn fields(f: &ContentFields) -> Fields<'_> {
    Fields {
        content_type: f.content_type.as_str(),
        title: &f.title,
        body: f.body.as_deref(),
        category: f.category.as_deref(),
        image_alt_text: f.image_alt_text.as_deref(),
        link_target_type: f.link_target_type.map(LinkTargetType::as_str),
        link_target_ref: f.link_target_ref.as_deref(),
        channel: f.channel.as_str(),
        site_page: f.site_page.map(SitePage::as_str),
    }
}

/// Validasi isian draf: bentuk dan aturan kanal (domain), lalu aturan render aman isi (F1-T111).
fn validate(input: ContentFields) -> Result<ContentFields, ContentError> {
    input.normalized().map_err(ContentError::Validation)
}

/// AC1 (US-1101): konten baru selalu berstatus draf.
pub(super) async fn create(
    pool: &PgPool,
    clock: &dyn Clock,
    actor: &Principal,
    input: ContentFields,
) -> Result<ContentView, ContentError> {
    let f = validate(input)?;
    let id = new_id();
    repo::insert_draft(pool, id, actor.account_id(), &fields(&f)).await?;
    tracing::info!(cms_content_id = %id, "cms_content_created");
    get(pool, clock, id).await
}

/// Pratinjau dan detail: konten apa pun yang tidak terhapus (AC1 "dapat dipratinjau").
pub(super) async fn get(
    pool: &PgPool,
    clock: &dyn Clock,
    id: Uuid,
) -> Result<ContentView, ContentError> {
    let row = repo::find(pool, id).await?.ok_or(ContentError::NotFound)?;
    ContentView::from_row(row, clock.now())
}

/// Saringan daftar.
#[derive(Debug, Clone, Default)]
pub(super) struct ListQuery {
    pub(super) status: Option<DisplayStatus>,
    pub(super) channel: Option<Channel>,
    pub(super) site_page: Option<SitePage>,
    pub(super) cursor: Option<Uuid>,
    pub(super) limit: Option<i64>,
}

pub(super) async fn list(
    pool: &PgPool,
    clock: &dyn Clock,
    q: ListQuery,
) -> Result<(Vec<ContentView>, Meta), ContentError> {
    let limit = q.limit.unwrap_or(DEFAULT_PAGE_SIZE).clamp(1, MAX_PAGE_SIZE);
    let now = clock.now();
    let mut rows = repo::list(
        pool,
        &repo::ListFilter {
            status: q.status.map(DisplayStatus::as_str),
            channel: q.channel.map(Channel::as_str),
            site_page: q.site_page.map(SitePage::as_str),
            before_id: q.cursor,
            limit: limit.saturating_add(1),
            now,
        },
    )
    .await?;
    let has_more = i64::try_from(rows.len()).map_or(true, |n| n > limit);
    rows.truncate(usize::try_from(limit).map_err(|_| ContentError::Internal("limit"))?);
    let next_cursor = if has_more {
        rows.last().map(|r| r.id.hyphenated().to_string())
    } else {
        None
    };
    let items = rows
        .into_iter()
        .map(|r| ContentView::from_row(r, now))
        .collect::<Result<Vec<_>, _>>()?;
    Ok((
        items,
        Meta {
            next_cursor,
            has_more,
        },
    ))
}

/// Mengubah draf (status lain = 409; mengubah konten tayang = versi baru).
pub(super) async fn update(
    pool: &PgPool,
    clock: &dyn Clock,
    actor: &Principal,
    id: Uuid,
    input: ContentFields,
) -> Result<ContentView, ContentError> {
    let f = validate(input)?;
    let mut tx = pool.begin().await?;
    let row = repo::lock(&mut tx, id)
        .await?
        .filter(|r| r.deleted_at.is_none())
        .ok_or(ContentError::NotFound)?;
    let status = parse(
        EditorialStatus::parse(&row.editorial_status),
        "editorial_status",
    )?;
    if !domain::can_edit(status) {
        return Err(ContentError::StatusConflict);
    }
    if !repo::update_draft(&mut tx, id, actor.account_id(), &fields(&f)).await? {
        return Err(ContentError::StatusConflict);
    }
    tx.commit().await?;
    tracing::info!(cms_content_id = %id, "cms_content_updated");
    get(pool, clock, id).await
}

/// Menjadwalkan draf (US-1101 AC2): `start = None` = tayang sekarang. Gambar wajib bertekst alternatif
/// (US-1802 AC5). Halaman tetap situs: versi lama berakhir saat versi baru mulai (US-1802 AC2).
pub(super) async fn schedule(
    pool: &PgPool,
    clock: &dyn Clock,
    actor: &Principal,
    id: Uuid,
    start: Option<OffsetDateTime>,
    end: Option<OffsetDateTime>,
) -> Result<ContentView, ContentError> {
    let now = clock.now();
    let window = PublishWindow::new(start, end, now).map_err(ContentError::Validation)?;
    let mut tx = pool.begin().await?;
    let row = repo::lock(&mut tx, id)
        .await?
        .filter(|r| r.deleted_at.is_none())
        .ok_or(ContentError::NotFound)?;
    let view = ContentView::from_row(row, now)?;
    if !domain::can_schedule(view.editorial_status) {
        return Err(ContentError::StatusConflict);
    }
    domain::publish_requirements(view.image_file_id.is_some(), view.image_alt_text.as_deref())
        .map_err(ContentError::Validation)?;
    if let Some(page) = domain::fixed_site_page(view.content_type, view.site_page) {
        let page_no = SitePage::ALL
            .iter()
            .position(|p| *p == page)
            .and_then(|i| i32::try_from(i).ok())
            .ok_or(ContentError::Internal("site_page"))?;
        repo::lock_fixed_page(&mut tx, page_no).await?;
        let others = repo::scheduled_fixed_versions(&mut tx, page.as_str(), id)
            .await?
            .into_iter()
            .map(|v| {
                Ok(OtherVersion {
                    id: v.id,
                    start: parse(v.publish_start_at, "publish_start_at")?,
                    end: v.publish_end_at,
                })
            })
            .collect::<Result<Vec<_>, ContentError>>()?;
        match domain::plan_fixed_page(&window, &others) {
            FixedPagePlan::Conflict => return Err(ContentError::StatusConflict),
            FixedPagePlan::Proceed { end_previous } if !end_previous.is_empty() => {
                repo::end_versions(&mut tx, &end_previous, actor.account_id(), window.start)
                    .await?;
            }
            FixedPagePlan::Proceed { .. } => {}
        }
    }
    if !repo::schedule(&mut tx, id, actor.account_id(), window.start, window.end).await? {
        return Err(ContentError::StatusConflict);
    }
    tx.commit().await?;
    tracing::info!(cms_content_id = %id, "cms_content_scheduled");
    get(pool, clock, id).await
}

/// Mengarsipkan draf atau konten terjadwal (US-1101 AC2: tidak tampil lagi).
pub(super) async fn archive(
    pool: &PgPool,
    clock: &dyn Clock,
    actor: &Principal,
    id: Uuid,
) -> Result<ContentView, ContentError> {
    let mut tx = pool.begin().await?;
    let row = repo::lock(&mut tx, id)
        .await?
        .filter(|r| r.deleted_at.is_none())
        .ok_or(ContentError::NotFound)?;
    let status = parse(
        EditorialStatus::parse(&row.editorial_status),
        "editorial_status",
    )?;
    if !domain::can_archive(status) || !repo::archive(&mut tx, id, actor.account_id()).await? {
        return Err(ContentError::StatusConflict);
    }
    tx.commit().await?;
    tracing::info!(cms_content_id = %id, "cms_content_archived");
    get(pool, clock, id).await
}

fn audit_actor(p: &Principal) -> Result<Actor, ContentError> {
    let account_type =
        Code::new(p.account_type().as_str()).map_err(|_| ContentError::Internal("actor"))?;
    let roles = p
        .sub_roles()
        .iter()
        .map(|r| Code::new(r.as_str()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| ContentError::Internal("actor"))?;
    Ok(Actor::account(p.account_id(), account_type, roles))
}

/// Aksi E-16 atas konten.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SoftDeleteAction {
    Deleted,
    Restored,
}

/// Event E-16 (PRD §7.1) untuk soft delete atau pemulihan konten: waktu hapus sebagai nilai after (hapus)
/// atau before (pulihkan); alasan hanya nama field (teks bebas tidak masuk audit, ADR-009 butir 7).
pub(super) fn e16_event(
    action: SoftDeleteAction,
    actor: Actor,
    id: Uuid,
    deleted_at: OffsetDateTime,
    ctx: &AuditContext,
) -> Result<AuditEvent, ContentError> {
    let (name, before, after) = match action {
        SoftDeleteAction::Deleted => ("cms_content_deleted", None, Some(deleted_at)),
        SoftDeleteAction::Restored => ("cms_content_restored", Some(deleted_at), None),
    };
    let code = Code::new(name).map_err(|_| ContentError::Internal("code"))?;
    Ok(AuditEvent::new(
        EventCode::E16,
        code,
        actor,
        ObjectRef::with_uuid(&CMS_CONTENT_OBJECT, id),
    )
    .change(FieldChange::new(
        "deleted_at",
        before.map(Value::Timestamp),
        after.map(Value::Timestamp),
    ))
    .change(FieldChange::changed("delete_reason"))
    .outcome(Outcome::success())
    .context(ctx))
}

fn audited<T>(r: Result<T, AuditedTxError<ContentError>>) -> Result<T, ContentError> {
    r.map_err(|e| match e {
        AuditedTxError::Work(w) => w,
        AuditedTxError::Audit(_) => ContentError::Internal("audit"),
        AuditedTxError::Database { .. } => ContentError::Internal("database"),
    })
}

/// Soft delete dengan alasan wajib (US-1101 AC3, ADR-003) + E-16 dalam satu transaksi.
pub(super) async fn delete(
    pool: &PgPool,
    clock: &dyn Clock,
    actor: &Principal,
    ctx: AuditContext,
    id: Uuid,
    reason: &str,
) -> Result<(), ContentError> {
    let reason = domain::delete_reason(reason).map_err(ContentError::Validation)?;
    let audit_actor = audit_actor(actor)?;
    let actor_id = actor.account_id();
    audited(
        audit::in_audited_tx(pool, clock, move |conn| {
            Box::pin(async move {
                let deleted_at = repo::soft_delete(conn, id, actor_id, &reason)
                    .await?
                    .ok_or(ContentError::NotFound)?;
                let event =
                    e16_event(SoftDeleteAction::Deleted, audit_actor, id, deleted_at, &ctx)?;
                Ok(((), vec![event]))
            })
        })
        .await,
    )?;
    tracing::info!(cms_content_id = %id, "cms_content_deleted");
    Ok(())
}

/// Memulihkan konten terhapus (US-1101 AC3) + E-16. Status editorial tidak berubah; halaman tetap yang
/// bentrok dengan versi terbuka lain ditolak `409` (uqp, pertanyaan S-7 wireframe C14 masih terbuka).
pub(super) async fn restore(
    pool: &PgPool,
    clock: &dyn Clock,
    actor: &Principal,
    ctx: AuditContext,
    id: Uuid,
) -> Result<ContentView, ContentError> {
    let audit_actor = audit_actor(actor)?;
    let actor_id = actor.account_id();
    audited(
        audit::in_audited_tx(pool, clock, move |conn| {
            Box::pin(async move {
                let row = repo::lock(conn, id).await?.ok_or(ContentError::NotFound)?;
                let Some(deleted_at) = row.deleted_at else {
                    return Err(ContentError::NotFound);
                };
                if !repo::restore(conn, id, actor_id).await? {
                    return Err(ContentError::NotFound);
                }
                let event = e16_event(
                    SoftDeleteAction::Restored,
                    audit_actor,
                    id,
                    deleted_at,
                    &ctx,
                )?;
                Ok(((), vec![event]))
            })
        })
        .await,
    )?;
    tracing::info!(cms_content_id = %id, "cms_content_restored");
    get(pool, clock, id).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use audit::{Source, SourceComponent};
    use foundation::{CorrelationId, FixedClock};
    use std::net::{IpAddr, Ipv4Addr};
    use time::macros::datetime;

    fn ctx() -> AuditContext {
        AuditContext::new(
            CorrelationId::default(),
            Source::component(SourceComponent::Backoffice)
                .ip(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 7)))
                .user_agent("Mozilla/5.0 uji"),
        )
    }

    fn actor() -> Actor {
        Actor::account(
            Uuid::from_u128(0x0193_6f8e_9c01_7e02_8f03_1a2b_3c4d_5e0a),
            Code::new("rs_admin").unwrap(),
            vec![Code::new("content").unwrap()],
        )
    }

    // trace: US-1101 AC3
    #[test]
    fn deletion_and_restore_events_are_e16_without_reason_text() {
        let at = datetime!(2027-03-01 01:00:00 UTC);
        let clock = FixedClock::new(at);
        let id = Uuid::from_u128(0x0193_6f8e_9c01_7e02_8f03_1a2b_3c4d_5e0b);
        let del = e16_event(SoftDeleteAction::Deleted, actor(), id, at, &ctx())
            .unwrap()
            .finish(&clock)
            .unwrap();
        let p = del.payload();
        assert_eq!(p["event_code"], "e_16");
        assert_eq!(p["action_name"], "cms_content_deleted");
        assert_eq!(p["object_type"], "cms_content");
        assert_eq!(p["object_id"], id.hyphenated().to_string());
        assert_eq!(p["actor_account_type"], "rs_admin");
        assert_eq!(p["actor_roles"], serde_json::json!(["content"]));
        assert_eq!(p["source_component"], "backoffice");
        assert_eq!(p["source_ip"], "10.0.0.7");
        assert_eq!(p["outcome"], "success");
        assert_eq!(
            p["changed_fields"],
            serde_json::json!(["deleted_at", "delete_reason"])
        );
        assert_eq!(p["after_values"]["deleted_at"], "2027-03-01T01:00:00Z");
        assert!(p.get("before_values").is_none_or(|v| v.is_null()));
        assert!(p["after_values"].get("delete_reason").is_none());
        let res = e16_event(SoftDeleteAction::Restored, actor(), id, at, &ctx())
            .unwrap()
            .finish(&clock)
            .unwrap();
        let p = res.payload();
        assert_eq!(p["action_name"], "cms_content_restored");
        assert_eq!(p["before_values"]["deleted_at"], "2027-03-01T01:00:00Z");
    }

    #[test]
    fn audit_policy_is_valid_and_rejects_unknown_fields() {
        let at = datetime!(2027-03-01 01:00:00 UTC);
        let ev = AuditEvent::new(
            EventCode::E16,
            Code::new("cms_content_deleted").unwrap(),
            actor(),
            ObjectRef::with_uuid(&CMS_CONTENT_OBJECT, Uuid::from_u128(1)),
        )
        .change(FieldChange::new("title", None, Some(Value::text("x"))))
        .outcome(Outcome::success())
        .context(&ctx());
        assert!(ev.finish(&FixedClock::new(at)).is_err());
    }
}
