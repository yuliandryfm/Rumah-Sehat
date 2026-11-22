//! Event audit dengan field PRD §7.2 dan pembentukan payload `audit_outbox` yang sudah dimasking.
//!
//! Pemanggil menyusun [`AuditEvent`] dari bagian bertipe; [`AuditEvent::finish`] memvalidasi field
//! wajib, menerapkan daftar izin dan masking (§7.3), lalu menghasilkan [`AuditRecord`] berisi ID event
//! (UUIDv7), waktu server dari [`Clock`], dan payload JSON. Tidak ada tempat untuk password, PIN, OTP,
//! token, atau rahasia MFA di API ini (ADR-009 butir 7).

use crate::code::{Code, EventCode};
use crate::error::{AuditError, AuditErrorKind};
use crate::policy::{ContentClass, ObjectPolicy};
use crate::value::{FieldChange, mask, rfc3339};
use foundation::{Clock, CorrelationId, Uuid, new_id};
use serde::Serialize;
use serde_json::{Map, Value as Json};
use std::fmt;
use std::net::IpAddr;
use time::OffsetDateTime;

/// Versi bentuk payload `audit_outbox`; dinaikkan bila bentuknya berubah (dibaca relay, F1-T24).
pub const PAYLOAD_VERSION: u16 = 1;
/// Batas panjang user agent setelah disaring.
pub const USER_AGENT_MAX_CHARS: usize = 512;
/// Batas jumlah subjek per event; lebih dari ini dipecah pemanggil (mis. export besar).
pub const MAX_SUBJECTS: usize = 10_000;
/// Batas versi objek (6 digit).
pub const OBJECT_VERSION_MAX: i32 = 999_999;

/// Jenis aktor (DD B1 `actor_type`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActorType {
    /// Akun pengguna (pasien, tenaga kesehatan, admin, pengguna RME).
    Account,
    /// Proses otomatis (job, worker).
    System,
    /// Sistem rekanan atau fasyankes lewat integrasi.
    PartnerSystem,
    /// Sistem Ruang Sehat sebagai pengirim (mis. di audit RME).
    RuangSehatSystem,
}

impl ActorType {
    fn as_str(self) -> &'static str {
        match self {
            Self::Account => "account",
            Self::System => "system",
            Self::PartnerSystem => "partner_system",
            Self::RuangSehatSystem => "ruang_sehat_system",
        }
    }
}

/// Aktor event (PRD §7.2: ID akun, jenis akun, peran dan sub-peran, atau "sistem").
#[derive(Debug, Clone)]
pub struct Actor {
    kind: ActorType,
    account_id: Option<Uuid>,
    account_type: Option<Code>,
    roles: Vec<Code>,
    partner_id: Option<Uuid>,
}

impl Actor {
    /// Akun pengguna dengan jenis akun dan peran/sub-peran saat aksi.
    pub fn account(account_id: Uuid, account_type: Code, roles: Vec<Code>) -> Self {
        Self {
            kind: ActorType::Account,
            account_id: Some(account_id),
            account_type: Some(account_type),
            roles,
            partner_id: None,
        }
    }

    /// Proses otomatis.
    pub fn system() -> Self {
        Self {
            kind: ActorType::System,
            account_id: None,
            account_type: None,
            roles: Vec::new(),
            partner_id: None,
        }
    }

    /// Sistem rekanan/fasyankes pengirim integrasi.
    pub fn partner_system(partner_id: Uuid) -> Self {
        Self {
            kind: ActorType::PartnerSystem,
            account_id: None,
            account_type: None,
            roles: Vec::new(),
            partner_id: Some(partner_id),
        }
    }

    /// Sistem Ruang Sehat sebagai pengirim.
    pub fn ruang_sehat_system() -> Self {
        Self {
            kind: ActorType::RuangSehatSystem,
            account_id: None,
            account_type: None,
            roles: Vec::new(),
            partner_id: None,
        }
    }

    /// Jenis aktor.
    pub fn kind(&self) -> ActorType {
        self.kind
    }
}

/// ID objek (DD B1 `object_id` berupa teks): UUID atau kode tertutup (mis. jenis export), bukan teks bebas.
#[derive(Debug, Clone)]
pub enum ObjectId {
    /// UUID objek.
    Uuid(Uuid),
    /// Kode tertutup.
    Code(Code),
}

/// Objek yang terkena aksi: jenis (dari kebijakannya), ID, dan versi bila konten klinis/dokumen.
#[derive(Debug, Clone)]
pub struct ObjectRef {
    policy: &'static ObjectPolicy,
    id: Option<ObjectId>,
    version: Option<i32>,
}

impl ObjectRef {
    /// Objek berjenis `policy.object_type()`.
    pub fn new(policy: &'static ObjectPolicy, id: Option<ObjectId>) -> Self {
        Self {
            policy,
            id,
            version: None,
        }
    }

    /// Objek ber-UUID.
    pub fn with_uuid(policy: &'static ObjectPolicy, id: Uuid) -> Self {
        Self::new(policy, Some(ObjectId::Uuid(id)))
    }

    /// Versi konten klinis atau dokumen.
    pub fn version(mut self, version: i32) -> Self {
        self.version = Some(version);
        self
    }
}

/// Jenis subjek data (DD B3 `subject_kind`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SubjectKind {
    /// Penerima layanan Ruang Sehat (`care_recipient.id`).
    CareRecipient,
    /// Pasien RME (`rme_patient.id`).
    RmePatient,
}

impl SubjectKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::CareRecipient => "care_recipient",
            Self::RmePatient => "rme_patient",
        }
    }
}

/// Subjek data yang datanya terkena (PRD §7.2, BR-106, BR-107).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Subject {
    id: Uuid,
    kind: SubjectKind,
}

impl Subject {
    /// Penerima layanan Ruang Sehat.
    pub fn care_recipient(id: Uuid) -> Self {
        Self {
            id,
            kind: SubjectKind::CareRecipient,
        }
    }

    /// Pasien RME.
    pub fn rme_patient(id: Uuid) -> Self {
        Self {
            id,
            kind: SubjectKind::RmePatient,
        }
    }
}

/// Komponen sumber (DD B1 `source_component`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceComponent {
    /// Aplikasi pasien.
    PatientApp,
    /// Aplikasi tenaga kesehatan.
    PractitionerApp,
    /// Backoffice.
    Backoffice,
    /// Portal rekanan.
    PartnerPortal,
    /// Member area HR perusahaan `web/korporat` (realm `korporat`, ADR-027; F1-T100).
    CorporatePortal,
    /// RME minimalis.
    Rme,
    /// Worker/proses otomatis.
    Worker,
    /// Integrasi (Partner Integration API, webhook penyedia).
    Integration,
}

impl SourceComponent {
    fn as_str(self) -> &'static str {
        match self {
            Self::PatientApp => "patient_app",
            Self::PractitionerApp => "practitioner_app",
            Self::Backoffice => "backoffice",
            Self::PartnerPortal => "partner_portal",
            Self::CorporatePortal => "corporate_portal",
            Self::Rme => "rme",
            Self::Worker => "worker",
            Self::Integration => "integration",
        }
    }
}

/// Sumber aksi (PRD §7.2): alamat IP, komponen, versi aplikasi, perangkat atau user agent.
#[derive(Clone)]
pub struct Source {
    component: SourceComponent,
    ip: Option<IpAddr>,
    user_agent: Option<String>,
    app_version: Option<String>,
}

// Debug tanpa alamat IP dan user agent (data pribadi, ADR-010 butir 1).
impl fmt::Debug for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Source")
            .field("component", &self.component)
            .finish_non_exhaustive()
    }
}

impl Source {
    /// Sumber dengan komponen saja (proses otomatis).
    pub fn component(component: SourceComponent) -> Self {
        Self {
            component,
            ip: None,
            user_agent: None,
            app_version: None,
        }
    }

    /// Alamat IP peminta.
    pub fn ip(mut self, ip: IpAddr) -> Self {
        self.ip = Some(ip);
        self
    }

    /// User agent atau perangkat; disaring dan dipotong saat event dibentuk.
    pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = Some(user_agent.into());
        self
    }

    /// Versi aplikasi klien (`[0-9A-Za-z.+_-]`, 1–32 karakter; selain itu ditolak saat event dibentuk).
    pub fn app_version(mut self, version: impl Into<String>) -> Self {
        self.app_version = Some(version.into());
        self
    }
}

/// Konteks permintaan atau proses: sumber dan ID korelasi (PRD §7.2). Dibangun sekali per permintaan
/// oleh lapisan HTTP/worker (kelak dari `RequestContext`), bukan disusun ulang per handler.
#[derive(Debug, Clone)]
pub struct AuditContext {
    correlation_id: CorrelationId,
    source: Source,
}

impl AuditContext {
    /// Konteks dari ID korelasi dan sumber.
    pub fn new(correlation_id: CorrelationId, source: Source) -> Self {
        Self {
            correlation_id,
            source,
        }
    }
}

/// Hasil aksi (PRD §7.2: sukses atau gagal, dengan kode alasan).
#[derive(Debug, Clone)]
pub enum Outcome {
    /// Sukses, kode alasan opsional.
    Success(Option<Code>),
    /// Gagal, kode alasan wajib (mis. `access_denied`, `account_locked`).
    Failure(Code),
}

impl Outcome {
    /// Sukses tanpa kode alasan.
    pub fn success() -> Self {
        Self::Success(None)
    }

    /// Gagal dengan kode alasan.
    pub fn failure(reason: Code) -> Self {
        Self::Failure(reason)
    }
}

/// Pengiriman data pribadi ke pihak luar (E-17): tujuan dan jenis data, tanpa isi (BR-107).
#[derive(Debug, Clone)]
pub struct Disclosure {
    recipient: Code,
    data_categories: Vec<Code>,
}

impl Disclosure {
    /// Tujuan (mis. `satusehat`, `payment_gateway`, `partner_system`) dan jenis data.
    pub fn new(recipient: Code, data_categories: Vec<Code>) -> Self {
        Self {
            recipient,
            data_categories,
        }
    }
}

/// Event audit yang sedang disusun.
#[derive(Debug, Clone)]
pub struct AuditEvent {
    code: EventCode,
    action: Code,
    actor: Actor,
    object: ObjectRef,
    subjects: Vec<Subject>,
    tenant_id: Option<Uuid>,
    changes: Vec<FieldChange>,
    outcome: Option<Outcome>,
    context: Option<AuditContext>,
    disclosure: Option<Disclosure>,
}

impl AuditEvent {
    /// Event baru: kode §7.1, nama aksi, aktor, dan objek (semuanya wajib).
    pub fn new(code: EventCode, action: Code, actor: Actor, object: ObjectRef) -> Self {
        Self {
            code,
            action,
            actor,
            object,
            subjects: Vec::new(),
            tenant_id: None,
            changes: Vec::new(),
            outcome: None,
            context: None,
            disclosure: None,
        }
    }

    /// Menambah subjek data.
    pub fn subject(mut self, subject: Subject) -> Self {
        self.subjects.push(subject);
        self
    }

    /// Tenant (`partner_id` di Ruang Sehat, `fasyankes_id` di RME).
    pub fn tenant(mut self, tenant_id: Uuid) -> Self {
        self.tenant_id = Some(tenant_id);
        self
    }

    /// Menambah perubahan field (wajib ada di daftar izin jenis objek).
    pub fn change(mut self, change: FieldChange) -> Self {
        self.changes.push(change);
        self
    }

    /// Hasil aksi.
    pub fn outcome(mut self, outcome: Outcome) -> Self {
        self.outcome = Some(outcome);
        self
    }

    /// Sumber dan ID korelasi.
    pub fn context(mut self, context: &AuditContext) -> Self {
        self.context = Some(context.clone());
        self
    }

    /// Tujuan dan jenis data (hanya E-17, dan wajib untuk E-17).
    pub fn disclosure(mut self, disclosure: Disclosure) -> Self {
        self.disclosure = Some(disclosure);
        self
    }

    /// Kode event.
    pub fn code(&self) -> EventCode {
        self.code
    }

    /// Seperti [`AuditEvent::finish`] untuk penolakan akses ke data sensitif: hanya E-05 dengan hasil
    /// gagal, tanpa perubahan field dan tanpa pengungkapan. Selain itu ditolak
    /// (`InvalidCombination`/`failed_access_only`), sehingga event aksi bisnis atau akses berhasil tidak
    /// dapat ditulis di luar transaksi aksinya (KEPUTUSAN-283).
    pub fn finish_failed_access(self, clock: &dyn Clock) -> Result<FailedAccessRecord, AuditError> {
        let failed = matches!(self.outcome, Some(Outcome::Failure(_)));
        if self.code != EventCode::E05
            || !failed
            || !self.changes.is_empty()
            || self.disclosure.is_some()
        {
            return Err(AuditError::new(
                AuditErrorKind::InvalidCombination,
                "failed_access_only",
            ));
        }
        Ok(FailedAccessRecord(self.finish(clock)?))
    }

    /// Memvalidasi, memasking, dan membentuk record siap tulis. `clock` = jam server aplikasi
    /// (`occurred_at`, ADR-009 butir 6).
    pub fn finish(self, clock: &dyn Clock) -> Result<AuditRecord, AuditError> {
        let policy = self.object.policy;
        policy.validate()?;
        let context = self
            .context
            .ok_or(AuditError::new(AuditErrorKind::MissingField, "context"))?;
        let outcome = self
            .outcome
            .ok_or(AuditError::new(AuditErrorKind::MissingField, "outcome"))?;
        let source = context.source;
        // Versi objek kecil; nilai besar (mis. nomor rekening 10 digit sebagai versi) ditolak.
        if self
            .object
            .version
            .is_some_and(|v| !(0..=OBJECT_VERSION_MAX).contains(&v))
        {
            return Err(AuditError::new(
                AuditErrorKind::InvalidValue,
                "object_version",
            ));
        }

        // Alamat IP wajib untuk aksi pengguna dan integrasi (PRD §7.2 "Sumber").
        let needs_ip = matches!(
            self.actor.kind,
            ActorType::Account | ActorType::PartnerSystem | ActorType::RuangSehatSystem
        );
        if needs_ip && source.ip.is_none() {
            return Err(AuditError::new(AuditErrorKind::MissingField, "source_ip"));
        }
        match (self.code, &self.disclosure) {
            (EventCode::E17, None) => {
                return Err(AuditError::new(AuditErrorKind::MissingField, "disclosure"));
            }
            (EventCode::E17, Some(d)) if d.data_categories.is_empty() => {
                return Err(AuditError::new(
                    AuditErrorKind::MissingField,
                    "disclosure_data_categories",
                ));
            }
            (c, Some(_)) if c != EventCode::E17 => {
                return Err(AuditError::new(
                    AuditErrorKind::InvalidCombination,
                    "disclosure_only_for_e17",
                ));
            }
            _ => {}
        }
        let app_version = match source.app_version {
            Some(v) if is_valid_app_version(&v) => Some(v),
            Some(_) => {
                return Err(AuditError::new(AuditErrorKind::InvalidValue, "app_version"));
            }
            None => None,
        };
        let user_agent = source.user_agent.map(|ua| clean_user_agent(&ua));

        let mut subjects: Vec<Subject> = Vec::new();
        for s in self.subjects {
            if !subjects.contains(&s) {
                subjects.push(s);
            }
        }
        if subjects.len() > MAX_SUBJECTS {
            return Err(AuditError::new(AuditErrorKind::InvalidValue, "subjects"));
        }

        let mut changed_fields: Vec<String> = Vec::new();
        let mut before = Map::new();
        let mut after = Map::new();
        for ch in &self.changes {
            let rule = policy.rule(&ch.field).ok_or(AuditError::new(
                AuditErrorKind::FieldNotAllowed,
                "field_not_in_policy",
            ))?;
            if changed_fields.iter().any(|f| f == rule.name()) {
                return Err(AuditError::new(
                    AuditErrorKind::InvalidCombination,
                    "duplicate_change",
                ));
            }
            changed_fields.push(rule.name().to_string());
            if policy.class() == ContentClass::ClinicalOrDocument {
                continue;
            }
            if let Some(v) = &ch.before
                && let Some(j) = mask(rule.kind(), v)?
            {
                before.insert(rule.name().to_string(), j);
            }
            if let Some(v) = &ch.after
                && let Some(j) = mask(rule.kind(), v)?
            {
                after.insert(rule.name().to_string(), j);
            }
        }

        let event_id = new_id();
        let occurred_at = clock.now();
        let (outcome_str, reason) = match outcome {
            Outcome::Success(r) => ("success", r),
            Outcome::Failure(r) => ("failure", Some(r)),
        };
        let payload = Payload {
            payload_version: PAYLOAD_VERSION,
            event_id: event_id.hyphenated().to_string(),
            occurred_at: rfc3339(occurred_at)?,
            event_code: self.code.as_str(),
            action_name: self.action.as_str().to_string(),
            actor_type: self.actor.kind.as_str(),
            actor_account_id: self.actor.account_id.map(|u| u.hyphenated().to_string()),
            actor_account_type: self.actor.account_type.map(|c| c.as_str().to_string()),
            actor_roles: self
                .actor
                .roles
                .iter()
                .map(|c| c.as_str().to_string())
                .collect(),
            actor_partner_id: self.actor.partner_id.map(|u| u.hyphenated().to_string()),
            object_type: policy.object_type(),
            object_id: self.object.id.map(|id| match id {
                ObjectId::Uuid(u) => u.hyphenated().to_string(),
                ObjectId::Code(c) => c.as_str().to_string(),
            }),
            object_version: self.object.version,
            subjects: subjects
                .iter()
                .map(|s| PayloadSubject {
                    subject_id: s.id.hyphenated().to_string(),
                    subject_kind: s.kind.as_str(),
                })
                .collect(),
            tenant_id: self.tenant_id.map(|u| u.hyphenated().to_string()),
            source_component: source.component.as_str(),
            source_ip: source.ip.map(|ip| ip.to_string()),
            user_agent,
            app_version,
            changed_fields: (!changed_fields.is_empty()).then_some(changed_fields),
            before_values: (!before.is_empty()).then_some(Json::Object(before)),
            after_values: (!after.is_empty()).then_some(Json::Object(after)),
            disclosure_recipient: self
                .disclosure
                .as_ref()
                .map(|d| d.recipient.as_str().to_string()),
            disclosure_data_categories: self.disclosure.as_ref().map(|d| {
                d.data_categories
                    .iter()
                    .map(|c| c.as_str().to_string())
                    .collect()
            }),
            outcome: outcome_str,
            outcome_reason_code: reason.map(|c| c.as_str().to_string()),
            correlation_id: context.correlation_id.to_string(),
        };
        let payload = serde_json::to_value(&payload)
            .map_err(|_| AuditError::new(AuditErrorKind::Serialization, "payload"))?;
        Ok(AuditRecord {
            event_id,
            occurred_at,
            code: self.code,
            payload,
        })
    }
}

/// Versi aplikasi: `[0-9A-Za-z.+_-]`, 1–32 karakter, paling banyak 8 digit seluruhnya (mis.
/// `1.0.3+20271101`), sehingga nomor identitas/kartu/rekening tidak dapat diselipkan.
fn is_valid_app_version(v: &str) -> bool {
    !v.is_empty()
        && v.len() <= 32
        && v.chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '+' | '_' | '-'))
        && v.chars().filter(char::is_ascii_digit).count() <= APP_VERSION_MAX_DIGITS
}

/// Batas digit seluruhnya di versi aplikasi.
const APP_VERSION_MAX_DIGITS: usize = 8;
/// Deret digit di user agent yang lebih panjang dari ini disamarkan (versi peramban punya deret ≤ 4).
const USER_AGENT_MAX_DIGIT_RUN: usize = 5;

/// User agent: karakter kontrol dibuang, pola terlarang disamarkan (`scrub_text`), deret angka lebih dari
/// 5 digit diganti `[REDACTED]`, lalu dipotong. User agent wajar memuat banyak angka versi, sehingga
/// jumlah angka seluruhnya tidak dibatasi (batas yang diketahui: nomor yang dipecah menjadi kelompok
/// ≤ 5 digit dapat lolos; nilai ini berasal dari klien peminta sendiri).
fn clean_user_agent(ua: &str) -> String {
    let no_control: String = ua
        .chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect();
    let scrubbed = observability::redact::scrub_text(&no_control);
    let mut out = String::with_capacity(scrubbed.len());
    let mut run = String::new();
    for c in scrubbed.chars().chain(std::iter::once(' ')) {
        if c.is_numeric() {
            run.push(c);
            continue;
        }
        if run.chars().count() > USER_AGENT_MAX_DIGIT_RUN {
            out.push_str("[REDACTED]");
        } else {
            out.push_str(&run);
        }
        run.clear();
        out.push(c);
    }
    out.pop();
    out.chars().take(USER_AGENT_MAX_CHARS).collect()
}

/// Bentuk payload `audit_outbox` versi 1. Nama field = kolom DD B1/B3 (dipetakan relay, F1-T24).
#[derive(Serialize)]
struct Payload {
    payload_version: u16,
    event_id: String,
    occurred_at: String,
    event_code: &'static str,
    action_name: String,
    actor_type: &'static str,
    actor_account_id: Option<String>,
    actor_account_type: Option<String>,
    actor_roles: Vec<String>,
    actor_partner_id: Option<String>,
    object_type: &'static str,
    object_id: Option<String>,
    object_version: Option<i32>,
    subjects: Vec<PayloadSubject>,
    tenant_id: Option<String>,
    source_component: &'static str,
    source_ip: Option<String>,
    user_agent: Option<String>,
    app_version: Option<String>,
    changed_fields: Option<Vec<String>>,
    before_values: Option<Json>,
    after_values: Option<Json>,
    disclosure_recipient: Option<String>,
    disclosure_data_categories: Option<Vec<String>>,
    outcome: &'static str,
    outcome_reason_code: Option<String>,
    correlation_id: String,
}

#[derive(Serialize)]
struct PayloadSubject {
    subject_id: String,
    subject_kind: &'static str,
}

/// Event yang sudah divalidasi dan dimasking, siap ditulis ke `audit_outbox` (DD A19.1).
#[derive(Clone)]
pub struct AuditRecord {
    event_id: Uuid,
    occurred_at: OffsetDateTime,
    code: EventCode,
    payload: Json,
}

// Debug tanpa payload (memuat alamat IP dan ID akun).
impl fmt::Debug for AuditRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AuditRecord")
            .field("event_id", &self.event_id)
            .field("code", &self.code)
            .finish_non_exhaustive()
    }
}

impl AuditRecord {
    /// ID event (UUIDv7) = `audit_outbox.id`.
    pub fn event_id(&self) -> Uuid {
        self.event_id
    }

    /// Waktu aksi menurut jam server aplikasi.
    pub fn occurred_at(&self) -> OffsetDateTime {
        self.occurred_at
    }

    /// Kode event.
    pub fn code(&self) -> EventCode {
        self.code
    }

    /// Payload JSON yang sudah dimasking (kolom `audit_outbox.payload`).
    pub fn payload(&self) -> &Json {
        &self.payload
    }
}

/// Event penolakan akses ke data sensitif (E-05 dengan hasil gagal, tanpa perubahan field dan tanpa
/// pengungkapan) yang sudah divalidasi dan dimasking. Satu-satunya jenis event yang boleh ditulis di luar
/// transaksi aksi bisnis ([`crate::insert_failed_access`]): event ini tidak menyertai perubahan data,
/// sehingga atomisitas aksi dan event (KEPUTUSAN-283, ADR-009 butir 2) tidak dapat dilewati lewatnya.
/// Hanya dibangun [`AuditEvent::finish_failed_access`].
#[derive(Clone, Debug)]
pub struct FailedAccessRecord(AuditRecord);

impl FailedAccessRecord {
    /// ID event (UUIDv7) = `audit_outbox.id`.
    pub fn event_id(&self) -> Uuid {
        self.0.event_id
    }

    pub(crate) fn record(&self) -> &AuditRecord {
        &self.0
    }
}
