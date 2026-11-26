//! CR-035 (T1; keputusan user Q10 = B2, 2026-11-26; TD-45, TD-46): NIB unik hanya di antara perusahaan Aktif.
//! Pengajuan dengan NIB yang sudah dipakai perusahaan lain masuk antrean Verifikator dengan penanda
//! `nib_used_by_other_company` (antrean dan detail); persetujuan ditolak `409` `4090116` bila NIB sudah dipakai
//! perusahaan Aktif lain, termasuk dua persetujuan bersamaan (`uqp_corporate_client_nib_active`). Proses `api`
//! lengkap terhadap PostgreSQL 18 sekali pakai (`scripts/db-ephemeral.sh`, role runtime `rs_api`). Tanpa variabel
//! `RS_TEST_DB_*` test GAGAL. Data sintetis saja.
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

use audit_support::assert_event_complete;
use corporate_documents_support::{
    Company, QUEUE, World, client_status, code, digits, insert_client, insert_verifier,
    seed_all_documents, world_with,
};
use foundation::Uuid;
use serde_json::Value;
use sqlx::PgPool;
use std::sync::Arc;

async fn plain_world() -> World {
    world_with(Arc::new(common::unused_file_storage())).await
}

/// Perusahaan berstatus `status` dengan NIB `nib` dan ketiga dokumen.
async fn company_with_nib(pool: &PgPool, status: &str, nib: &str, age_minutes: i32) -> Company {
    let c = insert_client(pool, status, age_minutes).await;
    sqlx::query!(
        "UPDATE app.corporate_client SET nib = $2 WHERE id = $1",
        c.id,
        nib
    )
    .execute(pool)
    .await
    .unwrap();
    seed_all_documents(pool, c.id).await;
    Company {
        nib: nib.to_string(),
        ..c
    }
}

async fn queue_item(w: &World, verifier: &str, id: Uuid) -> Value {
    let r = w
        .verifier_get(verifier, &format!("{QUEUE}?limit=100&readiness=all"))
        .await;
    assert_eq!(r.status, 200, "{}", r.body);
    r.json()["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["corporate_client_id"] == id.to_string())
        .cloned()
        .unwrap_or_else(|| panic!("{id} tidak ada di antrean"))
}

async fn detail(w: &World, verifier: &str, id: Uuid) -> Value {
    let r = w.verifier_get(verifier, &format!("{QUEUE}/{id}")).await;
    assert_eq!(r.status, 200, "{}", r.body);
    r.json()["data"].clone()
}

async fn decisions(pool: &PgPool, id: Uuid) -> i64 {
    sqlx::query_scalar!(
        r#"SELECT count(*) AS "n!" FROM app.corporate_verification_decision WHERE corporate_client_id = $1"#,
        id
    )
    .fetch_one(pool)
    .await
    .unwrap()
}

// trace: CR-035 (penanda "NIB sudah dipakai perusahaan lain" di antrean dan detail Verifikator)
#[tokio::test(flavor = "multi_thread")]
async fn queue_and_detail_flag_submissions_whose_nib_is_used_by_another_company() {
    let w = plain_world().await;
    let verifier = insert_verifier(&w.app).await;
    let shared = digits(13);
    let a = company_with_nib(&w.app, "pending_verification", &shared, 30).await;
    let b = company_with_nib(&w.app, "pending_verification", &shared, 20).await;
    let unique = company_with_nib(&w.app, "pending_verification", &digits(13), 10).await;
    for (c, flag) in [(&a, true), (&b, true), (&unique, false)] {
        assert_eq!(
            queue_item(&w, &verifier, c.id).await["nib_used_by_other_company"],
            flag
        );
        let d = detail(&w, &verifier, c.id).await;
        assert_eq!(d["nib_used_by_other_company"], flag);
        // Detail tidak membuka perusahaan lain pemilik NIB yang sama.
        let other = if c.id == a.id { b.id } else { a.id };
        assert!(!d.to_string().contains(&other.to_string()));
    }
    // Perusahaan terhapus lunak tidak dihitung.
    sqlx::query!(
        "UPDATE app.corporate_client SET deleted_at = now(), delete_reason = 'uji' WHERE id = $1",
        b.id
    )
    .execute(&w.app)
    .await
    .unwrap();
    assert_eq!(
        queue_item(&w, &verifier, a.id).await["nib_used_by_other_company"],
        false
    );
}

// trace: CR-035 (NIB unik hanya di antara perusahaan Aktif): persetujuan ditolak 4090116 tanpa efek dan tercatat
// gagal; penolakan tetap boleh; pemilik NIB yang Ditolak/Nonaktif tidak menghalangi
#[tokio::test(flavor = "multi_thread")]
async fn approval_is_refused_when_the_nib_belongs_to_another_active_company() {
    let w = plain_world().await;
    let verifier = insert_verifier(&w.app).await;
    let shared = digits(13);
    let active = company_with_nib(&w.app, "active", &shared, 60).await;
    let newcomer = company_with_nib(&w.app, "pending_verification", &shared, 30).await;
    assert_eq!(
        detail(&w, &verifier, newcomer.id).await["nib_used_by_other_company"],
        true
    );

    let r = w.decide(&verifier, newcomer.id, None).await;
    assert_eq!(
        (r.status, code(&r).as_str()),
        (409, "4090116"),
        "{}",
        r.body
    );
    assert!(
        !r.body.contains(&active.id.to_string()),
        "perusahaan lain tidak terbuka"
    );
    assert!(!r.body.contains(&shared), "NIB tidak digemakan");
    assert_eq!(
        client_status(&w.app, newcomer.id).await,
        ("pending_verification".to_string(), 1)
    );
    assert_eq!(
        decisions(&w.app, newcomer.id).await,
        0,
        "tanpa catatan keputusan"
    );
    assert_eq!(client_status(&w.app, active.id).await.0, "active");
    let evs = w
        .tenant_events("e_10", "corporate_verification_approved", newcomer.id)
        .await;
    assert_eq!(evs.len(), 1, "{evs:?}");
    assert_event_complete(&evs[0], "e_10");
    assert_eq!(evs[0]["outcome"], "failure");
    assert_eq!(
        evs[0]["outcome_reason_code"],
        "nib_in_use_by_active_company"
    );
    assert!(!evs[0].to_string().contains(&shared));
    assert!(!evs[0].to_string().contains(&active.id.to_string()));

    // Penolakan tetap boleh.
    let r = w
        .decide(&verifier, newcomer.id, Some("company_data_mismatch"))
        .await;
    assert_eq!(r.status, 200, "{}", r.body);

    // Pemilik NIB yang Ditolak atau Nonaktif tidak menghalangi persetujuan.
    let nib2 = digits(13);
    company_with_nib(&w.app, "rejected", &nib2, 90).await;
    company_with_nib(&w.app, "inactive", &nib2, 80).await;
    let ok = company_with_nib(&w.app, "pending_verification", &nib2, 10).await;
    let r = w.decide(&verifier, ok.id, None).await;
    assert_eq!(r.status, 200, "{}", r.body);
    assert_eq!(client_status(&w.app, ok.id).await.0, "active");
}

// trace: CR-035, US-1902 AC4 (negatif): dua persetujuan bersamaan atas dua pengajuan dengan NIB sama = tepat satu
// perusahaan Aktif; yang kalah 409 4090116 tanpa efek (`uqp_corporate_client_nib_active`)
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_approvals_of_two_submissions_with_one_nib_activate_exactly_one() {
    let w = plain_world().await;
    let verifier = insert_verifier(&w.app).await;
    for _ in 0..3 {
        let shared = digits(13);
        let a = company_with_nib(&w.app, "pending_verification", &shared, 30).await;
        let b = company_with_nib(&w.app, "pending_verification", &shared, 20).await;
        let (va, vb) = (
            w.documents_version(&verifier, a.id).await,
            w.documents_version(&verifier, b.id).await,
        );
        let (ra, rb) = tokio::join!(
            w.decide_with(&verifier, a.id, None, &va),
            w.decide_with(&verifier, b.id, None, &vb)
        );
        let mut outcomes = [(ra.status, code(&ra)), (rb.status, code(&rb))];
        outcomes.sort();
        assert_eq!(
            outcomes,
            [(200, "2000000".to_string()), (409, "4090116".to_string())],
            "{} | {}",
            ra.body,
            rb.body
        );
        let active = sqlx::query_scalar!(
            r#"SELECT count(*) AS "n!" FROM app.corporate_client
               WHERE nib = $1 AND status = 'active' AND deleted_at IS NULL"#,
            shared
        )
        .fetch_one(&w.app)
        .await
        .unwrap();
        assert_eq!(active, 1);
        assert_eq!(
            decisions(&w.app, a.id).await + decisions(&w.app, b.id).await,
            1,
            "hanya pemenang yang punya catatan keputusan"
        );
    }
}
