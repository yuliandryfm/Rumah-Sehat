//! Test integrasi pemuatan kunci blind index NIK dari OpenBao KV v2 (F1-T28; skema fisik §7.2;
//! ADR-013 §4) terhadap OpenBao 2.7.2 mode dev sungguhan. Butuh Docker; tanpa Docker test ini GAGAL.
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use base64::Engine;
use common::OpenBao;
use crypto::{CryptoError, HmacSha256Key, Nik, load_nik_blind_index_keyring};

const PATH: &str = "ruang-sehat/blind-index/nik";
const NIK: &str = "3200000000000001"; // NIK sintetis

fn b64(bytes: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

fn random_key() -> [u8; 32] {
    let mut k = [0u8; 32];
    crypto::fill_random(&mut k).unwrap();
    k
}

fn expected(key: &[u8; 32], nik: &str) -> [u8; 32] {
    let mut msg = b"rs.nik.v1\0".to_vec();
    msg.extend_from_slice(nik.as_bytes());
    HmacSha256Key::new(key).unwrap().sign(&msg)
}

#[tokio::test(flavor = "multi_thread")]
async fn nik_keys_are_loaded_from_kv_and_all_versions_are_usable() {
    let bao = OpenBao::start();
    let k1 = random_key();
    let k2 = random_key();
    bao.bao_ok(&[
        "kv",
        "put",
        &format!("secret/{PATH}"),
        &format!("1={}", b64(&k1)),
        &format!("2={}", b64(&k2)),
    ]);

    let ring = load_nik_blind_index_keyring(&bao.config(), "secret", PATH)
        .await
        .unwrap();
    assert_eq!(ring.current_version(), 2);
    assert_eq!(ring.versions(), vec![2, 1]);
    let nik = Nik::parse(NIK).unwrap();
    let current = ring.compute(&nik).unwrap();
    assert_eq!(current.key_version, 2);
    assert_eq!(current.value, expected(&k2, NIK));
    let candidates = ring.lookup_candidates(&nik);
    assert_eq!(candidates.len(), 2);
    assert_eq!(candidates[1].key_version, 1);
    assert_eq!(candidates[1].value, expected(&k1, NIK));

    // Path tidak ada, token salah, mount/path tidak sah.
    assert_eq!(
        load_nik_blind_index_keyring(&bao.config(), "secret", "tidak/ada")
            .await
            .err(),
        Some(CryptoError::KmsRejected { status: 404 })
    );
    assert_eq!(
        load_nik_blind_index_keyring(&bao.config_with_token("uji-token-salah"), "secret", PATH)
            .await
            .err(),
        Some(CryptoError::KmsRejected { status: 403 })
    );
    assert_eq!(
        load_nik_blind_index_keyring(&bao.config(), "secret", "../sys")
            .await
            .err(),
        Some(CryptoError::InvalidConfig)
    );

    // Isi rahasia yang tidak sesuai format (nama versi bukan angka, kunci 16 byte) ditolak seluruhnya.
    bao.bao_ok(&["kv", "put", "secret/rusak", &format!("v1={}", b64(&k1))]);
    assert_eq!(
        load_nik_blind_index_keyring(&bao.config(), "secret", "rusak")
            .await
            .err(),
        Some(CryptoError::InvalidConfig)
    );
    bao.bao_ok(&[
        "kv",
        "put",
        "secret/pendek",
        &format!("1={}", b64(&k1[..16])),
    ]);
    assert_eq!(
        load_nik_blind_index_keyring(&bao.config(), "secret", "pendek")
            .await
            .err(),
        Some(CryptoError::InvalidConfig)
    );
}
