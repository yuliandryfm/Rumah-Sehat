//! Test integrasi KEK OpenBao Transit dan wadah DEK terhadap OpenBao 2.7.2 mode dev sungguhan
//! (F1-T27; skema fisik §7.3; ADR-013 §3, §5). Butuh Docker; tanpa Docker test ini GAGAL.
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::OpenBao;
use crypto::{
    Container, ContainerType, CryptoError, DataKeyRecord, EncryptionContext, KeyEncryptionKey,
    Keyring, KeyringConfig, NewDataKey, System, TransitConfig, TransitKek,
};
use foundation::{FixedClock, Sensitive, new_id};
use std::sync::Arc;
use time::macros::datetime;

const KEY: &str = "rs-dek-kek";
const NIK: &[u8] = b"3200000000000001"; // NIK sintetis

fn setup() -> OpenBao {
    let bao = OpenBao::start();
    bao.bao_ok(&["secrets", "enable", "transit"]);
    bao.bao_ok(&[
        "write",
        "-f",
        &format!("transit/keys/{KEY}"),
        "type=aes256-gcm96",
    ]);
    bao
}

fn transit(bao: &OpenBao, key: &str) -> TransitKek {
    TransitKek::new(
        &bao.config(),
        TransitConfig {
            mount: "transit".into(),
            key_name: key.into(),
        },
    )
    .unwrap()
}

fn keyring(bao: &OpenBao) -> Keyring<TransitKek> {
    Keyring::new(
        transit(bao, KEY),
        KeyringConfig {
            system: System::RuangSehat,
            cache_ttl: time::Duration::minutes(5),
            cache_capacity: 100,
        },
        Arc::new(FixedClock::new(datetime!(2027-03-01 09:00:00 +7))),
    )
    .unwrap()
}

fn record(row: &NewDataKey) -> DataKeyRecord {
    DataKeyRecord {
        id: row.id,
        container_type: row.container_type.as_str().to_string(),
        container_id: row.container_id,
        wrapped_dek: Some(Sensitive::new(row.wrapped_dek.expose().clone())),
        kek_ref: row.kek_ref.clone(),
        shredded_at: None,
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn transit_wraps_and_unwraps_dek_and_rejects_tampering() {
    let bao = setup();
    let kek = transit(&bao, KEY);

    let g1 = kek.generate_dek().await.unwrap();
    let g2 = kek.generate_dek().await.unwrap();
    assert_eq!(g1.plaintext.expose().len(), 32);
    assert_ne!(g1.plaintext.expose(), g2.plaintext.expose());
    assert!(
        g1.wrapped.expose().starts_with("vault:v1:"),
        "format ciphertext Transit"
    );
    assert_eq!(g1.kek_ref, "openbao-transit:transit:rs-dek-kek:v1");

    let back = kek
        .unwrap_dek(g1.wrapped.expose(), &g1.kek_ref)
        .await
        .unwrap();
    assert_eq!(back.expose(), g1.plaintext.expose());

    // Ciphertext KEK diubah satu karakter: OpenBao menolak (autentikasi GCM gagal).
    let w = g1.wrapped.expose().clone();
    let last = w.len() - 3;
    let flipped = if &w[last..last + 1] == "A" { "B" } else { "A" };
    let tampered = format!("{}{}{}", &w[..last], flipped, &w[last + 1..]);
    match kek.unwrap_dek(&tampered, &g1.kek_ref).await {
        Err(CryptoError::KmsRejected { status }) => {
            assert!((400..500).contains(&status), "{status}")
        }
        other => panic!(
            "ciphertext diubah harus ditolak, didapat {:?}",
            other.map(|_| ())
        ),
    }

    // DEK terbungkus kunci lain tidak dibuka KEK ini; rujukan KEK lain ditolak sebelum memanggil OpenBao.
    assert_eq!(
        kek.unwrap_dek(&w, "openbao-transit:transit:rme-dek-kek:v1")
            .await
            .err(),
        Some(CryptoError::KekMismatch)
    );

    // Token salah: ditolak OpenBao (403).
    let wrong = TransitKek::new(
        &bao.config_with_token("uji-token-salah"),
        TransitConfig {
            mount: "transit".into(),
            key_name: KEY.into(),
        },
    )
    .unwrap();
    assert_eq!(
        wrong.generate_dek().await.err(),
        Some(CryptoError::KmsRejected { status: 403 })
    );
    assert_eq!(
        wrong.unwrap_dek(&w, &g1.kek_ref).await.err(),
        Some(CryptoError::KmsRejected { status: 403 })
    );

    // Kunci Transit yang tidak ada.
    match transit(&bao, "tidak-ada").generate_dek().await {
        Err(CryptoError::KmsRejected { status }) => {
            assert!((400..500).contains(&status), "{status}")
        }
        other => panic!(
            "kunci tidak ada harus ditolak, didapat {:?}",
            other.map(|_| ())
        ),
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn rotated_kek_still_unwraps_old_dek() {
    // Rotasi KEK cara utama: versi lama tetap dapat dibuka (SRC-222, skema fisik §7.3).
    let bao = setup();
    let kek = transit(&bao, KEY);
    let old = kek.generate_dek().await.unwrap();
    bao.bao_ok(&["write", "-f", &format!("transit/keys/{KEY}/rotate")]);
    let new = kek.generate_dek().await.unwrap();
    assert_eq!(new.kek_ref, "openbao-transit:transit:rs-dek-kek:v2");
    assert!(new.wrapped.expose().starts_with("vault:v2:"));
    let back = kek
        .unwrap_dek(old.wrapped.expose(), &old.kek_ref)
        .await
        .unwrap();
    assert_eq!(back.expose(), old.plaintext.expose());
    // kek_ref yang tidak cocok dengan versi di ciphertext ditolak.
    assert_eq!(
        kek.unwrap_dek(old.wrapped.expose(), &new.kek_ref)
            .await
            .err(),
        Some(CryptoError::KekMismatch)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn data_key_round_trip_and_shredded_or_moved_keys_fail() {
    let bao = setup();
    let writer = keyring(&bao);
    let doc_id = new_id();
    let container = Container::new(ContainerType::IdentityDocument, doc_id);
    let (row, key) = writer.create_data_key(container).await.unwrap();
    assert!(
        row.kek_ref
            .starts_with("openbao-transit:transit:rs-dek-kek:v")
    );
    let ct = key
        .seal(EncryptionContext::IDENTITY_DOCUMENT_NIK, doc_id, NIK)
        .unwrap();
    assert_eq!(ct.len(), NIK.len() + 29);

    // Proses lain (cache kosong) membuka DEK lewat OpenBao.
    let reader = keyring(&bao);
    let opened = reader
        .open_data_key(&record(&row), container)
        .await
        .unwrap();
    let pt = opened
        .open(EncryptionContext::IDENTITY_DOCUMENT_NIK, doc_id, &ct)
        .unwrap();
    assert_eq!(pt.expose().as_slice(), NIK);

    // DEK dimusnahkan (wrapped_dek NULL + shredded_at): tidak dapat dipakai, juga oleh proses yang
    // masih menyimpan DEK di cache.
    let mut shredded = record(&row);
    shredded.wrapped_dek = None;
    shredded.shredded_at = Some(datetime!(2027-03-02 00:00:00 UTC));
    assert_eq!(
        reader.open_data_key(&shredded, container).await.err(),
        Some(CryptoError::KeyShredded)
    );
    assert_eq!(
        writer.open_data_key(&shredded, container).await.err(),
        Some(CryptoError::KeyShredded)
    );
    assert_eq!(
        keyring(&bao)
            .open_data_key(&shredded, container)
            .await
            .err(),
        Some(CryptoError::KeyShredded)
    );

    // container_type salah untuk container_id yang sama ditolak.
    let wrong_type = Container::new(ContainerType::StatusNote, doc_id);
    assert_eq!(
        keyring(&bao)
            .open_data_key(&record(&row), wrong_type)
            .await
            .err(),
        Some(CryptoError::ContainerMismatch)
    );

    // wrapped_dek baris lain ditukar ke baris ini: DEK yang terbuka bukan DEK baris ini, sehingga
    // ciphertext tidak terbuka (AAD memuat data_key_id, bahan kunci pun berbeda).
    let other = Container::new(ContainerType::IdentityDocument, new_id());
    let (other_row, _) = writer.create_data_key(other).await.unwrap();
    let mut swapped = record(&row);
    swapped.wrapped_dek = Some(Sensitive::new(other_row.wrapped_dek.expose().clone()));
    swapped.kek_ref = other_row.kek_ref.clone();
    let wrong_key = keyring(&bao)
        .open_data_key(&swapped, container)
        .await
        .unwrap();
    assert_eq!(
        wrong_key
            .open(EncryptionContext::IDENTITY_DOCUMENT_NIK, doc_id, &ct)
            .err(),
        Some(CryptoError::DecryptionFailed)
    );
}
