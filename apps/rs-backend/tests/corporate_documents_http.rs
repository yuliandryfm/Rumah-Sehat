//! F1-T102 (T1; US-1901 AC2; ADR-018 butir 2, 3, 6; keputusan user Q8 dan Q9): unggah dokumen NIB, NPWP, dan
//! bukti kewenangan oleh HR lewat upload slot, validasi isi, penetralan PDF, penggantian, dan URL unduh
//! berumur pendek, lewat proses `api` lengkap terhadap PostgreSQL 18 sekali pakai (`scripts/db-ephemeral.sh`,
//! role runtime `rs_api`) dan SeaweedFS efemeral (KEPUTUSAN-233, Docker). Tanpa variabel `RS_TEST_DB_*` atau
//! Docker test GAGAL (tidak dilewati).
//!
//! Pemeriksa PDF di sini pengganti ber-penanda (`StubInspector`): yang diuji adalah alur backend (PDF aman
//! disimpan apa adanya, berkonten aktif/terenkripsi ditolak dan dibuang, pemeriksa mati tidak membakar slot).
//! Penganalisis keluaran qpdf dan penanganan proses ada di `platform/pdf.rs`; qpdf asli di
//! `corporate_documents_pdf_qpdf.rs`.
//!
//! Skenario negatif (coding-standards §7.3): tipe palsu (isi bukan PDF/JPG/PNG walau dinyatakan PDF), ukuran
//! berlebih, PDF berbahaya, metadata foto, URL presigned kedaluwarsa dan dirusak, slot tidak dapat dipakai ulang,
//! audit tanpa data berkas atau rahasia.
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
    CountingInspector, DOCUMENTS, DOWNLOADS, Hr, StubInspector, World, active_kinds, code,
    insert_client, insert_hr, insert_verifier, pdf, seed_all_documents, world,
};
use files::{METADATA_MARKER, SeaweedFs, contains, jpeg, jpeg_with_exif, png, png_with_metadata};
use foundation::{Clock, FixedClock, SystemClock, Uuid, new_id};
use object_storage::VersionListCursor;
use rs_backend::modules::platform::api::{
    FileError, FileStorage, LogicalBucket, PdfInspector, UploadMethod, UploadPurpose,
    UploadSlotRequest, create_upload_slot, purge_expired_upload_slots, validate_upload,
};
use serde_json::{Value, json};
use sqlx::PgPool;
use std::process::Command;
use std::sync::Arc;
use std::time::Duration;

struct Fixture {
    _sw: SeaweedFs,
    storage: Arc<FileStorage>,
    w: World,
}

async fn fixture_with_clock(clock: Arc<dyn Clock>) -> Fixture {
    fixture_with(clock, Arc::new(StubInspector)).await
}

async fn fixture_with(clock: Arc<dyn Clock>, inspector: Arc<dyn PdfInspector>) -> Fixture {
    let sw = SeaweedFs::start().await;
    let storage = sw
        .file_storage(UploadMethod::Post, Duration::from_secs(600))
        .await
        .with_pdf_inspector(inspector);
    // Bucket `license_document` (dokumen perusahaan) dibuat dengan versi objek aktif seperti produksi.
    sw.create_other_buckets(&storage).await;
    let storage = Arc::new(storage);
    let w = world(Arc::clone(&storage), clock).await;
    Fixture {
        _sw: sw,
        storage,
        w,
    }
}

async fn fixture() -> Fixture {
    fixture_with_clock(Arc::new(SystemClock)).await
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

/// Semua kunci berversi di bawah prefiks bucket dokumen perusahaan.
async fn keys_under(storage: &FileStorage, prefix: &str) -> Vec<String> {
    let bucket = storage.bucket_name(LogicalBucket::LicenseDocument);
    let mut out = Vec::new();
    let mut cursor: Option<VersionListCursor> = None;
    loop {
        let page = storage
            .object_store()
            .list_version_keys(&bucket, prefix, cursor.as_ref(), files::now())
            .await
            .unwrap();
        out.extend(page.keys);
        match page.next {
            Some(n) => cursor = Some(n),
            None => break,
        }
    }
    out.sort();
    out.dedup();
    out
}

/// Unduhan lewat URL bertanda tangan (klien tanpa kredensial): (kode HTTP, header huruf kecil, isi).
fn download(url: &str) -> (u16, String, Vec<u8>) {
    let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("corporate-documents-unduh");
    std::fs::create_dir_all(&dir).unwrap();
    let body = dir.join(files::random_hex(8));
    let headers = dir.join(files::random_hex(8));
    let out = Command::new("curl")
        .args(["-s", "-w", "%{http_code}", "-o"])
        .arg(&body)
        .arg("-D")
        .arg(&headers)
        .arg(url)
        .output()
        .unwrap();
    let code: u16 = String::from_utf8(out.stdout)
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    let h = std::fs::read_to_string(&headers).unwrap_or_default();
    let b = std::fs::read(&body).unwrap_or_default();
    let _ = std::fs::remove_file(body);
    let _ = std::fs::remove_file(headers);
    (code, h.to_ascii_lowercase(), b)
}

/// Baris dokumen aktif dan berkasnya.
struct DocRow {
    stored_file_id: Uuid,
    bucket: String,
    object_key: String,
    category: String,
    mime_type: String,
    size_bytes: i32,
    sha256: Vec<u8>,
    owner_account_id: Option<Uuid>,
    container_type: Option<String>,
}

async fn active_doc(pool: &PgPool, client: Uuid, kind: &str) -> DocRow {
    let r = sqlx::query!(
        "SELECT f.id AS stored_file_id, f.bucket, f.object_key, f.category, f.mime_type, f.size_bytes, f.sha256, \
                f.owner_account_id, f.container_type \
         FROM app.corporate_client_document d JOIN app.stored_file f ON f.id = d.stored_file_id \
         WHERE d.corporate_client_id = $1 AND d.document_kind = $2 AND d.deleted_at IS NULL AND f.deleted_at IS NULL",
        client,
        kind,
    )
    .fetch_one(pool)
    .await
    .unwrap();
    DocRow {
        stored_file_id: r.stored_file_id,
        bucket: r.bucket,
        object_key: r.object_key,
        category: r.category,
        mime_type: r.mime_type,
        size_bytes: r.size_bytes,
        sha256: r.sha256,
        owner_account_id: r.owner_account_id,
        container_type: r.container_type,
    }
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

async fn slot_consumed(pool: &PgPool, slot: &str) -> bool {
    files::slot_consumed(pool, Uuid::parse_str(slot).unwrap()).await
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

async fn download_url(w: &World, hr: &Hr, kind: &str) -> Value {
    let r = w
        .hr_post(hr, DOWNLOADS, &json!({"document_kind": kind}))
        .await;
    assert_eq!(r.status, 200, "{}", r.body);
    r.json()["data"].clone()
}

// trace: US-1901 AC2, Q8, Q9, ADR-018 butir 2, 3, 6 (tiga dokumen lewat slot; siap diverifikasi saat lengkap)
#[tokio::test(flavor = "multi_thread")]
async fn hr_uploads_three_documents_which_are_validated_stored_and_audited_without_secrets() {
    let f = fixture().await;
    let w = &f.w;
    let (c, hr) = company_with_hr(&w.app, "pending_verification").await;
    let verifier = insert_verifier(&w.app).await;

    // Belum lengkap: tidak tampil di antrean bawaan, tampil sebagai "belum lengkap".
    let id = c.id.hyphenated().to_string();
    assert!(!w.queue_ids(&verifier, None).await.contains(&id));
    assert!(
        w.queue_ids(&verifier, Some("incomplete"))
            .await
            .contains(&id)
    );

    // NIB: foto JPEG berisi EXIF (lokasi sintetis); NPWP: PNG dengan chunk metadata; bukti kewenangan: PDF.
    let jpeg_bytes = jpeg_with_exif(64, 48);
    assert!(contains(&jpeg_bytes, METADATA_MARKER));
    let png_bytes = png_with_metadata(64, 48);
    assert!(contains(&png_bytes, METADATA_MARKER));
    let pdf_bytes = pdf("bukti-kewenangan-sintetis");

    let r = w
        .upload_document(&hr, "nib", "image/jpeg", &jpeg_bytes)
        .await;
    assert_eq!(r.status, 201, "{}", r.body);
    let d = &r.json()["data"];
    assert_eq!(d["complete"], false);
    assert_eq!(d["documents"][0]["kind"], "nib");
    assert_eq!(d["documents"][0]["uploaded"], true);
    assert!(d["documents"][0]["uploaded_at"].is_string());
    assert_eq!(d["documents"][1]["uploaded"], false);
    assert!(!w.queue_ids(&verifier, None).await.contains(&id));

    let r = w
        .upload_document(&hr, "npwp", "image/png", &png_bytes)
        .await;
    assert_eq!(r.status, 201, "{}", r.body);
    assert_eq!(r.json()["data"]["complete"], false);
    let r = w
        .upload_document(&hr, "authority_proof", "application/pdf", &pdf_bytes)
        .await;
    assert_eq!(r.status, 201, "{}", r.body);
    assert_eq!(r.header("cache-control"), Some("no-store"));
    assert_eq!(r.json()["data"]["complete"], true);
    assert_eq!(r.json()["data"]["status"], "pending_verification");

    // Lengkap: tampil di antrean bawaan ("siap diverifikasi") dan tidak lagi di "belum lengkap".
    assert!(w.queue_ids(&verifier, None).await.contains(&id));
    assert!(
        !w.queue_ids(&verifier, Some("incomplete"))
            .await
            .contains(&id)
    );
    assert_eq!(
        active_kinds(&w.app, c.id).await,
        ["authority_proof", "nib", "npwp"]
    );

    // Metadata berkas: berkas tercatat milik HR pengunggah, kategori dan bucket dokumen perusahaan, wadah
    // retensi kosong (DD §0.7 belum memuat dokumen perusahaan), checksum dan ukuran = isi tersimpan.
    let bucket = f.storage.bucket_name(LogicalBucket::LicenseDocument);
    for (kind, mime) in [
        ("nib", "image/jpeg"),
        ("npwp", "image/png"),
        ("authority_proof", "application/pdf"),
    ] {
        let row = active_doc(&w.app, c.id, kind).await;
        assert_eq!(row.category, "corporate_document", "{kind}");
        assert_eq!(row.bucket, "license_document", "{kind}");
        assert_eq!(row.mime_type, mime, "{kind}");
        assert_eq!(row.owner_account_id, Some(hr.account_id), "{kind}");
        assert_eq!(row.container_type, None, "{kind}");
        assert_eq!(row.sha256.len(), 32);
        assert!(row.size_bytes > 0);
        assert!(row.object_key.starts_with("files/"), "{}", row.object_key);
        assert_eq!(
            f.storage
                .object_store()
                .head_object(&bucket, &row.object_key, files::now())
                .await
                .unwrap(),
            Some(u64::try_from(row.size_bytes).unwrap()),
            "{kind}"
        );
    }
    // Unggahan mentah (`incoming/`) beserta semua versinya sudah dihapus; hanya hasil validasi yang tersisa.
    assert!(keys_under(&f.storage, "incoming/").await.is_empty());
    assert_eq!(keys_under(&f.storage, "files/").await.len(), 3);

    // Unduh lewat URL berumur pendek: lampiran, tanpa cache, isi sesuai; metadata foto sudah terbuang.
    let started = files::now();
    let mut sha_hexes = Vec::new();
    for (kind, mime, ext) in [
        ("nib", "image/jpeg", "jpg"),
        ("npwp", "image/png", "png"),
        ("authority_proof", "application/pdf", "pdf"),
    ] {
        let d = download_url(w, &hr, kind).await;
        assert_eq!(d["mime_type"], mime);
        let expires = time::OffsetDateTime::parse(
            d["expires_at"].as_str().unwrap(),
            &time::format_description::well_known::Rfc3339,
        )
        .unwrap();
        let ttl = (expires - started).whole_seconds();
        assert!(
            (290..=310).contains(&ttl),
            "umur URL {kind}: {ttl} detik (5 menit)"
        );
        let url = d["url"].as_str().unwrap().to_string();
        let (status, headers, body) = download(&url);
        assert_eq!(status, 200, "{kind}");
        assert!(
            headers.contains("content-disposition: attachment"),
            "{headers}"
        );
        assert!(headers.contains(&format!(".{ext}")), "{headers}");
        assert!(
            headers.contains(&format!("content-type: {mime}")),
            "{headers}"
        );
        assert!(headers.contains("no-store"), "{headers}");
        let row = active_doc(&w.app, c.id, kind).await;
        assert_eq!(body.len(), usize::try_from(row.size_bytes).unwrap());
        assert_eq!(crypto::sha256(&body).to_vec(), row.sha256);
        assert!(
            !contains(&body, METADATA_MARKER),
            "metadata {kind} tidak terbuang"
        );
        if kind == "npwp" {
            let chunks = files::png_chunk_types(&body);
            assert!(
                !chunks.iter().any(|c| c == "tEXt" || c == "eXIf"),
                "{chunks:?}"
            );
        }
        if kind == "authority_proof" {
            assert_eq!(body, pdf_bytes, "PDF aman disimpan apa adanya");
        }
        // URL dirusak (tanda tangan diubah, atau kunci objek lain) ditolak penyedia.
        let tampered = url.replace("X-Amz-Signature=", "X-Amz-Signature=00");
        assert_eq!(download(&tampered).0, 403, "{kind} tanda tangan dirusak");
        let other_key = url.replace(
            &row.object_key,
            "files/00000000-0000-7000-8000-000000000000",
        );
        assert_eq!(download(&other_key).0, 403, "{kind} kunci objek diganti");
        sha_hexes.push(hex(&row.sha256));
    }

    // Verifikator dapat menyetujui pengajuan yang lengkap; setelah Aktif dokumen tidak dapat diubah HR.
    let r = w.decide(&verifier, c.id, None).await;
    assert_eq!(r.status, 200, "{}", r.body);
    let r = w.slot(&hr, "image/png", 100).await;
    assert_eq!(
        (r.status, code(&r).as_str()),
        (409, "4090112"),
        "{}",
        r.body
    );
    let r = w
        .hr_post(&hr, DOWNLOADS, &json!({"document_kind": "nib"}))
        .await;
    assert_eq!((r.status, code(&r).as_str()), (409, "4090112"));

    // Audit: tiga E-10 sukses lengkap (jenis dokumen saja), tiga E-12 unduhan; tanpa isi, kunci objek,
    // checksum, penanda metadata, NIB, atau NPWP di payload mana pun milik perusahaan ini.
    let uploaded = w
        .tenant_events("e_10", "corporate_document_uploaded", c.id)
        .await;
    assert_eq!(uploaded.len(), 3);
    for e in &uploaded {
        assert_event_complete(e, "e_10");
        assert_eq!(e["outcome"], "success");
        assert_eq!(e["object_type"], "corporate_client_document");
        assert_eq!(e["actor_account_id"], hr.account_id.to_string());
        assert_eq!(e["actor_account_type"], "corporate_admin");
        assert_eq!(e["source_component"], "corporate_portal");
        assert_eq!(e["after_values"]["replaced"], false);
        assert!(e["source_ip"].is_string());
    }
    let mut kinds: Vec<&str> = uploaded
        .iter()
        .map(|e| e["after_values"]["document_kind"].as_str().unwrap())
        .collect();
    kinds.sort_unstable();
    assert_eq!(kinds, ["authority_proof", "nib", "npwp"]);
    let downloads = w
        .tenant_events("e_12", "stored_file_downloaded", c.id)
        .await;
    assert_eq!(downloads.len(), 3);
    for e in &downloads {
        assert_event_complete(e, "e_12");
    }
    let marker = String::from_utf8_lossy(METADATA_MARKER).to_string();
    // Penanda yang tidak boleh ada di payload mana pun milik perusahaan ini.
    let mut forbidden = vec![
        "files/".to_string(),
        "incoming/".to_string(),
        marker.clone(),
        c.nib.clone(),
        c.npwp.clone(),
        "X-Amz".to_string(),
        "image/jpeg".to_string(),
    ];
    forbidden.extend(sha_hexes.clone());
    // F1-T109: semua event perusahaan ini (E-10 unggah, E-12 unduh) lolos pemeriksa korporat bersama: tenant =
    // perusahaan, tanpa URL unduh/kunci objek/email/NIB/NPWP/deret digit panjang.
    let all = w.all_tenant_events(c.id).await;
    let tenant = c.id.to_string();
    let mut raw: Vec<&str> = forbidden.iter().map(String::as_str).collect();
    raw.push(hr.email.as_str());
    audit_support::assert_corporate_events_clean(&all, &["e_10", "e_12"], Some(&tenant), &raw);
    audit_support::corporate_checker_rejects_leaks(&all[0], &tenant);
    for e in w.all_tenant_events(c.id).await {
        let text = e.to_string();
        for needle in &forbidden {
            assert!(
                !text.contains(needle.as_str()),
                "payload audit memuat {needle}: {text}"
            );
        }
    }
    // Penanda khas perusahaan ini juga tidak muncul di outbox mana pun (termasuk event tanpa tenant).
    let mut unique = vec![marker, c.nib.clone(), c.npwp.clone()];
    unique.extend(sha_hexes);
    for needle in &unique {
        assert_eq!(
            w.outbox_containing(needle).await,
            0,
            "outbox memuat {needle}"
        );
    }
}

// trace: ADR-018 butir 2 (jenis dari isi, bukan header/ekstensi), NFR-SEC-08, ADR-018 butir 6 (PDF berbahaya)
#[tokio::test(flavor = "multi_thread")]
async fn rejected_content_is_removed_never_recorded_and_audited_as_failed_upload() {
    let f = fixture().await;
    let w = &f.w;
    let (c, hr) = company_with_hr(&w.app, "pending_verification").await;
    assert_eq!(stored_file_count(&w.app, hr.account_id).await, 0);

    let html = b"<html><script>alert(1)</script></html>".to_vec();
    let js_pdf = {
        let mut p = pdf("berbahaya");
        p.extend_from_slice(b"\n1 0 obj << /S /JavaScript /JS (app.alert(1)) >> endobj\n");
        p
    };
    let encrypted_pdf = pdf("ENCRYPTED");
    let mut padded_pdf = b"\n  ".to_vec();
    padded_pdf.extend_from_slice(&pdf("awalan-sampah"));
    // (nama, jenis yang dinyatakan, isi): tipe palsu dan PDF yang tidak aman.
    let cases: Vec<(&str, &str, Vec<u8>)> = vec![
        ("html bernama pdf", "application/pdf", html.clone()),
        ("html bernama png", "image/png", html),
        ("png dinyatakan jpeg", "image/jpeg", png(8, 8)),
        ("jpeg dinyatakan pdf", "application/pdf", jpeg(8, 8)),
        ("pdf dinyatakan png", "image/png", pdf("pdf-bukan-png")),
        (
            "magic jpeg tanpa gambar",
            "image/jpeg",
            vec![0xFF, 0xD8, 0xFF, 0xE0, 0, 1],
        ),
        ("pdf berisi javascript", "application/pdf", js_pdf),
        ("pdf terenkripsi", "application/pdf", encrypted_pdf),
        ("pdf dengan awalan sampah", "application/pdf", padded_pdf),
    ];
    for (name, declared, bytes) in &cases {
        let (slot, provider) = w.slot_and_upload(&hr, declared, bytes).await;
        assert!(
            (200..300).contains(&provider),
            "{name}: penyedia {provider}"
        );
        let r = w.confirm(&hr, "authority_proof", &slot).await;
        assert_eq!(
            (r.status, code(&r).as_str()),
            (415, "4150001"),
            "{name}: {}",
            r.body
        );
        // Slot menjadi tombstone: tidak dapat dikonfirmasi lagi (404), tidak ada dokumen atau berkas baru.
        assert!(slot_consumed(&w.app, &slot).await, "{name}");
        let again = w.confirm(&hr, "authority_proof", &slot).await;
        assert_eq!(
            (again.status, code(&again).as_str()),
            (404, "4040001"),
            "{name}"
        );
    }
    assert!(active_kinds(&w.app, c.id).await.is_empty());
    assert_eq!(stored_file_count(&w.app, hr.account_id).await, 0);
    // Berkas ditolak dihapus beserta semua versinya: tidak ada yang tersisa di bucket.
    assert!(keys_under(&f.storage, "incoming/").await.is_empty());
    assert!(keys_under(&f.storage, "files/").await.is_empty());

    // Setiap penolakan isi tercatat E-10 hasil gagal dengan kode tertutup; tanpa isi atau kunci objek.
    let failed: Vec<Value> = w
        .tenant_events("e_10", "corporate_document_uploaded", c.id)
        .await
        .into_iter()
        .filter(|e| e["outcome"] == "failure")
        .collect();
    assert_eq!(failed.len(), cases.len());
    for e in &failed {
        assert_event_complete(e, "e_10");
        assert_eq!(e["outcome_reason_code"], "file_type_rejected");
        assert_eq!(e["object_type"], "corporate_client");
        assert_eq!(e["actor_account_id"], hr.account_id.to_string());
    }
    for needle in ["incoming/", "files/", "alert(1)", "JavaScript", "uji-"] {
        for e in w.all_tenant_events(c.id).await {
            assert!(!e.to_string().contains(needle), "{needle}: {e}");
        }
    }

    // Ukuran berlebih: penyedia menolak unggahan di atas batas kebijakan slot (foto 5 MiB, PDF 10 MiB); slot
    // tanpa berkas dijawab 422 "belum diunggah" dan tidak ada yang tercatat.
    for (declared, size) in [
        ("image/jpeg", 5_242_881usize),
        ("application/pdf", 10_485_761),
    ] {
        let mut oversized = vec![0u8; size];
        oversized[..5].copy_from_slice(if declared == "application/pdf" {
            b"%PDF-"
        } else {
            &[0xFF, 0xD8, 0xFF, 0xE0, 0]
        });
        let r = w.slot(&hr, declared, size - 1).await;
        assert_eq!(r.status, 201, "{}", r.body);
        let slot = r.json()["data"]["upload_slot_id"]
            .as_str()
            .unwrap()
            .to_string();
        let instruction = {
            let d = &r.json()["data"];
            rs_backend::modules::platform::api::UploadInstruction::Post {
                url: d["url"].as_str().unwrap().to_string(),
                fields: d["fields"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|x| {
                        (
                            x["name"].as_str().unwrap().to_string(),
                            x["value"].as_str().unwrap().to_string(),
                        )
                    })
                    .collect(),
            }
        };
        let provider = files::upload(&instruction, &oversized);
        assert!(
            (400..500).contains(&provider),
            "{declared}: penyedia menerima {provider}"
        );
        let r = w.confirm(&hr, "nib", &slot).await;
        assert_eq!(r.status, 422, "{declared}: {}", r.body);
        assert!(!slot_consumed(&w.app, &slot).await);
    }
    assert!(active_kinds(&w.app, c.id).await.is_empty());
    assert_eq!(stored_file_count(&w.app, hr.account_id).await, 0);

    // Pemeriksa PDF tidak berjalan: bukan salah berkas, jadi 503 tanpa membakar slot dan tanpa menghapus berkas.
    let inspector_down = pdf("INSPECTOR-DOWN");
    let (slot, provider) = w
        .slot_and_upload(&hr, "application/pdf", &inspector_down)
        .await;
    assert!((200..300).contains(&provider));
    let before_events = w
        .tenant_events("e_10", "corporate_document_uploaded", c.id)
        .await
        .len();
    for _ in 0..2 {
        let r = w.confirm(&hr, "authority_proof", &slot).await;
        assert_eq!(
            (r.status, code(&r).as_str()),
            (503, "5030001"),
            "{}",
            r.body
        );
    }
    assert!(!slot_consumed(&w.app, &slot).await);
    assert_eq!(keys_under(&f.storage, "incoming/").await.len(), 1);
    assert_eq!(
        w.tenant_events("e_10", "corporate_document_uploaded", c.id)
            .await
            .len(),
        before_events,
        "kegagalan pemeriksa bukan penolakan berkas"
    );
    assert!(active_kinds(&w.app, c.id).await.is_empty());

    // Jenis dokumen di luar daftar atau slot yang tidak ada.
    let r = w.confirm(&hr, "ktp", &slot).await;
    assert_eq!(r.status, 422, "{}", r.body);
    let r = w.confirm(&hr, "nib", "bukan-uuid").await;
    assert_eq!(r.status, 422, "{}", r.body);
    let r = w.confirm(&hr, "nib", &new_id().to_string()).await;
    assert_eq!((r.status, code(&r).as_str()), (404, "4040001"));
}

// trace: US-1901 AC6 (perusahaan ditolak dapat memperbaiki), ADR-003 (penggantian = hapus lunak), KEPUTUSAN-290
#[tokio::test(flavor = "multi_thread")]
async fn replacing_a_document_soft_deletes_the_old_one_and_concurrent_confirms_keep_one_active() {
    let f = fixture().await;
    let w = &f.w;
    // Perusahaan ditolak yang memperbaiki dokumen: ketiga dokumen lama (tiruan) diganti.
    let (c, hr) = company_with_hr(&w.app, "rejected").await;
    sqlx::query!(
        "INSERT INTO app.corporate_verification_decision \
           (id, created_by, corporate_client_id, submission_round, decision, reason_code, decided_by) \
         VALUES ($1, $2, $3, 1, 'rejected', 'nib_document_invalid', $2)",
        new_id(),
        hr.account_id,
        c.id,
    )
    .execute(&w.app)
    .await
    .unwrap();
    seed_all_documents(&w.app, c.id).await;
    let old_nib = active_doc(&w.app, c.id, "nib").await;

    let r = w.hr_get(&hr, DOCUMENTS).await;
    assert_eq!(r.json()["data"]["status"], "rejected");
    assert_eq!(r.json()["data"]["rejection_reason"], "nib_document_invalid");
    assert_eq!(r.json()["data"]["complete"], true);

    let v1 = png_with_metadata(32, 32);
    let r = w.upload_document(&hr, "nib", "image/png", &v1).await;
    assert_eq!(r.status, 201, "{}", r.body);
    let first = active_doc(&w.app, c.id, "nib").await;
    assert_ne!(first.stored_file_id, old_nib.stored_file_id);
    let v2 = pdf("nib-versi-dua");
    let r = w.upload_document(&hr, "nib", "application/pdf", &v2).await;
    assert_eq!(r.status, 201, "{}", r.body);
    let second = active_doc(&w.app, c.id, "nib").await;
    assert_eq!(second.mime_type, "application/pdf");

    // Dokumen lama dan berkasnya terhapus lunak dengan alasan `replaced`; hanya satu yang aktif.
    let rows = sqlx::query!(
        "SELECT d.deleted_at IS NOT NULL AS \"deleted!\", d.delete_reason, d.deleted_by, f.deleted_at IS NOT NULL AS \"file_deleted!\", \
                f.delete_reason AS file_reason, d.stored_file_id \
         FROM app.corporate_client_document d JOIN app.stored_file f ON f.id = d.stored_file_id \
         WHERE d.corporate_client_id = $1 AND d.document_kind = 'nib'",
        c.id
    )
    .fetch_all(&w.app)
    .await
    .unwrap();
    assert_eq!(rows.len(), 3);
    assert_eq!(rows.iter().filter(|r| !r.deleted).count(), 1);
    for r in rows.iter().filter(|r| r.deleted) {
        assert_eq!(r.delete_reason.as_deref(), Some("replaced"));
        assert_eq!(r.deleted_by, Some(hr.account_id));
        assert!(r.file_deleted);
        assert_eq!(r.file_reason.as_deref(), Some("replaced"));
    }
    // Yang aktif adalah versi terbaru dan itulah yang diunduh.
    let d = download_url(w, &hr, "nib").await;
    let (status, _, body) = download(d["url"].as_str().unwrap());
    assert_eq!(status, 200);
    assert_eq!(body, v2);
    // Berkas lama tidak dapat diunduh lagi lewat jenis itu.
    assert_eq!(active_kinds(&w.app, c.id).await.len(), 3);

    // E-10 penggantian: replaced = true pada dokumen yang menggantikan.
    let events = w
        .tenant_events("e_10", "corporate_document_uploaded", c.id)
        .await;
    assert_eq!(events.len(), 2);
    for e in &events {
        assert_event_complete(e, "e_10");
        assert_eq!(e["after_values"]["replaced"], true);
        assert_eq!(e["after_values"]["document_kind"], "nib");
    }
    // F1-T109 (KEPUTUSAN-384): E-16 atas setiap dokumen lama yang dihapus lunak, di transaksi yang sama, dengan
    // waktu dan kode alasan saja; objeknya dokumen lama (bukan berkas).
    let deleted = w
        .tenant_events("e_16", "corporate_document_soft_deleted", c.id)
        .await;
    assert_eq!(deleted.len(), 2, "{deleted:?}");
    let mut deleted_ids: Vec<&str> = deleted
        .iter()
        .map(|e| e["object_id"].as_str().unwrap())
        .collect();
    deleted_ids.sort_unstable();
    let mut expected_ids: Vec<String> = sqlx::query_scalar!(
        "SELECT id FROM app.corporate_client_document \
         WHERE corporate_client_id = $1 AND document_kind = 'nib' AND deleted_at IS NOT NULL",
        c.id
    )
    .fetch_all(&w.app)
    .await
    .unwrap()
    .into_iter()
    .map(|id| id.to_string())
    .collect();
    expected_ids.sort_unstable();
    assert_eq!(deleted_ids, expected_ids);
    for e in &deleted {
        assert_eq!(e["object_type"], "corporate_client_document");
        assert_eq!(e["outcome"], "success");
        assert_eq!(e["actor_account_id"], hr.account_id.to_string());
        assert_eq!(e["after_values"]["delete_reason"], "replaced");
        assert_eq!(e["after_values"]["document_kind"], "nib");
        assert!(e["after_values"]["deleted_at"].is_string());
        assert_eq!(e["changed_fields"].as_array().unwrap().len(), 3);
    }
    let tenant = c.id.to_string();
    audit_support::assert_corporate_events_clean(
        &w.all_tenant_events(c.id).await,
        &["e_10", "e_12", "e_16"],
        Some(&tenant),
        &[
            c.nib.as_str(),
            c.npwp.as_str(),
            hr.email.as_str(),
            old_nib.object_key.as_str(),
            first.object_key.as_str(),
            second.object_key.as_str(),
        ],
    );

    // Dua konfirmasi bersamaan untuk jenis yang sama: berurutan oleh kunci baris perusahaan, keduanya sah,
    // tepat satu dokumen aktif tersisa (UQp), yang lain terhapus lunak.
    let (s1, p1) = w.slot_and_upload(&hr, "image/png", &png(16, 16)).await;
    let (s2, p2) = w
        .slot_and_upload(&hr, "application/pdf", &pdf("bersamaan"))
        .await;
    assert!((200..300).contains(&p1) && (200..300).contains(&p2));
    let (r1, r2) = tokio::join!(
        w.confirm(&hr, "authority_proof", &s1),
        w.confirm(&hr, "authority_proof", &s2)
    );
    assert_eq!(
        (r1.status, r2.status),
        (201, 201),
        "{} | {}",
        r1.body,
        r2.body
    );
    let active = sqlx::query_scalar!(
        r#"SELECT count(*) AS "n!" FROM app.corporate_client_document
           WHERE corporate_client_id = $1 AND document_kind = 'authority_proof' AND deleted_at IS NULL"#,
        c.id
    )
    .fetch_one(&w.app)
    .await
    .unwrap();
    assert_eq!(active, 1);
    let total = sqlx::query_scalar!(
        r#"SELECT count(*) AS "n!" FROM app.corporate_client_document
           WHERE corporate_client_id = $1 AND document_kind = 'authority_proof'"#,
        c.id
    )
    .fetch_one(&w.app)
    .await
    .unwrap();
    assert_eq!(total, 3, "seed + dua konfirmasi bersamaan");
}

// trace: ADR-018 butir 3 (URL unduh berumur pendek, kedaluwarsa ditolak penyedia)
#[tokio::test(flavor = "multi_thread")]
async fn expired_download_url_is_rejected_by_the_provider() {
    // Jam aplikasi 6 menit di belakang: URL 5 menit sudah kedaluwarsa saat diterbitkan; unggahan memakai slot
    // 10 menit sehingga tetap berlaku.
    let f = fixture_with_clock(Arc::new(FixedClock::new(
        files::now() - time::Duration::minutes(6),
    )))
    .await;
    let w = &f.w;
    let (c, hr) = company_with_hr(&w.app, "pending_verification").await;
    let r = w
        .upload_document(&hr, "nib", "image/png", &png(16, 16))
        .await;
    assert_eq!(r.status, 201, "{}", r.body);
    let d = download_url(w, &hr, "nib").await;
    let (status, _, _) = download(d["url"].as_str().unwrap());
    assert_eq!(status, 403, "URL kedaluwarsa harus ditolak penyedia");
    assert_eq!(active_kinds(&w.app, c.id).await, ["nib"]);
}

/// Slot PDF untuk akun `owner` yang sudah diunggah ke object storage; mengembalikan ID slot.
async fn uploaded_pdf_slot(f: &Fixture, owner: Uuid, marker: &str) -> Uuid {
    let bytes = pdf(marker);
    let ticket = create_upload_slot(
        &f.w.app,
        &SystemClock,
        &f.storage,
        UploadSlotRequest {
            owner_account_id: owner,
            purpose: UploadPurpose::CorporateDocument,
            content_type: "application/pdf".to_string(),
            size_bytes: bytes.len() as u64,
        },
    )
    .await
    .unwrap();
    let code = files::upload(&ticket.instruction, &bytes);
    assert!((200..300).contains(&code), "unggah {code}");
    ticket.upload_slot_id
}

// trace: review T1 F1-T102 (izin konkurensi pemeriksa PDF ikut task pemeriksa, bukan permintaan)
#[tokio::test(flavor = "multi_thread")]
async fn cancelled_validation_keeps_its_permit_until_the_inspection_finishes() {
    let inspector = Arc::new(CountingInspector::new(Duration::from_millis(2500)));
    let max = Arc::clone(&inspector.max_concurrent);
    let calls = Arc::clone(&inspector.calls);
    let f = fixture_with(Arc::new(SystemClock), inspector).await; // konkurensi validasi = 2
    let (_c, hr) = company_with_hr(&f.w.app, "pending_verification").await;
    let mut slots = Vec::new();
    for i in 0..4 {
        slots.push(uploaded_pdf_slot(&f, hr.account_id, &format!("batal-{i}")).await);
    }
    let validate = |slot: Uuid| {
        let (pool, storage) = (f.w.app.clone(), Arc::clone(&f.storage));
        async move {
            validate_upload(
                &pool,
                &SystemClock,
                &storage,
                hr.account_id,
                slot,
                UploadPurpose::CorporateDocument,
            )
            .await
        }
    };
    // Dua permintaan dibatalkan 800 ms setelah pemeriksaan berjalan (dua izin terpakai).
    let cancelled = tokio::join!(
        tokio::time::timeout(Duration::from_millis(800), validate(slots[0])),
        tokio::time::timeout(Duration::from_millis(800), validate(slots[1])),
    );
    assert!(
        cancelled.0.is_err() && cancelled.1.is_err(),
        "harus terbatalkan"
    );
    // Dua berikutnya langsung diminta: tanpa izin yang ikut task, keduanya berjalan sekarang (4 serentak).
    let (a, b) = tokio::join!(validate(slots[2]), validate(slots[3]));
    let (a, b) = (a.unwrap(), b.unwrap());
    assert_eq!(a.mime_type(), "application/pdf");
    assert_eq!(b.mime_type(), "application/pdf");
    assert_eq!(
        max.load(std::sync::atomic::Ordering::SeqCst),
        2,
        "pemeriksaan serentak tidak boleh melebihi batas konkurensi walau permintaan dibatalkan"
    );
    assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 4);
}

// trace: review T1 F1-T102 (klaim atomik: satu slot diperiksa sekali pada satu waktu)
#[tokio::test(flavor = "multi_thread")]
async fn parallel_confirmations_of_one_slot_run_the_inspector_once() {
    let inspector = Arc::new(CountingInspector::new(Duration::from_millis(600)));
    let calls = Arc::clone(&inspector.calls);
    let f = fixture_with(Arc::new(SystemClock), inspector).await;
    let (_c, hr) = company_with_hr(&f.w.app, "pending_verification").await;
    let slot = uploaded_pdf_slot(&f, hr.account_id, "sekali").await;
    let run = || {
        let (pool, storage) = (f.w.app.clone(), Arc::clone(&f.storage));
        async move {
            validate_upload(
                &pool,
                &SystemClock,
                &storage,
                hr.account_id,
                slot,
                UploadPurpose::CorporateDocument,
            )
            .await
        }
    };
    let results = tokio::join!(run(), run(), run(), run());
    let all = [results.0, results.1, results.2, results.3];
    let ok = all.iter().filter(|r| r.is_ok()).count();
    assert_eq!(ok, 1, "tepat satu menang");
    for r in all.iter().filter(|r| r.is_err()) {
        assert!(matches!(r, Err(FileError::NotFound)), "{r:?}");
    }
    assert_eq!(
        calls.load(std::sync::atomic::Ordering::SeqCst),
        1,
        "pemeriksa hanya sekali"
    );
}

// trace: review T1 F1-T102 (kuota slot tetap berlaku walau pembersih berjalan di tengah jendela)
#[tokio::test(flavor = "multi_thread")]
async fn slot_quota_survives_cleanup_inside_the_window() {
    let f = fixture().await;
    let w = &f.w;
    let (_c, hr) = company_with_hr(&w.app, "pending_verification").await;
    for i in 0..30 {
        let r = w.slot(&hr, "image/png", 1024).await;
        assert_eq!(r.status, 201, "slot ke-{} {}", i + 1, r.body);
    }
    let r = w.slot(&hr, "image/png", 1024).await;
    assert_eq!((r.status, code(&r).as_str()), (429, "4290001"));
    // Semua slot sudah lewat kedaluwarsa + jeda pembersihan, tetapi baru dibuat (di dalam jendela kuota 1 jam).
    sqlx::query!(
        "UPDATE app.upload_slot SET expires_at = now() - interval '16 minutes' WHERE owner_account_id = $1",
        hr.account_id
    )
    .execute(&w.app)
    .await
    .unwrap();
    let count = || async {
        sqlx::query_scalar!(
            r#"SELECT count(*) AS "n!" FROM app.upload_slot WHERE owner_account_id = $1"#,
            hr.account_id
        )
        .fetch_one(&w.app)
        .await
        .unwrap()
    };
    purge_expired_upload_slots(&w.app, &SystemClock, &f.storage, 1000)
        .await
        .unwrap();
    assert_eq!(
        count().await,
        30,
        "pembersih tidak boleh menghapus slot di dalam jendela kuota"
    );
    let r = w.slot(&hr, "image/png", 1024).await;
    assert_eq!(
        (r.status, code(&r).as_str()),
        (429, "4290001"),
        "kuota tetap menolak"
    );
    // Setelah jendela lewat, baris dibersihkan dan kuota pulih.
    sqlx::query!(
        "UPDATE app.upload_slot SET created_at = now() - interval '2 hours' WHERE owner_account_id = $1",
        hr.account_id
    )
    .execute(&w.app)
    .await
    .unwrap();
    purge_expired_upload_slots(&w.app, &SystemClock, &f.storage, 1000)
        .await
        .unwrap();
    assert_eq!(count().await, 0);
    assert_eq!(w.slot(&hr, "image/png", 1024).await.status, 201);
}
