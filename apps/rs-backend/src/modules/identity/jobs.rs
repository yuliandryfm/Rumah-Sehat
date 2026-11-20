//! Pengambil event Keycloak (F1-T34; ADR-009 butir 2, ADR-011 butir 10, skema fisik §6.4 kunci 7001002).
//!
//! Satu pengambil aktif: kunci advisory `platform_jobs::lock::RS_KEYCLOAK_EVENT_PULLER` dipegang di koneksi
//! khusus, dan posisi baca (`idp_event_cursor`) dibaca serta dimajukan **lewat koneksi itu**, sehingga
//! instans yang kehilangan kunci tidak dapat memajukan posisi. Event ditulis per event dengan
//! [`audit::in_audited_tx`] (pustaka memegang transaksi): tanda terima `idp_event_receipt` (UNIQUE
//! realm + ID event Keycloak) dan baris `audit_outbox` ter-commit atau batal bersama. Tanda terima itulah
//! pengaman terakhir terhadap duplikasi bila dua pengambil sempat berjalan bersamaan atau halaman dibaca
//! ulang.
//!
//! Algoritma per realm (urutan Keycloak hanya menurut `time`, tanpa pemecah seri; SRC-620):
//! 1. `from = posisi − tumpang-tindih` (bawaan 120 detik: event yang commit terlambat atau jam node
//!    Keycloak yang sedikit berbeda tetap terambil; event yang sama dilewati lewat tanda terima).
//! 2. Ambil halaman `time >= from` urut naik. Halaman tidak penuh: semua event sampai akhir sudah dibaca,
//!    posisi = waktu event terakhir, putaran selesai.
//! 3. Halaman penuh dengan waktu terakhir `t > from`: semua event `< t` pasti ada di halaman; posisi = `t`,
//!    halaman berikutnya `from = t` (event lain di milidetik `t` terbaca ulang lalu dilewati).
//! 4. Halaman penuh yang seluruhnya di milidetik `from`: ambil `dateFrom = dateTo = from` dengan batas
//!    [`SAME_MS_LIMIT`]; bila batas itu pun penuh, putaran gagal tanpa memajukan posisi (peringatan).
//!
//! Posisi dimajukan setelah setiap halaman (tahan restart: event setelah posisi tersimpan dibaca ulang dan
//! dilewati lewat tanda terima). Batas yang diketahui: event yang muncul di penyimpanan Keycloak lebih dari
//! `overlap` setelah waktunya sendiri tidak terambil (`[TIDAK DIKETAHUI]` lama commit event Keycloak di
//! klaster 2 node; nilai bawaan dapat dinaikkan lewat konfigurasi).
//!
//! Isi event yang dicatat: kode E-01–E-04, nama aksi, aktor (`account.id` bila subjek Keycloak terpetakan ke
//! akun aktif; bila tidak, aktor `system` tanpa ID), objek `account`, alamat IP dan komponen sumber, hasil
//! dan kode alasan dari daftar izin, waktu event Keycloak sebagai `occurred_at`. Email/username, token, ID
//! sesi, dan teks alasan Keycloak tidak pernah dibaca (lihat `domain`).
//!
//! Event yang tidak dapat dicatat (bentuk tidak valid, atau ditolak pustaka audit, misalnya akun dikenal
//! tanpa alamat IP yang diwajibkan PRD §7.2) menghentikan putaran realm itu di event tersebut tanpa
//! memajukan posisi (gagal-aman: tidak ada event yang dilewati diam-diam), dengan security log
//! `idp_event_pull_failed` dan, setelah ambang, `idp_event_pull_delayed`.
//!
//! Keterlambatan: bila putaran terakhir yang berhasil untuk suatu realm lebih tua dari ambang (bawaan 5
//! menit, ADR-011 butir 10), security log `idp_event_pull_delayed` paling sering sekali per menit.

use super::domain::{IdpRealm, InvalidEvent, KcEvent, Mapped, RawEvent};
use super::keycloak::{KeycloakClient, KeycloakError};
use super::repo::{self, AccountRef};
use audit::{
    Actor, AuditContext, AuditErrorKind, AuditEvent, AuditedTxError, Code, ObjectPolicy, ObjectRef,
    Outcome, Source,
};
use foundation::{Clock, CorrelationId, FixedClock, Sensitive};
use observability::security::{self, IdpEventPullFailure};
use platform_jobs::lock::RS_KEYCLOAK_EVENT_PULLER;
use platform_jobs::{LockAttempt, SingletonLock};
use sqlx::postgres::PgPoolOptions;
use sqlx::{Connection, PgConnection, PgPool};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use std::time::Duration;
use time::OffsetDateTime;

/// Batas event dalam satu milidetik yang dapat ditangani (langkah 4 algoritma).
pub const SAME_MS_LIMIT: usize = 5_000;
/// Batas halaman per putaran per realm; sisanya dilanjutkan putaran berikutnya.
pub const MAX_PAGES_PER_ROUND: usize = 200;
/// Rentang ukuran halaman yang diterima.
pub const PAGE_SIZE_RANGE: std::ops::RangeInclusive<usize> = 1..=500;
/// Tanda terima disimpan sekian lama di luar jendela tumpang-tindih sebelum dihapus.
pub const RECEIPT_MARGIN: time::Duration = time::Duration::hours(1);
/// Jeda minimum antar-peringatan keterlambatan per realm.
const LAG_ALERT_REPEAT: time::Duration = time::Duration::minutes(1);

/// Objek event autentikasi: akun aplikasi (tanpa field before/after).
static ACCOUNT_OBJECT: ObjectPolicy = ObjectPolicy::non_clinical("account", &[]);

/// Pengaturan putaran pengambilan.
#[derive(Debug, Clone)]
pub struct PullOptions {
    /// Realm yang diambil, urut.
    pub realms: Vec<IdpRealm>,
    /// Ukuran halaman (`max` Admin REST), [`PAGE_SIZE_RANGE`].
    pub page_size: usize,
    /// Jendela tumpang-tindih pembacaan ulang.
    pub overlap: time::Duration,
    /// Ambang peringatan keterlambatan.
    pub lag_alert: time::Duration,
}

/// Galat pengambilan. Tidak membawa isi event, respons, atau pesan database.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PullError {
    /// Admin REST Keycloak gagal atau tidak tersedia.
    Keycloak(KeycloakError),
    /// Event dari Keycloak tidak valid (nama field).
    InvalidEvent(&'static str),
    /// Event ditolak pustaka audit atau `audit_outbox` gagal ditulis.
    Audit(AuditErrorKind),
    /// Query database gagal (SQLSTATE bila ada).
    Database {
        /// SQLSTATE.
        sqlstate: Option<String>,
    },
    /// Lebih dari [`SAME_MS_LIMIT`] event di satu milidetik.
    TooManyEventsInOneMillisecond,
    /// Kunci yang diberikan bukan kunci pengambil event (7001002).
    WrongLock,
}

impl fmt::Display for PullError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Keycloak(e) => write!(f, "{e}"),
            Self::InvalidEvent(field) => write!(f, "event Keycloak tidak valid: {field}"),
            Self::Audit(kind) => write!(f, "event audit gagal ditulis: {kind:?}"),
            Self::Database { sqlstate } => write!(
                f,
                "database gagal (SQLSTATE {})",
                sqlstate.as_deref().unwrap_or("-")
            ),
            Self::TooManyEventsInOneMillisecond => {
                write!(f, "lebih dari {SAME_MS_LIMIT} event dalam satu milidetik")
            }
            Self::WrongLock => f.write_str("kunci bukan milik pengambil event Keycloak"),
        }
    }
}

impl std::error::Error for PullError {}

impl PullError {
    fn stage(&self) -> IdpEventPullFailure {
        match self {
            Self::Keycloak(KeycloakError::Token(_)) => IdpEventPullFailure::KeycloakAuth,
            Self::Keycloak(_) => IdpEventPullFailure::KeycloakUnavailable,
            Self::InvalidEvent(_) => IdpEventPullFailure::InvalidEvent,
            Self::Audit(_) => IdpEventPullFailure::AuditWrite,
            Self::Database { .. } | Self::WrongLock => IdpEventPullFailure::Database,
            Self::TooManyEventsInOneMillisecond => IdpEventPullFailure::TooManyEvents,
        }
    }
}

fn db_err(e: &sqlx::Error) -> PullError {
    PullError::Database {
        sqlstate: e
            .as_database_error()
            .and_then(|d| d.code().map(|c| c.into_owned())),
    }
}

/// Hasil satu putaran satu realm.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RoundReport {
    /// Event yang diterima dari Keycloak (termasuk bacaan ulang).
    pub fetched: usize,
    /// Event baru yang ditulis ke `audit_outbox`.
    pub written: usize,
    /// Event yang sudah pernah dicatat (dilewati).
    pub duplicates: usize,
    /// Event berjenis di luar pemetaan (dilewati).
    pub ignored: usize,
    /// Posisi baca setelah putaran.
    pub position_ms: i64,
}

impl RoundReport {
    fn add(&mut self, o: RecordOutcome) {
        match o {
            RecordOutcome::Written => self.written += 1,
            RecordOutcome::Duplicate => self.duplicates += 1,
            RecordOutcome::Ignored => self.ignored += 1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RecordOutcome {
    Written,
    Duplicate,
    Ignored,
}

/// Galat di dalam transaksi teraudit.
#[derive(Debug)]
enum TxWork {
    Duplicate,
    Db(sqlx::Error),
    Audit(AuditErrorKind),
}

impl fmt::Display for TxWork {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Duplicate => f.write_str("duplikat"),
            Self::Db(_) => f.write_str("database"),
            Self::Audit(k) => write!(f, "audit {k:?}"),
        }
    }
}

/// Pengambil event dengan pemantau keterlambatan per realm.
#[derive(Debug)]
pub struct Puller {
    options: PullOptions,
    client: Arc<KeycloakClient>,
    last_ok: HashMap<IdpRealm, OffsetDateTime>,
    last_alert: HashMap<IdpRealm, OffsetDateTime>,
    started: OffsetDateTime,
}

impl Puller {
    /// Pengambil baru; `started` = waktu mulai proses (acuan keterlambatan bila realm belum pernah
    /// berhasil diambil).
    pub fn new(options: PullOptions, client: Arc<KeycloakClient>, started: OffsetDateTime) -> Self {
        Self {
            options,
            client,
            last_ok: HashMap::new(),
            last_alert: HashMap::new(),
            started,
        }
    }

    /// Satu putaran untuk semua realm. Galat per realm dicatat ke security log tanpa menghentikan realm
    /// lain dan tanpa memajukan posisinya; `Err` hanya bila koneksi kunci tidak dapat dipakai lagi (sesi
    /// harus diulang dan kunci diambil ulang).
    pub async fn round(
        &mut self,
        lock: &mut SingletonLock,
        pool: &PgPool,
        clock: &dyn Clock,
    ) -> Result<Vec<(IdpRealm, Result<RoundReport, PullError>)>, PullError> {
        if lock.key() != RS_KEYCLOAK_EVENT_PULLER {
            return Err(PullError::WrongLock);
        }
        let mut out = Vec::with_capacity(self.options.realms.len());
        for realm in self.options.realms.clone() {
            if !self.last_ok.contains_key(&realm) {
                let cursor = repo::load_cursor(lock.connection(), realm)
                    .await
                    .map_err(|e| db_err(&e))?;
                self.last_ok
                    .insert(realm, cursor.last_pulled_at.unwrap_or(self.started));
            }
            let result = pull_realm(lock, pool, &self.client, realm, &self.options, clock).await;
            match &result {
                Ok(r) => {
                    self.last_ok.insert(realm, clock.now());
                    if r.fetched > 0 {
                        tracing::info!(
                            realm = realm.as_str(),
                            fetched = r.fetched,
                            written = r.written,
                            duplicates = r.duplicates,
                            ignored = r.ignored,
                            position_ms = r.position_ms,
                            "keycloak_event_pull_round"
                        );
                    }
                }
                Err(e) => {
                    security::idp_event_pull_failed(realm.as_str(), e.stage());
                    if matches!(e, PullError::Database { .. }) {
                        // Koneksi kunci mungkin putus: hentikan sesi supaya kunci diambil ulang.
                        self.check_lag(realm, clock.now());
                        return Err(e.clone());
                    }
                }
            }
            self.check_lag(realm, clock.now());
            out.push((realm, result));
        }
        Ok(out)
    }

    fn check_lag(&mut self, realm: IdpRealm, now: OffsetDateTime) {
        let last = self.last_ok.get(&realm).copied().unwrap_or(self.started);
        let lag = now - last;
        if lag <= self.options.lag_alert {
            return;
        }
        if self
            .last_alert
            .get(&realm)
            .is_none_or(|t| now - *t >= LAG_ALERT_REPEAT)
        {
            security::idp_event_pull_delayed(
                realm.as_str(),
                u64::try_from(lag.whole_seconds()).unwrap_or(0),
            );
            self.last_alert.insert(realm, now);
        }
    }
}

/// Satu putaran satu realm (lihat dokumentasi modul).
pub async fn pull_realm(
    lock: &mut SingletonLock,
    pool: &PgPool,
    client: &KeycloakClient,
    realm: IdpRealm,
    options: &PullOptions,
    clock: &dyn Clock,
) -> Result<RoundReport, PullError> {
    if lock.key() != RS_KEYCLOAK_EVENT_PULLER {
        return Err(PullError::WrongLock);
    }
    let page_size = options
        .page_size
        .clamp(*PAGE_SIZE_RANGE.start(), *PAGE_SIZE_RANGE.end());
    let overlap_ms = i64::try_from(options.overlap.whole_milliseconds())
        .unwrap_or(0)
        .max(0);
    let cursor = repo::load_cursor(lock.connection(), realm)
        .await
        .map_err(|e| db_err(&e))?;
    let mut position = cursor.position_ms;
    let mut from = position.saturating_sub(overlap_ms).max(0);
    let mut report = RoundReport::default();
    let correlation = CorrelationId::new();
    for _ in 0..MAX_PAGES_PER_ROUND {
        let page = client
            .fetch_events(realm, from, None, page_size, clock)
            .await
            .map_err(PullError::Keycloak)?;
        let n = page.events.len();
        let last = process(pool, realm, page.events, correlation, &mut report).await?;
        match last {
            Some(t) if n >= page_size && t > from => {
                position = position.max(t);
                from = t;
            }
            Some(t) if n >= page_size => {
                // Halaman penuh di satu milidetik `t == from`.
                let all = client
                    .fetch_events(realm, t, Some(t), SAME_MS_LIMIT, clock)
                    .await
                    .map_err(PullError::Keycloak)?;
                if all.events.len() >= SAME_MS_LIMIT {
                    return Err(PullError::TooManyEventsInOneMillisecond);
                }
                process(pool, realm, all.events, correlation, &mut report).await?;
                position = position.max(t);
                from = t.saturating_add(1);
            }
            _ => {
                if let Some(t) = last {
                    position = position.max(t);
                }
                let now = clock.now();
                repo::advance_cursor(lock.connection(), realm, position, Some(now), now)
                    .await
                    .map_err(|e| db_err(&e))?;
                let cutoff_ms = position.saturating_sub(overlap_ms);
                if let Ok(cutoff) =
                    OffsetDateTime::from_unix_timestamp_nanos(i128::from(cutoff_ms) * 1_000_000)
                {
                    repo::prune_receipts(lock.connection(), realm, cutoff - RECEIPT_MARGIN)
                        .await
                        .map_err(|e| db_err(&e))?;
                }
                report.position_ms = position;
                return Ok(report);
            }
        }
        repo::advance_cursor(lock.connection(), realm, position, None, clock.now())
            .await
            .map_err(|e| db_err(&e))?;
    }
    report.position_ms = position;
    Ok(report)
}

/// Memvalidasi dan mencatat satu halaman; mengembalikan waktu event terbesar.
async fn process(
    pool: &PgPool,
    realm: IdpRealm,
    raw: Vec<RawEvent>,
    correlation: CorrelationId,
    report: &mut RoundReport,
) -> Result<Option<i64>, PullError> {
    let mut last = None;
    report.fetched += raw.len();
    for r in raw {
        let ev = r
            .validate(realm)
            .map_err(|InvalidEvent(f)| PullError::InvalidEvent(f))?;
        last = Some(last.map_or(ev.time_ms, |l: i64| l.max(ev.time_ms)));
        report.add(record(pool, realm, ev, correlation).await?);
    }
    Ok(last)
}

/// Mencatat satu event: tanda terima + event audit dalam satu transaksi teraudit.
async fn record(
    pool: &PgPool,
    realm: IdpRealm,
    ev: KcEvent,
    correlation: CorrelationId,
) -> Result<RecordOutcome, PullError> {
    let Some(mapped) = ev.mapped else {
        return Ok(RecordOutcome::Ignored);
    };
    // occurred_at = waktu event di server Keycloak (ADR-009 butir 6), bukan waktu pengambilan.
    let at = FixedClock::new(ev.time);
    let result = audit::in_audited_tx(pool, &at, move |conn| {
        Box::pin(async move {
            if !repo::insert_receipt(conn, realm, ev.id, ev.time)
                .await
                .map_err(TxWork::Db)?
            {
                return Err(TxWork::Duplicate);
            }
            let account = match ev.user_id.as_deref() {
                Some(subject) => repo::find_account(conn, realm, subject)
                    .await
                    .map_err(TxWork::Db)?,
                None => None,
            };
            let event =
                build_event(&ev, mapped, account.as_ref(), correlation).map_err(TxWork::Audit)?;
            Ok((account.map(|a| a.id), vec![event]))
        })
    })
    .await;
    match result {
        Ok(account_id) => {
            security::idp_event(
                realm.as_str(),
                mapped.code.as_str(),
                mapped.action,
                account_id,
                mapped.failure,
            );
            Ok(RecordOutcome::Written)
        }
        Err(AuditedTxError::Work(TxWork::Duplicate)) => Ok(RecordOutcome::Duplicate),
        Err(AuditedTxError::Work(TxWork::Db(e))) => Err(db_err(&e)),
        Err(AuditedTxError::Work(TxWork::Audit(k))) => Err(PullError::Audit(k)),
        Err(AuditedTxError::Audit(e)) => Err(PullError::Audit(e.kind())),
        Err(AuditedTxError::Database { sqlstate }) => Err(PullError::Database { sqlstate }),
    }
}

/// Menyusun event audit dari event Keycloak yang sudah dipetakan.
fn build_event(
    ev: &KcEvent,
    mapped: Mapped,
    account: Option<&AccountRef>,
    correlation: CorrelationId,
) -> Result<AuditEvent, AuditErrorKind> {
    let code = |s: &str| Code::new(s).map_err(|e| e.kind());
    let mut source = Source::component(ev.component);
    if let Some(ip) = ev.ip {
        source = source.ip(ip);
    }
    let ctx = AuditContext::new(correlation, source);
    let (actor, object) = match account {
        Some(a) => {
            let roles = a
                .roles
                .iter()
                .map(|r| code(r))
                .collect::<Result<Vec<_>, _>>()?;
            (
                Actor::account(a.id, code(&a.account_type)?, roles),
                ObjectRef::with_uuid(&ACCOUNT_OBJECT, a.id),
            )
        }
        // Subjek tidak dikenal (mis. login dengan email yang tidak terdaftar): tanpa ID dan tanpa email.
        None => (Actor::system(), ObjectRef::new(&ACCOUNT_OBJECT, None)),
    };
    let outcome = match mapped.failure {
        None => Outcome::success(),
        Some(reason) => Outcome::failure(code(reason)?),
    };
    Ok(
        AuditEvent::new(mapped.code, code(mapped.action)?, actor, object)
            .outcome(outcome)
            .context(&ctx),
    )
}

/// Pengaturan proses `rs-backend worker` untuk pengambil event.
pub struct PullerSettings {
    /// Putaran.
    pub options: PullOptions,
    /// Login `rs_worker` ke `ruang_sehat` (rahasia).
    pub database_url: Sensitive<String>,
    /// Jeda antar-putaran (bawaan 60 detik, ADR-011 butir 10 "setiap menit").
    pub poll_interval: Duration,
    /// Jeda coba ulang setelah galat sesi atau saat siaga.
    pub retry_interval: Duration,
}

impl fmt::Debug for PullerSettings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PullerSettings")
            .field("options", &self.options)
            .finish_non_exhaustive()
    }
}

/// Menjalankan pengambil sampai proses dihentikan: satu aktif (kunci 7001002), instans lain siaga.
pub async fn run_puller(
    settings: PullerSettings,
    client: Arc<KeycloakClient>,
    clock: Arc<dyn Clock>,
) {
    let pool = match PgPoolOptions::new()
        .max_connections(4)
        .connect_lazy(settings.database_url.expose())
    {
        Ok(p) => p,
        Err(_) => {
            tracing::error!("keycloak_event_puller_database_url_invalid");
            return;
        }
    };
    let mut puller = Puller::new(settings.options.clone(), client, clock.now());
    loop {
        if let Err(e) = session(&settings, &pool, &mut puller, clock.as_ref()).await {
            security::idp_event_pull_failed("-", e.stage());
            tracing::warn!(error = %e, "keycloak_event_puller_restart");
            // Database/kunci tidak tersedia juga menahan pengambilan: keterlambatan tetap diperingatkan.
            for realm in settings.options.realms.clone() {
                puller.check_lag(realm, clock.now());
            }
        }
        tokio::time::sleep(settings.retry_interval).await;
    }
}

async fn session(
    settings: &PullerSettings,
    pool: &PgPool,
    puller: &mut Puller,
    clock: &dyn Clock,
) -> Result<(), PullError> {
    let mut conn = PgConnection::connect(settings.database_url.expose())
        .await
        .map_err(|e| db_err(&e))?;
    let mut lock = loop {
        match SingletonLock::try_acquire(conn, RS_KEYCLOAK_EVENT_PULLER).await {
            Ok(LockAttempt::Acquired(l)) => break l,
            Ok(LockAttempt::Busy(c)) => {
                conn = c;
                tokio::time::sleep(settings.retry_interval).await;
            }
            Err(_) => return Err(PullError::Database { sqlstate: None }),
        }
    };
    loop {
        puller.round(&mut lock, pool, clock).await?;
        tokio::time::sleep(settings.poll_interval).await;
    }
}
