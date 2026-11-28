//! F1-T102 (T1; ADR-018 butir 6, KEPUTUSAN-86): penetralan PDF dengan **qpdf asli** (12.4.2, `docs/05-stack.md`
//! §1.4), bukan pengganti. Semua test di berkas ini bertanda `#[ignore]` karena butuh biner qpdf yang tidak
//! dipasang `scripts/verify.sh` (alat luar repo; pemasangannya keputusan user dan task infrastruktur CI):
//!
//! ```text
//! RS_TEST_QPDF=/jalur/ke/qpdf bash scripts/db-ephemeral.sh cargo test -p rs-backend \
//!   --test corporate_documents_pdf_qpdf --locked -- --ignored
//! ```
//!
//! Tanpa `RS_TEST_QPDF`, dicari `~/.local/share/ruang-sehat/qpdf-12.4.2/bin/qpdf`. Bila biner tidak ada, test
//! yang dijalankan dengan `--ignored` GAGAL (tidak dilewati diam-diam). Penganalisis keluaran dan penanganan
//! proses (batas waktu, keluaran, berkas sementara) diuji tanpa qpdf di `platform/pdf.rs`; alur HTTP dengan
//! pengganti pemeriksa di `corporate_documents_http.rs`.
//!
//! Yang dibuktikan di sini terhadap qpdf asli: PDF biasa lolos; konten aktif (JavaScript, OpenAction, Launch,
//! EmbeddedFile, AcroForm, XFA, nama di-`#`-escape) ditolak, termasuk yang disembunyikan di object stream
//! terkompresi dan di objek yatim; PDF terenkripsi, rusak (xref dipulihkan), dan bukan PDF ditolak; PDF yang
//! lolos dan yang berbahaya lewat alur unggah penuh.
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

use corporate_documents_support::{code, insert_client, insert_hr, world};
use files::SeaweedFs;
use foundation::SystemClock;
use observability::EnvSource;
use rs_backend::modules::platform::api::{
    PdfInspector, PdfVerdict, QpdfConfig, QpdfInspector, UploadMethod,
};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::time::Duration;

fn qpdf_path() -> PathBuf {
    let env = EnvSource::from_process();
    let explicit: String = env.optional_or("RS_TEST_QPDF", String::new()).unwrap();
    if !explicit.trim().is_empty() {
        return PathBuf::from(explicit.trim());
    }
    let home: String = env.optional_or("HOME", String::new()).unwrap();
    let path = Path::new(&home).join(".local/share/ruang-sehat/qpdf-12.4.2/bin/qpdf");
    assert!(
        path.exists(),
        "qpdf tidak ditemukan: set RS_TEST_QPDF atau pasang di {}",
        path.display()
    );
    path
}

fn inspector() -> QpdfInspector {
    let mut config = QpdfConfig::new(qpdf_path());
    config.timeout = Duration::from_secs(20);
    // Seperti produksi (start api): direktori berkas sementara disiapkan (0700) sebelum pemeriksa dipakai (TD-65).
    // Direktori unik per pemeriksa: `prepare` menghapus sisa `rs-pdf-*`, jangan sampai mengenai test paralel.
    config.temp_dir = std::env::temp_dir().join(format!(
        "rs-pdf-inspect-test-{}",
        foundation::new_id().hyphenated()
    ));
    QpdfInspector::prepare(&config).unwrap();
    QpdfInspector::new(config)
}

/// PDF 1.7 minimal dengan xref klasik yang benar. `catalog_extra` ditambahkan ke katalog; `extra` = objek
/// tambahan bernomor 4 dan seterusnya (boleh yatim).
fn build_pdf(catalog_extra: &str, extra: &[&str]) -> Vec<u8> {
    let mut objs: Vec<String> = vec![
        format!("<< /Type /Catalog /Pages 2 0 R {catalog_extra} >>"),
        "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_string(),
        "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 200 200] >>".to_string(),
    ];
    objs.extend(extra.iter().map(|s| (*s).to_string()));
    let mut out = b"%PDF-1.7\n%\xE2\xE3\xCF\xD3\n".to_vec();
    let mut offsets = Vec::new();
    for (i, o) in objs.iter().enumerate() {
        offsets.push(out.len());
        out.extend(format!("{} 0 obj\n{}\nendobj\n", i + 1, o).bytes());
    }
    let xref = out.len();
    out.extend(format!("xref\n0 {}\n0000000000 65535 f \n", objs.len() + 1).bytes());
    for off in offsets {
        out.extend(format!("{off:010} 00000 n \n").bytes());
    }
    out.extend(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
            objs.len() + 1
        )
        .bytes(),
    );
    out
}

fn scratch(name: &str, bytes: &[u8]) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("corporate-documents-qpdf");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(format!("{}-{name}", files::random_hex(6)));
    std::fs::write(&path, bytes).unwrap();
    path
}

/// Menjalankan qpdf sebagai alat (membangun masukan uji): mengembalikan isi berkas keluaran.
fn qpdf_tool(args: &[&str], input: &[u8]) -> Vec<u8> {
    let input_path = scratch("in.pdf", input);
    let output_path = scratch("out.pdf", b"");
    let status = Command::new(qpdf_path())
        .args(args)
        .arg(&input_path)
        .arg(&output_path)
        .status()
        .unwrap();
    assert!(
        status.success() || status.code() == Some(3),
        "qpdf {args:?} gagal: {status}"
    );
    let out = std::fs::read(&output_path).unwrap();
    let _ = std::fs::remove_file(input_path);
    let _ = std::fs::remove_file(output_path);
    out
}

async fn verdict(bytes: &[u8]) -> PdfVerdict {
    inspector().inspect(bytes).await
}

#[tokio::test]
#[ignore = "butuh biner qpdf 12.4.2 (RS_TEST_QPDF); lihat dokumentasi berkas"]
async fn plain_pdf_passes_and_active_content_is_rejected() {
    assert_eq!(verdict(&build_pdf("", &[])).await, PdfVerdict::Safe);
    // Tautan (URI) bukan konten aktif.
    let link = build_pdf(
        "",
        &[
            "<< /Type /Annot /Subtype /Link /Rect [0 0 1 1] /A << /S /URI /URI (https://contoh.example/) >> >>",
        ],
    );
    assert_eq!(verdict(&link).await, PdfVerdict::Safe);

    // Halaman pindaian biasa: kamus grup transparansi (`/S /Transparency`) bukan aksi (daftar izin `/S`).
    let scan = build_pdf(
        "",
        &["<< /Type /Page /Parent 2 0 R /Group << /S /Transparency /CS /DeviceRGB >> >>"],
    );
    assert_eq!(verdict(&scan).await, PdfVerdict::Safe);

    // PDF sah yang memuat `/S` di luar kamus aksi (review T1 N1): hint stream PDF terlinearisasi (`/S <angka>`),
    // OutputIntent PDF/A, soft mask, dan transisi halaman tidak boleh ditolak.
    let linearized = qpdf_tool(&["--linearize"], &build_pdf("", &[]));
    assert_eq!(
        verdict(&linearized).await,
        PdfVerdict::Safe,
        "terlinearisasi"
    );
    let pdfa = build_pdf(
        "/OutputIntents [<< /Type /OutputIntent /S /GTS_PDFA1 /OutputConditionIdentifier (sRGB) >>]",
        &[],
    );
    assert_eq!(verdict(&pdfa).await, PdfVerdict::Safe, "PDF/A");
    let softmask = build_pdf(
        "",
        &["<< /Type /ExtGState /SMask << /Type /Mask /S /Luminosity /G 3 0 R >> >>"],
    );
    assert_eq!(verdict(&softmask).await, PdfVerdict::Safe, "soft mask");
    let transition = build_pdf(
        "",
        &["<< /Type /Page /Parent 2 0 R /Trans << /Type /Trans /S /Wipe /D 1 >> >>"],
    );
    assert_eq!(verdict(&transition).await, PdfVerdict::Safe, "transisi");
    // Aksi `/Launch` lewat /OpenAction yang dirujuk, dan jenis aksi tak dikenal di kamus ber-/Type /Action.
    let unknown_action = build_pdf("/OpenAction 4 0 R", &["<< /Type /Action /S /Foo >>"]);
    assert_eq!(verdict(&unknown_action).await, PdfVerdict::ActiveContent);

    let bad: Vec<(&str, Vec<u8>)> = vec![
        (
            "jenis aksi di luar daftar izin (/S /Thread)",
            build_pdf("", &["<< /S /Thread /D 3 0 R >>"]),
        ),
        (
            "parameter peluncuran /Win",
            build_pdf("", &["<< /Win << /F (cmd.exe) >> >>"]),
        ),
        (
            "OpenAction + JavaScript",
            build_pdf(
                "/OpenAction 4 0 R",
                &["<< /S /JavaScript /JS (app.alert\\(1\\);) >>"],
            ),
        ),
        (
            "Launch di objek yatim",
            build_pdf("", &["<< /S /Launch /F (cmd.exe) >>"]),
        ),
        (
            "JavaScript di objek yatim",
            build_pdf("", &["<< /S /JavaScript /JS (x) >>"]),
        ),
        (
            "nama JavaScript di-#-escape",
            build_pdf("/Names << /Java#53cript 4 0 R >>", &["<< /Names [] >>"]),
        ),
        (
            "EmbeddedFile (Filespec)",
            build_pdf(
                "",
                &[
                    "<< /Type /Filespec /F (a.exe) /EF << /F 5 0 R >> >>",
                    "<< /Type /EmbeddedFile >>",
                ],
            ),
        ),
        ("AcroForm", build_pdf("/AcroForm << /Fields [] >>", &[])),
        (
            "XFA",
            build_pdf("/AcroForm << /XFA 4 0 R >>", &["<< /Dummy true >>"]),
        ),
        (
            "aksi tambahan halaman",
            build_pdf(
                "",
                &["<< /Type /Page /Parent 2 0 R /AA << /O << /S /JavaScript /JS (x) >> >> >>"],
            ),
        ),
    ];
    for (name, bytes) in bad {
        assert_eq!(verdict(&bytes).await, PdfVerdict::ActiveContent, "{name}");
    }
}

#[tokio::test]
#[ignore = "butuh biner qpdf 12.4.2 (RS_TEST_QPDF); lihat dokumentasi berkas"]
async fn duplicate_s_keys_are_never_safe() {
    // Kunci `/S` ganda dalam satu kamus: penampil berbeda memilih nilai pertama atau terakhir. JSON qpdf hanya
    // memuat satu nilai; perilaku qpdf atas kunci ganda `[TIDAK DIKETAHUI]`, jadi kedua urutan wajib tidak
    // pernah dinilai aman (ditolak sebagai konten aktif atau tidak terbaca, mis. peringatan qpdf kode keluar 3).
    for dict in [
        "<< /S /URI /S /Launch /F (cmd.exe) >>",
        "<< /S /Launch /F (cmd.exe) /S /URI >>",
        "<< /S /URI /S /Thread >>",
        "<< /S /Thread /S /URI >>",
    ] {
        let pdf = build_pdf("/OpenAction 4 0 R", &[dict]);
        assert_ne!(verdict(&pdf).await, PdfVerdict::Safe, "{dict}");
    }
}

#[tokio::test]
#[ignore = "butuh biner qpdf 12.4.2 (RS_TEST_QPDF); lihat dokumentasi berkas"]
async fn javascript_hidden_in_a_compressed_object_stream_is_still_found() {
    let raw = build_pdf(
        "/OpenAction 4 0 R",
        &["<< /S /JavaScript /JS (app.alert\\(1\\);) >>"],
    );
    // Kata kunci mentah ada di berkas asal; setelah masuk object stream terkompresi tidak lagi terbaca di byte.
    let packed = qpdf_tool(&["--object-streams=generate", "--compress-streams=y"], &raw);
    assert!(
        !packed.windows(10).any(|w| w == b"JavaScript"),
        "JavaScript masih terbaca di byte berkas: pengujian tidak bermakna"
    );
    assert_eq!(verdict(&packed).await, PdfVerdict::ActiveContent);
    // Pembanding: PDF aman yang sama dipadatkan tetap lolos.
    let safe = qpdf_tool(
        &["--object-streams=generate", "--compress-streams=y"],
        &build_pdf("", &[]),
    );
    assert_eq!(verdict(&safe).await, PdfVerdict::Safe);
}

#[tokio::test]
#[ignore = "butuh biner qpdf 12.4.2 (RS_TEST_QPDF); lihat dokumentasi berkas"]
async fn encrypted_damaged_and_non_pdf_inputs_are_rejected() {
    // Terenkripsi dengan password pengguna: tidak dapat diperiksa tanpa password.
    let encrypted = qpdf_tool(
        &[
            "--encrypt",
            "kata-sandi-pengguna",
            "kata-sandi-pemilik",
            "256",
            "--",
        ],
        &build_pdf("", &[]),
    );
    assert_eq!(verdict(&encrypted).await, PdfVerdict::Unreadable);
    // Hanya password pemilik (password pengguna kosong): qpdf membukanya tanpa password, tetapi `/Encrypt`
    // di keluaran membuatnya ditolak (klaim KEPUTUSAN-346: berkas terenkripsi ditolak).
    let owner_only = qpdf_tool(
        &["--encrypt", "", "kata-sandi-pemilik", "256", "--"],
        &build_pdf("", &[]),
    );
    assert_ne!(verdict(&owner_only).await, PdfVerdict::Safe);
    // xref rusak (offset digeser): qpdf memulihkan dengan peringatan (kode keluar 3), ditolak kebijakan.
    let mut damaged = build_pdf("", &[]);
    let text = String::from_utf8_lossy(&damaged).to_string();
    let at = text.rfind("startxref\n").unwrap() + "startxref\n".len();
    damaged.splice(at..at + 1, *b"9");
    assert_eq!(verdict(&damaged).await, PdfVerdict::Unreadable);
    // Bukan PDF walau diawali penanda.
    assert_eq!(
        verdict(b"%PDF-1.7\nini-bukan-pdf\n%%EOF\n").await,
        PdfVerdict::Unreadable
    );
    assert_eq!(verdict(b"").await, PdfVerdict::Unreadable);
}

#[tokio::test]
#[ignore = "butuh biner qpdf 12.4.2 dan Docker; lihat dokumentasi berkas"]
async fn real_qpdf_in_the_full_upload_flow() {
    let sw = SeaweedFs::start().await;
    let storage = sw
        .file_storage(UploadMethod::Post, Duration::from_secs(600))
        .await
        .with_pdf_inspector(Arc::new(inspector()));
    sw.create_other_buckets(&storage).await;
    let w = world(Arc::new(storage), Arc::new(SystemClock)).await;
    let c = insert_client(&w.app, "pending_verification", 0).await;
    let hr = insert_hr(&w.app, c.id, "active").await;

    let good = build_pdf("", &[]);
    let r = w
        .upload_document(&hr, "authority_proof", "application/pdf", &good)
        .await;
    assert_eq!(r.status, 201, "{}", r.body);

    let bad = build_pdf(
        "/OpenAction 4 0 R",
        &["<< /S /JavaScript /JS (app.alert\\(1\\);) >>"],
    );
    let (slot, provider) = w.slot_and_upload(&hr, "application/pdf", &bad).await;
    assert!((200..300).contains(&provider));
    let r = w.confirm(&hr, "nib", &slot).await;
    assert_eq!(
        (r.status, code(&r).as_str()),
        (415, "4150001"),
        "{}",
        r.body
    );
    assert_eq!(
        corporate_documents_support::active_kinds(&w.app, c.id).await,
        ["authority_proof"]
    );
}
