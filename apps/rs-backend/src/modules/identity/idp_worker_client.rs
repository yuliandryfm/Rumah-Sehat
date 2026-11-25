//! Klien Admin REST pengguna Keycloak untuk worker IdP (ADR-030 butir 1-3, 6). Satu-satunya pemegang kredensial
//! klien `manage-users` realm yang disediakan worker; hanya boleh dirakit di subcommand `worker` dan modul
//! worker IdP (arch-check `idp-admin-client`).
//!
//! Permukaan sengaja sempit: tipe ini hanya menyediakan operasi daftar tertutup ADR-030 butir 3 (buat pengguna
//! tanpa kredensial dengan atribut `rs_intent_id`, cari menurut atribut, baca keadaan pengguna, kirim email aksi,
//! nonaktifkan, akhiri sesi, hapus). Reset password, hapus kredensial, ubah email/username/atribut, pemetaan
//! peran, grup, dan impersonation tidak punya metode.
//!
//! Endpoint (kode sumber `UserResource` Keycloak 26.8.2, SRC-672, dan `UsersResource.getUsers`, SRC-852; pencarian
//! atribut `q=<nama>:<nilai>` dan `briefRepresentation=false` mengembalikan `attributes`, diamati
//! `scripts/keycloak-check.sh` langkah pekerja IdP):
//! - `POST /admin/realms/{realm}/users` → 201 + `Location`; 409 bila username/email sudah ada.
//! - `GET /admin/realms/{realm}/users?q=rs_intent_id:<id>&briefRepresentation=false&max=2`.
//! - `GET /admin/realms/{realm}/users/{id}` dan `GET .../users/{id}/credentials` (hanya `type` dibaca).
//! - `PUT .../users/{id}/execute-actions-email?lifespan=<detik>` (204).
//! - `PUT .../users/{id}` `{"enabled":false}` (204), `POST .../users/{id}/logout` (204), `DELETE .../users/{id}`.
//!
//! Isi jawaban yang memuat data pribadi (email, username, nama) tidak pernah diuraikan ke tipe Rust: hanya `id`,
//! `enabled`, atribut `rs_intent_id`, dan jenis kredensial.

use super::domain::IdpRealm;
use super::idp_domain::{ActivationPlan, INTENT_ATTRIBUTE, KcUserFacts};
use super::keycloak::{
    InvalidKeycloakAddress, KeycloakAddress, KeycloakClient, KeycloakError, RealmCredential,
    valid_user_id,
};
use foundation::{Clock, Uuid};
use http::{Method, StatusCode};
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::time::Duration;

/// Hasil `POST users`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreateOutcome {
    /// Dibuat; ID pengguna dari `Location`.
    Created(String),
    /// Username/email sudah dipakai (409).
    Conflict,
}

/// Klien pengguna Keycloak worker IdP.
pub struct IdpUserAdmin {
    client: KeycloakClient,
    realms: Vec<IdpRealm>,
}

impl fmt::Debug for IdpUserAdmin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IdpUserAdmin")
            .field("realms", &self.realms)
            .finish_non_exhaustive()
    }
}

#[derive(Deserialize)]
struct KcUser {
    id: String,
    #[serde(default)]
    enabled: bool,
    #[serde(default)]
    attributes: BTreeMap<String, Vec<String>>,
}

#[derive(Deserialize)]
struct KcCredential {
    #[serde(rename = "type")]
    kind: String,
}

fn status(reply_status: StatusCode, ok: StatusCode) -> Result<(), KeycloakError> {
    match reply_status {
        s if s == ok => Ok(()),
        StatusCode::CONFLICT => Err(KeycloakError::Conflict),
        StatusCode::NOT_FOUND => Err(KeycloakError::NotFound),
        s => Err(KeycloakError::Status(s.as_u16())),
    }
}

impl IdpUserAdmin {
    /// Klien untuk realm yang punya kredensial worker. Realm tanpa kredensial tidak pernah dipanggil.
    pub fn new(
        addr: KeycloakAddress,
        timeout: Duration,
        credentials: HashMap<IdpRealm, RealmCredential>,
    ) -> Result<Self, InvalidKeycloakAddress> {
        let mut realms: Vec<IdpRealm> = credentials.keys().copied().collect();
        realms.sort_by_key(|r| r.as_str());
        Ok(Self {
            client: KeycloakClient::new(addr, timeout, credentials)?,
            realms,
        })
    }

    /// Realm yang dilayani.
    pub fn serves(&self, realm: IdpRealm) -> bool {
        self.realms.contains(&realm)
    }

    fn users_path(realm: IdpRealm) -> String {
        format!("/admin/realms/{}/users", realm.as_str())
    }

    fn user_path(realm: IdpRealm, user_id: &str) -> Result<String, KeycloakError> {
        if !valid_user_id(user_id) {
            return Err(KeycloakError::NotFound);
        }
        Ok(format!("{}/{user_id}", Self::users_path(realm)))
    }

    async fn call(
        &self,
        realm: IdpRealm,
        method: Method,
        path: &str,
        json: Option<Vec<u8>>,
        clock: &dyn Clock,
    ) -> Result<super::keycloak::Reply, KeycloakError> {
        if !self.serves(realm) {
            return Err(KeycloakError::Token(0));
        }
        self.client
            .admin_call(realm, method, path, json, clock)
            .await
    }

    /// `POST users`: pengguna aktif tanpa kredensial, email sebagai username, `emailVerified=false`, atribut
    /// `rs_intent_id` = `intent`.
    pub async fn create_user(
        &self,
        realm: IdpRealm,
        email: &str,
        intent: Uuid,
        clock: &dyn Clock,
    ) -> Result<CreateOutcome, KeycloakError> {
        let body = serde_json::to_vec(&serde_json::json!({
            "username": email,
            "email": email,
            "enabled": true,
            "emailVerified": false,
            "attributes": { INTENT_ATTRIBUTE: [intent.hyphenated().to_string()] },
        }))
        .map_err(|_| KeycloakError::Json)?;
        let reply = self
            .call(
                realm,
                Method::POST,
                &Self::users_path(realm),
                Some(body),
                clock,
            )
            .await?;
        match status(reply.status, StatusCode::CREATED) {
            Ok(()) => {}
            Err(KeycloakError::Conflict) => return Ok(CreateOutcome::Conflict),
            Err(e) => return Err(e),
        }
        let id = reply
            .location
            .as_deref()
            .and_then(|l| l.rsplit('/').next())
            .filter(|id| valid_user_id(id))
            .ok_or(KeycloakError::Json)?;
        Ok(CreateOutcome::Created(id.to_string()))
    }

    /// Pengguna yang atribut `rs_intent_id`-nya tepat satu nilai yang sama persis dengan `intent` (hanya baca,
    /// ADR-030 butir 6). Lebih dari satu pengguna cocok = keadaan tidak sah ([`KeycloakError::Conflict`]).
    pub async fn find_by_intent(
        &self,
        realm: IdpRealm,
        intent: Uuid,
        clock: &dyn Clock,
    ) -> Result<Option<String>, KeycloakError> {
        let want = intent.hyphenated().to_string();
        let path = format!(
            "{}?q={INTENT_ATTRIBUTE}:{want}&briefRepresentation=false&first=0&max=2",
            Self::users_path(realm)
        );
        let reply = self.call(realm, Method::GET, &path, None, clock).await?;
        status(reply.status, StatusCode::OK)?;
        let users: Vec<KcUser> =
            serde_json::from_slice(&reply.body).map_err(|_| KeycloakError::Json)?;
        let matching: Vec<KcUser> = users
            .into_iter()
            .filter(|u| {
                matches!(u.attributes.get(INTENT_ATTRIBUTE).map(Vec::as_slice), Some([v]) if *v == want)
            })
            .collect();
        match matching.as_slice() {
            [] => Ok(None),
            [u] if valid_user_id(&u.id) => Ok(Some(u.id.clone())),
            [_] => Err(KeycloakError::Json),
            _ => Err(KeycloakError::Conflict),
        }
    }

    /// Keadaan pengguna (`None` bila tidak ada).
    pub async fn user_facts(
        &self,
        realm: IdpRealm,
        user_id: &str,
        clock: &dyn Clock,
    ) -> Result<Option<KcUserFacts>, KeycloakError> {
        let path = Self::user_path(realm, user_id)?;
        let reply = self.call(realm, Method::GET, &path, None, clock).await?;
        match status(reply.status, StatusCode::OK) {
            Ok(()) => {}
            Err(KeycloakError::NotFound) => return Ok(None),
            Err(e) => return Err(e),
        }
        let user: KcUser = serde_json::from_slice(&reply.body).map_err(|_| KeycloakError::Json)?;
        let reply = self
            .call(
                realm,
                Method::GET,
                &format!("{path}/credentials"),
                None,
                clock,
            )
            .await?;
        match status(reply.status, StatusCode::OK) {
            Ok(()) => {}
            Err(KeycloakError::NotFound) => return Ok(None),
            Err(e) => return Err(e),
        }
        let creds: Vec<KcCredential> =
            serde_json::from_slice(&reply.body).map_err(|_| KeycloakError::Json)?;
        Ok(Some(KcUserFacts {
            intent_attribute: user
                .attributes
                .get(INTENT_ATTRIBUTE)
                .cloned()
                .unwrap_or_default(),
            credential_types: creds.into_iter().map(|c| c.kind).collect(),
            enabled: user.enabled,
        }))
    }

    /// `execute-actions-email` dengan susunan aksi dan masa berlaku dari [`ActivationPlan`] (ditetapkan kode,
    /// bukan muatan job). Tanpa `client_id`/`redirect_uri`.
    pub async fn send_actions_email(
        &self,
        realm: IdpRealm,
        user_id: &str,
        plan: ActivationPlan,
        clock: &dyn Clock,
    ) -> Result<(), KeycloakError> {
        let path = format!(
            "{}/execute-actions-email?lifespan={}",
            Self::user_path(realm, user_id)?,
            plan.lifespan_seconds
        );
        let body = serde_json::to_vec(plan.actions).map_err(|_| KeycloakError::Json)?;
        let reply = self
            .call(realm, Method::PUT, &path, Some(body), clock)
            .await?;
        status(reply.status, StatusCode::NO_CONTENT)
    }

    /// `PUT users/{id}` `enabled=false`.
    pub async fn disable(
        &self,
        realm: IdpRealm,
        user_id: &str,
        clock: &dyn Clock,
    ) -> Result<(), KeycloakError> {
        let path = Self::user_path(realm, user_id)?;
        let body = serde_json::to_vec(&serde_json::json!({ "enabled": false }))
            .map_err(|_| KeycloakError::Json)?;
        let reply = self
            .call(realm, Method::PUT, &path, Some(body), clock)
            .await?;
        status(reply.status, StatusCode::NO_CONTENT)
    }

    /// `POST users/{id}/logout`.
    pub async fn logout(
        &self,
        realm: IdpRealm,
        user_id: &str,
        clock: &dyn Clock,
    ) -> Result<(), KeycloakError> {
        let path = format!("{}/logout", Self::user_path(realm, user_id)?);
        let reply = self.call(realm, Method::POST, &path, None, clock).await?;
        status(reply.status, StatusCode::NO_CONTENT)
    }

    /// `DELETE users/{id}`; sudah tidak ada = sukses.
    pub async fn delete(
        &self,
        realm: IdpRealm,
        user_id: &str,
        clock: &dyn Clock,
    ) -> Result<(), KeycloakError> {
        let path = Self::user_path(realm, user_id)?;
        let reply = self.call(realm, Method::DELETE, &path, None, clock).await?;
        match status(reply.status, StatusCode::NO_CONTENT) {
            Err(KeycloakError::NotFound) => Ok(()),
            other => other,
        }
    }
}
