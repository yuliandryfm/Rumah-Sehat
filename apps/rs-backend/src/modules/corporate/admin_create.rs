//! Use case perusahaan dibuatkan Verifikator (F1-T105, T1; US-1903 AC1-AC4; BR-136 `[USER] C21-P4` 4a;
//! ADR-030 butir 2, 5, 7; KEPUTUSAN-303, KEPUTUSAN-306; KEPUTUSAN-328/329).
//!
//! Akses: `POST /backoffice/v1/corporate-clients`, kebijakan `Policy::rs_admin(&[Verifier])` (`policy.rs`):
//! sub-peran lain termasuk Super Admin `403`, HR realm `korporat` `401`, keduanya tercatat di security log
//! oleh middleware (US-1902 AC5, pola F1-T104). Persetujuan orang kedua (G-27, M-49) TIDAK ada: phase plan G-27
//! menetapkan F1-T105 dibangun tanpa persetujuan orang kedua, ditambah bila user memutuskan.
//!
//! Urutan di satu transaksi `audit::in_audited_tx` (KEPUTUSAN-125: tanpa panggilan HTTP selama transaksi
//! terbuka; efek Keycloak hanya dijalankan worker IdP):
//! 1. [`register_hr_invitation`] lebih dulu: kunci advisory (realm, email) adalah kunci PERTAMA, lalu email
//!    diperiksa belum dipakai akun belum terhapus, akun HR `pending_provisioning` disisipkan, niat `create_user`
//!    dan job `idp_create_user` dicatat (ADR-030 butir 5). Email sudah dipakai = `409` `4090111` (hanya
//!    Verifikator, staf berwenang, yang melihat pembeda ini; undangan HR oleh HR seragam, P30-2).
//! 2. NIB diperiksa belum dipakai perusahaan **Aktif** belum terhapus (`409` `4090110`, pesan umum tanpa nama
//!    perusahaan lain, US-1903 AC4); constraint `uqp_corporate_client_nib_active` menjaga balapan
//!    (`ON CONFLICT DO NOTHING`, hasil nol baris = NIB ganda). Sejak CR-035 (keputusan user Q10 = B2: NIB unik
//!    hanya di antara perusahaan Aktif) pengajuan Menunggu verifikasi atau Ditolak dengan NIB yang sama tidak
//!    menghalangi; pengajuan itu kemudian tidak dapat disetujui (`4090116`).
//! 3. Perusahaan disisipkan **langsung Aktif** (`origin = created_by_admin`, putaran 1) tanpa keputusan
//!    verifikasi: Verifikator sudah memeriksa dokumen di luar sistem (AC1). Keanggotaan HR pertama disisipkan
//!    di transaksi yang sama (akun terikat ke perusahaan sejak undangan, US-1904 AC2).
//! 4. Event: E-10 perusahaan dibuat (pelaku Verifikator, field `origin` = `created_by_admin` sebagai sumber
//!    "dibuat admin", AC3), E-10 akun HR diundang, E-11 keanggotaan diberikan; semua `tenant_id` = perusahaan.
//!    E-04 (kredensial) menyusul dari event Keycloak saat HR mengatur password lewat tautan aktivasi
//!    (`identity::domain::map_event`), bukan dari permintaan ini.
//!
//! Email aktivasi dikirim worker IdP setelah pengguna Keycloak dibuat (ADR-030 butir 3, 8): tautan berlaku 24
//! jam dan sekali pakai (KEPUTUSAN-306, token aksi Keycloak; AC2 diuji di `keycloak_check.py`
//! `cek_undangan_hr`). Backend tidak punya jalur "undang ulang": email yang sama ditolak selama akunnya belum
//! terhapus, sehingga tidak ada akun atau niat ganda.
//!
//! Dokumen perusahaan (NIB, NPWP, bukti kewenangan) TIDAK dilampirkan di endpoint ini: jalur unggah dokumen
//! perusahaan (upload slot, validasi isi, penetralan PDF ADR-018) adalah F1-T102 yang belum ada; perusahaan
//! dibuatkan dapat dilengkapi dokumennya lewat jalur itu `[TIDAK DIKETAHUI]` (lihat laporan F1-T105).
//!
//! Gagal (email atau NIB ganda): tidak ada efek; E-10 hasil gagal di transaksi tersendiri dengan objek akun
//! Verifikator (perusahaan yang diminta belum ada) supaya tidak membuka perusahaan atau akun lain.

use super::api::CorporateVerification;
use super::error::CorporateError;
use super::repo::{self, NewAdminClient};
use crate::modules::identity::api::{
    HrInviteError, IntentActor, normalize_hr_email, register_hr_invitation,
};
use audit::{
    Actor, AuditContext, AuditEvent, AuditedTxError, Code, EventCode, FieldChange, FieldRule,
    ObjectPolicy, ObjectRef, Outcome, Source, SourceComponent, Value, in_audited_tx,
};
use authz::Principal;
use envelope::FieldError;
use foundation::{Clock, FixedClock, Sensitive, Uuid, new_id};
use sqlx::PgPool;
use std::convert::Infallible;
use std::fmt;
use std::net::IpAddr;

const CLIENT_CREATED_ACTION: &str = "corporate_client_created_by_admin";
const HR_INVITED_ACTION: &str = "corporate_admin_account_invited";
/// E-04 pembuatan akun HR (keputusan user Q28 = A, KEPUTUSAN-380): dicatat bersama E-10 di transaksi yang sama.
pub(super) const ACCOUNT_CREATED_ACTION: &str = "account_created";
const MEMBERSHIP_GRANTED_ACTION: &str = "corporate_membership_granted";
const CREATE_REJECTED_ACTION: &str = "corporate_client_create_rejected";
const NIB_TAKEN_REASON: &str = "nib_already_registered";
const EMAIL_TAKEN_REASON: &str = "hr_email_in_use";

/// Percobaan menulis event hasil gagal di transaksi tersendiri (coding-standards §4.4 butir 3).
pub(super) const FAILED_AUDIT_ATTEMPTS: usize = 3;

/// Batas panjang sama dengan CHECK `ck_corporate_client_*` (pertahanan lapis kedua, coding-standards §5.1).
pub(super) const LEGAL_NAME_MAX: usize = 200;
const NIB_MAX: usize = 64;
const NPWP_MAX: usize = 64;
pub(super) const ADDRESS_MAX: usize = 500;
pub(super) const PHONE_MAX: usize = 32;

/// Kebijakan audit perusahaan dibuat admin: hanya status, sumber (`origin`), dan putaran pengajuan. Nama,
/// NIB, NPWP, alamat, telepon, dan email HR tidak terdaftar sehingga pustaka menolaknya (NPWP Terbatas).
pub(super) static CLIENT_POLICY: ObjectPolicy = ObjectPolicy::non_clinical(
    "corporate_client",
    &[
        FieldRule::value("status"),
        FieldRule::value("origin"),
        FieldRule::value("submission_round"),
    ],
);

/// Kebijakan audit akun HR yang diundang dan keanggotaannya: status, jenis akun, ID keanggotaan, ID perusahaan.
/// Email dan nama tidak pernah dicatat.
pub(super) static HR_ACCOUNT_POLICY: ObjectPolicy = ObjectPolicy::non_clinical(
    "account",
    &[
        FieldRule::value("status"),
        FieldRule::value("account_type"),
        FieldRule::value("corporate_member_id"),
        FieldRule::value("corporate_client_id"),
    ],
);

/// Objek event penolakan: akun Verifikator pelaku (tanpa field).
static ACTOR_ACCOUNT_POLICY: ObjectPolicy = ObjectPolicy::non_clinical("account", &[]);

/// Isian pembuatan perusahaan oleh Verifikator. NPWP (Terbatas) dan email HR dibungkus [`Sensitive`]; `Debug`
/// manual tanpa keduanya.
pub struct NewCorporateClientInput {
    /// Nama badan usaha.
    pub legal_name: String,
    /// NIB (format resmi `[TIDAK DIKETAHUI]`: hanya panjang dan karakter dasar diperiksa).
    pub nib: String,
    /// NPWP (Terbatas).
    pub npwp: Sensitive<String>,
    /// Alamat kantor.
    pub office_address: String,
    /// Telepon kantor.
    pub office_phone: String,
    /// Email HR pertama (penerima undangan aktivasi).
    pub hr_email: Sensitive<String>,
}

impl fmt::Debug for NewCorporateClientInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NewCorporateClientInput")
            .finish_non_exhaustive()
    }
}

/// Hasil pembuatan perusahaan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatedCorporateClientView {
    /// Perusahaan baru (langsung Aktif).
    pub corporate_client_id: Uuid,
    /// Keanggotaan HR pertama.
    pub corporate_member_id: Uuid,
    /// Akun HR pertama (`pending_provisioning` sampai worker membuat pengguna Keycloak-nya).
    pub hr_account_id: Uuid,
}

/// Isian tervalidasi (dipangkas, batas panjang, tanpa karakter kontrol).
struct Validated {
    legal_name: String,
    nib: String,
    npwp: String,
    office_address: String,
    office_phone: String,
    hr_email: String,
}

pub(super) fn field(name: &str, code: &str, message: &str) -> FieldError {
    FieldError::new(name, code, message)
}

/// Teks dipangkas 1..=`max` karakter tanpa karakter kontrol; `single_token` melarang spasi di tengah.
pub(super) fn text(raw: &str, max: usize, single_token: bool) -> Option<String> {
    let t = raw.trim();
    let n = t.chars().count();
    let ok = (1..=max).contains(&n)
        && !t.chars().any(char::is_control)
        && !(single_token && t.chars().any(char::is_whitespace));
    ok.then(|| t.to_string())
}

fn validate(input: &NewCorporateClientInput) -> Result<Validated, CorporateError> {
    let mut errors = Vec::new();
    let mut take = |name: &str, raw: &str, max: usize, token: bool, message: &str| {
        let v = text(raw, max, token);
        if v.is_none() {
            errors.push(field(name, "invalid", message));
        }
        v.unwrap_or_default()
    };
    let legal_name = take(
        "legal_name",
        &input.legal_name,
        LEGAL_NAME_MAX,
        false,
        "Nama badan usaha wajib, maksimal 200 karakter",
    );
    let nib = take(
        "nib",
        &input.nib,
        NIB_MAX,
        true,
        "NIB wajib, maksimal 64 karakter tanpa spasi",
    );
    let npwp = take(
        "npwp",
        input.npwp.expose(),
        NPWP_MAX,
        true,
        "NPWP wajib, maksimal 64 karakter tanpa spasi",
    );
    let office_address = take(
        "office_address",
        &input.office_address,
        ADDRESS_MAX,
        false,
        "Alamat kantor wajib, maksimal 500 karakter",
    );
    let office_phone = take(
        "office_phone",
        &input.office_phone,
        PHONE_MAX,
        false,
        "Telepon kantor wajib, maksimal 32 karakter",
    );
    let hr_email = normalize_hr_email(input.hr_email.expose()).unwrap_or_else(|| {
        errors.push(field(
            "hr_email",
            "invalid",
            "Email HR harus berupa alamat email yang sah",
        ));
        String::new()
    });
    if errors.is_empty() {
        Ok(Validated {
            legal_name,
            nib,
            npwp,
            office_address,
            office_phone,
            hr_email,
        })
    } else {
        Err(CorporateError::Validation(errors))
    }
}

pub(super) fn db_error(_: sqlx::Error) -> CorporateError {
    // Pesan sqlx dapat memuat isi baris (DETAIL); tidak diteruskan (TD-03).
    CorporateError::Database
}

fn client_ip(principal: &Principal, route: &str) -> Result<IpAddr, CorporateError> {
    principal.client_ip().ok_or_else(|| {
        tracing::error!(route = route, "client_ip_missing");
        CorporateError::Internal("client_ip_missing")
    })
}

fn audit_context(principal: &Principal, ip: IpAddr) -> AuditContext {
    AuditContext::new(
        principal.correlation_id(),
        Source::component(SourceComponent::Backoffice).ip(ip),
    )
}

pub(super) fn code(v: &str) -> Result<Code, CorporateError> {
    Code::new(v).map_err(|_| CorporateError::Internal("audit_code"))
}

pub(super) fn code_value(v: &str) -> Result<Value, CorporateError> {
    Value::code(v).map_err(|_| CorporateError::Internal("audit_value"))
}

fn actor(principal: &Principal) -> Result<Actor, CorporateError> {
    let roles = principal
        .sub_roles()
        .iter()
        .map(|r| code(r.as_str()))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Actor::account(
        principal.account_id(),
        code(principal.account_type().as_str())?,
        roles,
    ))
}

fn event(
    ev: EventCode,
    action: &str,
    principal: &Principal,
    ctx: &AuditContext,
    object: ObjectRef,
    tenant: Option<Uuid>,
    outcome: Outcome,
) -> Result<AuditEvent, CorporateError> {
    let mut e = AuditEvent::new(ev, code(action)?, actor(principal)?, object)
        .outcome(outcome)
        .context(ctx);
    if let Some(t) = tenant {
        e = e.tenant(t);
    }
    Ok(e)
}

/// US-1903 AC1-AC4: membuat perusahaan Aktif dan mengundang HR pertama. Lihat dokumentasi modul.
pub(super) async fn create(
    s: &CorporateVerification<'_>,
    principal: &Principal,
    route: &str,
    input: &NewCorporateClientInput,
) -> Result<CreatedCorporateClientView, CorporateError> {
    let ip = client_ip(principal, route)?;
    let v = validate(input)?;
    let ctx = audit_context(principal, ip);
    let now = s.clock.now();
    let tx_principal = principal.clone();
    let tx_ctx = ctx.clone();
    let result = in_audited_tx(s.pool, s.clock, move |conn| {
        Box::pin(async move {
            let actor_id = tx_principal.account_id();
            // 1. Akun HR + niat (kunci advisory email pertama).
            let invitee = register_hr_invitation(
                conn,
                &v.hr_email,
                IntentActor {
                    actor_type: "rs_admin",
                    account_id: Some(actor_id),
                },
                tx_principal.correlation_id(),
                &FixedClock::new(now),
            )
            .await
            .map_err(|e| match e {
                HrInviteError::EmailInUse => CorporateError::HrEmailAlreadyRegistered,
                HrInviteError::InvalidEmail => CorporateError::Validation(vec![field(
                    "hr_email",
                    "invalid",
                    "Email HR harus berupa alamat email yang sah",
                )]),
                HrInviteError::Database | HrInviteError::Queue => CorporateError::Database,
            })?;
            // 2. NIB ganda di antara perusahaan Aktif (CR-035).
            if repo::active_client_with_nib(conn, &v.nib)
                .await
                .map_err(db_error)?
                .is_some()
            {
                return Err(CorporateError::NibAlreadyRegistered);
            }
            // 3. Perusahaan Aktif + keanggotaan HR pertama.
            let client_id = new_id();
            let inserted = repo::insert_admin_client(
                conn,
                &NewAdminClient {
                    id: client_id,
                    created_by: actor_id,
                    legal_name: &v.legal_name,
                    nib: &v.nib,
                    npwp: &v.npwp,
                    office_address: &v.office_address,
                    office_phone: &v.office_phone,
                    now,
                },
            )
            .await
            .map_err(db_error)?;
            if !inserted {
                return Err(CorporateError::NibAlreadyRegistered);
            }
            let member_id = new_id();
            repo::insert_member(
                conn,
                member_id,
                client_id,
                invitee.account_id,
                actor_id,
                now,
            )
            .await
            .map_err(db_error)?;
            // 4. Event.
            let created = event(
                EventCode::E10,
                CLIENT_CREATED_ACTION,
                &tx_principal,
                &tx_ctx,
                ObjectRef::with_uuid(&CLIENT_POLICY, client_id),
                Some(client_id),
                Outcome::success(),
            )?
            .change(FieldChange::new(
                "status",
                None,
                Some(code_value("active")?),
            ))
            .change(FieldChange::new(
                "origin",
                None,
                Some(code_value("created_by_admin")?),
            ))
            .change(FieldChange::new(
                "submission_round",
                None,
                Some(Value::Int(1)),
            ));
            let invited = event(
                EventCode::E10,
                HR_INVITED_ACTION,
                &tx_principal,
                &tx_ctx,
                ObjectRef::with_uuid(&HR_ACCOUNT_POLICY, invitee.account_id),
                Some(client_id),
                Outcome::success(),
            )?
            .change(FieldChange::new(
                "account_type",
                None,
                Some(code_value("corporate_admin")?),
            ))
            .change(FieldChange::new(
                "status",
                None,
                Some(code_value("pending_provisioning")?),
            ));
            let account_created = event(
                EventCode::E04,
                ACCOUNT_CREATED_ACTION,
                &tx_principal,
                &tx_ctx,
                ObjectRef::with_uuid(&HR_ACCOUNT_POLICY, invitee.account_id),
                Some(client_id),
                Outcome::success(),
            )?
            .change(FieldChange::new(
                "account_type",
                None,
                Some(code_value("corporate_admin")?),
            ));
            let granted = event(
                EventCode::E11,
                MEMBERSHIP_GRANTED_ACTION,
                &tx_principal,
                &tx_ctx,
                ObjectRef::with_uuid(&HR_ACCOUNT_POLICY, invitee.account_id),
                Some(client_id),
                Outcome::success(),
            )?
            .change(FieldChange::new(
                "corporate_member_id",
                None,
                Some(Value::Uuid(member_id)),
            ))
            .change(FieldChange::new(
                "corporate_client_id",
                None,
                Some(Value::Uuid(client_id)),
            ));
            Ok((
                CreatedCorporateClientView {
                    corporate_client_id: client_id,
                    corporate_member_id: member_id,
                    hr_account_id: invitee.account_id,
                },
                vec![created, invited, account_created, granted],
            ))
        })
    })
    .await;
    match result {
        Ok(v) => Ok(v),
        Err(e) => {
            let e = match e {
                AuditedTxError::Work(e) => e,
                AuditedTxError::Audit(_) => CorporateError::Internal("audit"),
                AuditedTxError::Database { .. } => CorporateError::Database,
            };
            let reason = match e {
                CorporateError::NibAlreadyRegistered => Some(NIB_TAKEN_REASON),
                CorporateError::HrEmailAlreadyRegistered => Some(EMAIL_TAKEN_REASON),
                _ => None,
            };
            if let Some(reason) = reason {
                record_rejected(s.pool, s.clock, principal, &ctx, reason).await;
            }
            Err(e)
        }
    }
}

/// E-10 hasil gagal (NIB atau email ganda) di transaksi tersendiri, dicoba ulang; kegagalan tidak mengubah
/// jawaban (coding-standards §4.4 butir 3).
async fn record_rejected(
    pool: &PgPool,
    clock: &dyn Clock,
    principal: &Principal,
    ctx: &AuditContext,
    reason: &'static str,
) {
    let Ok(event) = code(reason).and_then(|r| {
        event(
            EventCode::E10,
            CREATE_REJECTED_ACTION,
            principal,
            ctx,
            ObjectRef::with_uuid(&ACTOR_ACCOUNT_POLICY, principal.account_id()),
            None,
            Outcome::failure(r),
        )
    }) else {
        tracing::error!("failed_outcome_audit_not_built");
        return;
    };
    for attempt in 1..=FAILED_AUDIT_ATTEMPTS {
        let ev = event.clone();
        let result = in_audited_tx(pool, clock, move |_conn| {
            Box::pin(async move { Ok::<_, Infallible>(((), vec![ev])) })
        })
        .await;
        match result {
            Ok(()) => return,
            Err(_) if attempt < FAILED_AUDIT_ATTEMPTS => {}
            Err(_) => tracing::error!("failed_outcome_audit_not_written"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> NewCorporateClientInput {
        NewCorporateClientInput {
            legal_name: "  PT Uji Sintetis  ".to_string(),
            nib: " 1234567890123 ".to_string(),
            npwp: Sensitive::new("012345678901234".to_string()),
            office_address: "Jl. Uji No. 1".to_string(),
            office_phone: "021-0000000".to_string(),
            hr_email: Sensitive::new(" HR@Uji.Example ".to_string()),
        }
    }

    // trace: US-1903 AC1 (isian dipangkas; email HR ternormalisasi)
    #[test]
    fn valid_input_is_trimmed_and_email_normalized() {
        let v = validate(&input()).ok().unwrap_or_else(|| unreachable!());
        assert_eq!(v.legal_name, "PT Uji Sintetis");
        assert_eq!(v.nib, "1234567890123");
        assert_eq!(v.hr_email, "hr@uji.example");
    }

    // trace: US-1903 AC1, coding-standards §5.1 (batas panjang dan karakter kontrol = galat isian)
    #[test]
    fn invalid_fields_are_reported_per_field() {
        let mut i = input();
        i.legal_name = "x".repeat(201);
        i.nib = "12 34".to_string();
        i.npwp = Sensitive::new(String::new());
        i.office_address = "alamat\u{7}".to_string();
        i.office_phone = "0".repeat(33);
        i.hr_email = Sensitive::new("bukan-email".to_string());
        let Err(CorporateError::Validation(errors)) = validate(&i) else {
            unreachable!("harus galat validasi");
        };
        let mut names: Vec<_> = errors.iter().map(|e| e.field.as_str()).collect();
        names.sort_unstable();
        assert_eq!(
            names,
            vec![
                "hr_email",
                "legal_name",
                "nib",
                "npwp",
                "office_address",
                "office_phone"
            ]
        );
    }

    // trace: DD A20 (NPWP Terbatas tidak pernah masuk payload audit), ADR-009 butir 7
    #[test]
    fn audit_policies_admit_no_identifying_field_and_actions_are_codes() {
        for p in [&CLIENT_POLICY, &HR_ACCOUNT_POLICY, &ACTOR_ACCOUNT_POLICY] {
            assert!(p.validate().is_ok());
            for f in [
                "npwp",
                "nib",
                "legal_name",
                "office_address",
                "office_phone",
                "email",
                "hr_email",
            ] {
                assert!(p.fields().iter().all(|r| r.name() != f), "{f}");
            }
        }
        for a in [
            CLIENT_CREATED_ACTION,
            HR_INVITED_ACTION,
            MEMBERSHIP_GRANTED_ACTION,
            CREATE_REJECTED_ACTION,
            NIB_TAKEN_REASON,
            EMAIL_TAKEN_REASON,
        ] {
            assert!(Code::new(a).is_ok(), "{a}");
        }
    }
}
