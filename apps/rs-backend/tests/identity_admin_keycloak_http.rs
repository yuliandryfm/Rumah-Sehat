//! F1-T65 (US-208): klien Admin REST Keycloak untuk pengguna realm `admin` terhadap tiruan HTTP di
//! 127.0.0.1. Bentuk permintaan dan jawaban mengikuti kode sumber `UsersResource`/`UserResource` Keycloak
//! 26.8.2 (SRC-672): `POST /admin/realms/{realm}/users` → 201 + `Location`, 409 bila username/email sudah
//! dipakai; `PUT .../execute-actions-email?lifespan=` → 204; `PUT .../users/{id}` → 204;
//! `POST .../logout` → 204; `DELETE` → 204/404. Perilaku Keycloak sungguhan (hak klien, undangan sekali
//! pakai) diuji `scripts/keycloak-check.sh`.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use axum::Router;
use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, Method, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use foundation::{FixedClock, Sensitive};
use rs_backend::modules::identity::api::{
    ACTIVATION_ACTIONS, IdpRealm, KeycloakAddress, KeycloakClient, KeycloakError, RealmCredential,
};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use time::macros::datetime;

const SECRET: &str = "rahasia-uji-admin-manager-0123456789";
const USER_ID: &str = "2924aef4-5f3c-4ab1-a8d1-c259bd6fdd6c";

#[derive(Default)]
struct Fake {
    requests: Mutex<Vec<(Method, String, Value)>>,
    conflict: Mutex<bool>,
    missing: Mutex<bool>,
}

type S = Arc<Fake>;

async fn any(
    State(f): State<S>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let path = uri.path().to_string();
    let full = uri.to_string();
    if path == "/realms/admin/protocol/openid-connect/token" {
        let form = String::from_utf8(body.to_vec()).unwrap();
        if form.contains("client_id=rs-backend-admin-manager")
            && form.contains(&format!("client_secret={SECRET}"))
        {
            return (
                StatusCode::OK,
                axum::Json(json!({"access_token": "token-akses-uji", "expires_in": 300})),
            )
                .into_response();
        }
        return StatusCode::UNAUTHORIZED.into_response();
    }
    if headers.get("authorization").and_then(|v| v.to_str().ok()) != Some("Bearer token-akses-uji")
    {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let json_body: Value = if body.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&body).unwrap()
    };
    f.requests
        .lock()
        .unwrap()
        .push((method.clone(), full.clone(), json_body));
    let user = format!("/admin/realms/admin/users/{USER_ID}");
    match (method.as_str(), path.as_str()) {
        ("POST", "/admin/realms/admin/users") => {
            if *f.conflict.lock().unwrap() {
                return StatusCode::CONFLICT.into_response();
            }
            let mut h = HeaderMap::new();
            h.insert(
                "location",
                format!("http://127.0.0.1/admin/realms/admin/users/{USER_ID}")
                    .parse()
                    .unwrap(),
            );
            (StatusCode::CREATED, h).into_response()
        }
        ("PUT", p) if p == format!("{user}/execute-actions-email") => {
            StatusCode::NO_CONTENT.into_response()
        }
        ("PUT", p) | ("DELETE", p) if p == user => {
            if *f.missing.lock().unwrap() {
                StatusCode::NOT_FOUND.into_response()
            } else {
                StatusCode::NO_CONTENT.into_response()
            }
        }
        ("POST", p) if p == format!("{user}/logout") => StatusCode::NO_CONTENT.into_response(),
        _ => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn start() -> (S, KeycloakClient) {
    let f: S = Arc::new(Fake::default());
    let app = Router::new().fallback(any).with_state(f.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    let client = KeycloakClient::new(
        format!("http://{addr}").parse::<KeycloakAddress>().unwrap(),
        Duration::from_secs(5),
        HashMap::from([(
            IdpRealm::Admin,
            RealmCredential {
                client_id: "rs-backend-admin-manager".into(),
                secret: Sensitive::new(SECRET.into()),
            },
        )]),
    )
    .unwrap();
    (f, client)
}

#[tokio::test]
async fn admin_user_lifecycle_calls_match_keycloak_admin_rest() {
    let (f, kc) = start().await;
    let clock = FixedClock::new(datetime!(2027-03-01 09:00:00 +7));
    let id = kc
        .create_user(IdpRealm::Admin, "calon@example.com", &clock)
        .await
        .unwrap();
    assert_eq!(id, USER_ID);
    kc.send_activation_email(IdpRealm::Admin, &id, 86_400, &clock)
        .await
        .unwrap();
    kc.disable_user(IdpRealm::Admin, &id, &clock).await.unwrap();
    kc.delete_user(IdpRealm::Admin, &id, &clock).await.unwrap();

    let reqs = f.requests.lock().unwrap().clone();
    let user = format!("/admin/realms/admin/users/{USER_ID}");
    assert_eq!(reqs.len(), 5);
    assert_eq!(reqs[0].0, Method::POST);
    assert_eq!(
        reqs[0].2,
        json!({"username": "calon@example.com", "email": "calon@example.com",
               "enabled": true, "emailVerified": false})
    );
    assert_eq!(
        (reqs[1].0.clone(), reqs[1].1.clone()),
        (
            Method::PUT,
            format!("{user}/execute-actions-email?lifespan=86400")
        )
    );
    assert_eq!(reqs[1].2, json!(ACTIVATION_ACTIONS));
    assert_eq!(
        ACTIVATION_ACTIONS,
        ["VERIFY_EMAIL", "UPDATE_PASSWORD", "CONFIGURE_TOTP"]
    );
    assert_eq!(
        (reqs[2].0.clone(), reqs[2].2.clone()),
        (Method::PUT, json!({"enabled": false}))
    );
    assert_eq!(
        (reqs[3].0.clone(), reqs[3].1.clone()),
        (Method::POST, format!("{user}/logout"))
    );
    assert_eq!(
        (reqs[4].0.clone(), reqs[4].1.clone()),
        (Method::DELETE, user)
    );
}

#[tokio::test]
async fn conflicts_missing_users_and_unsafe_ids_are_reported() {
    let (f, kc) = start().await;
    let clock = FixedClock::new(datetime!(2027-03-01 09:00:00 +7));
    *f.conflict.lock().unwrap() = true;
    assert_eq!(
        kc.create_user(IdpRealm::Admin, "ada@example.com", &clock)
            .await
            .unwrap_err(),
        KeycloakError::Conflict
    );
    *f.missing.lock().unwrap() = true;
    // Hapus pengguna yang sudah tidak ada = sukses (kompensasi idempoten); nonaktifkan = NotFound.
    assert!(
        kc.delete_user(IdpRealm::Admin, USER_ID, &clock)
            .await
            .is_ok()
    );
    assert_eq!(
        kc.disable_user(IdpRealm::Admin, USER_ID, &clock)
            .await
            .unwrap_err(),
        KeycloakError::NotFound
    );
    let before = f.requests.lock().unwrap().len();
    for bad in ["../users", "a/b", "a?x=1", ""] {
        assert_eq!(
            kc.send_activation_email(IdpRealm::Admin, bad, 86_400, &clock)
                .await
                .unwrap_err(),
            KeycloakError::NotFound,
            "{bad:?}"
        );
    }
    assert_eq!(
        f.requests.lock().unwrap().len(),
        before,
        "ID tidak aman tidak dikirim"
    );
    // Realm tanpa kredensial: token tidak dapat diminta.
    assert!(matches!(
        kc.create_user(IdpRealm::Pasien, "x@example.com", &clock)
            .await,
        Err(KeycloakError::Token(_))
    ));
}
