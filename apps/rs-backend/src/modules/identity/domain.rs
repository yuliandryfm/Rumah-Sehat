//! Pemetaan event Keycloak 26.8.2 ke E-01–E-04 (PRD §7.1, ADR-011 butir 10, KEPUTUSAN-85 "pemetaan event
//! ditetapkan per versi saat implementasi di Tahap 8"). Murni, tanpa I/O.
//!
//! Dasar versi: enum `EventType`, konstanta `Details`/`Errors`, dan pemancar event `UpdatePassword`/
//! `UpdateTotp` di kode sumber tag 26.8.2 (docs/sources.md SRC-622, SRC-623). Event usang `UPDATE_PASSWORD`,
//! `UPDATE_TOTP`, dan `REMOVE_TOTP` (beserta `_ERROR`) TIDAK dipetakan: Keycloak 26.8.2 memancarkannya sebagai
//! salinan dari `UPDATE_CREDENTIAL`/`REMOVE_CREDENTIAL` untuk aksi yang sama, sehingga memetakannya akan
//! mencatat satu perubahan kredensial dua kali.
//!
//! Isi event Keycloak yang memuat data pribadi atau rahasia (`details.username` = email karena email sebagai
//! username, `details.email`, `details.reason` teks bebas, `token_id`, `refresh_token_id`, `code_id`,
//! `sessionId`, `redirect_uri`) tidak pernah dibaca ke tipe Rust: [`KcEvent`] hanya menguraikan field yang
//! dipakai pemetaan, dan `details` hanya `credential_type` yang dipetakan ke nama aksi tertutup.
//! Kode alasan berasal dari daftar izin [`reason_code`], bukan diteruskan dari Keycloak.

use audit::{EventCode, SourceComponent};
use foundation::Uuid;
use serde::Deserialize;
use std::net::IpAddr;
use std::str::FromStr;
use time::OffsetDateTime;

/// Realm Keycloak milik backend Ruang Sehat (DD A1.1 `idp_realm`; ADR-011 butir 11: realm `rme` bukan milik
/// proses ini).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IdpRealm {
    /// Realm pasien.
    Pasien,
    /// Realm tenaga kesehatan (belum dibuat sebagai kode; F1-T31 baru `pasien` dan `admin`).
    Nakes,
    /// Realm admin Ruang Sehat dan admin rekanan.
    Admin,
}

impl IdpRealm {
    /// Nama realm (= nilai `account.idp_realm`).
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pasien => "pasien",
            Self::Nakes => "nakes",
            Self::Admin => "admin",
        }
    }

    /// Komponen sumber bila `clientId` event tidak dikenal.
    fn default_component(self) -> SourceComponent {
        match self {
            Self::Pasien => SourceComponent::PatientApp,
            Self::Nakes => SourceComponent::PractitionerApp,
            Self::Admin => SourceComponent::Backoffice,
        }
    }
}

impl FromStr for IdpRealm {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pasien" => Ok(Self::Pasien),
            "nakes" => Ok(Self::Nakes),
            "admin" => Ok(Self::Admin),
            _ => Err(()),
        }
    }
}

/// Jenis event Keycloak yang diambil (parameter `type` Admin REST). Nama yang tidak ada di enum
/// `EventType` membuat Keycloak 26.8.2 menjawab HTTP 500 (diamati, SRC-621), jadi daftar ini wajib persis.
pub const PULLED_TYPES: &[&str] = &[
    "LOGIN",
    "LOGIN_ERROR",
    "USER_DISABLED_BY_TEMPORARY_LOCKOUT",
    "USER_DISABLED_BY_PERMANENT_LOCKOUT",
    "LOGOUT",
    "LOGOUT_ERROR",
    "REGISTER",
    "REGISTER_ERROR",
    "UPDATE_CREDENTIAL",
    "UPDATE_CREDENTIAL_ERROR",
    "REMOVE_CREDENTIAL",
    "REMOVE_CREDENTIAL_ERROR",
    "RESET_PASSWORD",
    "RESET_PASSWORD_ERROR",
    "SEND_RESET_PASSWORD",
    "SEND_RESET_PASSWORD_ERROR",
];

/// Batas panjang ID pengguna Keycloak yang diterima (UUID 36 karakter; sisa ruang untuk ID federasi).
const MAX_USER_ID_LEN: usize = 64;
/// Batas panjang `clientId` yang dibaca.
const MAX_CLIENT_ID_LEN: usize = 255;

/// Satu event dari `GET /admin/realms/{realm}/events` (EventRepresentation 26.8.2, SRC-621). Field lain
/// (termasuk `details.username`, `details.email`, `details.reason`, `details.token_id`, `sessionId`)
/// dilewati serde tanpa disalin.
#[derive(Deserialize)]
pub struct RawEvent {
    id: Option<String>,
    time: Option<i64>,
    #[serde(rename = "type")]
    kind: Option<String>,
    #[serde(rename = "clientId")]
    client_id: Option<String>,
    #[serde(rename = "userId")]
    user_id: Option<String>,
    #[serde(rename = "ipAddress")]
    ip_address: Option<String>,
    error: Option<String>,
    details: Option<RawDetails>,
}

#[derive(Deserialize)]
struct RawDetails {
    credential_type: Option<String>,
}

/// Event yang sudah divalidasi. Tidak memuat email, username, token, atau ID sesi.
#[derive(Clone, PartialEq, Eq)]
pub struct KcEvent {
    /// ID event Keycloak (UUID).
    pub id: Uuid,
    /// Waktu event di server Keycloak (milidetik epoch).
    pub time_ms: i64,
    /// Waktu event.
    pub time: OffsetDateTime,
    /// Pemetaan ke PRD §7.1; `None` = jenis yang tidak dicatat.
    pub mapped: Option<Mapped>,
    /// ID pengguna Keycloak (`account.idp_subject`), bila ada.
    pub user_id: Option<String>,
    /// Alamat IP peminta menurut Keycloak.
    pub ip: Option<IpAddr>,
    /// Komponen sumber dari `clientId`.
    pub component: SourceComponent,
}

// Debug tanpa ID pengguna dan alamat IP (data pribadi, ADR-010 butir 1).
impl std::fmt::Debug for KcEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KcEvent")
            .field("id", &self.id)
            .field("time_ms", &self.time_ms)
            .field("mapped", &self.mapped)
            .finish_non_exhaustive()
    }
}

/// Field event yang tidak valid (nama statis, tanpa nilai).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidEvent(pub &'static str);

/// Hasil pemetaan satu event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mapped {
    /// Kode event PRD §7.1.
    pub code: EventCode,
    /// Nama aksi (kode tertutup).
    pub action: &'static str,
    /// `None` = sukses; `Some(alasan)` = gagal dengan kode alasan tertutup.
    pub failure: Option<&'static str>,
}

impl RawEvent {
    /// Memvalidasi dan memetakan event. `realm` menentukan komponen bawaan.
    pub fn validate(self, realm: IdpRealm) -> Result<KcEvent, InvalidEvent> {
        let id = self
            .id
            .as_deref()
            .and_then(|s| Uuid::parse_str(s).ok())
            .ok_or(InvalidEvent("id"))?;
        let time_ms = self.time.filter(|t| *t >= 0).ok_or(InvalidEvent("time"))?;
        let time = OffsetDateTime::from_unix_timestamp_nanos(i128::from(time_ms) * 1_000_000)
            .map_err(|_| InvalidEvent("time"))?;
        let kind = self.kind.ok_or(InvalidEvent("type"))?;
        let credential = self.details.and_then(|d| d.credential_type);
        let mapped = map_event(&kind, self.error.as_deref(), credential.as_deref());
        let user_id = match self.user_id {
            Some(u) if u.is_empty() => None,
            Some(u) if u.len() <= MAX_USER_ID_LEN && u.bytes().all(is_id_byte) => Some(u),
            Some(_) => return Err(InvalidEvent("userId")),
            None => None,
        };
        let ip = match self.ip_address.as_deref() {
            None | Some("") => None,
            Some(s) => Some(IpAddr::from_str(s).map_err(|_| InvalidEvent("ipAddress"))?),
        };
        let component = self
            .client_id
            .as_deref()
            .filter(|c| c.len() <= MAX_CLIENT_ID_LEN)
            .and_then(component_for_client)
            .unwrap_or_else(|| realm.default_component());
        Ok(KcEvent {
            id,
            time_ms,
            time,
            mapped,
            user_id,
            ip,
            component,
        })
    }
}

fn is_id_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b == b':' || b == b'.'
}

/// Klien realm sebagai kode (`infra/keycloak/realms/*.json`) ke komponen sumber DD B1.
fn component_for_client(client_id: &str) -> Option<SourceComponent> {
    match client_id {
        "rs-pasien-mobile" => Some(SourceComponent::PatientApp),
        "rs-backoffice-bff" => Some(SourceComponent::Backoffice),
        "rs-portal-bff" => Some(SourceComponent::PartnerPortal),
        _ => None,
    }
}

/// Pemetaan jenis event (dan `credential_type` untuk perubahan kredensial) ke kode PRD dan nama aksi.
/// Jenis di luar [`PULLED_TYPES`] tidak dipetakan.
pub fn map_event(kind: &str, error: Option<&str>, credential: Option<&str>) -> Option<Mapped> {
    let (base, is_error) = match kind.strip_suffix("_ERROR") {
        Some(b) => (b, true),
        None => (kind, false),
    };
    if !PULLED_TYPES.contains(&kind) {
        return None;
    }
    let (code, action) = match base {
        "LOGIN" => (EventCode::E01, "login"),
        "USER_DISABLED_BY_TEMPORARY_LOCKOUT" => (EventCode::E02, "account_locked_temporarily"),
        "USER_DISABLED_BY_PERMANENT_LOCKOUT" => (EventCode::E02, "account_locked_permanently"),
        "LOGOUT" => (EventCode::E03, "logout"),
        "REGISTER" => (EventCode::E04, "account_registered"),
        "UPDATE_CREDENTIAL" => (EventCode::E04, credential_action(credential, false)),
        "REMOVE_CREDENTIAL" => (EventCode::E04, credential_action(credential, true)),
        "RESET_PASSWORD" => (EventCode::E04, "password_reset"),
        "SEND_RESET_PASSWORD" => (EventCode::E04, "password_reset_requested"),
        _ => return None,
    };
    // LOGIN_ERROR adalah E-02 (login gagal, termasuk akun terkunci), bukan E-01 gagal.
    let (code, action) = if base == "LOGIN" && is_error {
        (EventCode::E02, "login_failed")
    } else {
        (code, action)
    };
    let failure = is_error.then(|| reason_code(error));
    Some(Mapped {
        code,
        action,
        failure,
    })
}

fn credential_action(credential: Option<&str>, removed: bool) -> &'static str {
    match (credential, removed) {
        (Some("password"), false) => "password_changed",
        (Some("password"), true) => "password_removed",
        (Some("otp"), false) => "otp_configured",
        (Some("otp"), true) => "otp_removed",
        (Some("recovery-authn-codes"), false) => "recovery_codes_created",
        (Some("recovery-authn-codes"), true) => "recovery_codes_removed",
        (_, false) => "credential_updated",
        (_, true) => "credential_removed",
    }
}

/// Kode alasan gagal dari daftar izin konstanta `Errors` Keycloak 26.8.2 (SRC-623). Nilai lain, atau galat
/// kosong, menjadi `idp_error_other`; teks Keycloak tidak pernah diteruskan.
pub fn reason_code(error: Option<&str>) -> &'static str {
    match error {
        Some("invalid_user_credentials") => "invalid_user_credentials",
        Some("user_not_found") => "user_not_found",
        Some("user_disabled") => "user_disabled",
        Some("user_temporarily_disabled") => "user_temporarily_disabled",
        Some("resolve_required_actions") => "resolve_required_actions",
        Some("invalid_authentication_session") => "invalid_authentication_session",
        Some("invalid_user") => "invalid_user",
        Some("invalid_client") => "invalid_client",
        Some("invalid_redirect_uri") => "invalid_redirect_uri",
        Some("invalid_code") => "invalid_code",
        Some("expired_code") => "expired_code",
        Some("cookie_not_found") => "cookie_not_found",
        Some("not_allowed") => "not_allowed",
        Some("user_session_not_found") => "user_session_not_found",
        Some("email_send_failed") => "email_send_failed",
        Some("invalid_email") => "invalid_email",
        Some("email_in_use") => "email_in_use",
        Some("username_in_use") => "username_in_use",
        Some("invalid_registration") => "invalid_registration",
        Some("password_rejected") => "password_rejected",
        Some("password_missing") => "password_missing",
        Some("password_confirm_error") => "password_confirm_error",
        Some("access_denied") => "access_denied",
        Some("generic_authentication_error") => "generic_authentication_error",
        _ => "idp_error_other",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Nama `EventType` Keycloak 26.8.2 yang relevan (kode sumber tag 26.8.2, SRC-622), tanpa yang bertanda
    /// `@Deprecated`.
    const KC_26_8_2_TYPES: &[&str] = &[
        "LOGIN",
        "LOGIN_ERROR",
        "REGISTER",
        "REGISTER_ERROR",
        "LOGOUT",
        "LOGOUT_ERROR",
        "CODE_TO_TOKEN",
        "CODE_TO_TOKEN_ERROR",
        "CLIENT_LOGIN",
        "CLIENT_LOGIN_ERROR",
        "UPDATE_CREDENTIAL",
        "UPDATE_CREDENTIAL_ERROR",
        "REMOVE_CREDENTIAL",
        "REMOVE_CREDENTIAL_ERROR",
        "RESET_PASSWORD",
        "RESET_PASSWORD_ERROR",
        "SEND_RESET_PASSWORD",
        "SEND_RESET_PASSWORD_ERROR",
        "USER_DISABLED_BY_PERMANENT_LOCKOUT",
        "USER_DISABLED_BY_TEMPORARY_LOCKOUT",
        "VERIFY_EMAIL",
        "VERIFY_EMAIL_ERROR",
    ];

    #[test]
    fn pulled_types_exist_in_keycloak_26_8_2_and_exclude_deprecated_copies() {
        for t in PULLED_TYPES {
            assert!(
                KC_26_8_2_TYPES.contains(t),
                "{t} bukan EventType 26.8.2 (Keycloak menjawab 500)"
            );
        }
        for deprecated in [
            "UPDATE_PASSWORD",
            "UPDATE_TOTP",
            "REMOVE_TOTP",
            "UPDATE_PASSWORD_ERROR",
        ] {
            assert!(!PULLED_TYPES.contains(&deprecated));
            assert_eq!(map_event(deprecated, None, None), None);
        }
        let mut seen = std::collections::HashSet::new();
        assert!(PULLED_TYPES.iter().all(|t| seen.insert(*t)), "duplikat");
    }

    #[test]
    fn event_types_map_to_e01_e04() {
        let m = |k: &str, e: Option<&str>, c: Option<&str>| {
            map_event(k, e, c).map(|x| (x.code, x.action, x.failure))
        };
        assert_eq!(
            m("LOGIN", None, None),
            Some((EventCode::E01, "login", None))
        );
        assert_eq!(
            m("LOGIN_ERROR", Some("invalid_user_credentials"), None),
            Some((
                EventCode::E02,
                "login_failed",
                Some("invalid_user_credentials")
            ))
        );
        assert_eq!(
            m("LOGIN_ERROR", Some("user_temporarily_disabled"), None),
            Some((
                EventCode::E02,
                "login_failed",
                Some("user_temporarily_disabled")
            ))
        );
        assert_eq!(
            m("USER_DISABLED_BY_TEMPORARY_LOCKOUT", None, None),
            Some((EventCode::E02, "account_locked_temporarily", None))
        );
        assert_eq!(
            m("LOGOUT", None, None),
            Some((EventCode::E03, "logout", None))
        );
        assert_eq!(
            m("REGISTER", None, None),
            Some((EventCode::E04, "account_registered", None))
        );
        assert_eq!(
            m("UPDATE_CREDENTIAL", None, Some("password")),
            Some((EventCode::E04, "password_changed", None))
        );
        assert_eq!(
            m("UPDATE_CREDENTIAL", None, Some("otp")),
            Some((EventCode::E04, "otp_configured", None))
        );
        assert_eq!(
            m("UPDATE_CREDENTIAL", None, Some("recovery-authn-codes")),
            Some((EventCode::E04, "recovery_codes_created", None))
        );
        assert_eq!(
            m("REMOVE_CREDENTIAL", None, Some("otp")),
            Some((EventCode::E04, "otp_removed", None))
        );
        assert_eq!(
            m("UPDATE_CREDENTIAL", None, Some("webauthn")),
            Some((EventCode::E04, "credential_updated", None))
        );
        assert_eq!(
            m(
                "UPDATE_CREDENTIAL_ERROR",
                Some("password_rejected"),
                Some("password")
            ),
            Some((
                EventCode::E04,
                "password_changed",
                Some("password_rejected")
            ))
        );
        assert_eq!(
            m("RESET_PASSWORD", None, None),
            Some((EventCode::E04, "password_reset", None))
        );
        assert_eq!(
            m("SEND_RESET_PASSWORD_ERROR", Some("email_send_failed"), None),
            Some((
                EventCode::E04,
                "password_reset_requested",
                Some("email_send_failed")
            ))
        );
        for ignored in [
            "CLIENT_LOGIN",
            "CODE_TO_TOKEN",
            "REFRESH_TOKEN",
            "VERIFY_EMAIL",
            "login",
            "",
        ] {
            assert_eq!(m(ignored, None, None), None, "{ignored}");
        }
    }

    #[test]
    fn reason_codes_come_from_allowlist_only() {
        assert_eq!(reason_code(Some("user_not_found")), "user_not_found");
        assert_eq!(reason_code(None), "idp_error_other");
        let long = "x".repeat(300);
        for leak in [
            "budi@example.invalid",
            "Account is not fully set up",
            "USER_NOT_FOUND",
            long.as_str(),
        ] {
            assert_eq!(reason_code(Some(leak)), "idp_error_other", "{leak}");
        }
        for r in [
            "invalid_user_credentials",
            "email_send_failed",
            "password_confirm_error",
            "idp_error_other",
        ] {
            assert!(audit::Code::new(r).is_ok(), "{r}");
        }
    }

    fn raw(v: serde_json::Value) -> RawEvent {
        serde_json::from_value(v).unwrap()
    }

    #[test]
    fn real_event_shape_is_parsed_without_keeping_personal_data() {
        // Bentuk respons Keycloak 26.8.2 (SRC-621), username = email.
        let ev = raw(serde_json::json!({
            "id": "67a5bdd7-ac56-4601-9235-8a0ca7a400c3", "time": 1_795_168_199_998_i64, "type": "LOGIN",
            "realmId": "5cf927d1-f2b5-4d7b-aa5e-48394f57de41", "clientId": "rs-pasien-mobile",
            "userId": "2924aef4-5f3c-4ab1-a8d1-c259bd6fdd6c", "sessionId": "YK9dKN5WZg0tGfmg2RvbYtzf",
            "ipAddress": "172.17.0.1",
            "details": {"auth_method": "openid-connect", "token_id": "onrtro:94a37201",
                        "username": "budi@example.invalid", "refresh_token_id": "ca30a31c"}
        }))
        .validate(IdpRealm::Pasien)
        .unwrap();
        assert_eq!(ev.time_ms, 1_795_168_199_998);
        assert_eq!(ev.component, SourceComponent::PatientApp);
        assert_eq!(
            ev.user_id.as_deref(),
            Some("2924aef4-5f3c-4ab1-a8d1-c259bd6fdd6c")
        );
        assert_eq!(ev.ip, Some(IpAddr::from([172, 17, 0, 1])));
        let dbg = format!("{ev:?}");
        assert!(
            !dbg.contains("2924aef4") && !dbg.contains("172.17") && !dbg.contains('@'),
            "{dbg}"
        );
        let unknown_client = raw(serde_json::json!({
            "id": "67a5bdd7-ac56-4601-9235-8a0ca7a400c3", "time": 1, "type": "LOGOUT", "clientId": "lain"
        }))
        .validate(IdpRealm::Admin)
        .unwrap();
        assert_eq!(unknown_client.component, SourceComponent::Backoffice);
        assert_eq!((unknown_client.user_id, unknown_client.ip), (None, None));
    }

    #[test]
    fn malformed_events_are_rejected_with_field_name() {
        let field = |patch: &dyn Fn(&mut serde_json::Value)| {
            let mut v = serde_json::json!({
                "id": "67a5bdd7-ac56-4601-9235-8a0ca7a400c3", "time": 1, "type": "LOGIN"
            });
            patch(&mut v);
            raw(v)
                .validate(IdpRealm::Pasien)
                .err()
                .map(|InvalidEvent(f)| f)
        };
        assert_eq!(field(&|_| {}), None);
        assert_eq!(
            field(&|v| v["id"] = serde_json::json!("bukan-uuid")),
            Some("id")
        );
        assert_eq!(field(&|v| v["time"] = serde_json::json!(-1)), Some("time"));
        assert_eq!(
            field(&|v| {
                if let Some(o) = v.as_object_mut() {
                    o.remove("type");
                }
            }),
            Some("type")
        );
        assert_eq!(
            field(&|v| v["ipAddress"] = serde_json::json!("bukan ip")),
            Some("ipAddress")
        );
        assert_eq!(
            field(&|v| v["userId"] = serde_json::json!("a b")),
            Some("userId")
        );
        assert_eq!(
            field(&|v| v["userId"] = serde_json::json!("x".repeat(65))),
            Some("userId")
        );
    }
}
