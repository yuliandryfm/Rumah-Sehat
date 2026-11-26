//! Pemeriksa PDF (F1-T102, ADR-018 butir 6, KEPUTUSAN-86, `docs/05-stack.md` §1.4): PDF dari pengguna tidak
//! dirujuk sebelum diperiksa di server sendiri. CDR atau render ulang penuh belum tersedia, sehingga berlaku
//! jalur penolakan ADR-018 butir 6: PDF yang memuat konten aktif ditolak, dan PDF yang lolos disimpan apa adanya.
//!
//! Mekanisme: proses `qpdf --json=2` (dijalankan sebagai proses anak terpisah dengan batas waktu dan batas
//! keluaran) mengurai PDF dan menuliskan seluruh objeknya sebagai JSON, **termasuk isi object stream** (objek
//! yang tersembunyi di dalam stream terkompresi tampil sebagai objek tingkat atas), sehingga pencarian kata
//! kunci mentah di byte berkas (yang dapat dilewati dengan kompresi atau `#`-encoding nama) tidak dipakai.
//! Nama PDF di JSON v2 berbentuk string berawalan `/` dalam bentuk kanonik (`#xx` sudah diuraikan) dan kunci
//! kamus adalah nama `[SRC-860]`. Penganalisis [`analyze_qpdf_json`] menelusuri **seluruh** pohon JSON (kunci
//! dan nilai string) tanpa bergantung pada letak struktur, jadi tidak ada jalur persembunyian per struktur.
//!
//! Gagal-tertutup: kode keluar selain 0 (rusak, perlu password, ada peringatan pemulihan), keluaran melebihi
//! batas, batas waktu, JSON yang tidak berbentuk keluaran qpdf v2, atau tanda konten aktif = berkas ditolak.
//! Hanya kegagalan menjalankan qpdf itu sendiri (biner tidak ada atau tidak dapat dieksekusi) yang bukan
//! kesalahan berkas: [`PdfVerdict::Unavailable`] (503, berkas tidak dihapus). qpdf tidak menerima masukan dari
//! stdin (berkas harus dapat di-seek `[SRC-861]`), sehingga PDF ditulis sementara ke berkas berizin 0600 dan
//! dihapus segera setelah proses selesai.
//!
//! Batas yang diketahui: isi PDF yang lolos tidak diubah (data setelah `%%EOF` dan objek yatim tetap ada);
//! tanpa antivirus (`[TIDAK DIKETAHUI]`, stack §1.4); `/URI` (tautan) dan `/AcroForm` dibedakan: tautan
//! diizinkan, `/AcroForm` ditolak sesuai KEPUTUSAN-86 (PDF bertanda tangan digital memuat `/AcroForm`).

use serde_json::Value;
use std::future::Future;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::pin::Pin;
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

/// Hasil pemeriksaan satu PDF.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PdfVerdict {
    /// Tidak ada konten aktif yang dikenal; boleh disimpan.
    Safe,
    /// Memuat konten aktif (daftar [`ACTIVE_CONTENT_NAMES`]).
    ActiveContent,
    /// Tidak dapat diperiksa karena berkasnya (rusak, terenkripsi, terlalu rumit, melewati batas waktu atau
    /// keluaran): ditolak.
    Unreadable,
    /// Pemeriksa tidak dapat dijalankan (bukan kesalahan berkas): berkas tidak ditolak, permintaan dijawab 503.
    Unavailable,
}

/// Future hasil pemeriksaan.
pub type PdfInspectFuture<'a> = Pin<Box<dyn Future<Output = PdfVerdict> + Send + 'a>>;

/// Pemeriksa PDF. Implementasi produksi [`QpdfInspector`]; test boleh memasang pengganti.
pub trait PdfInspector: Send + Sync {
    /// Memeriksa isi PDF. Tidak mengubah, menyimpan, atau mencatat isi berkas.
    fn inspect<'a>(&'a self, bytes: &'a [u8]) -> PdfInspectFuture<'a>;
}

/// Nama PDF yang menandai konten aktif atau penyisipan berkas (KEPUTUSAN-86: JavaScript, OpenAction, Launch,
/// EmbeddedFile, AcroForm/XFA; ditambah aksi sejenis). Dibandingkan persis (huruf besar/kecil berarti).
pub const ACTIVE_CONTENT_NAMES: &[&str] = &[
    "/JavaScript",
    "/JS",
    "/OpenAction",
    "/AA",
    "/Launch",
    "/EmbeddedFile",
    "/EmbeddedFiles",
    "/EF",
    "/RichMedia",
    "/XFA",
    "/AcroForm",
    "/SubmitForm",
    "/ImportData",
    "/GoToR",
    "/GoToE",
    "/Movie",
    "/Sound",
    "/Rendition",
    "/3D",
    // Parameter peluncuran khusus platform (aksi Launch lama) dan berkas terenkripsi (F1-T102 review T1).
    "/Win",
    "/Mac",
    "/Unix",
    "/Encrypt",
];

/// Nilai `/S` (jenis aksi atau subtipe kamus) yang diizinkan (daftar izin, review T1 F1-T102): kamus yang punya
/// `/S` selain nilai ini ditolak, sehingga jenis aksi baru atau yang tidak dikenal tidak lolos hanya karena
/// belum ada di daftar tolak. Aksi yang diizinkan: `/URI` (tautan), `/GoTo` (lompat di dokumen), `/Named`
/// (navigasi baku). Nilai sah non-aksi (supaya PDF pindaian dan PDF bertag biasa tidak ditolak): `/Transparency`
/// (kamus grup), jenis label halaman `/D /R /r /A /a`, gaya garis `/S /D /B /I /U` (kamus border), dan tipe
/// struktur baku PDF 32000-1 §14.8.4 (`/Document /P /Span /Figure` dan seterusnya). Tipe struktur khusus
/// (dipetakan lewat RoleMap) tidak ada di daftar, sehingga PDF bertag dengan tipe khusus ditolak; batas yang
/// diketahui dan dicatat di TD-48. Kunci `/S` ganda dalam satu kamus tidak terlihat di JSON qpdf (hanya satu
/// nilai); perilaku qpdf atasnya `[TIDAK DIKETAHUI]` dan diuji di `corporate_documents_pdf_qpdf.rs`.
pub const ALLOWED_S_VALUES: &[&str] = &[
    "/URI",
    "/GoTo",
    "/Named",
    "/Transparency",
    "/D",
    "/R",
    "/r",
    "/A",
    "/a",
    "/S",
    "/B",
    "/I",
    "/U",
    "/Document",
    "/Part",
    "/Art",
    "/Sect",
    "/Div",
    "/BlockQuote",
    "/Caption",
    "/TOC",
    "/TOCI",
    "/Index",
    "/NonStruct",
    "/Private",
    "/P",
    "/H",
    "/H1",
    "/H2",
    "/H3",
    "/H4",
    "/H5",
    "/H6",
    "/L",
    "/LI",
    "/Lbl",
    "/LBody",
    "/Table",
    "/TR",
    "/TH",
    "/TD",
    "/THead",
    "/TBody",
    "/TFoot",
    "/Span",
    "/Quote",
    "/Note",
    "/Reference",
    "/BibEntry",
    "/Code",
    "/Link",
    "/Annot",
    "/Ruby",
    "/RB",
    "/RT",
    "/RP",
    "/Warichu",
    "/WT",
    "/WP",
    "/Figure",
    "/Formula",
    "/Form",
];

/// Batas waktu bawaan satu pemeriksaan (detik). Usulan teknis F1-T102.
pub const DEFAULT_PDF_INSPECT_TIMEOUT_SECONDS: u64 = 10;
/// Batas keluaran JSON bawaan (byte): 32 MiB. Usulan teknis F1-T102 (PDF 10 MiB dengan puluhan ribu objek).
pub const DEFAULT_PDF_INSPECT_MAX_OUTPUT_BYTES: usize = 32 * 1024 * 1024;
/// Batas tunggu hasil pembaca keluaran setelah proses selesai.
const READER_GRACE: Duration = Duration::from_secs(2);
/// Jeda polling proses anak.
const POLL_INTERVAL: Duration = Duration::from_millis(10);
/// Percobaan menjalankan biner yang baru ditulis (ETXTBSY).
const SPAWN_RETRIES: usize = 5;

/// Konfigurasi [`QpdfInspector`].
#[derive(Debug, Clone)]
pub struct QpdfConfig {
    /// Path biner `qpdf` (versi di `docs/05-stack.md`: 12.4.2).
    pub program: PathBuf,
    /// Batas waktu satu pemeriksaan.
    pub timeout: Duration,
    /// Batas keluaran JSON.
    pub max_output_bytes: usize,
    /// Direktori berkas sementara khusus pemeriksa (berkas 0600, dihapus setelah proses). Disiapkan
    /// [`QpdfInspector::prepare`] saat start (0700, sisa `rs-pdf-*` dibersihkan); jangan menunjuk `/tmp` langsung.
    pub temp_dir: PathBuf,
    /// Batas ruang alamat proses qpdf dalam KiB (`ulimit -v`, RLIMIT_AS). Usulan teknis F1-T102: 2 GiB.
    pub memory_limit_kib: u64,
    /// Batas waktu CPU proses qpdf dalam detik (`ulimit -t`, RLIMIT_CPU); bawaan batas waktu + 5.
    pub cpu_limit_seconds: u64,
}

/// Bawaan batas ruang alamat proses qpdf (KiB): 2 GiB.
pub const DEFAULT_PDF_INSPECT_MEMORY_LIMIT_KIB: u64 = 2 * 1024 * 1024;
/// Nama subdirektori bawaan berkas sementara pemeriksa di dalam direktori sementara sistem.
pub const DEFAULT_PDF_TEMP_SUBDIR: &str = "rs-pdf-inspect";

impl QpdfConfig {
    /// Konfigurasi bawaan untuk biner di `program`.
    pub fn new(program: impl Into<PathBuf>) -> Self {
        Self {
            program: program.into(),
            timeout: Duration::from_secs(DEFAULT_PDF_INSPECT_TIMEOUT_SECONDS),
            max_output_bytes: DEFAULT_PDF_INSPECT_MAX_OUTPUT_BYTES,
            temp_dir: std::env::temp_dir().join(DEFAULT_PDF_TEMP_SUBDIR),
            memory_limit_kib: DEFAULT_PDF_INSPECT_MEMORY_LIMIT_KIB,
            cpu_limit_seconds: DEFAULT_PDF_INSPECT_TIMEOUT_SECONDS + 5,
        }
    }
}

/// Pemeriksa PDF berbasis proses `qpdf`.
#[derive(Debug, Clone)]
pub struct QpdfInspector {
    config: Arc<QpdfConfig>,
}

impl QpdfInspector {
    /// Pemeriksa dengan `config`.
    pub fn new(config: QpdfConfig) -> Self {
        Self {
            config: Arc::new(config),
        }
    }

    /// Menyiapkan direktori berkas sementara saat start: dibuat bila belum ada, izin 0700 (hanya pemilik proses),
    /// dan sisa berkas `rs-pdf-*` dari proses yang mati dihapus. Berkas lain di direktori itu tidak disentuh.
    /// Direktori adalah milik pemeriksa saja: izinnya diubah ke 0700.
    pub fn prepare(config: &QpdfConfig) -> std::io::Result<()> {
        std::fs::create_dir_all(&config.temp_dir)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&config.temp_dir, std::fs::Permissions::from_mode(0o700))?;
        }
        for entry in std::fs::read_dir(&config.temp_dir)? {
            let entry = entry?;
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.starts_with("rs-pdf-") && entry.file_type()?.is_file() {
                std::fs::remove_file(entry.path())?;
            }
        }
        Ok(())
    }
}

impl PdfInspector for QpdfInspector {
    fn inspect<'a>(&'a self, bytes: &'a [u8]) -> PdfInspectFuture<'a> {
        Box::pin(async move {
            let config = Arc::clone(&self.config);
            let data = bytes.to_vec();
            // Proses dan analisis JSON sama-sama di thread blocking: permintaan yang dibatalkan tidak menghentikan
            // pekerjaan ini, dan pemanggil menahan izin konkurensinya sampai selesai (service.rs).
            let joined = tokio::task::spawn_blocking(move || match run_qpdf(&config, &data) {
                Ok(output) => analyze_qpdf_json(&output),
                Err(RunFailure::Unavailable) => PdfVerdict::Unavailable,
                Err(RunFailure::Rejected) => PdfVerdict::Unreadable,
            })
            .await;
            joined.unwrap_or_else(|_| {
                tracing::error!("pdf_inspect_task_failed");
                PdfVerdict::Unavailable
            })
        })
    }
}

/// Sebab gagal menjalankan proses.
#[derive(Debug, PartialEq, Eq)]
enum RunFailure {
    /// qpdf tidak dapat dijalankan (bukan kesalahan berkas).
    Unavailable,
    /// qpdf berjalan tetapi menolak berkas, melewati batas waktu atau keluaran.
    Rejected,
}

/// Berkas sementara yang dihapus saat dibuang.
struct TempFile(PathBuf);

impl Drop for TempFile {
    fn drop(&mut self) {
        if std::fs::remove_file(&self.0).is_err() {
            tracing::warn!("pdf_temp_file_remove_failed");
        }
    }
}

fn write_temp(config: &QpdfConfig, data: &[u8]) -> Result<TempFile, RunFailure> {
    let path = config
        .temp_dir
        .join(format!("rs-pdf-{}.pdf", foundation::new_id().hyphenated()));
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(&path).map_err(|_| {
        tracing::error!("pdf_temp_file_create_failed");
        RunFailure::Unavailable
    })?;
    // Dibungkus sebelum menulis: kegagalan tulis tetap menghapus berkas.
    let guard = TempFile(path);
    file.write_all(data).map_err(|_| {
        tracing::error!("pdf_temp_file_write_failed");
        RunFailure::Unavailable
    })?;
    drop(file);
    Ok(guard)
}

/// Pembungkus POSIX tanpa `unsafe`: memasang batas ruang alamat dan waktu CPU (`ulimit`, upaya terbaik: platform
/// tanpa `ulimit -v`, misalnya macOS, melewatinya diam-diam) lalu `exec` qpdf. Argumen: batas KiB, batas detik,
/// program, lalu argumen program. Kode keluar 126/127 (tidak dapat dieksekusi / tidak ditemukan) dibaca
/// sebagai pemeriksa tidak tersedia.
const LIMIT_WRAPPER: &str =
    r#"ulimit -v "$1" 2>/dev/null; ulimit -t "$2" 2>/dev/null; shift 2; exec "$@""#;

fn spawn_qpdf(config: &QpdfConfig, input: &std::path::Path) -> Result<std::process::Child, ()> {
    for attempt in 0..SPAWN_RETRIES {
        let spawned = Command::new("/bin/sh")
            .arg("-c")
            .arg(LIMIT_WRAPPER)
            .arg("sh")
            .arg(config.memory_limit_kib.to_string())
            .arg(config.cpu_limit_seconds.to_string())
            .arg(&config.program)
            .args([
                "--json=2",
                "--json-key=qpdf",
                "--json-stream-data=none",
                "--no-warn",
            ])
            .arg(input)
            .env_clear()
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn();
        match spawned {
            Ok(child) => return Ok(child),
            // Biner yang baru ditulis proses lain dapat sesaat "sibuk"; coba lagi sebentar.
            Err(e)
                if e.kind() == std::io::ErrorKind::ExecutableFileBusy
                    && attempt + 1 < SPAWN_RETRIES =>
            {
                std::thread::sleep(Duration::from_millis(20));
            }
            Err(_) => return Err(()),
        }
    }
    Err(())
}

fn run_qpdf(config: &QpdfConfig, data: &[u8]) -> Result<Vec<u8>, RunFailure> {
    // Biner tidak ada atau bukan berkas: bukan kesalahan berkas (pembungkus sh akan menjawab 127 juga).
    if !config.program.is_file() {
        tracing::error!("pdf_inspector_binary_missing");
        return Err(RunFailure::Unavailable);
    }
    let temp = write_temp(config, data)?;
    let mut child = spawn_qpdf(config, &temp.0).map_err(|()| {
        tracing::error!("pdf_inspector_spawn_failed");
        RunFailure::Unavailable
    })?;
    let overflow = Arc::new(AtomicBool::new(false));
    let (tx, rx) = std::sync::mpsc::channel::<Vec<u8>>();
    // Pembaca keluaran di thread sendiri; hasilnya dikirim lewat saluran dan thread tidak pernah di-`join`:
    // proses turunan yang masih memegang ujung pipa (misalnya cucu proses) tidak boleh menahan permintaan.
    if let Some(mut out) = child.stdout.take() {
        let overflow = Arc::clone(&overflow);
        let max = config.max_output_bytes;
        std::thread::spawn(move || {
            let mut collected = Vec::new();
            let mut chunk = [0u8; 64 * 1024];
            loop {
                match out.read(&mut chunk) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        collected.extend_from_slice(chunk.get(..n).unwrap_or_default());
                        if collected.len() > max {
                            overflow.store(true, Ordering::Release);
                            break;
                        }
                    }
                }
            }
            let _ = tx.send(collected);
        });
    }
    let deadline = Instant::now() + config.timeout;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Some(status),
            Ok(None) => {
                if overflow.load(Ordering::Acquire) || Instant::now() >= deadline {
                    break None;
                }
                std::thread::sleep(POLL_INTERVAL);
            }
            Err(_) => break None,
        }
    };
    if status.is_none() {
        // Melewati batas waktu atau keluaran: hentikan proses (dan tunggu supaya tidak menjadi zombie).
        let _ = child.kill();
        let _ = child.wait();
    }
    match status {
        Some(s) if matches!(s.code(), Some(126 | 127)) => {
            tracing::error!("pdf_inspector_not_executable");
            Err(RunFailure::Unavailable)
        }
        Some(s) if s.success() && !overflow.load(Ordering::Acquire) => {
            // Proses sudah selesai: ujung pipa tertutup dan pembaca segera mengirim hasilnya. Batas tunggu
            // pendek melindungi dari pemegang ujung pipa lain; tanpa hasil = ditolak.
            rx.recv_timeout(READER_GRACE)
                .map_err(|_| RunFailure::Rejected)
        }
        _ => Err(RunFailure::Rejected),
    }
}

/// Apakah `s` (nama PDF, dengan atau tanpa awalan `n:` JSON qpdf) menandai konten aktif.
fn is_active_name(s: &str) -> bool {
    let name = s.strip_prefix("n:").unwrap_or(s);
    ACTIVE_CONTENT_NAMES.contains(&name)
}

/// Apakah nilai `/S` termasuk [`ALLOWED_S_VALUES`] (nama kanonik, awalan `n:` dibuang).
fn is_allowed_s(v: &Value) -> bool {
    v.as_str().is_some_and(|s| {
        let name = s.strip_prefix("n:").unwrap_or(s);
        ALLOWED_S_VALUES.contains(&name)
    })
}

/// Menelusuri seluruh pohon JSON (kunci dan nilai string) tanpa rekursi; `true` bila ada nama konten aktif.
fn contains_active_name(root: &Value) -> bool {
    let mut stack = vec![root];
    while let Some(v) = stack.pop() {
        match v {
            Value::Object(map) => {
                // Daftar izin `/S`: nilai di luar [`ALLOWED_S_VALUES`] (atau bukan nama) ditolak.
                if map.get("/S").is_some_and(|s| !is_allowed_s(s)) {
                    return true;
                }
                for (k, child) in map {
                    if is_active_name(k) {
                        return true;
                    }
                    stack.push(child);
                }
            }
            Value::Array(items) => stack.extend(items.iter()),
            Value::String(s) if is_active_name(s) => return true,
            _ => {}
        }
    }
    false
}

/// Menganalisis keluaran `qpdf --json=2 --json-key=qpdf` (format: `[SRC-860]`). Keluaran yang bukan JSON v2
/// berbentuk `{"qpdf":[{"jsonversion":2,...},{"obj:N G R":...,"trailer":...}]}` (tanpa `trailer`) dianggap
/// tidak dapat diperiksa, jadi opsi yang salah atau keluaran kosong tidak pernah berarti "aman".
pub fn analyze_qpdf_json(output: &[u8]) -> PdfVerdict {
    let Ok(root) = serde_json::from_slice::<Value>(output) else {
        return PdfVerdict::Unreadable;
    };
    let Some(parts) = root.get("qpdf").and_then(Value::as_array) else {
        return PdfVerdict::Unreadable;
    };
    if parts.len() != 2 {
        return PdfVerdict::Unreadable;
    }
    let version_ok = parts
        .first()
        .and_then(|h| h.get("jsonversion"))
        .and_then(Value::as_u64)
        == Some(2);
    let objects = parts.get(1).and_then(Value::as_object);
    let has_trailer = objects.is_some_and(|o| o.contains_key("trailer"));
    if !version_ok || !has_trailer {
        return PdfVerdict::Unreadable;
    }
    if contains_active_name(&root) {
        PdfVerdict::ActiveContent
    } else {
        PdfVerdict::Safe
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn doc(objects: Value) -> Vec<u8> {
        let mut objs = objects.as_object().cloned().unwrap_or_default();
        objs.entry("trailer")
            .or_insert(json!({"value": {"/Root": "1 0 R", "/Size": 5}}));
        json!({"qpdf": [{"jsonversion": 2, "pdfversion": "1.7", "maxobjectid": 4}, objs]})
            .to_string()
            .into_bytes()
    }

    // trace: US-1901 AC2, ADR-018 butir 6 (PDF tanpa konten aktif lolos)
    #[test]
    fn plain_pdf_is_safe_and_links_are_allowed() {
        let out = doc(json!({
            "obj:1 0 R": {"value": {"/Type": "/Catalog", "/Pages": "2 0 R"}},
            "obj:3 0 R": {"value": {"/Type": "/Annot", "/Subtype": "/Link",
                "/A": {"/S": "/URI", "/URI": "u:https://contoh.example/"}}},
            "obj:4 0 R": {"stream": {"dict": {"/Filter": "/FlateDecode"}}},
        }));
        assert_eq!(analyze_qpdf_json(&out), PdfVerdict::Safe);
    }

    // trace: ADR-018 butir 6 (JavaScript, OpenAction, Launch, EmbeddedFile, AcroForm/XFA ditolak)
    #[test]
    fn every_active_content_marker_is_rejected_as_key_or_name_value() {
        for name in ACTIVE_CONTENT_NAMES {
            let as_key = doc(json!({"obj:1 0 R": {"value": {(*name): "x"}}}));
            assert_eq!(
                analyze_qpdf_json(&as_key),
                PdfVerdict::ActiveContent,
                "kunci {name}"
            );
            let as_value = doc(json!({"obj:1 0 R": {"value": {"/S": (*name)}}}));
            assert_eq!(
                analyze_qpdf_json(&as_value),
                PdfVerdict::ActiveContent,
                "nilai {name}"
            );
        }
    }

    #[test]
    fn markers_are_found_at_any_depth_including_stream_dicts_and_trailer() {
        let nested = doc(
            json!({"obj:9 0 R": {"value": {"/A": [[{"/B": {"/Names": ["u:x", {"/JS": "9 0 R"}]}}]]}}}),
        );
        assert_eq!(analyze_qpdf_json(&nested), PdfVerdict::ActiveContent);
        let in_stream = doc(json!({"obj:9 0 R": {"stream": {"dict": {"/Type": "/EmbeddedFile"}}}}));
        assert_eq!(analyze_qpdf_json(&in_stream), PdfVerdict::ActiveContent);
        let in_trailer =
            doc(json!({"trailer": {"value": {"/Root": "1 0 R", "/OpenAction": "5 0 R"}}}));
        assert_eq!(analyze_qpdf_json(&in_trailer), PdfVerdict::ActiveContent);
    }

    // Objek di dalam object stream tampil sebagai objek tingkat atas di JSON qpdf [SRC-860], sehingga
    // JavaScript yang disembunyikan di object stream terkompresi tetap terdeteksi.
    #[test]
    fn object_inside_object_stream_is_scanned_like_any_object() {
        let out = doc(json!({
            "obj:10 0 R": {"stream": {"dict": {"/Type": "/ObjStm", "/N": 1, "/First": 5}}},
            "obj:11 0 R": {"value": {"/Type": "/Action", "/S": "/JavaScript", "/JS": "u:app.alert(1)"}},
        }));
        assert_eq!(analyze_qpdf_json(&out), PdfVerdict::ActiveContent);
    }

    #[test]
    fn name_with_binary_prefix_and_lookalikes() {
        let prefixed = doc(json!({"obj:1 0 R": {"value": {"/S": "n:/JavaScript"}}}));
        assert_eq!(analyze_qpdf_json(&prefixed), PdfVerdict::ActiveContent);
        // Teks biasa yang kebetulan memuat kata itu (string "u:") atau nama berbeda huruf besar/kecil
        // bukan nama PDF aktif.
        let benign = doc(json!({"obj:1 0 R": {"value": {
            "/Title": "u:JavaScript untuk pemula", "/Note": "u:/JavaScript", "/Subtype": "/javascript",
            "/Producer": "u:OpenAction"}}}));
        assert_eq!(analyze_qpdf_json(&benign), PdfVerdict::Safe);
    }

    // trace: gagal-tertutup: keluaran yang bukan JSON v2 qpdf tidak pernah dianggap aman
    #[test]
    fn malformed_or_unexpected_output_is_unreadable() {
        let cases: Vec<Vec<u8>> = vec![
            Vec::new(),
            b"bukan json".to_vec(),
            b"{}".to_vec(),
            b"[]".to_vec(),
            json!({"qpdf": []}).to_string().into_bytes(),
            json!({"qpdf": [{"jsonversion": 2}]})
                .to_string()
                .into_bytes(),
            // versi JSON bukan 2
            json!({"qpdf": [{"jsonversion": 1}, {"trailer": {"value": {}}}]})
                .to_string()
                .into_bytes(),
            // tanpa trailer
            json!({"qpdf": [{"jsonversion": 2}, {"obj:1 0 R": {"value": {}}}]})
                .to_string()
                .into_bytes(),
            // objek bukan kamus
            json!({"qpdf": [{"jsonversion": 2}, []]})
                .to_string()
                .into_bytes(),
            // tiga bagian
            json!({"qpdf": [{"jsonversion": 2}, {"trailer": {}}, {}]})
                .to_string()
                .into_bytes(),
        ];
        for c in cases {
            assert_eq!(analyze_qpdf_json(&c), PdfVerdict::Unreadable);
        }
        // Pohon sangat dalam melewati batas rekursi pengurai: ditolak, bukan lolos.
        let deep = format!(
            "{{\"qpdf\":[{{\"jsonversion\":2}},{{\"trailer\":{}1{}}}]}}",
            "[".repeat(300),
            "]".repeat(300)
        );
        assert_eq!(analyze_qpdf_json(deep.as_bytes()), PdfVerdict::Unreadable);
    }

    // trace: review T1 F1-T102 (daftar izin /S: aksi tak dikenal ditolak, dokumen pindaian/bertag biasa lolos)
    #[test]
    fn s_values_follow_an_allow_list_not_a_deny_list() {
        for ok in [
            "/URI",
            "/GoTo",
            "/Named",
            "/Transparency",
            "/D",
            "/r",
            "/P",
            "/Span",
            "/Figure",
            "/Table",
        ] {
            let out = doc(json!({"obj:1 0 R": {"value": {"/S": ok}}}));
            assert_eq!(analyze_qpdf_json(&out), PdfVerdict::Safe, "{ok}");
        }
        // Halaman pindaian biasa: kamus grup transparansi dan label halaman.
        let scan = doc(json!({
            "obj:3 0 R": {"value": {"/Type": "/Page", "/Group": {"/S": "/Transparency", "/CS": "/DeviceRGB"}}},
            "obj:4 0 R": {"value": {"/Nums": [0, {"/S": "/D"}]}},
        }));
        assert_eq!(analyze_qpdf_json(&scan), PdfVerdict::Safe);
        // Jenis aksi yang tidak ada di daftar tolak mana pun tetap ditolak, juga dalam bentuk n: dan non-nama.
        for bad in [
            "/Thread",
            "/ResetForm",
            "/SetOCGState",
            "/Hide",
            "/Trans",
            "/GoToDp",
            "/Foo",
            "n:/Foo",
        ] {
            let out = doc(json!({"obj:1 0 R": {"value": {"/S": bad}}}));
            assert_eq!(analyze_qpdf_json(&out), PdfVerdict::ActiveContent, "{bad}");
        }
        for odd in [json!(1), json!("u:URI"), json!(["/URI"]), json!(null)] {
            let out = doc(json!({"obj:1 0 R": {"value": {"/S": odd}}}));
            assert_eq!(analyze_qpdf_json(&out), PdfVerdict::ActiveContent, "{odd}");
        }
    }

    #[test]
    fn launch_parameters_and_encryption_dictionary_are_rejected() {
        for key in ["/Win", "/Mac", "/Unix", "/Encrypt"] {
            let out = doc(json!({"obj:1 0 R": {"value": {(key): "x"}}}));
            assert_eq!(analyze_qpdf_json(&out), PdfVerdict::ActiveContent, "{key}");
        }
        // /Encrypt di trailer (bentuk dokumentasi qpdf: kamus enkripsi tetap tampil di keluaran).
        let encrypted = doc(json!({"trailer": {"value": {"/Root": "1 0 R", "/Encrypt": "9 0 R"}}}));
        assert_eq!(analyze_qpdf_json(&encrypted), PdfVerdict::ActiveContent);
    }

    // ---- proses anak: biner palsu (skrip) untuk menguji penanganan kegagalan, bukan qpdf asli ----

    #[cfg(unix)]
    mod process {
        use super::*;
        use std::os::unix::fs::PermissionsExt;

        const SAFE_JSON: &str = r#"{"qpdf":[{"jsonversion":2,"pdfversion":"1.7","maxobjectid":1},{"obj:1 0 R":{"value":{"/Type":"/Catalog"}},"trailer":{"value":{"/Root":"1 0 R"}}}]}"#;

        struct Scratch(PathBuf);

        impl Scratch {
            fn new() -> Self {
                let dir = std::env::temp_dir().join(format!("rs-pdf-uji-{}", foundation::new_id()));
                std::fs::create_dir_all(&dir).unwrap();
                Self(dir)
            }

            fn script(&self, body: &str) -> QpdfConfig {
                let path = self.0.join(format!("qpdf-{}", foundation::new_id()));
                std::fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
                std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).unwrap();
                let tmp = self.0.join("tmp");
                std::fs::create_dir_all(&tmp).unwrap();
                let mut c = QpdfConfig::new(path);
                c.temp_dir = tmp;
                c.timeout = Duration::from_secs(5);
                c
            }

            fn leftovers(&self) -> usize {
                std::fs::read_dir(self.0.join("tmp")).map_or(0, Iterator::count)
            }
        }

        impl Drop for Scratch {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all(&self.0);
            }
        }

        async fn inspect(config: QpdfConfig, bytes: &[u8]) -> PdfVerdict {
            QpdfInspector::new(config).inspect(bytes).await
        }

        // trace: gagal-tertutup; argumen qpdf persis; berkas sementara dihapus
        #[tokio::test]
        async fn qpdf_is_called_with_fixed_arguments_and_temp_file_is_removed() {
            let s = Scratch::new();
            let log = s.0.join("args.log");
            let cfg = s.script(&format!(
                "echo \"$@\" > '{}'; cp \"$5\" '{}.in' 2>/dev/null; cat <<'EOF'\n{SAFE_JSON}\nEOF",
                log.display(),
                log.display()
            ));
            assert_eq!(inspect(cfg, b"%PDF-1.7\n%%EOF\n").await, PdfVerdict::Safe);
            let args = std::fs::read_to_string(&log).unwrap();
            assert!(
                args.starts_with("--json=2 --json-key=qpdf --json-stream-data=none --no-warn /"),
                "{args}"
            );
            assert_eq!(
                std::fs::read(format!("{}.in", log.display())).unwrap(),
                b"%PDF-1.7\n%%EOF\n"
            );
            assert_eq!(s.leftovers(), 0, "berkas sementara tertinggal");
        }

        #[tokio::test]
        async fn active_content_in_output_is_rejected() {
            let s = Scratch::new();
            let bad = SAFE_JSON.replace("/Catalog", "/JavaScript");
            let cfg = s.script(&format!("cat <<'EOF'\n{bad}\nEOF"));
            assert_eq!(inspect(cfg, b"%PDF-1.7").await, PdfVerdict::ActiveContent);
            assert_eq!(s.leftovers(), 0);
        }

        #[tokio::test]
        async fn nonzero_exit_even_with_valid_output_is_rejected() {
            let s = Scratch::new();
            for code in [2, 3] {
                let cfg = s.script(&format!("cat <<'EOF'\n{SAFE_JSON}\nEOF\nexit {code}"));
                assert_eq!(
                    inspect(cfg, b"%PDF-1.7").await,
                    PdfVerdict::Unreadable,
                    "{code}"
                );
            }
            assert_eq!(s.leftovers(), 0);
        }

        #[tokio::test]
        async fn timeout_kills_the_process_and_rejects() {
            let s = Scratch::new();
            let mut cfg = s.script("sleep 30");
            cfg.timeout = Duration::from_millis(300);
            let started = Instant::now();
            assert_eq!(inspect(cfg, b"%PDF-1.7").await, PdfVerdict::Unreadable);
            assert!(started.elapsed() < Duration::from_secs(5));
            assert_eq!(s.leftovers(), 0);
        }

        #[tokio::test]
        async fn output_over_the_limit_is_rejected_without_waiting_for_the_process() {
            let s = Scratch::new();
            let mut cfg = s.script("yes x");
            cfg.max_output_bytes = 4096;
            let started = Instant::now();
            assert_eq!(inspect(cfg, b"%PDF-1.7").await, PdfVerdict::Unreadable);
            assert!(started.elapsed() < Duration::from_secs(5));
        }

        // trace: tanpa pemeriksa yang berjalan, berkas bukan salah pengguna (503), tidak pernah "aman"
        #[tokio::test]
        async fn missing_or_unusable_binary_is_unavailable() {
            let s = Scratch::new();
            let mut cfg = QpdfConfig::new(s.0.join("tidak-ada"));
            cfg.temp_dir = s.0.clone();
            assert_eq!(inspect(cfg, b"%PDF-1.7").await, PdfVerdict::Unavailable);
            // Direktori sementara yang tidak ada.
            let mut cfg = s.script("exit 0");
            cfg.temp_dir = s.0.join("tidak-ada-juga");
            assert_eq!(inspect(cfg, b"%PDF-1.7").await, PdfVerdict::Unavailable);
        }

        #[tokio::test]
        async fn temp_file_is_private() {
            let s = Scratch::new();
            let probe = s.0.join("mode.txt");
            let cfg = s.script(&format!(
                "stat -c %a \"$5\" > '{}'; cat <<'EOF'\n{SAFE_JSON}\nEOF",
                probe.display()
            ));
            assert_eq!(inspect(cfg, b"%PDF-1.7").await, PdfVerdict::Safe);
            assert_eq!(std::fs::read_to_string(&probe).unwrap().trim(), "600");
        }

        // trace: review T1 F1-T102 (batas sumber daya proses anak, tanpa unsafe)
        #[tokio::test]
        async fn resource_limits_are_applied_to_the_child_process() {
            let s = Scratch::new();
            let probe = s.0.join("limits.txt");
            let mut cfg = s.script(&format!(
                "{{ ulimit -v; ulimit -t; }} > '{}'; cat <<'EOF'\n{SAFE_JSON}\nEOF",
                probe.display()
            ));
            cfg.memory_limit_kib = 1_048_576;
            cfg.cpu_limit_seconds = 17;
            assert_eq!(inspect(cfg, b"%PDF-1.7").await, PdfVerdict::Safe);
            let text = std::fs::read_to_string(&probe).unwrap();
            let lines: Vec<&str> = text.lines().collect();
            assert_eq!(lines, ["1048576", "17"], "ulimit -v dan -t");
        }

        // trace: review T1 F1-T102 (direktori sementara 0700, sisa rs-pdf-* dibersihkan saat start)
        #[test]
        fn prepare_creates_private_dir_and_removes_only_stale_pdf_files() {
            let s = Scratch::new();
            let dir = s.0.join("kerja-pemeriksa");
            let mut cfg = QpdfConfig::new(s.0.join("qpdf"));
            cfg.temp_dir = dir.clone();
            QpdfInspector::prepare(&cfg).unwrap();
            let mode =
                |p: &std::path::Path| std::fs::metadata(p).unwrap().permissions().mode() & 0o777;
            assert_eq!(mode(&dir), 0o700);
            std::fs::write(dir.join("rs-pdf-sisa.pdf"), b"x").unwrap();
            std::fs::write(dir.join("lain.txt"), b"y").unwrap();
            std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o755)).unwrap();
            QpdfInspector::prepare(&cfg).unwrap();
            assert_eq!(mode(&dir), 0o700, "izin longgar dikembalikan ke 0700");
            assert!(!dir.join("rs-pdf-sisa.pdf").exists());
            assert!(dir.join("lain.txt").exists());
        }
    }
}
