//! CR-036 (T1; keputusan user Q20 = A 2026-11-27; KEPUTUSAN-410 s.d. KEPUTUSAN-414): HR mengoreksi data
//! perusahaan saat pengajuan Ditolak sekaligus mengajukan ulang (`POST /v1/corporate/company-corrections`) lewat
//! proses `api` lengkap terhadap PostgreSQL 18 sekali pakai (`scripts/db-ephemeral.sh`, role runtime `rs_api`).
//! Tanpa variabel `RS_TEST_DB_*` test GAGAL (tidak dilewati). Data sintetis saja.
//!
//! Skenario negatif (coding-standards §7.3, modul T1): status selain Ditolak, dokumen belum lengkap, validasi per
//! field, isolasi dua perusahaan (ID perusahaan lain diselipkan), balapan dua koreksi dan koreksi vs pengajuan
//! ulang biasa, NIB perusahaan Aktif lain diproses sama dengan NIB baru, audit tanpa nilai data perusahaan lewat
//! pemeriksa bersama, batas laju bersama pengajuan ulang.
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
    Company, DOCUMENTS, Hr, QUEUE, World, client_status, code, digits, insert_client, insert_hr,
    insert_verifier, seed_all_documents, seed_document, world_with,
};
use foundation::{Uuid, new_id};
use serde_json::{Value, json};
use sqlx::PgPool;
use std::sync::Arc;

const CORRECT: &str = "/v1/corporate/company-corrections";
const RESUBMIT: &str = "/v1/corporate/verification-resubmissions";
const CORRECTED: &str = "corporate_client_data_corrected";
const RESUBMITTED: &str = "corporate_verification_resubmitted";

async fn plain_world() -> World {
    world_with(Arc::new(common::unused_file_storage())).await
}

async fn company_with_hr(pool: &PgPool, status: &str) -> (Company, Hr) {
    let c = insert_client(pool, status, 30).await;
    let hr = insert_hr(pool, c.id, "active").await;
    (c, hr)
}

async fn seed_rejection(pool: &PgPool, client: Uuid, round: i32, by: Uuid) {
    sqlx::query!(
        "INSERT INTO app.corporate_verification_decision \
           (id, created_by, corporate_client_id, submission_round, decision, reason_code, decided_by) \
         VALUES ($1, $2, $3, $4, 'rejected', 'company_data_mismatch', $2)",
        new_id(),
        by,
        client,
        round,
    )
    .execute(pool)
    .await
    .unwrap();
}

/// Perusahaan Ditolak putaran 1 (alasan data tidak sesuai) dengan ketiga dokumen dan HR aktif.
async fn rejected_complete(pool: &PgPool) -> (Company, Hr) {
    let (c, hr) = company_with_hr(pool, "rejected").await;
    seed_all_documents(pool, c.id).await;
    seed_rejection(pool, c.id, 1, hr.account_id).await;
    (c, hr)
}

/// Data perusahaan tersimpan: (nama, NIB, NPWP, alamat, telepon).
async fn stored(pool: &PgPool, id: Uuid) -> (String, String, String, String, String) {
    let r = sqlx::query!(
        "SELECT legal_name, nib, npwp, office_address, office_phone FROM app.corporate_client WHERE id = $1",
        id
    )
    .fetch_one(pool)
    .await
    .unwrap();
    (
        r.legal_name,
        r.nib,
        r.npwp,
        r.office_address,
        r.office_phone,
    )
}

async fn correct(w: &World, hr: &Hr, body: &Value) -> common::Resp {
    w.hr_post(hr, CORRECT, body).await
}

async fn events(w: &World, action: &str, client: Uuid, outcome: &str) -> Vec<Value> {
    w.tenant_events("e_10", action, client)
        .await
        .into_iter()
        .filter(|e| e["outcome"] == outcome)
        .collect()
}

fn changed(e: &Value) -> Vec<String> {
    e["changed_fields"]
        .as_array()
        .map(|a| a.iter().map(|v| v.as_str().unwrap().to_string()).collect())
        .unwrap_or_default()
}

/// Isian koreksi lengkap dengan nilai sintetis baru.
fn full_correction(tag: &str) -> (Value, String, String) {
    let (nib, npwp) = (digits(13), digits(16));
    (
        json!({
            "legal_name": format!("PT Koreksi {tag}"),
            "nib": nib,
            "npwp": npwp,
            "office_address": format!("Jl. Koreksi {tag}"),
            "office_phone": "0219999999",
        }),
        nib,
        npwp,
    )
}

// trace: CR-036 (Q20 = A: HR mengoreksi data saat Ditolak lalu mengajukan ulang; Verifikator melihat data terbaru),
// US-1902 AC3, ADR-005 (E-10 nama field saja; NPWP Terbatas DD A20 tidak pernah di payload)
#[tokio::test(flavor = "multi_thread")]
async fn rejected_company_corrects_its_data_and_resubmits_in_one_audited_action() {
    let w = plain_world().await;
    let verifier = insert_verifier(&w.app).await;
    let (c, hr) = rejected_complete(&w.app).await;
    let old = stored(&w.app, c.id).await;
    let (body, nib, npwp) = full_correction("A1");

    let r = correct(&w, &hr, &body).await;
    assert_eq!(r.status, 200, "{}", r.body);
    assert_eq!(r.header("cache-control"), Some("no-store"));
    let d = &r.json()["data"];
    assert_eq!(d["status"], "pending_verification");
    assert_eq!(d["submission_round"], 2);
    assert_eq!(d["rejection_reason"], Value::Null);
    assert_eq!(d["company"]["legal_name"], "PT Koreksi A1");
    assert_eq!(d["company"]["nib"], nib.as_str());
    assert_eq!(d["company"]["npwp"], npwp.as_str());
    assert_eq!(d["company"]["office_address"], "Jl. Koreksi A1");
    assert_eq!(d["company"]["office_phone"], "0219999999");
    assert_eq!(
        stored(&w.app, c.id).await,
        (
            "PT Koreksi A1".to_string(),
            nib.clone(),
            npwp.clone(),
            "Jl. Koreksi A1".to_string(),
            "0219999999".to_string()
        )
    );
    assert_eq!(
        client_status(&w.app, c.id).await,
        ("pending_verification".to_string(), 2)
    );

    // Verifikator melihat data terbaru di antrean dan detail (baris terkini, bukan salinan putaran lama).
    assert!(
        w.queue_ids(&verifier, None)
            .await
            .contains(&c.id.to_string())
    );
    let r = w
        .verifier_get(&verifier, &format!("{QUEUE}/{}", c.id))
        .await;
    assert_eq!(r.status, 200, "{}", r.body);
    assert!(r.body.contains("PT Koreksi A1") && r.body.contains(&nib) && r.body.contains(&npwp));
    assert!(!r.body.contains(&old.1) && !r.body.contains(&old.2));

    // Audit: E-10 koreksi (nama field berubah saja) + E-10 pengajuan ulang, aktor HR, tenant = perusahaan.
    let corrected = events(&w, CORRECTED, c.id, "success").await;
    assert_eq!(corrected.len(), 1, "{corrected:?}");
    assert_eq!(corrected[0]["actor_account_id"], hr.account_id.to_string());
    assert_eq!(corrected[0]["object_type"], "corporate_client");
    assert_eq!(corrected[0]["object_id"], c.id.to_string());
    assert_eq!(
        changed(&corrected[0]),
        [
            "legal_name",
            "nib",
            "npwp",
            "office_address",
            "office_phone"
        ]
    );
    assert_eq!(events(&w, RESUBMITTED, c.id, "success").await.len(), 1);
    let tenant = c.id.to_string();
    audit_support::assert_corporate_events_clean(
        &w.all_tenant_events(c.id).await,
        &["e_10", "e_05"],
        Some(&tenant),
        &[
            old.0.as_str(),
            old.1.as_str(),
            old.2.as_str(),
            old.3.as_str(),
            nib.as_str(),
            npwp.as_str(),
            "PT Koreksi",
            "Jl. Koreksi",
            "0219999999",
            "0210000001",
            hr.email.as_str(),
        ],
    );
}

// trace: CR-036, KEPUTUSAN-411 (field tidak dikirim tidak berubah; nilai sama tidak dihitung koreksi)
#[tokio::test(flavor = "multi_thread")]
async fn only_sent_fields_change_and_unchanged_values_are_not_audited_as_corrections() {
    let w = plain_world().await;
    let (c, hr) = rejected_complete(&w.app).await;
    let old = stored(&w.app, c.id).await;
    let r = correct(
        &w,
        &hr,
        &json!({ "office_phone": "0218888888", "nib": old.1, "legal_name": null }),
    )
    .await;
    assert_eq!(r.status, 200, "{}", r.body);
    let now = stored(&w.app, c.id).await;
    assert_eq!(now.4, "0218888888");
    assert_eq!(
        (&now.0, &now.1, &now.2, &now.3),
        (&old.0, &old.1, &old.2, &old.3)
    );
    let corrected = events(&w, CORRECTED, c.id, "success").await;
    assert_eq!(corrected.len(), 1);
    assert_eq!(changed(&corrected[0]), ["office_phone"]);

    // Ditolak lagi, lalu "koreksi" dengan nilai yang sama persis: diajukan ulang tanpa event koreksi baru.
    let verifier = insert_verifier(&w.app).await;
    let r = w
        .decide(&verifier, c.id, Some("company_data_mismatch"))
        .await;
    assert_eq!(r.status, 200, "{}", r.body);
    let r = correct(&w, &hr, &json!({ "office_phone": "0218888888" })).await;
    assert_eq!(r.status, 200, "{}", r.body);
    assert_eq!(r.json()["data"]["submission_round"], 3);
    assert_eq!(events(&w, CORRECTED, c.id, "success").await.len(), 1);
    assert_eq!(events(&w, RESUBMITTED, c.id, "success").await.len(), 2);
}

// trace: CR-036 (negatif: hanya saat Ditolak dan dokumen lengkap; tanpa efek, penolakan aturan tercatat E-10 gagal)
#[tokio::test(flavor = "multi_thread")]
async fn correction_is_refused_outside_rejected_or_without_complete_documents() {
    let w = plain_world().await;
    let (body, nib, npwp) = full_correction("N1");

    // Menunggu verifikasi: 4090001, data tetap, E-10 gagal untuk koreksi dan pengajuan ulang.
    let (p, hr_p) = company_with_hr(&w.app, "pending_verification").await;
    seed_all_documents(&w.app, p.id).await;
    let before = stored(&w.app, p.id).await;
    let r = correct(&w, &hr_p, &body).await;
    assert_eq!(
        (r.status, code(&r).as_str()),
        (409, "4090001"),
        "{}",
        r.body
    );
    assert_eq!(stored(&w.app, p.id).await, before);
    assert_eq!(
        client_status(&w.app, p.id).await,
        ("pending_verification".to_string(), 1)
    );
    assert!(events(&w, CORRECTED, p.id, "success").await.is_empty());
    for action in [CORRECTED, RESUBMITTED] {
        let f = events(&w, action, p.id, "failure").await;
        assert_eq!(f.len(), 1, "{action}: {f:?}");
        assert_eq!(f[0]["outcome_reason_code"], "not_resubmittable");
        assert!(changed(&f[0]).is_empty());
    }
    let tenant_p = p.id.to_string();
    audit_support::assert_corporate_events_clean(
        &w.all_tenant_events(p.id).await,
        &["e_10"],
        Some(&tenant_p),
        &[
            nib.as_str(),
            npwp.as_str(),
            before.1.as_str(),
            before.2.as_str(),
            "PT Koreksi",
        ],
    );

    // Ditolak tanpa dokumen lengkap: 4090115, data tetap.
    let (inc, hr_inc) = company_with_hr(&w.app, "rejected").await;
    seed_rejection(&w.app, inc.id, 1, hr_inc.account_id).await;
    seed_document(&w.app, inc.id, "nib").await;
    let before = stored(&w.app, inc.id).await;
    let r = correct(&w, &hr_inc, &body).await;
    assert_eq!(
        (r.status, code(&r).as_str()),
        (409, "4090115"),
        "{}",
        r.body
    );
    assert_eq!(stored(&w.app, inc.id).await, before);
    assert_eq!(
        client_status(&w.app, inc.id).await,
        ("rejected".to_string(), 1)
    );
    for action in [CORRECTED, RESUBMITTED] {
        let f = events(&w, action, inc.id, "failure").await;
        assert_eq!(f.len(), 1, "{action}: {f:?}");
        assert_eq!(f[0]["outcome_reason_code"], "documents_incomplete");
    }

    // Aktif: 4090112 (gerbang sesi, tanpa event aksi), data terverifikasi tidak berubah.
    let (a, hr_a) = company_with_hr(&w.app, "active").await;
    let before = stored(&w.app, a.id).await;
    let r = correct(&w, &hr_a, &body).await;
    assert_eq!((r.status, code(&r).as_str()), (409, "4090112"));
    assert_eq!(stored(&w.app, a.id).await, before);
    assert!(w.tenant_events("e_10", CORRECTED, a.id).await.is_empty());

    // Nonaktif, akun HR belum aktif, token realm lain, tanpa token: 401 tanpa efek.
    let (i, hr_i) = company_with_hr(&w.app, "inactive").await;
    let r = correct(&w, &hr_i, &body).await;
    assert_eq!((r.status, code(&r).as_str()), (401, "4010002"));
    let (rc, _hr) = rejected_complete(&w.app).await;
    let before = stored(&w.app, rc.id).await;
    let pending_hr = insert_hr(&w.app, rc.id, "pending_provisioning").await;
    assert_eq!(correct(&w, &pending_hr, &body).await.status, 401);
    for header in [
        w.patient_header(&pending_hr.subject),
        w.verifier_header(&pending_hr.subject),
    ] {
        let r = common::request_json(w.addr, "POST", CORRECT, &[&header], &body).await;
        assert_eq!(r.status, 401, "{}", r.body);
    }
    let r = common::request_json(w.addr, "POST", CORRECT, &[], &body).await;
    assert_eq!(r.status, 401);
    assert_eq!(stored(&w.app, rc.id).await, before);
    assert!(stored(&w.app, i.id).await.0.starts_with("PT Dokumen Uji"));
}

// trace: CR-036 (validasi sama dengan pendaftaran KEPUTUSAN-325: galat per field, tanpa efek, tanpa event)
#[tokio::test(flavor = "multi_thread")]
async fn invalid_corrections_are_reported_per_field_without_effect() {
    let w = plain_world().await;
    let (c, hr) = rejected_complete(&w.app).await;
    let before = stored(&w.app, c.id).await;
    let fields_of = |r: &common::Resp| -> Vec<String> {
        r.json()["errors"]
            .as_array()
            .unwrap()
            .iter()
            .map(|e| e["field"].as_str().unwrap().to_string())
            .collect()
    };

    let r = correct(
        &w,
        &hr,
        &json!({
            "legal_name": "  ",
            "nib": "12345678901",
            "npwp": "990000000000001",
            "office_address": "x".repeat(501),
            "office_phone": "0".repeat(33),
        }),
    )
    .await;
    assert_eq!(
        (r.status, code(&r).as_str()),
        (422, "4220001"),
        "{}",
        r.body
    );
    let mut f = fields_of(&r);
    f.sort();
    assert_eq!(
        f,
        [
            "legal_name",
            "nib",
            "npwp",
            "office_address",
            "office_phone"
        ]
    );
    assert!(r.body.contains("NIB harus tepat 13 angka"));
    assert!(r.body.contains("tambahkan angka 0 di depan"));
    // Nilai yang dikirim tidak dipantulkan di galat.
    assert!(!r.body.contains("990000000000001"));

    // Isian kosong: bukan koreksi.
    let r = correct(&w, &hr, &json!({})).await;
    assert_eq!(
        (r.status, code(&r).as_str()),
        (422, "4220001"),
        "{}",
        r.body
    );
    assert_eq!(fields_of(&r), ["$"]);
    // Field tidak dikenal (mis. status, ID perusahaan) ditolak.
    for extra in [
        json!({ "office_phone": "0211111111", "status": "active" }),
        json!({ "office_phone": "0211111111", "corporate_client_id": new_id().to_string() }),
        json!({ "office_phone": 123 }),
    ] {
        let r = correct(&w, &hr, &extra).await;
        assert_eq!(
            (r.status, code(&r).as_str()),
            (422, "4220001"),
            "{}",
            r.body
        );
    }
    // Bukan JSON: 415.
    let r = common::request_with_body(
        w.addr,
        "POST",
        CORRECT,
        &[&w.hr_header(&hr), "Content-Type: text/plain"],
        Some(b"office_phone=1"),
    )
    .await;
    assert_eq!(r.status, 415, "{}", r.body);

    assert_eq!(stored(&w.app, c.id).await, before);
    assert_eq!(
        client_status(&w.app, c.id).await,
        ("rejected".to_string(), 1)
    );
    assert!(w.tenant_events("e_10", CORRECTED, c.id).await.is_empty());
    assert!(w.tenant_events("e_10", RESUBMITTED, c.id).await.is_empty());
}

// trace: US-1904 AC3 (isolasi tenant), H-18, CR-036: koreksi HR A tidak pernah menyentuh perusahaan B, termasuk
// bila ID B diselipkan di query
#[tokio::test(flavor = "multi_thread")]
async fn correction_only_ever_touches_the_callers_company() {
    let w = plain_world().await;
    let (a, hr_a) = rejected_complete(&w.app).await;
    let (b, hr_b) = rejected_complete(&w.app).await;
    let b_before = stored(&w.app, b.id).await;
    let (body, nib, _) = full_correction("ISO");
    let r = common::request_json(
        w.addr,
        "POST",
        &format!("{CORRECT}?corporate_client_id={}", b.id),
        &[&w.hr_header(&hr_a)],
        &body,
    )
    .await;
    assert_eq!(r.status, 200, "{}", r.body);
    for m in [
        b.id.to_string(),
        b.nib.clone(),
        b.npwp.clone(),
        b_before.0.clone(),
    ] {
        assert!(
            !r.body.contains(&m),
            "respons memuat data perusahaan B: {m}"
        );
    }
    assert_eq!(stored(&w.app, a.id).await.1, nib);
    assert_eq!(
        stored(&w.app, b.id).await,
        b_before,
        "perusahaan B tidak berubah"
    );
    assert_eq!(
        client_status(&w.app, b.id).await,
        ("rejected".to_string(), 1)
    );
    assert!(w.tenant_events("e_10", CORRECTED, b.id).await.is_empty());
    // Pembanding positif: HR B mengoreksi perusahaannya sendiri.
    let r = correct(&w, &hr_b, &json!({ "office_phone": "0217777777" })).await;
    assert_eq!(r.status, 200, "{}", r.body);
    assert_eq!(stored(&w.app, b.id).await.4, "0217777777");
    assert_eq!(stored(&w.app, a.id).await.4, "0219999999");
}

// trace: CR-036, US-1902 AC4 (negatif balapan): dua koreksi bersamaan dan koreksi vs pengajuan ulang biasa = tepat
// satu yang berlaku (kunci baris + UPDATE bersyarat status/putaran); data tersimpan = milik pemenang
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_corrections_and_resubmissions_apply_exactly_once() {
    let w = plain_world().await;
    let (c, hr) = rejected_complete(&w.app).await;
    let header = w.hr_header(&hr);
    let mut tasks = Vec::new();
    for i in 0..4 {
        let (addr, header) = (w.addr, header.clone());
        let (body, nib, _) = full_correction(&format!("R{i}"));
        tasks.push(tokio::spawn(async move {
            (
                Some(nib),
                common::request_json(addr, "POST", CORRECT, &[&header], &body).await,
            )
        }));
    }
    for _ in 0..2 {
        let (addr, header) = (w.addr, header.clone());
        tasks.push(tokio::spawn(async move {
            (
                None,
                common::request(addr, "POST", RESUBMIT, &[&header]).await,
            )
        }));
    }
    let mut winners = Vec::new();
    for t in tasks {
        let (nib, r) = t.await.unwrap();
        match r.status {
            200 => winners.push(nib),
            409 => assert_eq!(code(&r), "4090001", "{}", r.body),
            s => panic!("status tak terduga {s}: {}", r.body),
        }
    }
    assert_eq!(winners.len(), 1, "tepat satu permintaan berhasil");
    assert_eq!(
        client_status(&w.app, c.id).await,
        ("pending_verification".to_string(), 2)
    );
    let now = stored(&w.app, c.id).await;
    match &winners[0] {
        Some(nib) => {
            assert_eq!(&now.1, nib, "data tersimpan milik koreksi pemenang");
            assert_eq!(events(&w, CORRECTED, c.id, "success").await.len(), 1);
        }
        None => {
            assert_eq!(
                now.1, c.nib,
                "pengajuan ulang biasa menang: data tidak berubah"
            );
            assert!(events(&w, CORRECTED, c.id, "success").await.is_empty());
        }
    }
    assert_eq!(events(&w, RESUBMITTED, c.id, "success").await.len(), 1);
    // Setiap koreksi yang kalah tercatat gagal `not_resubmittable` (4 koreksi dikurangi pemenang bila koreksi).
    let lost_corrections = if winners[0].is_some() { 3 } else { 4 };
    assert_eq!(
        events(&w, CORRECTED, c.id, "failure").await.len(),
        lost_corrections
    );
    assert_eq!(events(&w, RESUBMITTED, c.id, "failure").await.len(), 5);
}

// trace: CR-035/KEPUTUSAN-365 di jalur koreksi (Q10 B2): NIB yang dipakai perusahaan Aktif lain diproses sama
// persis dengan NIB baru; penanda Verifikator dihitung saat dibaca dan persetujuan tetap ditolak 4090116
#[tokio::test(flavor = "multi_thread")]
async fn correcting_to_a_nib_used_by_an_active_company_behaves_like_a_fresh_nib() {
    let w = plain_world().await;
    let verifier = insert_verifier(&w.app).await;
    let (active, _hr) = company_with_hr(&w.app, "active").await;
    let (taken, taken_hr) = rejected_complete(&w.app).await;
    let (other, other_hr) = rejected_complete(&w.app).await;

    let r_taken = correct(&w, &taken_hr, &json!({ "nib": active.nib })).await;
    let r_fresh = correct(&w, &other_hr, &json!({ "nib": digits(13) })).await;
    assert_eq!(
        (r_taken.status, r_fresh.status),
        (200, 200),
        "{} {}",
        r_taken.body,
        r_fresh.body
    );
    // Bentuk jawaban identik selain nilai milik perusahaan masing-masing.
    let keys = |r: &common::Resp| {
        let mut k: Vec<String> = r.json()["data"]
            .as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect();
        k.sort();
        k
    };
    assert_eq!(keys(&r_taken), keys(&r_fresh));
    assert_eq!(
        r_taken.json()["response_code"],
        r_fresh.json()["response_code"]
    );
    assert_eq!(
        changed(&events(&w, CORRECTED, other.id, "success").await[0]),
        ["nib"]
    );

    let queue = w
        .verifier_get(&verifier, &format!("{QUEUE}?limit=100&readiness=all"))
        .await;
    assert_eq!(queue.status, 200, "{}", queue.body);
    let flag = |id: Uuid| {
        queue.json()["data"]
            .as_array()
            .unwrap()
            .iter()
            .find(|i| i["corporate_client_id"] == id.to_string())
            .map(|i| i["nib_used_by_other_company"].clone())
            .unwrap()
    };
    assert_eq!(flag(taken.id), json!(true));
    assert_eq!(flag(other.id), json!(false));
    // Keunikan di antara perusahaan Aktif tetap ditegakkan saat persetujuan.
    let r = w.decide(&verifier, taken.id, None).await;
    assert_eq!(
        (r.status, code(&r).as_str()),
        (409, "4090116"),
        "{}",
        r.body
    );
    assert_eq!(
        client_status(&w.app, taken.id).await,
        ("pending_verification".to_string(), 2)
    );
}

// trace: KEPUTUSAN-389, KEPUTUSAN-412: koreksi memakai batas laju pengajuan ulang per akun HR (10 per menit,
// dihitung bersama); permintaan yang dibatasi tidak menulis event dan tidak mengubah data
#[tokio::test(flavor = "multi_thread")]
async fn corrections_share_the_resubmission_rate_limit_per_hr_account() {
    let w = plain_world().await;
    let (c, hr) = rejected_complete(&w.app).await;
    let colleague = insert_hr(&w.app, c.id, "active").await;
    let first = correct(&w, &hr, &json!({ "office_phone": "0216666666" })).await;
    assert_eq!(first.status, 200, "{}", first.body);
    for i in 0..9 {
        let r = if i % 2 == 0 {
            correct(&w, &hr, &json!({ "office_phone": "0215555555" })).await
        } else {
            common::request(w.addr, "POST", RESUBMIT, &[&w.hr_header(&hr)]).await
        };
        assert_eq!(
            (r.status, code(&r).as_str()),
            (409, "4090001"),
            "{}",
            r.body
        );
    }
    let limited = correct(&w, &hr, &json!({ "office_phone": "0214444444" })).await;
    assert_eq!(
        (limited.status, code(&limited).as_str()),
        (429, "4290001"),
        "{}",
        limited.body
    );
    // 5 koreksi gagal + 4 pengajuan ulang gagal sebelum batas; yang dibatasi tidak menulis apa pun.
    assert_eq!(events(&w, CORRECTED, c.id, "failure").await.len(), 5);
    assert_eq!(events(&w, RESUBMITTED, c.id, "failure").await.len(), 9);
    assert_eq!(stored(&w.app, c.id).await.4, "0216666666");
    let r = correct(&w, &colleague, &json!({ "office_phone": "0213333333" })).await;
    assert_eq!(
        (r.status, code(&r).as_str()),
        (409, "4090001"),
        "{}",
        r.body
    );
    // Halaman status HR menampilkan data hasil koreksi (US-1901 AC6).
    let r = w.hr_get(&hr, DOCUMENTS).await;
    assert_eq!(r.json()["data"]["company"]["office_phone"], "0216666666");
}
