//! Use case profil dan alamat pasien (F1-T56, US-105 AC1-AC3). Pola sama dengan `service.rs`
//! (reference implementation US-106): validasi dan IP klien dulu, lalu satu `audit::in_audited_tx` untuk
//! semua tulisan basis data bersama event-nya.
//!
//! - Profil (US-105 AC1): nama, tanggal lahir, dan jenis kelamin ada di `care_recipient` pemilik akun,
//!   nomor HP di `patient` (DD A2.1-A2.2). Pemanggil hanya dapat membaca dan mengubah profil akunnya
//!   sendiri: tidak ada ID di path, sehingga tidak ada objek orang lain yang dapat dituju.
//! - Perubahan nama atau tanggal lahir pada identitas "Disetujui" membuat permintaan verifikasi baru
//!   berpemicu `profile_changed` lewat jalur tulis status (ADR-008, `status_flow::transition`), memakai
//!   dokumen dan NIK yang sama (AC3). Pengajuan seperti itu tidak dapat dibatalkan (`VerificationRules`).
//! - Alamat (AC2): banyak alamat per pasien, tepat satu alamat utama selama ada alamat. Aturan
//!   `[ASUMSI]` karena T-42 (perilaku saat alamat utama dihapus) belum tertulis di dokumen mana pun:
//!   alamat pertama otomatis utama; menandai alamat lain sebagai utama melepas yang lama; menghapus alamat
//!   utama menjadikan alamat aktif tertua berikutnya sebagai utama; alamat utama tidak dapat
//!   dinonaktifkan tanpa memilih penggantinya.
//! - Alamat diakses lewat `patient.account_id` akun pemanggil: ID yang tidak ada dan milik pasien lain
//!   memakai query yang sama dan menghasilkan `404` yang tidak dapat dibedakan (KU-03); percobaan itu
//!   dicatat sebagai event E-10 hasil gagal.
//! - Audit (PRD §7): E-10 untuk tambah/ubah profil dan alamat, E-16 untuk soft delete alamat. Nilai data
//!   pribadi (nama, tanggal lahir, nomor HP, alamat) tidak disalin ke audit trail: hanya nama field yang
//!   berubah (`FieldRule::name_only`), minimisasi data pribadi; nilai bertipe (status, `is_default`)
//!   dicatat apa adanya.

use super::api::{AddressInput, AddressView, ProfileInput, ProfileView};
use super::domain::{
    IdentityStatus, TriggerType, VerificationActor, VerificationRules, VerificationStatus,
};
use super::error::ProfileError;
use super::profile_rules::{
    Errors, MAX_ADDRESSES, RawAddress, ValidAddress, normalize_name, normalize_phone,
    parse_birth_date, parse_sex, requires_reverification, validate_address,
};
use super::repo::{
    self, AddressRow, AddressWrite, EventDetails, IvrStatusStore, OwnProfile, RecipientNik,
    RepoError,
};
use super::service::{
    RequestEvent, audit_context, client_ip, record_failed, request_event, status_change,
    transition_error,
};
use audit::{
    AuditContext, AuditEvent, AuditedTxError, Code, EventCode, FieldChange, FieldRule,
    ObjectPolicy, ObjectRef, Outcome, Subject, Value, in_audited_tx,
};
use authz::Principal;
use envelope::FieldError;
use foundation::{Uuid, new_id};
use observability::security;
use time::UtcOffset;

/// Alasan soft delete alamat (`address.delete_reason`, ADR-003).
const ADDRESS_DELETE_REASON: &str = "patient_request";

/// Objek profil untuk event E-10: hanya nama field yang berubah, tanpa nilai data pribadi.
pub(super) static PROFILE_POLICY: ObjectPolicy = ObjectPolicy::non_clinical(
    "care_recipient_profile",
    &[
        FieldRule::name_only("full_name"),
        FieldRule::name_only("birth_date"),
        FieldRule::name_only("sex"),
        FieldRule::name_only("phone"),
    ],
);

/// Objek alamat untuk event E-10/E-16: nama field berubah; penanda alamat utama berupa nilai boolean.
pub(super) static ADDRESS_POLICY: ObjectPolicy = ObjectPolicy::non_clinical(
    "address",
    &[
        FieldRule::name_only("label"),
        FieldRule::name_only("recipient_name"),
        FieldRule::name_only("recipient_phone"),
        FieldRule::name_only("address_line"),
        FieldRule::name_only("province_name"),
        FieldRule::name_only("city_name"),
        FieldRule::name_only("postal_code"),
        FieldRule::name_only("notes"),
        FieldRule::name_only("time_zone"),
        FieldRule::name_only("deleted_at"),
        FieldRule::value("is_default"),
    ],
);

/// Galat kerja transaksi: tidak ada perubahan (tanpa tulisan dan tanpa event) atau galat sebenarnya.
enum Abort {
    Unchanged,
    Error(ProfileError),
}

impl From<ProfileError> for Abort {
    fn from(e: ProfileError) -> Self {
        Self::Error(e)
    }
}

impl From<RepoError> for Abort {
    fn from(e: RepoError) -> Self {
        Self::Error(e.into())
    }
}

fn audited(e: AuditedTxError<Abort>) -> Result<(), ProfileError> {
    match e {
        AuditedTxError::Work(Abort::Unchanged) => Ok(()),
        AuditedTxError::Work(Abort::Error(e)) => Err(e),
        AuditedTxError::Audit(_) => Err(ProfileError::Internal("audit")),
        AuditedTxError::Database { .. } => Err(ProfileError::Database),
    }
}

fn identity_status(p: &OwnProfile) -> Result<IdentityStatus, ProfileError> {
    let status = p
        .identity_status
        .as_deref()
        .map(|c| VerificationStatus::parse(c).ok_or(ProfileError::Internal("corrupt_row")))
        .transpose()?;
    IdentityStatus::from_effective(status)
        .ok_or(ProfileError::Internal("cancelled_effective_request"))
}

fn view(p: OwnProfile) -> Result<ProfileView, ProfileError> {
    let status = identity_status(&p)?;
    Ok(ProfileView {
        care_recipient_id: p.care_recipient_id,
        full_name: p.full_name,
        birth_date: p.birth_date,
        sex: p.sex,
        phone: p.phone,
        identity_status: status.as_str(),
    })
}

fn address_view(a: AddressRow) -> AddressView {
    AddressView {
        id: a.id,
        label: a.label,
        recipient_name: a.recipient_name,
        recipient_phone: a.recipient_phone,
        address_line: a.address_line,
        province_name: a.province_name,
        city_name: a.city_name,
        postal_code: a.postal_code,
        notes: a.notes,
        time_zone: a.time_zone,
        is_default: a.is_default,
    }
}

fn code(v: &str) -> Result<Code, ProfileError> {
    Code::new(v).map_err(|_| ProfileError::Internal("audit_action"))
}

/// Profil akun pemanggil (US-105 AC1). Akun tanpa profil = `404`.
pub(super) async fn get_profile(
    s: &super::api::IdentityServices,
    principal: &Principal,
) -> Result<ProfileView, ProfileError> {
    let p = repo::own_profile(&s.pool, principal.account_id())
        .await?
        .ok_or(ProfileError::NotFound)?;
    view(p)
}

/// Menyimpan profil (US-105 AC1, AC1a, AC1b, AC3). Simpan tanpa perubahan tidak menulis apa pun dan tidak
/// menghasilkan event (TC-US-105-4); dua simpan paralel berjalan berurutan lewat kunci baris.
pub(super) async fn update_profile(
    s: &super::api::IdentityServices,
    principal: &Principal,
    route: &str,
    input: &ProfileInput<'_>,
) -> Result<ProfileView, ProfileError> {
    let wib = UtcOffset::from_hms(7, 0, 0).map_err(|_| ProfileError::Internal("offset"))?;
    let today_wib = s.clock.now().to_offset(wib).date();
    let mut errors = Errors::default();
    let full_name = errors.take(normalize_name("full_name", input.full_name, "Nama lengkap"));
    let birth_date = errors.take(parse_birth_date(input.birth_date, today_wib));
    let sex = errors.take(parse_sex(input.sex));
    let phone = match input.phone.map(str::trim).filter(|p| !p.is_empty()) {
        None => Some(None),
        Some(p) => errors
            .take(normalize_phone("phone", p, "Nomor HP"))
            .map(Some),
    };
    errors.into_result().map_err(ProfileError::Validation)?;
    let (Some(full_name), Some(birth_date), Some(sex), Some(phone)) =
        (full_name, birth_date, sex, phone)
    else {
        return Err(ProfileError::Internal("validation"));
    };
    let ip = client_ip(principal, route)?;
    let account_id = principal.account_id();
    let context = audit_context(principal, ip);
    let event_actor = principal.clone();

    let result = in_audited_tx(&s.pool, s.clock.as_ref(), move |conn| {
        Box::pin(async move {
            let before = repo::lock_own_profile(conn, account_id)
                .await?
                .ok_or(ProfileError::NotFound)?;
            let status = identity_status(&before)?;
            let name_changed = before.full_name != full_name;
            let birth_changed = before.birth_date != birth_date;
            let sex_changed = before.sex != sex;
            let phone_changed = before.phone != phone;
            if !(name_changed || birth_changed || sex_changed || phone_changed) {
                return Err(Abort::Unchanged);
            }
            let mut changes = Vec::new();
            if name_changed || birth_changed || sex_changed {
                repo::update_owner_profile(
                    conn,
                    before.care_recipient_id,
                    &full_name,
                    birth_date,
                    sex,
                    account_id,
                )
                .await?;
            }
            if phone_changed {
                repo::update_patient_phone(conn, before.patient_id, phone.as_deref(), account_id)
                    .await?;
            }
            for (changed, name) in [
                (name_changed, "full_name"),
                (birth_changed, "birth_date"),
                (sex_changed, "sex"),
                (phone_changed, "phone"),
            ] {
                if changed {
                    changes.push(FieldChange::changed(name));
                }
            }
            let mut profile_event = AuditEvent::new(
                EventCode::E10,
                code("profile_updated")?,
                super::service::actor(&event_actor)?,
                ObjectRef::with_uuid(&PROFILE_POLICY, before.care_recipient_id),
            )
            .subject(Subject::care_recipient(before.care_recipient_id))
            .outcome(Outcome::success())
            .context(&context);
            for c in changes {
                profile_event = profile_event.change(c);
            }
            let mut events = vec![profile_event];

            if requires_reverification(status, name_changed, birth_changed) {
                let latest = before
                    .latest_verification_request_id
                    .ok_or(ProfileError::Internal("approved_without_request"))?;
                let doc = repo::effective_document(conn, latest)
                    .await?
                    .ok_or(ProfileError::Internal("dangling_document"))?;
                let request_id = new_id();
                repo::insert_request(
                    conn,
                    request_id,
                    account_id,
                    before.care_recipient_id,
                    doc.document_id,
                    TriggerType::ProfileChanged,
                )
                .await?;
                let step = {
                    let mut store = IvrStatusStore {
                        conn: &mut *conn,
                        request_id,
                        details: EventDetails {
                            event_id: new_id(),
                            actor_type: "patient",
                            actor_account_id: account_id,
                            reason_code: None,
                            note_enc: None,
                            note_data_key_id: None,
                            correlation_id: event_actor.correlation_id(),
                        },
                    };
                    status_flow::transition(
                        &mut store,
                        &VerificationRules::new(
                            VerificationActor::Patient,
                            TriggerType::ProfileChanged,
                        ),
                        VerificationStatus::Pending,
                    )
                    .await
                    .map_err(transition_error)?
                };
                repo::set_effective_request(
                    conn,
                    before.care_recipient_id,
                    Some(request_id),
                    Some(RecipientNik {
                        bidx: &doc.nik_bidx,
                        key_version: doc.nik_bidx_key_version,
                        last4: &doc.nik_last4,
                    }),
                    account_id,
                )
                .await?;
                events.push(request_event(
                    &event_actor,
                    RequestEvent {
                        code: EventCode::E10,
                        action: "identity_reverification_requested",
                        request_id,
                        subject: before.care_recipient_id,
                        changes: vec![status_change(step.from, step.to)?],
                        outcome: Outcome::success(),
                        context: &context,
                    },
                )?);
            }
            Ok(((), events))
        })
    })
    .await;

    match result {
        Ok(()) => {}
        Err(e) => {
            let was_mismatch = matches!(
                &e,
                AuditedTxError::Work(Abort::Error(ProfileError::ProjectionMismatch))
            );
            audited(e)?;
            if was_mismatch {
                security::status_projection_mismatch("identity_verification_request", account_id);
            }
        }
    }
    get_profile(s, principal).await
}

/// Daftar alamat aktif akun pemanggil (alamat utama pertama). Akun tanpa profil = `404`.
pub(super) async fn list_addresses(
    s: &super::api::IdentityServices,
    principal: &Principal,
) -> Result<Vec<AddressView>, ProfileError> {
    let p = repo::own_profile(&s.pool, principal.account_id())
        .await?
        .ok_or(ProfileError::NotFound)?;
    list_for(s, p.patient_id).await
}

async fn list_for(
    s: &super::api::IdentityServices,
    patient_id: Uuid,
) -> Result<Vec<AddressView>, ProfileError> {
    Ok(repo::list_addresses(&s.pool, patient_id)
        .await?
        .into_iter()
        .map(address_view)
        .collect())
}

fn validate_input(input: &AddressInput<'_>) -> Result<ValidAddress, ProfileError> {
    validate_address(&RawAddress {
        label: input.label,
        recipient_name: input.recipient_name,
        recipient_phone: input.recipient_phone,
        address_line: input.address_line,
        province_name: input.province_name,
        city_name: input.city_name,
        postal_code: input.postal_code,
        notes: input.notes,
        time_zone: input.time_zone,
    })
    .map_err(ProfileError::Validation)
}

fn write_of<'a>(
    id: Uuid,
    patient_id: Uuid,
    account_id: Uuid,
    v: &'a ValidAddress,
    is_default: bool,
) -> AddressWrite<'a> {
    AddressWrite {
        id,
        patient_id,
        actor_account_id: account_id,
        label: v.label.as_deref(),
        recipient_name: &v.recipient_name,
        recipient_phone: &v.recipient_phone,
        address_line: &v.address_line,
        province_name: &v.province_name,
        city_name: &v.city_name,
        postal_code: v.postal_code.as_deref(),
        notes: v.notes.as_deref(),
        time_zone: v.time_zone,
        is_default,
    }
}

fn address_event(
    who: (&Principal, &AuditContext),
    event_code: EventCode,
    action: &str,
    address_id: Uuid,
    subject: Option<Uuid>,
    changes: Vec<FieldChange>,
    outcome: Outcome,
) -> Result<AuditEvent, ProfileError> {
    let (principal, context) = who;
    let mut event = AuditEvent::new(
        event_code,
        code(action)?,
        super::service::actor(principal)?,
        ObjectRef::with_uuid(&ADDRESS_POLICY, address_id),
    )
    .outcome(outcome)
    .context(context);
    if let Some(s) = subject {
        event = event.subject(Subject::care_recipient(s));
    }
    for c in changes {
        event = event.change(c);
    }
    Ok(event)
}

fn bool_change(before: Option<bool>, after: bool) -> FieldChange {
    FieldChange::new(
        "is_default",
        before.map(Value::Bool),
        Some(Value::Bool(after)),
    )
}

fn default_demoted_event(
    principal: &Principal,
    demoted: Uuid,
    subject: Uuid,
    context: &AuditContext,
) -> Result<AuditEvent, ProfileError> {
    address_event(
        (principal, context),
        EventCode::E10,
        "address_default_changed",
        demoted,
        Some(subject),
        vec![bool_change(Some(true), false)],
        Outcome::success(),
    )
}

fn field_names(v: &ValidAddress) -> Vec<FieldChange> {
    let mut f = vec![
        "recipient_name",
        "recipient_phone",
        "address_line",
        "province_name",
        "city_name",
        "time_zone",
    ];
    if v.label.is_some() {
        f.push("label");
    }
    if v.postal_code.is_some() {
        f.push("postal_code");
    }
    if v.notes.is_some() {
        f.push("notes");
    }
    f.into_iter().map(FieldChange::changed).collect()
}

/// Menambah alamat (US-105 AC2). Alamat pertama otomatis utama; `is_default = true` melepas yang lama.
pub(super) async fn create_address(
    s: &super::api::IdentityServices,
    principal: &Principal,
    route: &str,
    input: &AddressInput<'_>,
) -> Result<Vec<AddressView>, ProfileError> {
    let valid = validate_input(input)?;
    let ip = client_ip(principal, route)?;
    let account_id = principal.account_id();
    let want_default = input.is_default.unwrap_or(false);
    let context = audit_context(principal, ip);
    let event_actor = principal.clone();

    let result = in_audited_tx(&s.pool, s.clock.as_ref(), move |conn| {
        Box::pin(async move {
            let patient = repo::lock_patient(conn, account_id)
                .await?
                .ok_or(ProfileError::NotFound)?;
            let count = repo::count_addresses(conn, patient.patient_id).await?;
            if count >= MAX_ADDRESSES {
                return Err(Abort::Error(ProfileError::Validation(vec![
                    FieldError::new(
                        "$",
                        "too_many_addresses",
                        format!("Jumlah alamat paling banyak {MAX_ADDRESSES}"),
                    ),
                ])));
            }
            let is_default = want_default || count == 0;
            let mut events = Vec::new();
            if is_default
                && let Some(old) = repo::clear_default(conn, patient.patient_id, account_id).await?
            {
                events.push(default_demoted_event(
                    &event_actor,
                    old,
                    patient.care_recipient_id,
                    &context,
                )?);
            }
            let id = new_id();
            repo::insert_address(
                conn,
                &write_of(id, patient.patient_id, account_id, &valid, is_default),
            )
            .await?;
            let mut changes = field_names(&valid);
            changes.push(bool_change(None, is_default));
            events.insert(
                0,
                address_event(
                    (&event_actor, &context),
                    EventCode::E10,
                    "address_created",
                    id,
                    Some(patient.care_recipient_id),
                    changes,
                    Outcome::success(),
                )?,
            );
            Ok((patient.patient_id, events))
        })
    })
    .await;
    let patient_id = match result {
        Ok(p) => p,
        Err(e) => {
            audited(e)?;
            return Err(ProfileError::Internal("address_unchanged"));
        }
    };
    list_for(s, patient_id).await
}

/// Mencatat percobaan atas alamat yang tidak ditemukan (ID tidak ada atau milik pasien lain; KU-03, KU-06).
async fn record_address_not_found(
    s: &super::api::IdentityServices,
    principal: &Principal,
    action: &str,
    address_id: Uuid,
    context: &AuditContext,
) {
    let event_code = if action == "address_deleted" {
        EventCode::E16
    } else {
        EventCode::E10
    };
    let Ok(reason) = Code::new("not_found") else {
        return;
    };
    if let Ok(event) = address_event(
        (principal, context),
        event_code,
        action,
        address_id,
        None,
        Vec::new(),
        Outcome::failure(reason),
    ) {
        record_failed(&s.pool, s.clock.as_ref(), event).await;
    }
}

/// Mengubah alamat milik pemanggil (PUT penuh). `is_default = true` menjadikannya utama; `false` pada
/// alamat utama ditolak karena tepat satu alamat utama harus tetap ada.
pub(super) async fn update_address(
    s: &super::api::IdentityServices,
    principal: &Principal,
    route: &str,
    address_id: Uuid,
    input: &AddressInput<'_>,
) -> Result<Vec<AddressView>, ProfileError> {
    let valid = validate_input(input)?;
    let ip = client_ip(principal, route)?;
    let account_id = principal.account_id();
    let want_default = input.is_default;
    let context = audit_context(principal, ip);
    let failed_context = context.clone();
    let event_actor = principal.clone();

    let result = in_audited_tx(&s.pool, s.clock.as_ref(), move |conn| {
        Box::pin(async move {
            let patient = repo::lock_patient(conn, account_id)
                .await?
                .ok_or(ProfileError::NotFound)?;
            let current = repo::find_address(conn, patient.patient_id, address_id)
                .await?
                .ok_or(ProfileError::NotFound)?;
            if current.is_default && want_default == Some(false) {
                return Err(Abort::Error(ProfileError::Validation(vec![FieldError::new(
                    "is_default",
                    "default_required",
                    "Alamat utama tidak dapat dinonaktifkan; pilih alamat lain sebagai alamat utama",
                )])));
            }
            let is_default = current.is_default || want_default == Some(true);
            let mut changes = Vec::new();
            for (changed, name) in [
                (current.label != valid.label, "label"),
                (current.recipient_name != valid.recipient_name, "recipient_name"),
                (current.recipient_phone != valid.recipient_phone, "recipient_phone"),
                (current.address_line != valid.address_line, "address_line"),
                (current.province_name != valid.province_name, "province_name"),
                (current.city_name != valid.city_name, "city_name"),
                (current.postal_code != valid.postal_code, "postal_code"),
                (current.notes != valid.notes, "notes"),
                (current.time_zone != valid.time_zone, "time_zone"),
            ] {
                if changed {
                    changes.push(FieldChange::changed(name));
                }
            }
            let default_changed = is_default != current.is_default;
            if changes.is_empty() && !default_changed {
                return Err(Abort::Unchanged);
            }
            let mut events = Vec::new();
            if default_changed {
                if let Some(old) =
                    repo::clear_default(conn, patient.patient_id, account_id).await?
                {
                    events.push(default_demoted_event(
                        &event_actor,
                        old,
                        patient.care_recipient_id,
                        &context,
                    )?);
                }
                changes.push(bool_change(Some(current.is_default), is_default));
            }
            repo::update_address(
                conn,
                &write_of(address_id, patient.patient_id, account_id, &valid, is_default),
            )
            .await?;
            events.insert(
                0,
                address_event(
                    (&event_actor, &context),
                    EventCode::E10,
                    "address_updated",
                    address_id,
                    Some(patient.care_recipient_id),
                    changes,
                    Outcome::success(),
                )?,
            );
            Ok((patient.patient_id, events))
        })
    })
    .await;
    match result {
        Ok(patient_id) => list_for(s, patient_id).await,
        Err(AuditedTxError::Work(Abort::Unchanged)) => list_addresses(s, principal).await,
        Err(e) => {
            let e = audited(e)
                .err()
                .unwrap_or(ProfileError::Internal("address_update"));
            if matches!(e, ProfileError::NotFound) {
                record_address_not_found(
                    s,
                    principal,
                    "address_updated",
                    address_id,
                    &failed_context,
                )
                .await;
            }
            Err(e)
        }
    }
}

/// Menghapus alamat milik pemanggil (soft delete, ADR-003, E-16). Alamat utama yang dihapus digantikan
/// alamat aktif tertua berikutnya (`[ASUMSI]`, lihat komentar modul).
pub(super) async fn delete_address(
    s: &super::api::IdentityServices,
    principal: &Principal,
    route: &str,
    address_id: Uuid,
) -> Result<Vec<AddressView>, ProfileError> {
    let ip = client_ip(principal, route)?;
    let account_id = principal.account_id();
    let context = audit_context(principal, ip);
    let failed_context = context.clone();
    let event_actor = principal.clone();

    let result = in_audited_tx(&s.pool, s.clock.as_ref(), move |conn| {
        Box::pin(async move {
            let patient = repo::lock_patient(conn, account_id)
                .await?
                .ok_or(ProfileError::NotFound)?;
            let current = repo::find_address(conn, patient.patient_id, address_id)
                .await?
                .ok_or(ProfileError::NotFound)?;
            repo::soft_delete_address(
                conn,
                patient.patient_id,
                address_id,
                account_id,
                ADDRESS_DELETE_REASON,
            )
            .await?;
            let mut events = vec![address_event(
                (&event_actor, &context),
                EventCode::E16,
                "address_deleted",
                address_id,
                Some(patient.care_recipient_id),
                vec![FieldChange::changed("deleted_at")],
                Outcome::success(),
            )?];
            if current.is_default
                && let Some(promoted) =
                    repo::promote_oldest_address(conn, patient.patient_id, account_id).await?
            {
                events.push(address_event(
                    (&event_actor, &context),
                    EventCode::E10,
                    "address_default_changed",
                    promoted,
                    Some(patient.care_recipient_id),
                    vec![bool_change(Some(false), true)],
                    Outcome::success(),
                )?);
            }
            Ok((patient.patient_id, events))
        })
    })
    .await;
    match result {
        Ok(patient_id) => list_for(s, patient_id).await,
        Err(e) => {
            let e = audited(e)
                .err()
                .unwrap_or(ProfileError::Internal("address_delete"));
            if matches!(e, ProfileError::NotFound) {
                record_address_not_found(
                    s,
                    principal,
                    "address_deleted",
                    address_id,
                    &failed_context,
                )
                .await;
            }
            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // trace: ADR-009 butir 7 (daftar izin before/after sah); PRD §7.3 (tanpa data terlarang)
    #[test]
    fn audit_policies_are_valid() {
        PROFILE_POLICY.validate().unwrap();
        ADDRESS_POLICY.validate().unwrap();
    }
}
