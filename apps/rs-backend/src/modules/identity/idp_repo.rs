//! Query penyediaan identitas (ADR-030; migrasi 20261125100000): akun `pending_provisioning`, niat
//! (`idp_provisioning_intent`, kelas T), riwayat status (R) dan proyeksinya (P) lewat [`IntentStatusStore`]
//! (jalur tulis status ADR-008, crate status-flow), serta kunci advisory transaksi per (realm, email). Tabel niat
//! hanya `INSERT`; status hanya lewat `status_flow::transition`.

use super::domain::IdpRealm;
use super::idp_domain::{
    EMAIL_LOCK_NAMESPACE, IdpJobKind, IntentPurpose, IntentStatus, email_lock_key, normalize_email,
};
use foundation::{CorrelationId, Uuid};
use sqlx::PgConnection;
use status_flow::{Appended, Current, StatusStore, Step};
use time::OffsetDateTime;

/// Galat query (tanpa pesan database).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepoError {
    /// Galat database (SQLSTATE bila ada).
    Database(Option<String>),
    /// Nilai kolom di luar daftar yang dikenal kode.
    Corrupt,
}

impl std::fmt::Display for RepoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Database(Some(s)) => write!(f, "database (SQLSTATE {s})"),
            Self::Database(None) => f.write_str("database"),
            Self::Corrupt => f.write_str("nilai kolom tidak dikenal"),
        }
    }
}

pub(super) fn map_db(e: sqlx::Error) -> RepoError {
    RepoError::Database(
        e.as_database_error()
            .and_then(|d| d.code())
            .map(|c| c.into_owned()),
    )
}

/// Kunci advisory transaksi per (realm, email ternormalisasi) (ADR-030 butir 5, KEPUTUSAN-305). Dilepas saat
/// transaksi selesai.
pub(super) async fn lock_email(
    conn: &mut PgConnection,
    realm: IdpRealm,
    email: &str,
) -> Result<(), RepoError> {
    sqlx::query_scalar!(
        r#"SELECT 1::int4 AS "locked!" FROM pg_advisory_xact_lock($1::int4, $2::int4)"#,
        EMAIL_LOCK_NAMESPACE,
        email_lock_key(realm, email),
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(map_db)?;
    Ok(())
}

/// Ada akun belum terhapus dengan email yang sama (ternormalisasi) di realm itu.
pub(super) async fn live_account_with_email(
    conn: &mut PgConnection,
    realm: IdpRealm,
    email: &str,
) -> Result<bool, RepoError> {
    sqlx::query_scalar!(
        r#"SELECT EXISTS (
             SELECT 1 FROM app.account
              WHERE idp_realm = $1 AND lower(email) = $2 AND deleted_at IS NULL
           ) AS "exists!""#,
        realm.as_str(),
        normalize_email(email),
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(map_db)
}

/// Akun baru berstatus `pending_provisioning` tanpa `idp_subject`.
pub(super) struct NewPendingAccount<'a> {
    pub(super) id: Uuid,
    pub(super) account_type: &'a str,
    pub(super) realm: IdpRealm,
    pub(super) email: &'a str,
    pub(super) full_name: Option<&'a str>,
    pub(super) created_by: Option<Uuid>,
    pub(super) now: OffsetDateTime,
}

pub(super) async fn insert_pending_account(
    conn: &mut PgConnection,
    a: &NewPendingAccount<'_>,
) -> Result<(), RepoError> {
    sqlx::query!(
        "INSERT INTO app.account (id, created_by, updated_by, account_type, idp_realm, idp_subject, email, \
                                  status, status_changed_at, full_name) \
         VALUES ($1, $2, $2, $3, $4, NULL, $5, 'pending_provisioning', $6, $7)",
        a.id,
        a.created_by,
        a.account_type,
        a.realm.as_str(),
        normalize_email(a.email),
        a.now,
        a.full_name,
    )
    .execute(&mut *conn)
    .await
    .map_err(map_db)?;
    Ok(())
}

/// Niat baru.
pub(super) struct NewIntent {
    pub(super) id: Uuid,
    pub(super) created_by: Option<Uuid>,
    pub(super) account_id: Uuid,
    pub(super) realm: IdpRealm,
    pub(super) kind: IdpJobKind,
    pub(super) purpose: IntentPurpose,
    pub(super) origin_intent_id: Option<Uuid>,
    pub(super) correlation_id: CorrelationId,
}

pub(super) async fn insert_intent(conn: &mut PgConnection, i: &NewIntent) -> Result<(), RepoError> {
    sqlx::query!(
        "INSERT INTO app.idp_provisioning_intent \
             (id, created_by, account_id, idp_realm, operation, purpose, origin_intent_id, correlation_id) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        i.id,
        i.created_by,
        i.account_id,
        i.realm.as_str(),
        i.kind.operation(),
        i.purpose.as_str(),
        i.origin_intent_id,
        i.correlation_id.as_uuid(),
    )
    .execute(&mut *conn)
    .await
    .map_err(map_db)?;
    Ok(())
}

/// Niat beserta status dan keadaan akunnya, seperti tersimpan.
#[derive(Debug, Clone)]
pub(super) struct IntentRow {
    pub(super) id: Uuid,
    pub(super) account_id: Uuid,
    pub(super) realm: String,
    pub(super) operation: String,
    pub(super) purpose: String,
    pub(super) origin_intent_id: Option<Uuid>,
    pub(super) correlation_id: Uuid,
    pub(super) status: Option<String>,
    pub(super) account_status: String,
    /// Realm akun; sama dengan `realm` niat menurut FK komposit, diperiksa lagi worker sebelum memanggil Keycloak.
    pub(super) account_realm: String,
    pub(super) account_deleted: bool,
    pub(super) idp_subject: Option<String>,
}

pub(super) async fn load_intent(
    conn: &mut PgConnection,
    id: Uuid,
) -> Result<Option<IntentRow>, RepoError> {
    sqlx::query_as!(
        IntentRow,
        r#"SELECT i.id, i.account_id, i.idp_realm AS realm, i.operation, i.purpose, i.origin_intent_id,
                  i.correlation_id, s.status AS "status?", a.status AS account_status,
                  a.idp_realm AS account_realm,
                  (a.deleted_at IS NOT NULL) AS "account_deleted!", a.idp_subject
             FROM app.idp_provisioning_intent i
             JOIN app.account a ON a.id = i.account_id
             LEFT JOIN app.idp_provisioning_intent_state s ON s.idp_provisioning_intent_id = i.id
            WHERE i.id = $1"#,
        id,
    )
    .fetch_optional(&mut *conn)
    .await
    .map_err(map_db)
}

/// Email akun (salinan, KEPUTUSAN-45) untuk `POST users`/pencarian; dibaca saat dibutuhkan, tidak disalin ke niat
/// atau job.
pub(super) async fn account_email(
    conn: &mut PgConnection,
    account_id: Uuid,
) -> Result<Option<String>, RepoError> {
    sqlx::query_scalar!("SELECT email FROM app.account WHERE id = $1", account_id)
        .fetch_optional(&mut *conn)
        .await
        .map_err(map_db)
}

/// Operasi dan status niat asal.
pub(super) async fn origin_state(
    conn: &mut PgConnection,
    origin: Uuid,
) -> Result<Option<(String, Option<String>)>, RepoError> {
    let row = sqlx::query!(
        r#"SELECT i.operation, s.status AS "status?"
             FROM app.idp_provisioning_intent i
             LEFT JOIN app.idp_provisioning_intent_state s ON s.idp_provisioning_intent_id = i.id
            WHERE i.id = $1"#,
        origin,
    )
    .fetch_optional(&mut *conn)
    .await
    .map_err(map_db)?;
    Ok(row.map(|r| (r.operation, r.status)))
}

/// `pending_provisioning` → `pending_activation` dengan `idp_subject` (ADR-030 butir 5). `false` bila akun tidak
/// lagi menunggu penyediaan.
pub(super) async fn adopt_subject(
    conn: &mut PgConnection,
    account_id: Uuid,
    subject: &str,
    now: OffsetDateTime,
) -> Result<bool, RepoError> {
    let n = sqlx::query!(
        "UPDATE app.account \
            SET idp_subject = $2, status = 'pending_activation', status_changed_at = $3, updated_at = now() \
          WHERE id = $1 AND status = 'pending_provisioning' AND idp_subject IS NULL AND deleted_at IS NULL",
        account_id,
        subject,
        now,
    )
    .execute(&mut *conn)
    .await
    .map_err(map_db)?
    .rows_affected();
    Ok(n == 1)
}

/// Hapus lunak akun yang masih `pending_provisioning` (penyediaan gagal atau email sudah dipakai; ADR-030 butir
/// 5, ADR-003). Email kembali dapat dipakai mendaftar.
pub(super) async fn abandon_pending_account(
    conn: &mut PgConnection,
    account_id: Uuid,
    reason: &str,
) -> Result<bool, RepoError> {
    let n = sqlx::query!(
        "UPDATE app.account SET deleted_at = now(), delete_reason = $2, updated_at = now() \
          WHERE id = $1 AND status = 'pending_provisioning' AND deleted_at IS NULL",
        account_id,
        reason,
    )
    .execute(&mut *conn)
    .await
    .map_err(map_db)?
    .rows_affected();
    Ok(n == 1)
}

/// Isi event riwayat selain status.
#[derive(Debug, Clone)]
pub(super) struct IntentEventDetails {
    pub(super) event_id: Uuid,
    pub(super) reason_code: Option<&'static str>,
    pub(super) actor_type: &'static str,
    pub(super) actor_account_id: Option<Uuid>,
    pub(super) correlation_id: Uuid,
}

/// Riwayat dan proyeksi status satu niat di transaksi pemanggil.
pub(super) struct IntentStatusStore<'c> {
    pub(super) conn: &'c mut PgConnection,
    pub(super) intent_id: Uuid,
    pub(super) details: IntentEventDetails,
}

impl StatusStore<IntentStatus> for IntentStatusStore<'_> {
    type Error = RepoError;

    async fn current(&mut self) -> Result<Option<Current<IntentStatus>>, RepoError> {
        let row = sqlx::query!(
            "SELECT status, version FROM app.idp_provisioning_intent_state \
             WHERE idp_provisioning_intent_id = $1",
            self.intent_id,
        )
        .fetch_optional(&mut *self.conn)
        .await
        .map_err(map_db)?;
        row.map(|r| {
            Ok(Current {
                status: IntentStatus::parse(&r.status).ok_or(RepoError::Corrupt)?,
                version: r.version,
            })
        })
        .transpose()
    }

    async fn append(&mut self, step: Step<IntentStatus>) -> Result<Appended, RepoError> {
        // AB-36: versi yang sudah dipakai transaksi lain tidak disisipkan; status-flow mengevaluasi ulang.
        let n = sqlx::query!(
            "INSERT INTO app.idp_provisioning_intent_status_event \
                 (id, idp_provisioning_intent_id, version, from_status, to_status, reason_code, actor_type, \
                  actor_account_id, correlation_id) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) \
             ON CONFLICT ON CONSTRAINT uq_idp_intent_event_intent_id_version DO NOTHING",
            self.details.event_id,
            self.intent_id,
            step.version,
            step.from.map(IntentStatus::as_str),
            step.to.as_str(),
            self.details.reason_code,
            self.details.actor_type,
            self.details.actor_account_id,
            self.details.correlation_id,
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

    async fn project(&mut self, step: Step<IntentStatus>) -> Result<bool, RepoError> {
        let n = if step.version == 1 {
            sqlx::query!(
                "INSERT INTO app.idp_provisioning_intent_state (idp_provisioning_intent_id, status, version) \
                 VALUES ($1, $2, 1)",
                self.intent_id,
                step.to.as_str(),
            )
            .execute(&mut *self.conn)
            .await
        } else {
            sqlx::query!(
                "UPDATE app.idp_provisioning_intent_state \
                 SET status = $2, version = $3, updated_at = now() \
                 WHERE idp_provisioning_intent_id = $1 AND version = $3 - 1",
                self.intent_id,
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
