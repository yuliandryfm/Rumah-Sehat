//! ADR-030 langkah 3 (T1, CR-026, KEPUTUSAN-305): rahasia klien Keycloak `manage-users` worker IdP hanya dapat
//! dibaca identitas OpenBao `worker`. Policy sungguhan dari `infra/openbao/policies/` dimuat ke OpenBao mode dev
//! efemeral (Docker, image dari `infra/local/compose.yaml`); token per identitas dibuat dari policy itu. Tanpa
//! Docker test GAGAL (tidak dilewati).
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

#[path = "../../../crates/crypto/tests/common/mod.rs"]
mod bao;

use observability::EnvSource;
use rs_backend::modules::identity::api::{
    IdpRealm, IdpSecretError, IdpWorkerConfig, load_idp_credentials,
};
use std::path::Path;

const KV_PATH: &str = "rs/keycloak/korporat/idp-worker";
const SECRET: &str = "rahasia-sintetis-klien-idp-worker-0123456789";

fn policy(name: &str) -> String {
    let p = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../infra/openbao/policies")
        .join(format!("{name}.hcl"));
    std::fs::read_to_string(p).expect("berkas policy terbaca")
}

fn token_for(b: &bao::OpenBao, policy_name: &str) -> String {
    let out = b.bao(&[
        "token",
        "create",
        "-policy",
        policy_name,
        "-no-default-policy",
        "-field",
        "token",
    ]);
    assert!(out.status.success(), "token create gagal");
    String::from_utf8(out.stdout).unwrap().trim().to_string()
}

fn worker_config(b: &bao::OpenBao, token: &str) -> IdpWorkerConfig {
    IdpWorkerConfig::load(&EnvSource::from_pairs([
        ("IDP_WORKER_REALMS", "korporat"),
        ("KEYCLOAK_ADMIN_URL", "http://127.0.0.1:18080"),
        ("OPENBAO_ADDR", b.address.as_str()),
        ("OPENBAO_TOKEN", token),
        ("IDP_WORKER_KV_PATH_KORPORAT", KV_PATH),
    ]))
    .unwrap()
}

// trace: ADR-030 langkah 3, "policy OpenBao api ditolak membaca path rahasia"
#[tokio::test]
async fn only_the_worker_identity_can_read_the_manage_users_secret() {
    let b = bao::OpenBao::start();
    b.bao_stdin(&["policy", "write", "rs-api", "-"], &policy("rs-api"));
    b.bao_stdin(&["policy", "write", "rs-worker", "-"], &policy("rs-worker"));
    b.bao_ok(&[
        "kv",
        "put",
        "-mount=secret",
        KV_PATH,
        &format!("client_secret={SECRET}"),
    ]);
    b.bao_ok(&[
        "kv",
        "put",
        "-mount=secret",
        "ruang-sehat/blind-index/nik",
        "v1=sintetis",
    ]);

    // Identitas api: policy tidak memuat path rahasia worker → ditolak (gagal-tertutup).
    let api_token = token_for(&b, "rs-api");
    let denied = load_idp_credentials(&worker_config(&b, &api_token)).await;
    assert_eq!(denied.err(), Some(IdpSecretError::OpenBao));
    // Pembacaan langsung lewat klien KV juga ditolak (bukan sekadar field kosong).
    assert!(
        crypto::read_kv_secret(&b.config_with_token(&api_token), "secret", KV_PATH)
            .await
            .is_err()
    );
    // Policy api tetap memberi kebutuhannya sendiri (pembanding: penolakan di atas karena path, bukan token rusak).
    assert!(
        crypto::read_kv_secret(
            &b.config_with_token(&api_token),
            "secret",
            "ruang-sehat/blind-index/nik"
        )
        .await
        .is_ok()
    );

    // Identitas worker: hanya path rahasianya sendiri.
    let worker_token = token_for(&b, "rs-worker");
    let creds = load_idp_credentials(&worker_config(&b, &worker_token))
        .await
        .unwrap();
    let c = creds.get(&IdpRealm::Korporat).unwrap();
    assert_eq!(c.client_id, "rs-backend-idp-worker");
    assert_eq!(c.secret.expose(), SECRET);
    assert!(!format!("{c:?}").contains(SECRET));
    assert!(
        crypto::read_kv_secret(
            &b.config_with_token(&worker_token),
            "secret",
            "ruang-sehat/blind-index/nik"
        )
        .await
        .is_err()
    );

    // Entri tanpa field rahasia = gagal start, bukan rahasia kosong.
    b.bao_ok(&["kv", "put", "-mount=secret", KV_PATH, "lain=x"]);
    assert_eq!(
        load_idp_credentials(&worker_config(&b, &worker_token))
            .await
            .err(),
        Some(IdpSecretError::MissingField)
    );
}
