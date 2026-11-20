//! F1-T60 (reference implementation US-106): service + repository pengajuan identitas terhadap PostgreSQL
//! 18 (role runtime `rs_app`), SeaweedFS, Valkey, dan database audit (relay + verifier).
//!
//! Skenario positif dan negatif modul T1 (coding-standards §7.3): NIK terenkripsi dengan AAD dan blind
//! index semua versi kunci, foto lewat slot unggah, UP-40 lintas akun (termasuk registri versi kunci lama),
//! foto terkunci saat menunggu, pembatalan mengembalikan status sebelumnya, batas NIK berlapis (akun, NIK
//! lintas akun, IP) dengan pesan umum, objek milik orang lain 404, gagal-tertutup tanpa IP klien dan saat
//! Valkey mati, serta event E-10 tanpa NIK lengkap.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)] // berkas test

mod files;
mod identity_support;
mod valkey_support;

use crypto::{Container, ContainerType, EncryptionContext, Nik, load_data_key};
use foundation::Uuid;
use identity_support::{
    CLIENT_IP, Env, PATIENT_ROUTE, Principals, audit_events, new_family_member, new_patient,
    request_count, slot_consumed_or_gone, synthetic_nik,
};
use rs_backend::modules::platform::api::{FileError, UploadPurpose, validate_upload};
use rs_backend::modules::profile::api::{
    IdentityServices, ProfileError, cancel_identity_submission, identity_status,
    submit_identity_document,
};
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;

static SERIAL: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

struct Ctx {
    env: Env,
    principals: Principals,
    services: IdentityServices,
}

async fn ctx() -> Ctx {
    let env = Env::start().await;
    let principals = Principals::start().await;
    let services = env.services();
    Ctx {
        env,
        principals,
        services,
    }
}

/// Persetujuan tiruan langsung di database lewat jalur tulis yang sama dengan Verifikator (event v2 +
/// proyeksi + registri NIK pemilik akun dengan blind index versi `bidx_version`); keputusan Verifikator
/// sungguhan diuji di F1-T62.
async fn approve_in_db(
    env: &Env,
    care_recipient_id: Uuid,
    request_id: Uuid,
    nik: &str,
    v1_key: bool,
) {
    let nik = Nik::parse(nik).unwrap();
    let b = if v1_key {
        env.blind_index_v1.compute(&nik).unwrap()
    } else {
        env.blind_index.compute(&nik).unwrap()
    };
    let mut tx = env.pool.begin().await.unwrap();
    sqlx::query!(
        "INSERT INTO app.identity_verification_request_status_event \
             (id, identity_verification_request_id, version, from_status, to_status, actor_type, correlation_id) \
         VALUES ($1, $2, 2, 'pending', 'approved', 'rs_admin', $3)",
        foundation::new_id(),
        request_id,
        foundation::new_id(),
    )
    .execute(&mut *tx)
    .await
    .unwrap();
    sqlx::query!(
        "UPDATE app.identity_verification_request_state SET status = 'approved', version = 2 \
         WHERE identity_verification_request_id = $1 AND version = 1",
        request_id
    )
    .execute(&mut *tx)
    .await
    .unwrap();
    sqlx::query!(
        "INSERT INTO app.account_owner_nik_registry \
             (id, nik_bidx, nik_bidx_key_version, care_recipient_id, identity_verification_request_id) \
         VALUES ($1, $2, $3, $4, $5)",
        foundation::new_id(),
        &b.value[..],
        b.key_version,
        care_recipient_id,
        request_id,
    )
    .execute(&mut *tx)
    .await
    .unwrap();
    tx.commit().await.unwrap();
}

struct DocRow {
    nik_enc: Vec<u8>,
    nik_bidx: Vec<u8>,
    nik_bidx_key_version: i16,
    nik_last4: String,
    data_key_id: Uuid,
}

async fn document(pool: &PgPool, id: Uuid) -> DocRow {
    sqlx::query_as!(
        DocRow,
        "SELECT nik_enc, nik_bidx, nik_bidx_key_version, nik_last4, data_key_id \
         FROM app.identity_document WHERE id = $1",
        id
    )
    .fetch_one(pool)
    .await
    .unwrap()
}

// trace: US-106 AC1, AC2 (NIK terenkripsi + blind index, status Menunggu verifikasi, E-10)
#[tokio::test]
async fn submission_encrypts_nik_and_moves_status_to_pending() {
    let _g = SERIAL.lock().await;
    let c = ctx().await;
    let p = new_patient(&c.env.pool).await;
    let who = c.principals.patient(&p, true).await;
    let before = identity_status(&c.services, &who, PATIENT_ROUTE, p.owner_care_recipient_id)
        .await
        .unwrap();
    assert_eq!(before.status, "not_uploaded");
    assert!(before.can_submit && !before.can_cancel);

    let nik = synthetic_nik();
    let slot = c.env.uploaded_slot(p.account_id).await;
    let accepted = submit_identity_document(
        &c.services,
        &who,
        PATIENT_ROUTE,
        p.owner_care_recipient_id,
        &nik,
        slot,
    )
    .await
    .unwrap();
    assert_eq!(accepted.status, "pending");

    // NIK tidak tersimpan teks terbuka; terbuka kembali hanya dengan AAD baris dan wadah dokumen ini.
    let doc = document(&c.env.pool, accepted.identity_document_id).await;
    assert!(!files::contains(&doc.nik_enc, nik.as_bytes()));
    assert_eq!(doc.nik_last4, &nik[12..]);
    assert_eq!(
        doc.nik_bidx_key_version, 2,
        "blind index memakai versi kunci terbaru"
    );
    let parsed = Nik::parse(&nik).unwrap();
    assert_eq!(
        doc.nik_bidx,
        c.env.blind_index.compute(&parsed).unwrap().value.to_vec()
    );
    let mut conn = c.env.pool.acquire().await.unwrap();
    let record = load_data_key(&mut conn, doc.data_key_id)
        .await
        .unwrap()
        .unwrap();
    let container = Container::new(
        ContainerType::IdentityDocument,
        accepted.identity_document_id,
    );
    let dk = c
        .env
        .keyring
        .open_data_key(&record, container)
        .await
        .unwrap();
    let plain = dk
        .open(
            EncryptionContext::IDENTITY_DOCUMENT_NIK,
            accepted.identity_document_id,
            &doc.nik_enc,
        )
        .unwrap();
    assert_eq!(plain.expose().as_slice(), nik.as_bytes());
    // Ciphertext yang dipindah ke baris lain tidak dapat dibuka (AAD, KEPUTUSAN-130).
    assert!(
        dk.open(
            EncryptionContext::IDENTITY_DOCUMENT_NIK,
            foundation::new_id(),
            &doc.nik_enc
        )
        .is_err()
    );
    assert!(slot_consumed_or_gone(&c.env.pool, slot).await);

    let after = identity_status(&c.services, &who, PATIENT_ROUTE, p.owner_care_recipient_id)
        .await
        .unwrap();
    assert_eq!(after.status, "pending");
    assert_eq!(after.nik_last4.as_deref(), Some(&nik[12..]));
    assert!(after.can_cancel && !after.can_submit);

    let events = audit_events("e_10", p.owner_care_recipient_id).await;
    let submitted: Vec<_> = events
        .iter()
        .filter(|e| e.event.body.action_name == "identity_document_submitted")
        .collect();
    assert_eq!(submitted.len(), 1);
    let body = &submitted[0].event.body;
    assert_eq!(body.outcome, "success");
    assert_eq!(body.source_ip, Some(CLIENT_IP));
    let raw = serde_json::to_string(&body.after_values).unwrap();
    assert!(raw.contains(&nik[12..]), "4 digit terakhir NIK tercatat");
    assert!(
        !raw.contains(&nik),
        "NIK lengkap tidak pernah masuk audit (PRD §7.3)"
    );
}

// trace: US-106 AC2
#[tokio::test]
async fn nik_not_sixteen_digits_is_rejected_before_anything_is_stored() {
    let _g = SERIAL.lock().await;
    let c = ctx().await;
    let p = new_patient(&c.env.pool).await;
    let who = c.principals.patient(&p, true).await;
    let slot = c.env.uploaded_slot(p.account_id).await;
    for bad in [
        "123456789012345",
        "12345678901234567",
        "99ab567890123456",
        "",
    ] {
        let err = submit_identity_document(
            &c.services,
            &who,
            PATIENT_ROUTE,
            p.owner_care_recipient_id,
            bad,
            slot,
        )
        .await
        .unwrap_err();
        let ProfileError::Validation(fields) = &err else {
            panic!("bukan galat validasi: {err:?}");
        };
        assert_eq!(fields[0].field, "nik");
        assert_eq!(fields[0].message, "NIK harus 16 digit angka");
    }
    assert_eq!(
        request_count(&c.env.pool, p.owner_care_recipient_id).await,
        0
    );
    assert!(
        !files::slot_consumed(&c.env.pool, slot).await,
        "slot tidak tersentuh"
    );
}

// trace: US-106 AC5 (foto terkunci saat menunggu; batal mengembalikan status), AB-05a b
#[tokio::test]
async fn pending_submission_locks_photo_until_cancelled_and_cancel_restores_previous_status() {
    let _g = SERIAL.lock().await;
    let c = ctx().await;
    let p = new_patient(&c.env.pool).await;
    let who = c.principals.patient(&p, true).await;
    let cr = p.owner_care_recipient_id;
    let first_nik = synthetic_nik();
    let s1 = c.env.uploaded_slot(p.account_id).await;
    let first = submit_identity_document(&c.services, &who, PATIENT_ROUTE, cr, &first_nik, s1)
        .await
        .unwrap();

    // Menunggu: pengajuan kedua ditolak, berkasnya dibuang, status tidak berubah.
    let s2 = c.env.uploaded_slot(p.account_id).await;
    let err = submit_identity_document(&c.services, &who, PATIENT_ROUTE, cr, &synthetic_nik(), s2)
        .await
        .unwrap_err();
    assert!(matches!(err, ProfileError::SubmissionLocked), "{err:?}");
    assert_eq!(request_count(&c.env.pool, cr).await, 1);

    // Batal dari "Belum diunggah" → kembali "Belum diunggah".
    let v = cancel_identity_submission(&c.services, &who, PATIENT_ROUTE, cr)
        .await
        .unwrap();
    assert_eq!(v.status, "not_uploaded");
    assert_eq!(v.nik_last4, None);
    // Membatalkan lagi tanpa pengajuan menunggu: konflik.
    let err = cancel_identity_submission(&c.services, &who, PATIENT_ROUTE, cr)
        .await
        .unwrap_err();
    assert!(matches!(err, ProfileError::TransitionConflict), "{err:?}");

    // Disetujui, lalu pengajuan NIK baru dari "Disetujui" (AB-05a a), lalu batal → "Disetujui" NIK lama.
    let s3 = c.env.uploaded_slot(p.account_id).await;
    let second = submit_identity_document(&c.services, &who, PATIENT_ROUTE, cr, &first_nik, s3)
        .await
        .unwrap();
    assert_ne!(
        second.identity_verification_request_id,
        first.identity_verification_request_id
    );
    approve_in_db(
        &c.env,
        cr,
        second.identity_verification_request_id,
        &first_nik,
        false,
    )
    .await;
    let approved = identity_status(&c.services, &who, PATIENT_ROUTE, cr)
        .await
        .unwrap();
    assert_eq!(approved.status, "approved");
    assert!(approved.can_submit);

    let new_nik = synthetic_nik();
    let s4 = c.env.uploaded_slot(p.account_id).await;
    submit_identity_document(&c.services, &who, PATIENT_ROUTE, cr, &new_nik, s4)
        .await
        .unwrap();
    let pending = identity_status(&c.services, &who, PATIENT_ROUTE, cr)
        .await
        .unwrap();
    assert_eq!(pending.status, "pending");
    assert_eq!(pending.nik_last4.as_deref(), Some(&new_nik[12..]));
    let restored = cancel_identity_submission(&c.services, &who, PATIENT_ROUTE, cr)
        .await
        .unwrap();
    assert_eq!(restored.status, "approved");
    assert_eq!(
        restored.identity_verification_request_id,
        Some(second.identity_verification_request_id)
    );
    assert_eq!(restored.nik_last4.as_deref(), Some(&first_nik[12..]));

    let cancelled: Vec<_> = audit_events("e_10", cr)
        .await
        .into_iter()
        .filter(|e| e.event.body.action_name == "identity_submission_cancelled")
        .collect();
    assert_eq!(
        cancelled
            .iter()
            .filter(|e| e.event.body.outcome == "success")
            .count(),
        2
    );
    assert_eq!(
        cancelled
            .iter()
            .filter(|e| e.event.body.outcome == "failure")
            .count(),
        1,
        "batal tanpa pengajuan menunggu dicatat hasil gagal"
    );
}

// trace: US-106 AC5, US-201 AC5 (keputusan yang tersimpan lebih dulu menang atas pembatalan)
#[tokio::test]
async fn cancel_after_decision_is_rejected_and_status_kept() {
    let _g = SERIAL.lock().await;
    let c = ctx().await;
    let p = new_patient(&c.env.pool).await;
    let who = c.principals.patient(&p, true).await;
    let cr = p.owner_care_recipient_id;
    let nik = synthetic_nik();
    let slot = c.env.uploaded_slot(p.account_id).await;
    let s = submit_identity_document(&c.services, &who, PATIENT_ROUTE, cr, &nik, slot)
        .await
        .unwrap();
    approve_in_db(&c.env, cr, s.identity_verification_request_id, &nik, false).await;
    let err = cancel_identity_submission(&c.services, &who, PATIENT_ROUTE, cr)
        .await
        .unwrap_err();
    assert!(matches!(err, ProfileError::TransitionConflict), "{err:?}");
    let v = identity_status(&c.services, &who, PATIENT_ROUTE, cr)
        .await
        .unwrap();
    assert_eq!(v.status, "approved");
}

// trace: US-106 AC4, UP-40, TD-10 (registri versi kunci lama tetap terdeteksi)
#[tokio::test]
async fn owner_nik_already_approved_elsewhere_is_rejected_with_general_message() {
    let _g = SERIAL.lock().await;
    let c = ctx().await;
    let a = new_patient(&c.env.pool).await;
    let b = new_patient(&c.env.pool).await;
    let who_a = c.principals.patient(&a, true).await;
    let who_b = c.principals.patient(&b, true).await;
    let nik = synthetic_nik();
    let slot = c.env.uploaded_slot(a.account_id).await;
    let s = submit_identity_document(
        &c.services,
        &who_a,
        PATIENT_ROUTE,
        a.owner_care_recipient_id,
        &nik,
        slot,
    )
    .await
    .unwrap();
    // Registri ditulis dengan kunci versi 1 (sebelum rotasi); keyring kini versi 1 dan 2.
    approve_in_db(
        &c.env,
        a.owner_care_recipient_id,
        s.identity_verification_request_id,
        &nik,
        true,
    )
    .await;

    let slot_b = c.env.uploaded_slot(b.account_id).await;
    let files_before = files::keys_under(&c.env.storage, "files/").await;
    let err = submit_identity_document(
        &c.services,
        &who_b,
        PATIENT_ROUTE,
        b.owner_care_recipient_id,
        &nik,
        slot_b,
    )
    .await
    .unwrap_err();
    assert!(matches!(err, ProfileError::NikUnavailable), "{err:?}");
    assert_eq!(
        request_count(&c.env.pool, b.owner_care_recipient_id).await,
        0,
        "pengajuan tidak disimpan"
    );
    let v = identity_status(
        &c.services,
        &who_b,
        PATIENT_ROUTE,
        b.owner_care_recipient_id,
    )
    .await
    .unwrap();
    assert_eq!(v.status, "not_uploaded", "status identitas tidak berubah");
    // Berkas tervalidasi dibuang (discard_validated): hasil kode ulang tidak tersisa di bucket; slot
    // tetap ada tanpa `consumed_at` dan tidak dapat divalidasi ulang, lalu dibersihkan job pembersih
    // setelah kedaluwarsa (kontrak modul platform, TD-17).
    assert_eq!(
        files::keys_under(&c.env.storage, "files/").await,
        files_before,
        "hasil kode ulang foto pengajuan yang ditolak dibuang"
    );
    assert!(
        matches!(
            validate_upload(
                &c.env.pool,
                c.env.clock.as_ref(),
                &c.env.storage,
                b.account_id,
                slot_b,
                UploadPurpose::KtpPhoto,
            )
            .await,
            Err(FileError::NotFound)
        ),
        "slot pengajuan yang ditolak tidak dapat dipakai ulang"
    );

    // NIK yang sama sebagai anggota keluarga di akun lain tetap boleh (UP-40).
    let family = new_family_member(&c.env.pool, b.account_id).await;
    let slot_f = c.env.uploaded_slot(b.account_id).await;
    let f = submit_identity_document(&c.services, &who_b, PATIENT_ROUTE, family, &nik, slot_f)
        .await
        .unwrap();
    assert_eq!(f.status, "pending");
}

// trace: US-106 AC4 (batas berlapis: per akun 3/24 jam lalu ditahan, pesan umum sama, penahanan tanpa
// menyentuh berkas maupun registri)
#[tokio::test]
async fn three_rejections_hold_the_account_and_hold_skips_validation() {
    let _g = SERIAL.lock().await;
    let c = ctx().await;
    let attacker = new_patient(&c.env.pool).await;
    let who = c.principals.patient(&attacker, true).await;
    // Tiga NIK berbeda yang sudah disetujui sebagai pemilik akun lain.
    let mut niks = Vec::new();
    for _ in 0..3 {
        let o = new_patient(&c.env.pool).await;
        let who_o = c.principals.patient(&o, true).await;
        let n = synthetic_nik();
        let slot = c.env.uploaded_slot(o.account_id).await;
        let s = submit_identity_document(
            &c.services,
            &who_o,
            PATIENT_ROUTE,
            o.owner_care_recipient_id,
            &n,
            slot,
        )
        .await
        .unwrap();
        approve_in_db(
            &c.env,
            o.owner_care_recipient_id,
            s.identity_verification_request_id,
            &n,
            false,
        )
        .await;
        niks.push(n);
    }
    for n in &niks {
        let slot = c.env.uploaded_slot(attacker.account_id).await;
        let err = submit_identity_document(
            &c.services,
            &who,
            PATIENT_ROUTE,
            attacker.owner_care_recipient_id,
            n,
            slot,
        )
        .await
        .unwrap_err();
        assert!(matches!(err, ProfileError::NikUnavailable));
    }
    // Ditahan 24 jam: NIK baru yang sah pun dijawab pesan umum yang sama, tanpa validasi foto.
    let slot = c.env.uploaded_slot(attacker.account_id).await;
    let err = submit_identity_document(
        &c.services,
        &who,
        PATIENT_ROUTE,
        attacker.owner_care_recipient_id,
        &synthetic_nik(),
        slot,
    )
    .await
    .unwrap_err();
    assert!(matches!(err, ProfileError::NikUnavailable), "{err:?}");
    assert!(
        !files::slot_consumed(&c.env.pool, slot).await,
        "penahanan sebelum validasi berkas"
    );
    assert_eq!(
        request_count(&c.env.pool, attacker.owner_care_recipient_id).await,
        0
    );
}

// trace: US-106 AC4 (per NIK lintas akun: 3 penolakan NIK yang sama dari akun mana pun → NIK ditahan)
#[tokio::test]
async fn same_nik_rejected_three_times_across_accounts_is_held_for_everyone() {
    let _g = SERIAL.lock().await;
    let c = ctx().await;
    let holder = new_patient(&c.env.pool).await;
    let who_h = c.principals.patient(&holder, true).await;
    let nik = synthetic_nik();
    let slot = c.env.uploaded_slot(holder.account_id).await;
    let s = submit_identity_document(
        &c.services,
        &who_h,
        PATIENT_ROUTE,
        holder.owner_care_recipient_id,
        &nik,
        slot,
    )
    .await
    .unwrap();
    approve_in_db(
        &c.env,
        holder.owner_care_recipient_id,
        s.identity_verification_request_id,
        &nik,
        false,
    )
    .await;
    for _ in 0..3 {
        let x = new_patient(&c.env.pool).await;
        let who_x = c.principals.patient(&x, true).await;
        let slot = c.env.uploaded_slot(x.account_id).await;
        let err = submit_identity_document(
            &c.services,
            &who_x,
            PATIENT_ROUTE,
            x.owner_care_recipient_id,
            &nik,
            slot,
        )
        .await
        .unwrap_err();
        assert!(matches!(err, ProfileError::NikUnavailable));
    }
    // Akun keempat (belum pernah ditolak) ditolak lewat penahanan NIK, sebelum validasi berkas.
    let y = new_patient(&c.env.pool).await;
    let who_y = c.principals.patient(&y, true).await;
    let slot = c.env.uploaded_slot(y.account_id).await;
    let err = submit_identity_document(
        &c.services,
        &who_y,
        PATIENT_ROUTE,
        y.owner_care_recipient_id,
        &nik,
        slot,
    )
    .await
    .unwrap_err();
    assert!(matches!(err, ProfileError::NikUnavailable));
    assert!(!files::slot_consumed(&c.env.pool, slot).await);
}

// trace: US-106 AC4 (per alamat IP 10 penolakan per 24 jam)
#[tokio::test]
async fn ten_rejections_from_one_ip_hold_that_ip() {
    let _g = SERIAL.lock().await;
    let c = ctx().await;
    let holder = new_patient(&c.env.pool).await;
    let who_h = c.principals.patient(&holder, true).await;
    let nik = synthetic_nik();
    let slot = c.env.uploaded_slot(holder.account_id).await;
    let s = submit_identity_document(
        &c.services,
        &who_h,
        PATIENT_ROUTE,
        holder.owner_care_recipient_id,
        &nik,
        slot,
    )
    .await
    .unwrap();
    approve_in_db(
        &c.env,
        holder.owner_care_recipient_id,
        s.identity_verification_request_id,
        &nik,
        false,
    )
    .await;
    // Penahanan per NIK (3) akan lebih dulu berlaku untuk NIK yang sama, jadi penghitung IP dinaikkan
    // langsung untuk menguji ambang IP secara terpisah.
    let client = c.env.valkey.client(Duration::from_millis(500));
    for _ in 0..10 {
        client
            .incr(&format!("rs:profile:nik-limit:ip:{CLIENT_IP}:count"))
            .await
            .unwrap();
    }
    let fresh = new_patient(&c.env.pool).await;
    let who_f = c.principals.patient(&fresh, true).await;
    let slot = c.env.uploaded_slot(fresh.account_id).await;
    let err = submit_identity_document(
        &c.services,
        &who_f,
        PATIENT_ROUTE,
        fresh.owner_care_recipient_id,
        &synthetic_nik(),
        slot,
    )
    .await
    .unwrap_err();
    assert!(matches!(err, ProfileError::NikUnavailable), "{err:?}");
    assert!(!files::slot_consumed(&c.env.pool, slot).await);
}

// trace: KU-03 (ganti ID: milik orang lain dan tidak ada dijawab sama), US-201 AC6
#[tokio::test]
async fn foreign_or_unknown_care_recipient_is_not_found() {
    let _g = SERIAL.lock().await;
    let c = ctx().await;
    let a = new_patient(&c.env.pool).await;
    let b = new_patient(&c.env.pool).await;
    let who_a = c.principals.patient(&a, true).await;
    for target in [b.owner_care_recipient_id, foundation::new_id()] {
        let slot = c.env.uploaded_slot(a.account_id).await;
        let err = submit_identity_document(
            &c.services,
            &who_a,
            PATIENT_ROUTE,
            target,
            &synthetic_nik(),
            slot,
        )
        .await
        .unwrap_err();
        assert!(matches!(err, ProfileError::NotFound), "{err:?}");
        assert!(!files::slot_consumed(&c.env.pool, slot).await);
        assert!(matches!(
            identity_status(&c.services, &who_a, PATIENT_ROUTE, target)
                .await
                .unwrap_err(),
            ProfileError::NotFound
        ));
        assert!(matches!(
            cancel_identity_submission(&c.services, &who_a, PATIENT_ROUTE, target)
                .await
                .unwrap_err(),
            ProfileError::NotFound
        ));
    }
    assert_eq!(
        request_count(&c.env.pool, b.owner_care_recipient_id).await,
        0
    );
}

// trace: TD-22 (tanpa IP klien: gagal-tertutup sebelum ada efek), coding-standards §4.4 butir 1
#[tokio::test]
async fn missing_client_ip_fails_closed_without_side_effects() {
    let _g = SERIAL.lock().await;
    let c = ctx().await;
    let p = new_patient(&c.env.pool).await;
    let who = c.principals.patient(&p, false).await;
    let slot = c.env.uploaded_slot(p.account_id).await;
    let err = submit_identity_document(
        &c.services,
        &who,
        PATIENT_ROUTE,
        p.owner_care_recipient_id,
        &synthetic_nik(),
        slot,
    )
    .await
    .unwrap_err();
    assert!(matches!(err, ProfileError::Internal(_)), "{err:?}");
    assert_eq!(
        request_count(&c.env.pool, p.owner_care_recipient_id).await,
        0
    );
    assert!(!files::slot_consumed(&c.env.pool, slot).await);
}

// trace: US-106 AC4 (kontrol batas tidak dilewati diam-diam: Valkey mati = 503)
#[tokio::test]
async fn valkey_unavailable_fails_closed() {
    let _g = SERIAL.lock().await;
    let c = ctx().await;
    let services = IdentityServices::new(
        c.env.pool.clone(),
        Arc::clone(&c.env.clock),
        Arc::clone(&c.env.storage),
        c.env.keyring.clone(),
        Arc::clone(&c.env.blind_index),
        valkey::Valkey::connect_lazy(&valkey::ValkeyConfig {
            url: valkey_support::dead_url(),
            op_timeout: Duration::from_millis(200),
        })
        .unwrap(),
    );
    let p = new_patient(&c.env.pool).await;
    let who = c.principals.patient(&p, true).await;
    let slot = c.env.uploaded_slot(p.account_id).await;
    let err = submit_identity_document(
        &services,
        &who,
        PATIENT_ROUTE,
        p.owner_care_recipient_id,
        &synthetic_nik(),
        slot,
    )
    .await
    .unwrap_err();
    assert!(matches!(err, ProfileError::Database), "{err:?}");
    assert_eq!(
        request_count(&c.env.pool, p.owner_care_recipient_id).await,
        0
    );
}

/// Penghitung penolakan di Valkey diisi langsung tanpa kunci penahanan: meniru kegagalan `SET` kunci
/// penahanan tepat saat ambang tercapai (review N1 F1-T60).
async fn set_counter(c: &Ctx, key: &str, value: &[u8]) {
    c.env
        .valkey
        .client(Duration::from_millis(500))
        .set(key, value, Duration::from_secs(600))
        .await
        .unwrap();
}

// trace: US-106 AC4 (penahanan tetap berlaku bila kunci penahanan gagal ditulis saat ambang tercapai:
// penghitung >= ambang sudah menahan, per akun dan per NIK lintas akun)
#[tokio::test]
async fn counter_at_threshold_holds_even_without_hold_key() {
    let _g = SERIAL.lock().await;
    let c = ctx().await;

    // Per akun: 3 penolakan tercatat, kunci penahanan tidak ada.
    let a = new_patient(&c.env.pool).await;
    let who_a = c.principals.patient(&a, true).await;
    let account_key = format!(
        "rs:profile:nik-limit:account:{}:count",
        a.account_id.hyphenated()
    );
    set_counter(&c, &account_key, b"3").await;
    let slot = c.env.uploaded_slot(a.account_id).await;
    let err = submit_identity_document(
        &c.services,
        &who_a,
        PATIENT_ROUTE,
        a.owner_care_recipient_id,
        &synthetic_nik(),
        slot,
    )
    .await
    .unwrap_err();
    assert!(matches!(err, ProfileError::NikUnavailable), "{err:?}");
    assert_eq!(
        request_count(&c.env.pool, a.owner_care_recipient_id).await,
        0
    );
    assert!(
        !files::slot_consumed(&c.env.pool, slot).await,
        "penahanan diputuskan sebelum foto divalidasi"
    );

    // Per NIK lintas akun: 3 penolakan untuk NIK ini (kunci = SHA-256 blind index versi terbaru).
    let b = new_patient(&c.env.pool).await;
    let who_b = c.principals.patient(&b, true).await;
    let nik = synthetic_nik();
    let bidx = c
        .env
        .blind_index
        .compute(&Nik::parse(&nik).unwrap())
        .unwrap();
    let digest: String = crypto::sha256(&bidx.value)
        .iter()
        .map(|x| format!("{x:02x}"))
        .collect();
    set_counter(
        &c,
        &format!("rs:profile:nik-limit:nik:{digest}:count"),
        b"3",
    )
    .await;
    let slot_b = c.env.uploaded_slot(b.account_id).await;
    let err = submit_identity_document(
        &c.services,
        &who_b,
        PATIENT_ROUTE,
        b.owner_care_recipient_id,
        &nik,
        slot_b,
    )
    .await
    .unwrap_err();
    assert!(matches!(err, ProfileError::NikUnavailable), "{err:?}");
    assert_eq!(
        request_count(&c.env.pool, b.owner_care_recipient_id).await,
        0
    );

    // Di bawah ambang (2) tidak menahan.
    let d = new_patient(&c.env.pool).await;
    let who_d = c.principals.patient(&d, true).await;
    let d_key = format!(
        "rs:profile:nik-limit:account:{}:count",
        d.account_id.hyphenated()
    );
    set_counter(&c, &d_key, b"2").await;
    let slot_d = c.env.uploaded_slot(d.account_id).await;
    let ok = submit_identity_document(
        &c.services,
        &who_d,
        PATIENT_ROUTE,
        d.owner_care_recipient_id,
        &synthetic_nik(),
        slot_d,
    )
    .await
    .unwrap();
    assert_eq!(ok.status, "pending");
}
