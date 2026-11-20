//! Kunci akses SES dari OpenBao KV v2 (F1-T36, ADR-026 butir 5, KEPUTUSAN-79) terhadap OpenBao 2.7.2 mode dev
//! sekali pakai (image dari infra/local/compose.yaml). Kredensial adalah contoh fiktif AWS; tidak ada rahasia
//! di repo. Docker wajib ada: bila tidak, test GAGAL (tidak dilewati).
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

// Perlengkapan OpenBao efemeral yang sama dengan test crate crypto (satu sumber, tanpa salinan).
#[path = "../../crypto/tests/common/mod.rs"]
mod openbao;

use ses::{CredentialsError, FIELD_ACCESS_KEY_ID, SesError, load_from_openbao};

const ID: &str = "AKIAIOSFODNN7EXAMPLE";
const SECRET: &str = "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY";

#[tokio::test]
async fn credentials_are_loaded_from_kv_and_validated() {
    let bao = openbao::OpenBao::start();
    bao.bao_ok(&[
        "kv",
        "put",
        "secret/ruang-sehat/ses/email-relay",
        &format!("access_key_id={ID}"),
        &format!("secret_access_key={SECRET}"),
    ]);
    let c = load_from_openbao(&bao.config(), "secret", "ruang-sehat/ses/email-relay")
        .await
        .unwrap();
    assert_eq!(c.access_key_id, ID);
    assert_eq!(c.secret_access_key.expose(), SECRET);
    assert!(!format!("{c:?}").contains("EXAMPLE"));

    // Field hilang, bentuk salah, path tanpa izin/tidak ada, token salah: gagal tanpa nilai di galat.
    bao.bao_ok(&[
        "kv",
        "put",
        "secret/ses/tanpa-id",
        &format!("secret_access_key={SECRET}"),
    ]);
    assert_eq!(
        load_from_openbao(&bao.config(), "secret", "ses/tanpa-id")
            .await
            .err(),
        Some(CredentialsError::MissingField(FIELD_ACCESS_KEY_ID))
    );
    bao.bao_ok(&[
        "kv",
        "put",
        "secret/ses/pendek",
        &format!("access_key_id={ID}"),
        "secret_access_key=pendek",
    ]);
    let e = load_from_openbao(&bao.config(), "secret", "ses/pendek")
        .await
        .err()
        .unwrap();
    assert_eq!(
        e,
        CredentialsError::Invalid(SesError::InvalidConfig("secret_access_key"))
    );
    assert!(!format!("{e} {e:?}").contains("pendek"));
    assert!(matches!(
        load_from_openbao(&bao.config(), "secret", "ses/tidak-ada").await,
        Err(CredentialsError::OpenBao(_))
    ));
    assert!(matches!(
        load_from_openbao(
            &bao.config_with_token("token-salah"),
            "secret",
            "ruang-sehat/ses/email-relay"
        )
        .await,
        Err(CredentialsError::OpenBao(_))
    ));
}
