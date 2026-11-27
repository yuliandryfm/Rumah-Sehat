//! F1-T102 (T1; US-1901 AC2, AC6; US-1902 AC1; ADR-018, ADR-028 butir 1; keputusan user Q8 dan Q9): akses dan
//! isolasi dokumen perusahaan oleh HR serta antrean Verifikator yang memfilter pengajuan tanpa dokumen lengkap,
//! lewat proses `api` lengkap terhadap PostgreSQL 18 sekali pakai (`scripts/db-ephemeral.sh`, role runtime
//! `rs_api`). Tanpa object storage nyata: slot unggah dan URL unduh ditandatangani lokal; unggahan, validasi isi,
//! dan penggantian ada di `corporate_documents_http.rs`. Tanpa variabel `RS_TEST_DB_*` test GAGAL.
//!
//! Skenario negatif (coding-standards §7.3): HR belum aktif, status perusahaan salah (Aktif, Nonaktif), token
//! realm lain dan tanpa token, lintas tenant (slot dan dokumen perusahaan lain, isian ID perusahaan yang
//! diselipkan), kuota slot, jenis/ukuran di luar NFR-SEC-08, persetujuan tanpa dokumen lengkap.
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
    DOCUMENTS, DOWNLOADS, Hr, KINDS, SLOTS, StubInspector, World, active_kinds, client_status,
    code, insert_client, insert_hr, insert_verifier, seed_all_documents, seed_document, world_with,
};
use foundation::{Uuid, new_id};
use serde_json::{Value, json};
use sqlx::PgPool;
use std::sync::Arc;

async fn plain_world() -> World {
    world_with(Arc::new(common::unused_file_storage())).await
}

async fn world_with_pdf_support() -> World {
    world_with(Arc::new(
        common::unused_file_storage().with_pdf_inspector(Arc::new(StubInspector)),
    ))
    .await
}

/// Perusahaan berstatus `status` dengan satu HR aktif.
async fn company_with_hr(
    pool: &PgPool,
    status: &str,
) -> (corporate_documents_support::Company, Hr) {
    let c = insert_client(pool, status, 0).await;
    let hr = insert_hr(pool, c.id, "active").await;
    (c, hr)
}

/// Kode alasan tertutup untuk penolakan (daftar draf F1-T104).
async fn seed_rejection(pool: &PgPool, client: Uuid, decided_by: Uuid) {
    sqlx::query!(
        "INSERT INTO app.corporate_verification_decision \
           (id, created_by, corporate_client_id, submission_round, decision, reason_code, decided_by) \
         VALUES ($1, $2, $3, 1, 'rejected', 'authority_proof_insufficient', $2)",
        new_id(),
        decided_by,
        client,
    )
    .execute(pool)
    .await
    .unwrap();
}

async fn slot_row(pool: &PgPool, slot: &str) -> (Option<Uuid>, bool) {
    let id = Uuid::parse_str(slot).unwrap();
    let r = sqlx::query!(
        r#"SELECT owner_account_id, consumed_at IS NOT NULL AS "consumed!" FROM app.upload_slot WHERE id = $1"#,
        id
    )
    .fetch_one(pool)
    .await
    .unwrap();
    (r.owner_account_id, r.consumed)
}

/// Berkas tercatat milik akun `owner` (test lain berjalan bersamaan di database yang sama).
async fn stored_file_count(pool: &PgPool, owner: Uuid) -> i64 {
    sqlx::query_scalar!(
        r#"SELECT count(*) AS "n!" FROM app.stored_file WHERE owner_account_id = $1"#,
        owner
    )
    .fetch_one(pool)
    .await
    .unwrap()
}

/// Empat endpoint HR dengan isi sah; mengembalikan (nama, status, kode) per endpoint.
async fn probe_all(w: &World, hr: &Hr) -> Vec<(&'static str, u16, String)> {
    let slot = new_id().to_string();
    let rs = [
        ("list", w.hr_get(hr, DOCUMENTS).await),
        (
            "slot",
            w.hr_post(
                hr,
                SLOTS,
                &json!({"content_type": "image/png", "size_bytes": 1024}),
            )
            .await,
        ),
        ("confirm", w.confirm(hr, "nib", &slot).await),
        (
            "download",
            w.hr_post(hr, DOWNLOADS, &json!({"document_kind": "nib"}))
                .await,
        ),
    ];
    rs.into_iter()
        .map(|(n, r)| (n, r.status, code(&r)))
        .collect()
}

// trace: US-1901 AC6, Q8 (dokumen hanya saat menunggu verifikasi atau ditolak; HR melihat alasan penolakan)
#[tokio::test(flavor = "multi_thread")]
async fn documents_are_manageable_only_while_pending_or_rejected() {
    let w = plain_world().await;

    // Menunggu verifikasi: boleh, tanpa alasan penolakan, belum ada dokumen.
    let (_pending, hr) = company_with_hr(&w.app, "pending_verification").await;
    let r = w.hr_get(&hr, DOCUMENTS).await;
    assert_eq!(r.status, 200, "{}", r.body);
    assert_eq!(r.header("cache-control"), Some("no-store"));
    let d = &r.json()["data"];
    assert_eq!(d["status"], "pending_verification");
    assert_eq!(d["submission_round"], 1);
    assert_eq!(d["rejection_reason"], Value::Null);
    assert_eq!(d["complete"], false);
    let kinds: Vec<&str> = d["documents"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x["kind"].as_str().unwrap())
        .collect();
    assert_eq!(kinds, KINDS);
    assert!(
        d["documents"]
            .as_array()
            .unwrap()
            .iter()
            .all(|x| x["uploaded"] == false && x["uploaded_at"].is_null())
    );
    assert_eq!(
        w.slot(&hr, "image/png", 1024).await.status,
        201,
        "slot saat menunggu verifikasi"
    );

    // Ditolak: boleh; alasan penolakan putaran berjalan terlihat; dokumen lama tampil.
    let (rejected, hr_r) = company_with_hr(&w.app, "rejected").await;
    seed_rejection(&w.app, rejected.id, hr_r.account_id).await;
    seed_document(&w.app, rejected.id, "nib").await;
    let r = w.hr_get(&hr_r, DOCUMENTS).await;
    assert_eq!(r.status, 200, "{}", r.body);
    let d = &r.json()["data"];
    assert_eq!(d["status"], "rejected");
    assert_eq!(d["rejection_reason"], "authority_proof_insufficient");
    assert_eq!(d["documents"][0]["kind"], "nib");
    assert_eq!(d["documents"][0]["uploaded"], true);
    assert!(d["documents"][0]["uploaded_at"].is_string());
    assert_eq!(d["complete"], false);
    assert_eq!(w.slot(&hr_r, "image/jpeg", 2048).await.status, 201);

    // Aktif: semua endpoint dokumen dijawab 409 4090112 (dokumen terverifikasi tidak diubah).
    let (active, hr_a) = company_with_hr(&w.app, "active").await;
    seed_all_documents(&w.app, active.id).await;
    for (name, status, c) in probe_all(&w, &hr_a).await {
        assert_eq!((status, c.as_str()), (409, "4090112"), "{name}");
    }
    assert_eq!(
        active_kinds(&w.app, active.id).await.len(),
        3,
        "dokumen perusahaan Aktif tidak berubah"
    );

    // Nonaktif: ditolak middleware (akun/perusahaan nonaktif, 401 4010002), bukan 409.
    let (_inactive, hr_i) = company_with_hr(&w.app, "inactive").await;
    for (name, status, c) in probe_all(&w, &hr_i).await {
        assert_eq!((status, c.as_str()), (401, "4010002"), "{name}");
    }
}

// trace: US-1901 AC6 (akun HR belum diaktivasi tidak dapat memakai endpoint), Q8 (setelah aktivasi saja)
#[tokio::test(flavor = "multi_thread")]
async fn hr_without_activated_account_or_with_wrong_realm_is_refused() {
    let w = plain_world().await;
    let c = insert_client(&w.app, "pending_verification", 0).await;

    // HR belum aktif (undangan belum diaktivasi atau pengguna belum dibuat): token yang kebetulan ada ditolak.
    for status in ["pending_activation", "pending_provisioning", "inactive"] {
        let hr = insert_hr(&w.app, c.id, status).await;
        for (name, st, _) in probe_all(&w, &hr).await {
            assert_eq!(st, 401, "{status} {name}");
        }
    }

    // Token realm lain dan tanpa token.
    let hr = insert_hr(&w.app, c.id, "active").await;
    let verifier = insert_verifier(&w.app).await;
    let admin = w.verifier_header(&verifier);
    let patient = w.patient_header(&hr.subject);
    for header in [Some(admin), Some(patient), None] {
        let extra: Vec<&str> = header.iter().map(String::as_str).collect();
        for (method, path, body) in [
            ("GET", DOCUMENTS, None),
            (
                "POST",
                SLOTS,
                Some(json!({"content_type": "image/png", "size_bytes": 1})),
            ),
            (
                "POST",
                DOCUMENTS,
                Some(json!({"document_kind": "nib", "upload_slot_id": new_id().to_string()})),
            ),
            ("POST", DOWNLOADS, Some(json!({"document_kind": "nib"}))),
        ] {
            let r = match &body {
                Some(b) => common::request_json(w.addr, method, path, &extra, b).await,
                None => common::request(w.addr, method, path, &extra).await,
            };
            assert_eq!(r.status, 401, "{method} {path}");
        }
    }
    assert_eq!(stored_file_count(&w.app, hr.account_id).await, 0);
    assert!(active_kinds(&w.app, c.id).await.is_empty());
}

// trace: ADR-028 butir 1, US-1904 AC3, KU-03 (HR hanya menjangkau dokumen perusahaannya sendiri)
#[tokio::test(flavor = "multi_thread")]
async fn hr_reaches_only_its_own_companys_documents_slots_and_urls() {
    let w = plain_world().await;
    let (a, hr_a) = company_with_hr(&w.app, "pending_verification").await;
    let (b, hr_b) = company_with_hr(&w.app, "pending_verification").await;
    let a_file = seed_document(&w.app, a.id, "nib").await;

    // Status B tidak memuat dokumen A.
    let r = w.hr_get(&hr_b, DOCUMENTS).await;
    assert_eq!(r.status, 200);
    assert!(
        r.json()["data"]["documents"]
            .as_array()
            .unwrap()
            .iter()
            .all(|d| d["uploaded"] == false),
        "{}",
        r.body
    );
    let r = w.hr_get(&hr_a, DOCUMENTS).await;
    assert_eq!(r.json()["data"]["documents"][0]["uploaded"], true);

    // URL unduh: HR A mendapat URL berumur pendek untuk dokumennya; HR B 404 untuk jenis yang sama.
    let r = w
        .hr_post(&hr_a, DOWNLOADS, &json!({"document_kind": "nib"}))
        .await;
    assert_eq!(r.status, 200, "{}", r.body);
    assert_eq!(r.header("cache-control"), Some("no-store"));
    let d = &r.json()["data"];
    assert!(d["url"].as_str().unwrap().contains("X-Amz-Signature"));
    assert_eq!(d["mime_type"], "image/jpeg");
    let expires = time::OffsetDateTime::parse(
        d["expires_at"].as_str().unwrap(),
        &time::format_description::well_known::Rfc3339,
    )
    .unwrap();
    let ttl = (expires - files::now()).whole_seconds();
    assert!((290..=305).contains(&ttl), "umur URL {ttl} detik (5 menit)");
    let r = w
        .hr_post(&hr_b, DOWNLOADS, &json!({"document_kind": "nib"}))
        .await;
    assert_eq!(
        (r.status, code(&r).as_str()),
        (404, "4040001"),
        "{}",
        r.body
    );
    assert!(
        !r.body.contains(&a_file.to_string()) && !r.body.contains(&a.nib),
        "respons B memuat penanda A"
    );
    // Jenis di luar daftar = isian tidak valid (422), bukan 404 yang membedakan.
    let r = w
        .hr_post(&hr_a, DOWNLOADS, &json!({"document_kind": "ktp"}))
        .await;
    assert_eq!(r.status, 422, "{}", r.body);

    // E-12 unduhan HR tercatat dengan tenant A, tanpa kunci objek, dan tidak ada untuk B.
    let events = w
        .tenant_events("e_12", "stored_file_downloaded", a.id)
        .await;
    assert_eq!(events.len(), 1);
    assert_event_complete(&events[0], "e_12");
    assert_eq!(events[0]["actor_account_id"], hr_a.account_id.to_string());
    assert_eq!(events[0]["actor_account_type"], "corporate_admin");
    assert_eq!(events[0]["source_component"], "corporate_portal");
    assert!(events[0]["subjects"].as_array().is_none_or(Vec::is_empty));
    assert!(w.all_tenant_events(b.id).await.is_empty());
    for e in w.all_tenant_events(a.id).await {
        assert!(!e.to_string().contains("uji/"), "kunci objek di audit: {e}");
    }

    // Slot milik HR A tidak dapat dipakai HR B (404 sama dengan slot tidak ada); slot A tetap utuh.
    let r = w.slot(&hr_a, "image/png", 1024).await;
    assert_eq!(r.status, 201, "{}", r.body);
    let slot_a = r.json()["data"]["upload_slot_id"]
        .as_str()
        .unwrap()
        .to_string();
    let r = w.confirm(&hr_b, "nib", &slot_a).await;
    assert_eq!(
        (r.status, code(&r).as_str()),
        (404, "4040001"),
        "{}",
        r.body
    );
    let r = w.confirm(&hr_b, "nib", &new_id().to_string()).await;
    assert_eq!(
        (r.status, code(&r).as_str()),
        (404, "4040001"),
        "{}",
        r.body
    );
    let (owner, consumed) = slot_row(&w.app, &slot_a).await;
    assert_eq!(owner, Some(hr_a.account_id));
    assert!(!consumed, "slot A tidak boleh terpakai oleh HR B");
    assert!(active_kinds(&w.app, b.id).await.is_empty());
    assert_eq!(active_kinds(&w.app, a.id).await, ["nib"]);

    // ID perusahaan/anggota di isi atau query tidak diterima dan tidak mengubah apa pun (cakupan dari sesi).
    for (path, body) in [
        (
            DOWNLOADS,
            json!({"document_kind": "nib", "corporate_client_id": a.id.to_string()}),
        ),
        (
            DOCUMENTS,
            json!({"document_kind": "nib", "upload_slot_id": slot_a, "corporate_client_id": a.id.to_string()}),
        ),
        (
            SLOTS,
            json!({"content_type": "image/png", "size_bytes": 1, "corporate_client_id": a.id.to_string()}),
        ),
    ] {
        let r = w.hr_post(&hr_b, path, &body).await;
        assert_eq!(r.status, 422, "{path} {}", r.body);
    }
    let r = w
        .hr_get(&hr_b, &format!("{DOCUMENTS}?corporate_client_id={}", a.id))
        .await;
    assert_eq!(r.status, 200);
    assert!(
        r.json()["data"]["documents"]
            .as_array()
            .unwrap()
            .iter()
            .all(|d| d["uploaded"] == false),
        "query ID perusahaan lain tidak boleh mengubah cakupan"
    );
}

// trace: US-1901 AC2, NFR-SEC-08 (batas jumlah slot per akun; jenis dan ukuran di luar batas ditolak sejak slot)
#[tokio::test(flavor = "multi_thread")]
async fn slot_requests_are_limited_in_type_size_and_count() {
    let w = world_with_pdf_support().await;
    let (_c, hr) = company_with_hr(&w.app, "pending_verification").await;

    // Jenis: hanya JPG, PNG, PDF (menurut isi diperiksa saat konfirmasi; di sini yang dinyatakan).
    for bad in [
        "image/gif",
        "image/svg+xml",
        "application/zip",
        "text/html",
        "application/x-msdownload",
        "",
        "IMAGE/PNG",
    ] {
        let r = w.slot(&hr, bad, 1024).await;
        assert_eq!((r.status, code(&r).as_str()), (415, "4150001"), "{bad}");
    }
    // Ukuran: foto ≤ 5.242.880, PDF ≤ 10.485.760 (NFR-SEC-08); nol ditolak.
    let r = w.slot(&hr, "image/png", 5_242_881).await;
    assert_eq!((r.status, code(&r).as_str()), (413, "4130001"));
    let r = w.slot(&hr, "image/jpeg", 5_242_880).await;
    assert_eq!(r.status, 201, "{}", r.body);
    let r = w.slot(&hr, "application/pdf", 10_485_761).await;
    assert_eq!((r.status, code(&r).as_str()), (413, "4130001"));
    let r = w.slot(&hr, "application/pdf", 10_485_760).await;
    assert_eq!(r.status, 201, "{}", r.body);
    let r = w.slot(&hr, "image/png", 0).await;
    assert_eq!(r.status, 422, "{}", r.body);
    // Isi bukan JSON sah atau field tidak dikenal.
    let r = common::request_with_body(
        w.addr,
        "POST",
        SLOTS,
        &[&w.hr_header(&hr), "Content-Type: application/json"],
        Some(b"{bukan json"),
    )
    .await;
    assert_eq!(r.status, 400, "{}", r.body);
    let r = w
        .hr_post(
            &hr,
            SLOTS,
            &json!({"content_type": "image/png", "size_bytes": 1, "x": 1}),
        )
        .await;
    assert_eq!(r.status, 422);

    // Tanpa pemeriksa PDF terpasang, PDF ditolak sejak slot (gagal-tertutup, ADR-018 butir 6).
    let plain = plain_world().await;
    let (_c2, hr2) = company_with_hr(&plain.app, "pending_verification").await;
    let r = plain.slot(&hr2, "application/pdf", 1024).await;
    assert_eq!(
        (r.status, code(&r).as_str()),
        (415, "4150001"),
        "{}",
        r.body
    );
    assert_eq!(plain.slot(&hr2, "image/png", 1024).await.status, 201);

    // Jumlah: satu akun paling banyak 30 slot per jam; akun lain tidak terpengaruh.
    let (_c3, hr3) = company_with_hr(&w.app, "pending_verification").await;
    for i in 0..30 {
        let r = w.slot(&hr3, "image/png", 1024).await;
        assert_eq!(r.status, 201, "slot ke-{} {}", i + 1, r.body);
    }
    let r = w.slot(&hr3, "image/png", 1024).await;
    assert_eq!(
        (r.status, code(&r).as_str()),
        (429, "4290001"),
        "{}",
        r.body
    );
    let (_c4, hr4) = company_with_hr(&w.app, "pending_verification").await;
    assert_eq!(w.slot(&hr4, "image/png", 1024).await.status, 201);
}

// trace: Q8 (antrean memfilter atau menandai pengajuan tanpa dokumen lengkap), US-1902 AC1, AC2
#[tokio::test(flavor = "multi_thread")]
async fn queue_marks_incomplete_submissions_and_approval_requires_all_three_documents() {
    let w = plain_world().await;
    let verifier = insert_verifier(&w.app).await;
    let full = insert_client(&w.app, "pending_verification", 30).await;
    let partial = insert_client(&w.app, "pending_verification", 20).await;
    let none = insert_client(&w.app, "pending_verification", 10).await;
    seed_all_documents(&w.app, full.id).await;
    seed_document(&w.app, partial.id, "nib").await;
    seed_document(&w.app, partial.id, "npwp").await;
    let (full_s, partial_s, none_s) = (
        full.id.hyphenated().to_string(),
        partial.id.hyphenated().to_string(),
        none.id.hyphenated().to_string(),
    );

    // Bawaan: hanya yang siap diverifikasi.
    let ready = w.queue_ids(&verifier, None).await;
    assert!(ready.contains(&full_s));
    assert!(!ready.contains(&partial_s) && !ready.contains(&none_s));
    // `readiness=ready` = bawaan. Dibandingkan per anggota, bukan seluruh daftar: test lain di berkas ini berjalan
    // bersamaan dan dapat menambah pengajuan lengkap di antara dua pembacaan antrean.
    let explicit = w.queue_ids(&verifier, Some("ready")).await;
    assert!(explicit.contains(&full_s));
    assert!(!explicit.contains(&partial_s) && !explicit.contains(&none_s));
    // Belum lengkap: yang kurang dokumen, terlama dulu.
    let incomplete = w.queue_ids(&verifier, Some("incomplete")).await;
    assert!(incomplete.contains(&partial_s) && incomplete.contains(&none_s));
    assert!(!incomplete.contains(&full_s));
    let pos = |ids: &[String], id: &str| ids.iter().position(|x| x == id).unwrap();
    assert!(pos(&incomplete, &partial_s) < pos(&incomplete, &none_s));
    // Semua: ketiganya, dan penanda per item benar.
    let r = w
        .verifier_get(
            &verifier,
            &format!(
                "{}?limit=100&readiness=all",
                corporate_documents_support::QUEUE
            ),
        )
        .await;
    assert_eq!(r.status, 200, "{}", r.body);
    let items = r.json()["data"].as_array().unwrap().clone();
    for (id, expected) in [(&full_s, true), (&partial_s, false), (&none_s, false)] {
        let item = items
            .iter()
            .find(|i| i["corporate_client_id"] == id.as_str())
            .unwrap();
        assert_eq!(item["documents_complete"], expected, "{id}");
    }
    // Pilihan di luar daftar ditolak per field.
    let r = w
        .verifier_get(
            &verifier,
            &format!("{}?readiness=semua", corporate_documents_support::QUEUE),
        )
        .await;
    assert_eq!(r.status, 422, "{}", r.body);
    // Detail memuat daftar dokumen aktif (lengkap bila ketiganya ada).
    for (id, expected) in [(&full_s, true), (&partial_s, false)] {
        let r = w
            .verifier_get(
                &verifier,
                &format!("{}/{id}", corporate_documents_support::QUEUE),
            )
            .await;
        assert_eq!(r.status, 200, "{}", r.body);
        assert_eq!(
            r.json()["data"]["documents"].as_array().unwrap().len() == 3,
            expected
        );
    }

    // Persetujuan atas pengajuan yang dokumennya belum lengkap ditolak 409 4090113 tanpa efek.
    let not_detail = |evs: Vec<Value>| {
        evs.into_iter()
            .filter(|e| e["action_name"] != "corporate_detail_opened")
            .count()
    };
    let events_before = (
        not_detail(w.all_tenant_events(partial.id).await),
        not_detail(w.all_tenant_events(none.id).await),
    );
    for c in [&partial, &none] {
        let r = w.decide(&verifier, c.id, None).await;
        assert_eq!(
            (r.status, code(&r).as_str()),
            (409, "4090113"),
            "{}",
            r.body
        );
        assert_eq!(
            client_status(&w.app, c.id).await,
            ("pending_verification".to_string(), 1)
        );
        let decisions = sqlx::query_scalar!(
            r#"SELECT count(*) AS "n!" FROM app.corporate_verification_decision WHERE corporate_client_id = $1"#,
            c.id
        )
        .fetch_one(&w.app)
        .await
        .unwrap();
        assert_eq!(decisions, 0);
    }
    // F1-T109 (KEPUTUSAN-382): persetujuan yang ditolak aturan tidak menulis keputusan, tetapi tercatat tepat
    // satu E-10 hasil gagal `documents_incomplete` per percobaan (coding-standards §4.4 butir 3).
    for (c, before) in [(&partial, events_before.0), (&none, events_before.1)] {
        // Pembukaan detail oleh Verifikator (E-05, CR-037) tidak dihitung di sini.
        let all: Vec<_> = w
            .all_tenant_events(c.id)
            .await
            .into_iter()
            .filter(|e| e["action_name"] != "corporate_detail_opened")
            .collect();
        assert_eq!(all.len(), before + 1, "{all:?}");
        let refused: Vec<_> = all
            .into_iter()
            .filter(|e| e["action_name"] == "corporate_verification_approved")
            .collect();
        assert_eq!(refused.len(), 1, "{refused:?}");
        assert_eq!(refused[0]["outcome"], "failure");
        assert_eq!(refused[0]["outcome_reason_code"], "documents_incomplete");
        let tenant = c.id.to_string();
        audit_support::assert_corporate_events_clean(
            &refused,
            &["e_10"],
            Some(&tenant),
            &[c.nib.as_str(), c.npwp.as_str()],
        );
    }
    // Penolakan dengan alasan tetap boleh (Verifikator dapat menolak pengajuan yang tidak lengkap).
    let r = w
        .decide(&verifier, partial.id, Some("nib_document_invalid"))
        .await;
    assert_eq!(r.status, 200, "{}", r.body);
    assert_eq!(client_status(&w.app, partial.id).await.0, "rejected");
    // Yang lengkap dapat disetujui.
    let r = w.decide(&verifier, full.id, None).await;
    assert_eq!(r.status, 200, "{}", r.body);
    assert_eq!(client_status(&w.app, full.id).await.0, "active");
}

// trace: review T1 F1-T102 (keputusan Verifikator terikat versi dokumen yang dilihat)
#[tokio::test(flavor = "multi_thread")]
async fn decision_is_bound_to_the_document_version_the_verifier_saw() {
    let w = plain_world().await;
    let verifier = insert_verifier(&w.app).await;
    let c = insert_client(&w.app, "pending_verification", 5).await;
    seed_all_documents(&w.app, c.id).await;
    let seen = w.documents_version(&verifier, c.id).await;
    assert_eq!(seen.len(), 64);
    // Detail yang sama berulang = token sama; HR mengganti dokumen NIB sesudah Verifikator membuka detail.
    assert_eq!(seen, w.documents_version(&verifier, c.id).await);
    sqlx::query!(
        "UPDATE app.corporate_client_document SET deleted_at = now(), delete_reason = 'replaced' \
         WHERE corporate_client_id = $1 AND document_kind = 'nib' AND deleted_at IS NULL",
        c.id
    )
    .execute(&w.app)
    .await
    .unwrap();
    seed_document(&w.app, c.id, "nib").await;
    let events_before = w.all_tenant_events(c.id).await.len();
    // Menyetujui dan menolak dengan token lama ditolak 409 4090114 tanpa efek; token kosong/asal juga.
    for version in [seen.as_str(), "", "x", &"0".repeat(64)] {
        for reason in [None, Some("nib_document_invalid")] {
            let r = w.decide_with(&verifier, c.id, reason, version).await;
            assert_eq!(
                (r.status, code(&r).as_str()),
                (409, "4090114"),
                "{reason:?} {version}: {}",
                r.body
            );
        }
    }
    assert_eq!(
        client_status(&w.app, c.id).await,
        ("pending_verification".to_string(), 1)
    );
    // F1-T109 (KEPUTUSAN-382): tanpa keputusan tersimpan, tetapi setiap percobaan tercatat E-10 gagal
    // `documents_changed` (4 token x setuju/tolak).
    let after = w.all_tenant_events(c.id).await;
    let refused: Vec<_> = after.iter().skip(events_before).cloned().collect();
    assert_eq!(refused.len(), 8, "{refused:?}");
    for e in &refused {
        assert_eq!(e["outcome"], "failure");
        assert_eq!(e["outcome_reason_code"], "documents_changed");
        assert!(e["changed_fields"].as_array().is_none_or(Vec::is_empty));
    }
    let tenant = c.id.to_string();
    audit_support::assert_corporate_events_clean(
        &refused,
        &["e_10"],
        Some(&tenant),
        &[c.nib.as_str(), c.npwp.as_str(), seen.as_str()],
    );
    // Token wajib ada: tanpa field = 422; field asing = 422.
    let r = w
        .verifier_post(
            &verifier,
            &format!("{}/{}/approve", corporate_documents_support::QUEUE, c.id),
            &json!({}),
        )
        .await;
    assert_eq!(r.status, 422, "{}", r.body);
    // Setelah memuat ulang detail, token baru sah.
    let r = w.decide(&verifier, c.id, None).await;
    assert_eq!(r.status, 200, "{}", r.body);
    assert_eq!(client_status(&w.app, c.id).await.0, "active");
}
