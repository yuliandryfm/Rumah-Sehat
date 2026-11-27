//! F1-T102b (T1; US-1901 AC6, US-1902 AC3; TD-47; keputusan user Q14=A): profil perusahaan di
//! `GET /v1/corporate/documents` dan pengajuan ulang setelah ditolak `POST /v1/corporate/verification-resubmissions`
//! lewat proses `api` lengkap terhadap PostgreSQL 18 sekali pakai (`scripts/db-ephemeral.sh`, role runtime
//! `rs_api`). Tanpa variabel `RS_TEST_DB_*` test GAGAL (tidak dilewati). Data sintetis saja.
//!
//! Skenario negatif (coding-standards §7.3, modul T1): pengajuan ulang ganda dan bersamaan, status selain
//! Ditolak, dokumen belum lengkap, perusahaan lain tidak tersentuh (isolasi tenant), token realm lain, audit
//! tanpa data perusahaan.
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
    Company, DOCUMENTS, Hr, World, client_status, code, insert_client, insert_hr, insert_verifier,
    seed_all_documents, seed_document, world_with,
};
use foundation::{Uuid, new_id};
use serde_json::{Value, json};
use sqlx::PgPool;
use std::sync::Arc;

const RESUBMIT: &str = "/v1/corporate/verification-resubmissions";
const ACTION: &str = "corporate_verification_resubmitted";

async fn plain_world() -> World {
    world_with(Arc::new(common::unused_file_storage())).await
}

/// Perusahaan berstatus `status` dengan satu HR aktif.
async fn company_with_hr(pool: &PgPool, status: &str) -> (Company, Hr) {
    let c = insert_client(pool, status, 30).await;
    let hr = insert_hr(pool, c.id, "active").await;
    (c, hr)
}

/// Keputusan penolakan putaran `round` (kode alasan tertutup, daftar draf F1-T104).
async fn seed_rejection(pool: &PgPool, client: Uuid, round: i32, reason: &str, by: Uuid) {
    sqlx::query!(
        "INSERT INTO app.corporate_verification_decision \
           (id, created_by, corporate_client_id, submission_round, decision, reason_code, decided_by) \
         VALUES ($1, $2, $3, $4, 'rejected', $5, $2)",
        new_id(),
        by,
        client,
        round,
        reason,
    )
    .execute(pool)
    .await
    .unwrap();
}

/// Perusahaan Ditolak putaran 1 dengan ketiga dokumen dan HR aktif.
async fn rejected_complete(pool: &PgPool) -> (Company, Hr) {
    let (c, hr) = company_with_hr(pool, "rejected").await;
    seed_all_documents(pool, c.id).await;
    seed_rejection(pool, c.id, 1, "authority_proof_insufficient", hr.account_id).await;
    (c, hr)
}

async fn submitted_at(pool: &PgPool, id: Uuid) -> time::OffsetDateTime {
    sqlx::query_scalar!(
        "SELECT submitted_at FROM app.corporate_client WHERE id = $1",
        id
    )
    .fetch_one(pool)
    .await
    .unwrap()
}

async fn resubmit(w: &World, hr: &Hr) -> common::Resp {
    common::request(w.addr, "POST", RESUBMIT, &[&w.hr_header(hr)]).await
}

/// Event pengajuan ulang perusahaan `client` dengan hasil `outcome` (dan kode alasan bila gagal).
async fn resubmission_events(
    w: &World,
    client: Uuid,
    outcome: &str,
    reason: Option<&str>,
) -> Vec<Value> {
    w.tenant_events("e_10", ACTION, client)
        .await
        .into_iter()
        .filter(|e| {
            e["outcome"] == outcome
                && reason.is_none_or(|r| e["outcome_reason_code"].as_str() == Some(r))
        })
        .collect()
}

// trace: US-1901 AC6 (HR melihat status, alasan penolakan, dan profil perusahaannya), TD-47 b
#[tokio::test(flavor = "multi_thread")]
async fn status_page_shows_own_company_profile_only() {
    let w = plain_world().await;
    let (a, hr_a) = rejected_complete(&w.app).await;
    let (b, _hr_b) = company_with_hr(&w.app, "pending_verification").await;

    let r = w.hr_get(&hr_a, DOCUMENTS).await;
    assert_eq!(r.status, 200, "{}", r.body);
    assert_eq!(r.header("cache-control"), Some("no-store"));
    let d = &r.json()["data"];
    assert_eq!(d["status"], "rejected");
    assert_eq!(d["rejection_reason"], "authority_proof_insufficient");
    let company = &d["company"];
    assert_eq!(company["nib"], a.nib.as_str());
    assert_eq!(company["npwp"], a.npwp.as_str());
    assert!(
        company["legal_name"]
            .as_str()
            .unwrap()
            .starts_with("PT Dokumen Uji")
    );
    assert_eq!(company["office_address"], "Jl. Sintetis No. 1");
    assert_eq!(company["office_phone"], "0210000001");
    assert!(company["submitted_at"].is_string());
    // Tanpa ID atau data perusahaan lain.
    for m in [b.id.to_string(), b.nib.clone(), b.npwp.clone()] {
        assert!(!r.body.contains(&m), "data perusahaan lain bocor: {m}");
    }
    // Profil tidak menampilkan ID perusahaan maupun akun (perusahaan selalu dari sesi).
    assert!(!r.body.contains(&a.id.to_string()));
    assert!(!r.body.contains(&hr_a.account_id.to_string()));
}

// trace: US-1902 AC3 (HR memperbaiki lalu mengajukan ulang; status kembali Menunggu verifikasi dan masuk
// antrean lagi), TD-47 a, ADR-005 (E-10 tanpa data perusahaan)
#[tokio::test(flavor = "multi_thread")]
async fn rejected_company_resubmits_into_the_queue_with_a_new_round_and_audit() {
    let w = plain_world().await;
    let verifier = insert_verifier(&w.app).await;
    let (c, hr) = rejected_complete(&w.app).await;
    let before = submitted_at(&w.app, c.id).await;
    assert!(
        !w.queue_ids(&verifier, Some("all"))
            .await
            .contains(&c.id.to_string()),
        "pengajuan ditolak tidak ada di antrean"
    );

    let r = resubmit(&w, &hr).await;
    assert_eq!(r.status, 200, "{}", r.body);
    assert_eq!(r.header("cache-control"), Some("no-store"));
    let d = &r.json()["data"];
    assert_eq!(d["status"], "pending_verification");
    assert_eq!(d["submission_round"], 2);
    assert_eq!(
        d["rejection_reason"],
        Value::Null,
        "alasan putaran lama tidak tampil"
    );
    assert_eq!(d["complete"], true);
    assert_eq!(d["company"]["nib"], c.nib.as_str());
    assert_eq!(
        client_status(&w.app, c.id).await,
        ("pending_verification".to_string(), 2)
    );
    assert!(
        submitted_at(&w.app, c.id).await > before,
        "antrean memakai waktu pengajuan ulang"
    );
    assert!(
        w.queue_ids(&verifier, None)
            .await
            .contains(&c.id.to_string()),
        "pengajuan ulang masuk antrean siap diverifikasi"
    );

    // Audit E-10: status dan putaran saja, aktor HR, tenant = perusahaan; tanpa NIB/NPWP/nama.
    let events = w.tenant_events("e_10", ACTION, c.id).await;
    assert_eq!(events.len(), 1, "{events:?}");
    let e = &events[0];
    assert_event_complete(e, "e_10");
    assert_eq!(e["outcome"], "success");
    assert_eq!(e["actor_account_id"], hr.account_id.to_string());
    let dump = e.to_string();
    for secret in [
        c.nib.as_str(),
        c.npwp.as_str(),
        "PT Dokumen Uji",
        "Jl. Sintetis",
    ] {
        assert!(!dump.contains(secret), "audit memuat {secret}");
    }
    assert!(
        dump.contains("pending_verification") && dump.contains("rejected"),
        "{dump}"
    );
    // F1-T109: pemeriksa korporat bersama atas semua event perusahaan ini (pengajuan ulang + keputusan).
    let tenant = c.id.to_string();
    audit_support::assert_corporate_events_clean(
        &w.all_tenant_events(c.id).await,
        &["e_10"],
        Some(&tenant),
        &[
            c.nib.as_str(),
            c.npwp.as_str(),
            hr.email.as_str(),
            "PT Dokumen Uji",
        ],
    );

    // Verifikator memutuskan putaran baru: tidak terhalang keputusan putaran 1.
    let r = w
        .decide(&verifier, c.id, Some("company_data_mismatch"))
        .await;
    assert_eq!(r.status, 200, "{}", r.body);
    assert_eq!(r.json()["data"]["submission_round"], 2);
    let r = w.hr_get(&hr, DOCUMENTS).await;
    assert_eq!(
        r.json()["data"]["rejection_reason"],
        "company_data_mismatch"
    );
    // Ditolak lagi lalu diajukan ulang lagi: putaran 3, lalu disetujui.
    let r = resubmit(&w, &hr).await;
    assert_eq!(r.status, 200, "{}", r.body);
    assert_eq!(r.json()["data"]["submission_round"], 3);
    let r = w.decide(&verifier, c.id, None).await;
    assert_eq!(r.status, 200, "{}", r.body);
    assert_eq!(client_status(&w.app, c.id).await, ("active".to_string(), 3));
    // Perusahaan Aktif tidak dapat diajukan ulang (dokumen terverifikasi tidak diubah HR).
    let r = resubmit(&w, &hr).await;
    assert_eq!((r.status, code(&r).as_str()), (409, "4090112"));
}

// trace: US-1902 AC3 (negatif): hanya dari Ditolak, dokumen wajib lengkap, tanpa efek saat ditolak
#[tokio::test(flavor = "multi_thread")]
async fn resubmission_is_refused_outside_rejected_or_without_complete_documents() {
    let w = plain_world().await;

    // Menunggu verifikasi (belum diputuskan): 4090001, tanpa efek.
    let (p, hr_p) = company_with_hr(&w.app, "pending_verification").await;
    seed_all_documents(&w.app, p.id).await;
    let r = resubmit(&w, &hr_p).await;
    assert_eq!(
        (r.status, code(&r).as_str()),
        (409, "4090001"),
        "{}",
        r.body
    );
    assert_eq!(
        client_status(&w.app, p.id).await,
        ("pending_verification".to_string(), 1)
    );
    // F1-T109 (KEPUTUSAN-383): penolakan aturan tercatat E-10 gagal, tanpa event sukses.
    assert!(
        resubmission_events(&w, p.id, "success", None)
            .await
            .is_empty()
    );
    let refused = resubmission_events(&w, p.id, "failure", Some("not_resubmittable")).await;
    assert_eq!(refused.len(), 1, "{refused:?}");
    let tenant_p = p.id.to_string();
    audit_support::assert_corporate_events_clean(
        &refused,
        &["e_10"],
        Some(&tenant_p),
        &[p.nib.as_str(), p.npwp.as_str(), hr_p.email.as_str()],
    );
    assert_eq!(refused[0]["actor_account_id"], hr_p.account_id.to_string());
    assert_eq!(refused[0]["object_id"], p.id.to_string());
    assert!(
        refused[0]["changed_fields"]
            .as_array()
            .is_none_or(Vec::is_empty)
    );

    // Ditolak dengan dokumen belum lengkap: 4090115, tetap Ditolak putaran 1.
    let (inc, hr_inc) = company_with_hr(&w.app, "rejected").await;
    seed_rejection(
        &w.app,
        inc.id,
        1,
        "npwp_document_invalid",
        hr_inc.account_id,
    )
    .await;
    seed_document(&w.app, inc.id, "nib").await;
    seed_document(&w.app, inc.id, "authority_proof").await;
    let r = resubmit(&w, &hr_inc).await;
    assert_eq!(
        (r.status, code(&r).as_str()),
        (409, "4090115"),
        "{}",
        r.body
    );
    assert_eq!(
        client_status(&w.app, inc.id).await,
        ("rejected".to_string(), 1)
    );
    assert!(
        resubmission_events(&w, inc.id, "success", None)
            .await
            .is_empty()
    );
    let refused = resubmission_events(&w, inc.id, "failure", Some("documents_incomplete")).await;
    assert_eq!(refused.len(), 1, "{refused:?}");
    let tenant_inc = inc.id.to_string();
    audit_support::assert_corporate_events_clean(
        &refused,
        &["e_10"],
        Some(&tenant_inc),
        &[inc.nib.as_str(), inc.npwp.as_str(), hr_inc.email.as_str()],
    );

    // Aktif: 4090112 (seperti endpoint dokumen lain).
    let (a, hr_a) = company_with_hr(&w.app, "active").await;
    seed_all_documents(&w.app, a.id).await;
    let r = resubmit(&w, &hr_a).await;
    assert_eq!((r.status, code(&r).as_str()), (409, "4090112"));
    assert_eq!(client_status(&w.app, a.id).await, ("active".to_string(), 1));

    // Nonaktif: ditolak middleware.
    let (i, hr_i) = company_with_hr(&w.app, "inactive").await;
    let r = resubmit(&w, &hr_i).await;
    assert_eq!((r.status, code(&r).as_str()), (401, "4010002"));
    assert_eq!(
        client_status(&w.app, i.id).await,
        ("inactive".to_string(), 1)
    );

    // Akun HR belum aktif (penyediaan) dan token realm lain: 401, tanpa efek.
    let (r_c, _hr) = rejected_complete(&w.app).await;
    let pending_hr = insert_hr(&w.app, r_c.id, "pending_provisioning").await;
    let r = resubmit(&w, &pending_hr).await;
    assert_eq!(r.status, 401, "{}", r.body);
    for header in [
        w.patient_header(&pending_hr.subject),
        w.verifier_header(&pending_hr.subject),
    ] {
        let r = common::request(w.addr, "POST", RESUBMIT, &[&header]).await;
        assert_eq!(r.status, 401, "{}", r.body);
    }
    let r = common::request(w.addr, "POST", RESUBMIT, &[]).await;
    assert_eq!(r.status, 401);
    assert_eq!(
        client_status(&w.app, r_c.id).await,
        ("rejected".to_string(), 1)
    );
}

// trace: US-1902 AC3, AC4 (negatif): pengajuan ulang ganda dan bersamaan = satu putaran baru, satu event
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn double_and_concurrent_resubmissions_create_exactly_one_new_round() {
    let w = plain_world().await;
    let (c, hr) = rejected_complete(&w.app).await;
    let header = w.hr_header(&hr);
    let mut tasks = Vec::new();
    for _ in 0..6 {
        let (addr, header) = (w.addr, header.clone());
        tasks.push(tokio::spawn(async move {
            common::request(addr, "POST", RESUBMIT, &[&header]).await
        }));
    }
    let mut ok = 0;
    for t in tasks {
        let r = t.await.unwrap();
        match r.status {
            200 => ok += 1,
            409 => assert_eq!(code(&r), "4090001", "{}", r.body),
            s => panic!("status tak terduga {s}: {}", r.body),
        }
    }
    assert_eq!(ok, 1, "tepat satu pengajuan ulang berhasil");
    assert_eq!(
        client_status(&w.app, c.id).await,
        ("pending_verification".to_string(), 2)
    );
    assert_eq!(
        resubmission_events(&w, c.id, "success", None).await.len(),
        1
    );
    // F1-T109: setiap permintaan yang kalah tercatat E-10 gagal `not_resubmittable` (pola US-1902 AC4).
    assert_eq!(
        resubmission_events(&w, c.id, "failure", Some("not_resubmittable"))
            .await
            .len(),
        5
    );
    // Klik kedua setelah berhasil: 4090001 tanpa efek.
    let r = resubmit(&w, &hr).await;
    assert_eq!((r.status, code(&r).as_str()), (409, "4090001"));
    assert_eq!(client_status(&w.app, c.id).await.1, 2);
    assert_eq!(
        resubmission_events(&w, c.id, "success", None).await.len(),
        1
    );
    assert_eq!(
        resubmission_events(&w, c.id, "failure", Some("not_resubmittable"))
            .await
            .len(),
        6
    );
}

// trace: US-1904 AC3 (isolasi tenant), H-18: pengajuan ulang HR A tidak menyentuh perusahaan B; ID B yang
// diselipkan di query/isi tidak dipakai
#[tokio::test(flavor = "multi_thread")]
async fn resubmission_only_ever_touches_the_callers_company() {
    let w = plain_world().await;
    let (a, hr_a) = rejected_complete(&w.app).await;
    let (b, hr_b) = rejected_complete(&w.app).await;
    let r = common::request_json(
        w.addr,
        "POST",
        &format!("{RESUBMIT}?corporate_client_id={}", b.id),
        &[&w.hr_header(&hr_a)],
        &json!({ "corporate_client_id": b.id.to_string() }),
    )
    .await;
    assert_eq!(r.status, 200, "{}", r.body);
    for m in [b.id.to_string(), b.nib.clone(), b.npwp.clone()] {
        assert!(
            !r.body.contains(&m),
            "respons memuat data perusahaan B: {m}"
        );
    }
    assert_eq!(r.json()["data"]["company"]["nib"], a.nib.as_str());
    assert_eq!(
        client_status(&w.app, a.id).await,
        ("pending_verification".to_string(), 2)
    );
    assert_eq!(
        client_status(&w.app, b.id).await,
        ("rejected".to_string(), 1),
        "perusahaan B tidak berubah"
    );
    assert!(w.tenant_events("e_10", ACTION, b.id).await.is_empty());
    // Pembanding positif: HR B tetap dapat mengajukan ulang perusahaannya sendiri.
    let r = resubmit(&w, &hr_b).await;
    assert_eq!(r.status, 200, "{}", r.body);
    assert_eq!(
        client_status(&w.app, b.id).await,
        ("pending_verification".to_string(), 2)
    );
}

// trace: F1-T109 review keamanan butir 2 (KEPUTUSAN-389), NFR-SEC (batas laju per pengguna, 4290001): percobaan
// pengajuan ulang berulang dibatasi per akun HR (10 per menit) sebelum menyentuh database atau audit; permintaan
// yang dibatasi tidak menulis event, dan akun HR lain di perusahaan yang sama tidak ikut terbatasi.
#[tokio::test(flavor = "multi_thread")]
async fn repeated_resubmissions_are_rate_limited_per_hr_account() {
    let w = plain_world().await;
    let (c, hr) = rejected_complete(&w.app).await;
    let colleague = insert_hr(&w.app, c.id, "active").await;
    let first = resubmit(&w, &hr).await;
    assert_eq!(first.status, 200, "{}", first.body);
    for _ in 0..9 {
        let r = resubmit(&w, &hr).await;
        assert_eq!(
            (r.status, code(&r).as_str()),
            (409, "4090001"),
            "{}",
            r.body
        );
    }
    let limited = resubmit(&w, &hr).await;
    assert_eq!(
        (limited.status, code(&limited).as_str()),
        (429, "4290001"),
        "{}",
        limited.body
    );
    assert_eq!(
        resubmission_events(&w, c.id, "failure", Some("not_resubmittable"))
            .await
            .len(),
        9,
        "permintaan yang dibatasi tidak menulis audit"
    );
    assert_eq!(client_status(&w.app, c.id).await.1, 2);
    let r = resubmit(&w, &colleague).await;
    assert_eq!(
        (r.status, code(&r).as_str()),
        (409, "4090001"),
        "{}",
        r.body
    );
}
