//! CR-040 (T1; keputusan user Q34 = A 2026-11-27; KEPUTUSAN-442 s.d. KEPUTUSAN-445): Verifikator melihat penanda
//! "Data dikoreksi" beserta nilai sebelumnya pada setiap field yang dikoreksi HR saat pengajuan ulang setelah
//! Ditolak (CR-036). Proses `api` lengkap terhadap PostgreSQL 18 sekali pakai (`scripts/db-ephemeral.sh`, role
//! runtime `rs_api`). Tanpa variabel `RS_TEST_DB_*` test GAGAL (tidak dilewati). Data sintetis saja.
//!
//! Skenario negatif (coding-standards §7.3, modul T1): tanpa putaran sebelumnya (pengajuan pertama), tanpa field
//! yang berubah (pengajuan ulang dokumen saja dan koreksi dengan nilai sama), perbandingan hanya dengan putaran
//! sebelumnya (bukan putaran pertama), sub-peran admin selain Verifikator dan token HR ditolak tanpa nilai
//! sebelumnya di respons, NIB/NPWP sebelumnya termasking (nilai lengkap tidak tersimpan maupun terkirim), dan
//! catatan koreksi tidak dapat diubah atau dihapus role runtime (ADR-004).
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
    Company, Hr, QUEUE, World, digits, insert_client, insert_hr, insert_verifier,
    seed_all_documents, world_with,
};
use foundation::{Uuid, new_id};
use serde_json::{Value, json};
use sqlx::PgPool;
use std::sync::Arc;

const CORRECT: &str = "/v1/corporate/company-corrections";
const RESUBMIT: &str = "/v1/corporate/verification-resubmissions";
const MASK: &str = "************";

async fn plain_world() -> World {
    world_with(Arc::new(common::unused_file_storage())).await
}

/// Perusahaan Ditolak putaran 1 dengan ketiga dokumen dan HR aktif.
async fn rejected_complete(pool: &PgPool) -> (Company, Hr) {
    let c = insert_client(pool, "rejected", 30).await;
    let hr = insert_hr(pool, c.id, "active").await;
    seed_all_documents(pool, c.id).await;
    sqlx::query!(
        "INSERT INTO app.corporate_verification_decision \
           (id, created_by, corporate_client_id, submission_round, decision, reason_code, decided_by) \
         VALUES ($1, $2, $3, 1, 'rejected', 'company_data_mismatch', $2)",
        new_id(),
        hr.account_id,
        c.id,
    )
    .execute(pool)
    .await
    .unwrap();
    (c, hr)
}

async fn legal_name(pool: &PgPool, id: Uuid) -> String {
    sqlx::query_scalar!(
        "SELECT legal_name FROM app.corporate_client WHERE id = $1",
        id
    )
    .fetch_one(pool)
    .await
    .unwrap()
}

/// Detail Verifikator (200) sebagai JSON `data`.
async fn detail(w: &World, verifier: &str, id: Uuid) -> (Value, String) {
    let r = w.verifier_get(verifier, &format!("{QUEUE}/{id}")).await;
    assert_eq!(r.status, 200, "{}", r.body);
    (r.json()["data"].clone(), r.body)
}

/// `corrected_fields` sebagai pasangan (field, nilai sebelumnya).
fn marks(d: &Value) -> Vec<(String, String)> {
    d["corrected_fields"]
        .as_array()
        .expect("corrected_fields selalu ada (array)")
        .iter()
        .map(|f| {
            (
                f["field"].as_str().unwrap().to_string(),
                f["previous_value"].as_str().unwrap().to_string(),
            )
        })
        .collect()
}

async fn correction_rows(pool: &PgPool, id: Uuid) -> i64 {
    sqlx::query_scalar!(
        r#"SELECT count(*) AS "n!" FROM app.corporate_client_correction WHERE corporate_client_id = $1"#,
        id
    )
    .fetch_one(pool)
    .await
    .unwrap()
}

fn tail(v: &str) -> &str {
    &v[v.len() - 4..]
}

// trace: CR-040 (Q34 = A: penanda per field yang berubah + nilai sebelumnya; NIB/NPWP sebelumnya termasking seperti
// NIB antrean wireframe C13, nilai lengkap tidak disimpan; KEPUTUSAN-442/443)
#[tokio::test(flavor = "multi_thread")]
async fn verifier_sees_changed_fields_with_previous_values_and_masked_nib_npwp() {
    let w = plain_world().await;
    let verifier = insert_verifier(&w.app).await;
    let (c, hr) = rejected_complete(&w.app).await;
    let old_name = legal_name(&w.app, c.id).await;
    let (nib, npwp) = (digits(13), digits(16));
    let r = w
        .hr_post(
            &hr,
            CORRECT,
            &json!({
                "legal_name": "PT Koreksi Penanda",
                "nib": nib,
                "npwp": npwp,
                // dikirim dengan nilai sama: bukan koreksi, tanpa penanda
                "office_address": "Jl. Sintetis No. 1",
                "office_phone": "0219999999",
            }),
        )
        .await;
    assert_eq!(r.status, 200, "{}", r.body);

    let (d, body) = detail(&w, &verifier, c.id).await;
    assert_eq!(d["submission_round"], 2);
    assert_eq!(
        marks(&d),
        [
            ("legal_name".to_string(), old_name.clone()),
            ("nib".to_string(), format!("{MASK}{}", tail(&c.nib))),
            ("npwp".to_string(), format!("{MASK}{}", tail(&c.npwp))),
            ("office_phone".to_string(), "0210000001".to_string()),
        ]
    );
    // Nilai terbaru tetap lengkap untuk Verifikator; NIB/NPWP lama tidak pernah lengkap di respons.
    assert_eq!(d["nib"], nib.as_str());
    assert_eq!(d["npwp"], npwp.as_str());
    assert!(!body.contains(&c.nib) && !body.contains(&c.npwp), "{body}");

    // Tersimpan: satu baris putaran 2, NIB/NPWP hanya 4 karakter terakhir.
    let row = sqlx::query!(
        "SELECT submission_round, created_by, previous_legal_name, previous_nib_last4, previous_npwp_last4, \
                previous_office_address, previous_office_phone \
         FROM app.corporate_client_correction WHERE corporate_client_id = $1",
        c.id
    )
    .fetch_one(&w.app)
    .await
    .unwrap();
    assert_eq!(row.submission_round, 2);
    assert_eq!(row.created_by, Some(hr.account_id));
    assert_eq!(row.previous_legal_name.as_deref(), Some(old_name.as_str()));
    assert_eq!(row.previous_nib_last4.as_deref(), Some(tail(&c.nib)));
    assert_eq!(row.previous_npwp_last4.as_deref(), Some(tail(&c.npwp)));
    assert_eq!(row.previous_office_address, None);
    assert_eq!(row.previous_office_phone.as_deref(), Some("0210000001"));

    // Audit tetap tanpa nilai lengkap (pemeriksa bersama CR-036); pembukaan detail tercatat E-05 tanpa field.
    let all = w.all_tenant_events(c.id).await;
    audit_support::assert_corporate_correction_events_clean(
        &all,
        &["e_10", "e_05"],
        &c.id.to_string(),
        &[c.nib.as_str(), c.npwp.as_str(), nib.as_str(), npwp.as_str()],
    );
}

// trace: CR-040 (negatif: tanpa putaran sebelumnya = tanpa penanda; pengajuan ulang dokumen saja dan koreksi dengan
// nilai sama = tanpa penanda dan tanpa catatan)
#[tokio::test(flavor = "multi_thread")]
async fn no_marker_without_previous_round_or_without_changed_fields() {
    let w = plain_world().await;
    let verifier = insert_verifier(&w.app).await;

    // Pengajuan pertama (putaran 1).
    let first = insert_client(&w.app, "pending_verification", 10).await;
    seed_all_documents(&w.app, first.id).await;
    let (d, _) = detail(&w, &verifier, first.id).await;
    assert_eq!(d["submission_round"], 1);
    assert_eq!(d["corrected_fields"], json!([]));

    // Pengajuan ulang tanpa koreksi (dokumen saja).
    let (c, hr) = rejected_complete(&w.app).await;
    let r = common::request(w.addr, "POST", RESUBMIT, &[&w.hr_header(&hr)]).await;
    assert_eq!(r.status, 200, "{}", r.body);
    let (d, _) = detail(&w, &verifier, c.id).await;
    assert_eq!(d["submission_round"], 2);
    assert_eq!(d["corrected_fields"], json!([]));
    assert_eq!(correction_rows(&w.app, c.id).await, 0);

    // Koreksi yang semua nilainya sama dengan tersimpan.
    let (c2, hr2) = rejected_complete(&w.app).await;
    let name = legal_name(&w.app, c2.id).await;
    let r = w
        .hr_post(
            &hr2,
            CORRECT,
            &json!({ "legal_name": name, "nib": c2.nib, "office_phone": " 0210000001 " }),
        )
        .await;
    assert_eq!(r.status, 200, "{}", r.body);
    let (d, _) = detail(&w, &verifier, c2.id).await;
    assert_eq!(d["submission_round"], 2);
    assert_eq!(d["corrected_fields"], json!([]));
    assert_eq!(correction_rows(&w.app, c2.id).await, 0);
}

// trace: CR-040 (dibandingkan dengan putaran sebelumnya saja: putaran 3 menampilkan nilai putaran 2, bukan putaran 1;
// putaran tanpa koreksi setelah putaran berkoreksi tidak mewarisi penanda)
#[tokio::test(flavor = "multi_thread")]
async fn marker_compares_with_the_previous_round_only() {
    let w = plain_world().await;
    let verifier = insert_verifier(&w.app).await;
    let (c, hr) = rejected_complete(&w.app).await;

    let r = w
        .hr_post(
            &hr,
            CORRECT,
            &json!({ "office_address": "Jl. Putaran Dua" }),
        )
        .await;
    assert_eq!(r.status, 200, "{}", r.body);
    let (d, _) = detail(&w, &verifier, c.id).await;
    assert_eq!(
        marks(&d),
        [(
            "office_address".to_string(),
            "Jl. Sintetis No. 1".to_string()
        )]
    );

    // Ditolak lagi, koreksi alamat lagi: nilai sebelumnya = putaran 2.
    let r = w
        .decide(&verifier, c.id, Some("company_data_mismatch"))
        .await;
    assert_eq!(r.status, 200, "{}", r.body);
    let r = w
        .hr_post(
            &hr,
            CORRECT,
            &json!({ "office_address": "Jl. Putaran Tiga" }),
        )
        .await;
    assert_eq!(r.status, 200, "{}", r.body);
    let (d, _) = detail(&w, &verifier, c.id).await;
    assert_eq!(d["submission_round"], 3);
    assert_eq!(
        marks(&d),
        [("office_address".to_string(), "Jl. Putaran Dua".to_string())]
    );

    // Ditolak lagi, ajukan ulang tanpa koreksi: putaran 4 tanpa penanda.
    let r = w
        .decide(&verifier, c.id, Some("company_data_mismatch"))
        .await;
    assert_eq!(r.status, 200, "{}", r.body);
    let r = common::request(w.addr, "POST", RESUBMIT, &[&w.hr_header(&hr)]).await;
    assert_eq!(r.status, 200, "{}", r.body);
    let (d, _) = detail(&w, &verifier, c.id).await;
    assert_eq!(d["submission_round"], 4);
    assert_eq!(d["corrected_fields"], json!([]));
    assert_eq!(correction_rows(&w.app, c.id).await, 2);
}

/// Akun admin aktif dengan satu sub-peran `sub_role`; mengembalikan `idp_subject`.
async fn insert_admin(pool: &PgPool, sub_role: &str) -> String {
    let id = new_id();
    let subject = format!("admin-penanda-{}", files::random_hex(8));
    sqlx::query!(
        "INSERT INTO app.account (id, account_type, idp_realm, idp_subject, email, status, status_changed_at, full_name) \
         VALUES ($1, 'rs_admin', 'admin', $2, $3, 'active', now(), 'Admin Sintetis')",
        id,
        subject,
        format!("{subject}@example.com"),
    )
    .execute(pool)
    .await
    .unwrap();
    sqlx::query!(
        "INSERT INTO app.rs_admin_role (id, account_id, sub_role) VALUES ($1, $2, $3)",
        new_id(),
        id,
        sub_role,
    )
    .execute(pool)
    .await
    .unwrap();
    subject
}

// trace: CR-040 (otorisasi tetap Verifikator saja, sama dengan detail US-1902 AC5): sub-peran lain 403, HR 401,
// tanpa nilai sebelumnya di respons
#[tokio::test(flavor = "multi_thread")]
async fn non_verifiers_cannot_read_previous_values() {
    let w = plain_world().await;
    let (c, hr) = rejected_complete(&w.app).await;
    let old_name = legal_name(&w.app, c.id).await;
    let r = w
        .hr_post(&hr, CORRECT, &json!({ "legal_name": "PT Koreksi Rahasia" }))
        .await;
    assert_eq!(r.status, 200, "{}", r.body);
    let path = format!("{QUEUE}/{}", c.id);

    for role in [
        "super_admin",
        "customer_service",
        "finance",
        "content",
        "auditor",
        "privacy_officer",
    ] {
        let sub = insert_admin(&w.app, role).await;
        let r = w.verifier_get(&sub, &path).await;
        assert_eq!(r.status, 403, "{role}: {}", r.body);
        assert_eq!(corporate_documents_support::code(&r), "4030001", "{role}");
        assert!(!r.body.contains(&old_name) && !r.body.contains("corrected_fields"));
    }
    let r = common::request(w.addr, "GET", &path, &[&w.hr_header(&hr)]).await;
    assert_eq!(r.status, 401, "{}", r.body);
    assert!(!r.body.contains(&old_name));
}

// trace: CR-040, ADR-004 (catatan koreksi = Transaksi: role runtime tidak dapat mengubah atau menghapus)
#[tokio::test(flavor = "multi_thread")]
async fn correction_record_is_immutable_for_the_runtime_role() {
    let w = plain_world().await;
    let (c, hr) = rejected_complete(&w.app).await;
    let r = w
        .hr_post(&hr, CORRECT, &json!({ "office_phone": "0217777777" }))
        .await;
    assert_eq!(r.status, 200, "{}", r.body);
    let upd = sqlx::query!(
        "UPDATE app.corporate_client_correction SET previous_office_phone = '0' WHERE corporate_client_id = $1",
        c.id
    )
    .execute(&w.app)
    .await;
    assert!(upd.is_err(), "UPDATE harus ditolak");
    let del = sqlx::query!(
        "DELETE FROM app.corporate_client_correction WHERE corporate_client_id = $1",
        c.id
    )
    .execute(&w.app)
    .await;
    assert!(del.is_err(), "DELETE harus ditolak");
    assert_eq!(correction_rows(&w.app, c.id).await, 1);
}
