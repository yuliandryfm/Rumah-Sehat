//! Use case pendaftaran perusahaan mandiri oleh calon HR (F1-T101, T1; US-1901 AC1, AC3-AC7; BR-136;
//! ADR-027 butir 3; ADR-030 butir 2, 5, 7; KEPUTUSAN-303, KEPUTUSAN-321 s.d. KEPUTUSAN-327).
//!
//! Akses: `POST /v1/public/corporate-registrations` tanpa login (`PublicReason::CorporateRegistration`,
//! KEPUTUSAN-323): calon HR belum punya akun. Handler (`registration_http.rs`) menegakkan batas laju dan
//! menyamakan waktu respons; use case ini menegakkan isian, persetujuan, dan efek data.
//!
//! Urutan, semuanya sebelum transaksi dibuka tanpa efek: (1) validasi isian per field (422, tidak bergantung pada
//! keberadaan NIB atau email); (2) teks persetujuan HR yang berlaku diambil lewat `privacy::api` dengan
//! `DraftPolicy` aplikasi (production menolak teks draf, 503) dan pilihan butir diperiksa (`4090106` bila versi
//! berganti, 422 bila butir wajib tidak dipilih).
//!
//! Satu transaksi `audit::in_audited_tx` (KEPUTUSAN-125: tanpa panggilan Keycloak; efeknya dijalankan worker IdP):
//! 1. `register_hr_self_registration`: kunci advisory (realm, email) PERTAMA, email diperiksa belum dipakai akun
//!    belum terhapus, akun HR `pending_provisioning` tanpa password (KEPUTUSAN-303 R1), niat `create_user` dan job
//!    `idp_create_user` (tujuan `hr_self_registration`). Email aktivasi (VERIFY_EMAIL, UPDATE_PASSWORD,
//!    CONFIGURE_TOTP) dikirim worker setelah pengguna dibuat.
//! 2. NIB diperiksa dan perusahaan disisipkan `pending_verification` (`origin = self_registered`, putaran 1);
//!    `ON CONFLICT DO NOTHING` menjaga balapan NIB.
//! 3. Keanggotaan HR beserta jabatan, di transaksi yang sama (akun terikat ke perusahaan sejak awal).
//! 4. Bukti persetujuan per butir (`privacy::api::record_granted_consent`, subjek `CorporateHr`) dan event.
//!
//! Anti-enumerasi (KEPUTUSAN-324): email yang sudah dipakai, atau NIB yang sudah dipakai perusahaan belum terhapus
//! (status apa pun), TIDAK menghasilkan respons berbeda. Transaksi dibatalkan (nol efek: tanpa akun, niat, job,
//! perusahaan, bukti persetujuan) dan use case mengembalikan hasil yang sama dengan pendaftaran baru; penolakan
//! hanya tercatat di audit (E-10 hasil gagal, transaksi tersendiri, aktor `system`, objek = ID percobaan
//! baru tanpa NIB atau email di payload, sehingga tidak membuka perusahaan atau akun lain). Ini berbeda dari
//! `4090110` dan `4090111` jalur Verifikator (`admin_create.rs`): di sana pemanggil staf berwenang yang
//! terautentikasi dan perlu tahu penyebabnya; di sini pemanggil anonim.
//!
//! Pemberitahuan "Anda sudah punya akun" ke email terdaftar BELUM dikirim (jalur email aplikasi, modul Notifikasi,
//! `[TIDAK DIKETAHUI]`; keputusan user Q5=A: Fase 1 cukup pesan seragam di respons/UI).

use super::admin_create::{
    ADDRESS_MAX, CLIENT_POLICY, FAILED_AUDIT_ATTEMPTS, HR_ACCOUNT_POLICY, LEGAL_NAME_MAX,
    PHONE_MAX, code, code_value, db_error, field, text,
};
use super::domain::{nib_format, npwp_format};
use super::error::CorporateError;
use super::repo::{self, NewSelfRegisteredClient};
use crate::modules::identity::api::{
    HrInviteError, normalize_hr_email, register_hr_self_registration,
};
use crate::modules::privacy::api::{
    ConsentSubject, ConsentType, DraftPolicy, PrivacyError, SelectionCheck, active_consent_text,
    check_selection, record_granted_consent,
};
use audit::{
    Actor, AuditContext, AuditEvent, AuditedTxError, EventCode, FieldChange, ObjectPolicy,
    ObjectRef, Outcome, Source, SourceComponent, Value, in_audited_tx,
};
use foundation::{Clock, CorrelationId, FixedClock, Sensitive, Uuid, new_id};
use sqlx::PgPool;
use std::convert::Infallible;
use std::fmt;
use std::net::IpAddr;

const CLIENT_REGISTERED_ACTION: &str = "corporate_client_self_registered";
const HR_REGISTERED_ACTION: &str = "corporate_admin_account_registered";
const MEMBERSHIP_GRANTED_ACTION: &str = "corporate_membership_granted";
const REGISTRATION_REJECTED_ACTION: &str = "corporate_registration_rejected";
const NIB_TAKEN_REASON: &str = "nib_already_registered";
const EMAIL_TAKEN_REASON: &str = "hr_email_in_use";

/// Batas panjang nama dan jabatan HR (jabatan = CHECK `ck_corporate_member_job_title_length`).
const HR_NAME_MAX: usize = 120;
const JOB_TITLE_MAX: usize = 100;
/// Batas jumlah butir persetujuan yang dikirim (pertahanan; teks nyata memuat sedikit butir).
pub(super) const CONSENT_ITEMS_MAX: usize = 20;

/// Objek event penolakan: percobaan pendaftaran (ID acak baru), tanpa field.
static ATTEMPT_POLICY: ObjectPolicy = ObjectPolicy::non_clinical("corporate_registration", &[]);

/// Dependensi dan konteks permintaan pendaftaran (dipinjam dari keadaan aplikasi per permintaan).
#[derive(Clone, Copy)]
pub struct RegistrationDeps<'a> {
    /// Pool role aplikasi (`rs_app`).
    pub pool: &'a PgPool,
    /// Jam server (KEPUTUSAN-131).
    pub clock: &'a dyn Clock,
    /// Kebijakan teks persetujuan draf (production menolak, KEPUTUSAN-289).
    pub drafts: DraftPolicy,
    /// ID korelasi permintaan.
    pub correlation_id: CorrelationId,
    /// Alamat peer TCP peminta (wajib untuk audit "Sumber"; tanpa itu pendaftaran ditolak gagal-tertutup).
    pub client_ip: Option<IpAddr>,
}

impl fmt::Debug for RegistrationDeps<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RegistrationDeps").finish_non_exhaustive()
    }
}

/// Isian pendaftaran. NPWP (Terbatas), nama, dan email HR dibungkus [`Sensitive`]; `Debug` manual tanpa isian.
pub struct NewCorporateRegistration {
    /// Nama badan usaha.
    pub legal_name: String,
    /// NIB: tepat 13 angka (KEPUTUSAN-325).
    pub nib: String,
    /// NPWP badan: tepat 16 angka (KEPUTUSAN-325), Terbatas.
    pub npwp: Sensitive<String>,
    /// Alamat kantor.
    pub office_address: String,
    /// Telepon kantor.
    pub office_phone: String,
    /// Nama HR pendaftar.
    pub hr_full_name: Sensitive<String>,
    /// Jabatan HR pendaftar.
    pub hr_job_title: String,
    /// Email HR pendaftar (penerima tautan aktivasi).
    pub hr_email: Sensitive<String>,
    /// Versi teks persetujuan yang dilihat pendaftar.
    pub consent_text_version_id: Uuid,
    /// Butir persetujuan yang dicentang.
    pub consent_item_ids: Vec<Uuid>,
}

impl fmt::Debug for NewCorporateRegistration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NewCorporateRegistration")
            .finish_non_exhaustive()
    }
}

/// Isian tervalidasi.
struct Validated {
    legal_name: String,
    nib: String,
    npwp: String,
    office_address: String,
    office_phone: String,
    hr_full_name: String,
    hr_job_title: String,
    hr_email: String,
}

fn validate(input: &NewCorporateRegistration) -> Result<Validated, CorporateError> {
    let mut errors = Vec::new();
    let mut take = |name: &str, raw: &str, max: usize, message: &str| {
        let v = text(raw, max, false);
        if v.is_none() {
            errors.push(field(name, "invalid", message));
        }
        v.unwrap_or_default()
    };
    let legal_name = take(
        "legal_name",
        &input.legal_name,
        LEGAL_NAME_MAX,
        "Nama badan usaha wajib, maksimal 200 karakter",
    );
    let office_address = take(
        "office_address",
        &input.office_address,
        ADDRESS_MAX,
        "Alamat kantor wajib, maksimal 500 karakter",
    );
    let office_phone = take(
        "office_phone",
        &input.office_phone,
        PHONE_MAX,
        "Telepon kantor wajib, maksimal 32 karakter",
    );
    let hr_full_name = take(
        "hr_full_name",
        input.hr_full_name.expose(),
        HR_NAME_MAX,
        "Nama wajib, maksimal 120 karakter",
    );
    let hr_job_title = take(
        "hr_job_title",
        &input.hr_job_title,
        JOB_TITLE_MAX,
        "Jabatan wajib, maksimal 100 karakter",
    );
    let nib = nib_format(&input.nib).unwrap_or_else(|| {
        errors.push(field("nib", "invalid_format", "NIB harus tepat 13 angka"));
        String::new()
    });
    let npwp = npwp_format(input.npwp.expose()).unwrap_or_else(|| {
        errors.push(field(
            "npwp",
            "invalid_format",
            "NPWP harus tepat 16 angka (NPWP 15 angka lama: tambahkan angka 0 di depan)",
        ));
        String::new()
    });
    let hr_email = normalize_hr_email(input.hr_email.expose()).unwrap_or_else(|| {
        errors.push(field(
            "hr_email",
            "invalid",
            "Email harus berupa alamat email yang sah",
        ));
        String::new()
    });
    if input.consent_item_ids.len() > CONSENT_ITEMS_MAX {
        errors.push(field(
            "consent_item_ids",
            "invalid",
            "Terlalu banyak butir persetujuan",
        ));
    }
    if errors.is_empty() {
        Ok(Validated {
            legal_name,
            nib,
            npwp,
            office_address,
            office_phone,
            hr_full_name,
            hr_job_title,
            hr_email,
        })
    } else {
        Err(CorporateError::Validation(errors))
    }
}

fn client_ip(deps: &RegistrationDeps<'_>) -> Result<IpAddr, CorporateError> {
    deps.client_ip.ok_or_else(|| {
        tracing::error!("client_ip_missing");
        CorporateError::Internal("client_ip_missing")
    })
}

fn privacy_error(e: PrivacyError) -> CorporateError {
    match e {
        PrivacyError::Internal(w) => CorporateError::Internal(w),
        PrivacyError::RateLimited | PrivacyError::Unavailable => CorporateError::Database,
    }
}

fn hr_actor(account_id: Uuid) -> Result<Actor, CorporateError> {
    Ok(Actor::account(
        account_id,
        code("corporate_admin")?,
        Vec::new(),
    ))
}

fn build_event(
    ev: EventCode,
    action: &str,
    actor: Actor,
    ctx: &AuditContext,
    object: ObjectRef,
    tenant: Uuid,
) -> Result<AuditEvent, CorporateError> {
    Ok(AuditEvent::new(ev, code(action)?, actor, object)
        .tenant(tenant)
        .outcome(Outcome::success())
        .context(ctx))
}

/// US-1901 AC1, AC3-AC7: mendaftarkan perusahaan. `Ok(())` untuk pendaftaran baru MAUPUN untuk NIB atau email
/// yang sudah terdaftar (jawaban seragam, KEPUTUSAN-324). Lihat dokumentasi modul.
pub(super) async fn register(
    deps: &RegistrationDeps<'_>,
    input: &NewCorporateRegistration,
) -> Result<(), CorporateError> {
    let ip = client_ip(deps)?;
    let v = validate(input)?;
    let consent = active_consent_text(
        deps.pool,
        deps.clock,
        ConsentType::CorporateHrRegistration,
        deps.drafts,
    )
    .await
    .map_err(privacy_error)?
    .ok_or_else(|| {
        tracing::error!("corporate_consent_text_missing");
        CorporateError::Database
    })?;
    let selected = match check_selection(
        consent.version_id,
        &consent.items,
        input.consent_text_version_id,
        &input.consent_item_ids,
    ) {
        SelectionCheck::Accepted(ids) => ids,
        SelectionCheck::Outdated => return Err(CorporateError::ConsentOutdated),
        SelectionCheck::MissingRequired => {
            return Err(CorporateError::Validation(vec![field(
                "consent_item_ids",
                "required",
                "Persetujuan wajib dicentang untuk mendaftar",
            )]));
        }
    };
    let items: Vec<_> = consent
        .items
        .iter()
        .filter(|i| selected.contains(&i.id))
        .cloned()
        .collect();
    let version_id = consent.version_id;
    let ctx = AuditContext::new(
        deps.correlation_id,
        Source::component(SourceComponent::CorporatePortal).ip(ip),
    );
    let now = deps.clock.now();
    let correlation_id = deps.correlation_id;
    let tx_ctx = ctx.clone();
    let result = in_audited_tx(deps.pool, deps.clock, move |conn| {
        Box::pin(async move {
            // 1. Akun HR tanpa password + niat (kunci advisory email pertama).
            let hr = register_hr_self_registration(
                conn,
                &v.hr_email,
                &v.hr_full_name,
                correlation_id,
                &FixedClock::new(now),
            )
            .await
            .map_err(|e| match e {
                HrInviteError::EmailInUse => CorporateError::HrEmailAlreadyRegistered,
                HrInviteError::InvalidEmail => CorporateError::Validation(vec![field(
                    "hr_email",
                    "invalid",
                    "Email harus berupa alamat email yang sah",
                )]),
                HrInviteError::Database | HrInviteError::Queue => CorporateError::Database,
            })?;
            // 2. NIB ganda.
            if repo::client_with_nib(conn, &v.nib)
                .await
                .map_err(db_error)?
                .is_some()
            {
                return Err(CorporateError::NibAlreadyRegistered);
            }
            let client_id = new_id();
            let inserted = repo::insert_self_registered_client(
                conn,
                &NewSelfRegisteredClient {
                    id: client_id,
                    created_by: hr.account_id,
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
            // 3. Keanggotaan HR pendaftar.
            let member_id = new_id();
            repo::insert_member_with_title(
                conn,
                member_id,
                client_id,
                hr.account_id,
                &v.hr_job_title,
                now,
            )
            .await
            .map_err(db_error)?;
            // 4. Bukti persetujuan + event E-13 per butir.
            let mut events = record_granted_consent(
                conn,
                ConsentSubject::CorporateHr {
                    account_id: hr.account_id,
                    corporate_client_id: client_id,
                },
                version_id,
                &items,
                &tx_ctx,
                correlation_id.as_uuid(),
            )
            .await
            .map_err(privacy_error)?;
            let registered = build_event(
                EventCode::E10,
                CLIENT_REGISTERED_ACTION,
                hr_actor(hr.account_id)?,
                &tx_ctx,
                ObjectRef::with_uuid(&CLIENT_POLICY, client_id),
                client_id,
            )?
            .change(FieldChange::new(
                "status",
                None,
                Some(code_value("pending_verification")?),
            ))
            .change(FieldChange::new(
                "origin",
                None,
                Some(code_value("self_registered")?),
            ))
            .change(FieldChange::new(
                "submission_round",
                None,
                Some(Value::Int(1)),
            ));
            let account = build_event(
                EventCode::E10,
                HR_REGISTERED_ACTION,
                hr_actor(hr.account_id)?,
                &tx_ctx,
                ObjectRef::with_uuid(&HR_ACCOUNT_POLICY, hr.account_id),
                client_id,
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
            let granted = build_event(
                EventCode::E11,
                MEMBERSHIP_GRANTED_ACTION,
                hr_actor(hr.account_id)?,
                &tx_ctx,
                ObjectRef::with_uuid(&HR_ACCOUNT_POLICY, hr.account_id),
                client_id,
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
            events.extend([registered, account, granted]);
            Ok(((), events))
        })
    })
    .await;
    match result {
        Ok(()) => Ok(()),
        Err(e) => {
            let e = match e {
                AuditedTxError::Work(e) => e,
                AuditedTxError::Audit(_) => CorporateError::Internal("audit"),
                AuditedTxError::Database { .. } => CorporateError::Database,
            };
            let reason = match e {
                CorporateError::NibAlreadyRegistered => NIB_TAKEN_REASON,
                CorporateError::HrEmailAlreadyRegistered => EMAIL_TAKEN_REASON,
                other => return Err(other),
            };
            // Jawaban seragam: NIB atau email terdaftar tidak dapat dibedakan dari pendaftaran baru.
            record_rejected(deps.pool, deps.clock, &ctx, reason).await;
            Ok(())
        }
    }
}

/// E-10 hasil gagal (NIB atau email sudah terdaftar) di transaksi tersendiri, dicoba ulang; kegagalan tidak
/// mengubah jawaban (coding-standards §4.4 butir 3).
async fn record_rejected(
    pool: &PgPool,
    clock: &dyn Clock,
    ctx: &AuditContext,
    reason: &'static str,
) {
    let built = code(reason).and_then(|r| {
        Ok(AuditEvent::new(
            EventCode::E10,
            code(REGISTRATION_REJECTED_ACTION)?,
            Actor::system(),
            ObjectRef::with_uuid(&ATTEMPT_POLICY, new_id()),
        )
        .outcome(Outcome::failure(r))
        .context(ctx))
    });
    let Ok(event) = built else {
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

    fn input() -> NewCorporateRegistration {
        NewCorporateRegistration {
            legal_name: "  PT Uji Sintetis  ".to_string(),
            nib: " 1234567890123 ".to_string(),
            npwp: Sensitive::new("0123456789012345".to_string()),
            office_address: "Jl. Uji No. 1".to_string(),
            office_phone: "021-0000000".to_string(),
            hr_full_name: Sensitive::new("HR Uji".to_string()),
            hr_job_title: "Manajer SDM".to_string(),
            hr_email: Sensitive::new(" HR@Uji.Example ".to_string()),
            consent_text_version_id: Uuid::nil(),
            consent_item_ids: vec![],
        }
    }

    // trace: US-1901 AC1, AC3
    #[test]
    fn valid_input_is_trimmed_and_email_normalized() {
        let v = validate(&input()).ok().unwrap_or_else(|| unreachable!());
        assert_eq!(v.legal_name, "PT Uji Sintetis");
        assert_eq!(v.nib, "1234567890123");
        assert_eq!(v.npwp, "0123456789012345");
        assert_eq!(v.hr_email, "hr@uji.example");
    }

    // trace: US-1901 AC3 (format NIB 13 angka, NPWP 16 angka; per field)
    #[test]
    fn invalid_fields_are_reported_per_field() {
        let mut i = input();
        i.legal_name = "x".repeat(201);
        i.nib = "12345678901234".to_string();
        i.npwp = Sensitive::new("012345678901234".to_string());
        i.office_address = "alamat\u{7}".to_string();
        i.office_phone = "0".repeat(33);
        i.hr_full_name = Sensitive::new(String::new());
        i.hr_job_title = "j".repeat(101);
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
                "hr_full_name",
                "hr_job_title",
                "legal_name",
                "nib",
                "npwp",
                "office_address",
                "office_phone"
            ]
        );
    }

    // trace: DD A20 (NPWP Terbatas), ADR-009 butir 7: kebijakan audit tanpa field pengenal, aksi berupa kode
    #[test]
    fn audit_actions_are_codes_and_attempt_policy_is_valid() {
        assert!(ATTEMPT_POLICY.validate().is_ok());
        assert!(ATTEMPT_POLICY.fields().is_empty());
        for a in [
            CLIENT_REGISTERED_ACTION,
            HR_REGISTERED_ACTION,
            MEMBERSHIP_GRANTED_ACTION,
            REGISTRATION_REJECTED_ACTION,
            NIB_TAKEN_REASON,
            EMAIL_TAKEN_REASON,
        ] {
            assert!(audit::Code::new(a).is_ok(), "{a}");
        }
    }
}
