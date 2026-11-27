//! F1-T109 (T1; US-1501 AC1-AC2, US-1504 AC3, ADR-009 butir 5): event audit korporat ikut rantai audit
//! terverifikasi. Proses `api` lengkap (HR realm `korporat` dan Verifikator realm `admin`) terhadap PostgreSQL 18
//! sekali pakai (`scripts/db-ephemeral.sh`): HR mengajukan ulang perusahaan Ditolak, mengulang klik (ditolak aturan,
//! E-10 gagal), lalu Verifikator menyetujui. Semua event perusahaan diperiksa pemeriksa korporat bersama (lengkap,
//! tenant, tanpa data mentah), lalu dipindahkan relay sungguhan ke database audit dan diperiksa hash rantainya
//! (`event_hash` dihitung ulang, `prev_hash` menyambung).
//!
//! Berkas test tersendiri dengan satu test: relay menghapus baris `audit_outbox` yang sudah dipindahkan, sehingga
//! tidak boleh berjalan bersamaan dengan test lain yang membaca outbox (berkas test dijalankan berurutan). Tanpa
//! variabel `RS_TEST_DB_*` test GAGAL. Data sintetis saja.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)] // berkas test

mod audit_support;
mod common;
mod corporate_documents_support;
mod files;

use corporate_documents_support::{
    client_status, code, insert_client, insert_hr, insert_verifier, seed_all_documents, world_with,
};
use foundation::new_id;
use std::sync::Arc;

const RESUBMIT: &str = "/v1/corporate/verification-resubmissions";

// trace: F1-T109 (US-1501 AC1-AC2, US-1504 AC3): event korporat HR dan Verifikator lengkap, bertenant, tanpa data
// mentah, dan ada di rantai hash audit
#[tokio::test(flavor = "multi_thread")]
async fn corporate_events_are_clean_and_chained_in_the_audit_database() {
    let w = world_with(Arc::new(common::unused_file_storage())).await;
    let verifier = insert_verifier(&w.app).await;
    let c = insert_client(&w.app, "rejected", 30).await;
    let hr = insert_hr(&w.app, c.id, "active").await;
    seed_all_documents(&w.app, c.id).await;
    sqlx::query(
        "INSERT INTO app.corporate_verification_decision \
           (id, created_by, corporate_client_id, submission_round, decision, reason_code, decided_by) \
         VALUES ($1, $2, $3, 1, 'rejected', 'authority_proof_insufficient', $2)",
    )
    .bind(new_id())
    .bind(hr.account_id)
    .bind(c.id)
    .execute(&w.app)
    .await
    .unwrap();

    let header = w.hr_header(&hr);
    let r = common::request(w.addr, "POST", RESUBMIT, &[&header]).await;
    assert_eq!(r.status, 200, "{}", r.body);
    let r = common::request(w.addr, "POST", RESUBMIT, &[&header]).await;
    assert_eq!((r.status, code(&r).as_str()), (409, "4090001"));
    let r = w.decide(&verifier, c.id, None).await;
    assert_eq!(r.status, 200, "{}", r.body);
    assert_eq!(client_status(&w.app, c.id).await, ("active".to_string(), 2));

    let events = w.all_tenant_events(c.id).await;
    let mut kinds: Vec<(String, String)> = events
        .iter()
        .map(|e| {
            (
                e["action_name"].as_str().unwrap().to_string(),
                e["outcome"].as_str().unwrap().to_string(),
            )
        })
        .collect();
    kinds.sort();
    assert_eq!(
        kinds,
        [
            // CR-037: Verifikator membuka detail (membaca versi dokumen) sebelum memutuskan.
            ("corporate_detail_opened", "success"),
            ("corporate_verification_approved", "success"),
            ("corporate_verification_resubmitted", "failure"),
            ("corporate_verification_resubmitted", "success"),
        ]
        .map(|(a, o)| (a.to_string(), o.to_string()))
    );
    let tenant = c.id.to_string();
    audit_support::assert_corporate_events_clean(
        &events,
        &["e_05", "e_10"],
        Some(&tenant),
        &[
            c.nib.as_str(),
            c.npwp.as_str(),
            hr.email.as_str(),
            hr.subject.as_str(),
        ],
    );

    // Rantai: keempat event perusahaan ada di database audit dengan hash yang dapat dihitung ulang.
    assert_eq!(audit_support::relay_and_verify_tenant_chain(c.id).await, 4);
    assert!(
        w.all_tenant_events(c.id).await.is_empty(),
        "outbox sudah dipindahkan relay"
    );
}
