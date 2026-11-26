//! Perangkat test dokumen perusahaan oleh HR (F1-T102, T1; US-1901 AC2, AC6; ADR-018, Q8, Q9): proses `api`
//! lengkap (router produksi, resolver akun dan pembaca keanggotaan sungguhan, validator token dengan kunci realm
//! uji) di atas PostgreSQL 18 sekali pakai (`scripts/db-ephemeral.sh`, role runtime `rs_api`) dan, untuk test
//! berkas, SeaweedFS efemeral (`files::SeaweedFs`). Tanpa variabel `RS_TEST_DB_*` atau Docker test GAGAL
//! (tidak dilewati). Data sintetis saja.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    dead_code
)]

use crate::common::{
    self, ADMIN_AUD, ADMIN_ISS, KORPORAT_AUD, KORPORAT_ISS, PASIEN_AUD, PASIEN_ISS, Resp, TestKeys,
    request, request_json, token,
};
use crate::files::{self, random_hex};
use foundation::{Clock, SystemClock, Uuid, new_id};
use observability::EnvSource;
use rs_backend::app::{AppServices, AppState};
use rs_backend::modules::platform::api::{
    FileStorage, PdfInspectFuture, PdfInspector, PdfVerdict, UploadInstruction,
};
use serde_json::{Value, json};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use std::sync::Arc;

pub const DOCUMENTS: &str = "/v1/corporate/documents";
pub const SLOTS: &str = "/v1/corporate/document-upload-slots";
pub const DOWNLOADS: &str = "/v1/corporate/document-download-urls";
pub const QUEUE: &str = "/backoffice/v1/corporate-verifications";
pub const KINDS: [&str; 3] = ["nib", "npwp", "authority_proof"];

/// Pemeriksa PDF pengganti untuk test alur: penanda di isi menentukan hasil. Pemeriksaan qpdf asli ada di
/// `corporate_documents_pdf_qpdf.rs`; penganalisis keluaran dan penanganan proses diuji di `platform/pdf.rs`.
pub struct StubInspector;

impl PdfInspector for StubInspector {
    fn inspect<'a>(&'a self, bytes: &'a [u8]) -> PdfInspectFuture<'a> {
        Box::pin(async move {
            let has = |needle: &[u8]| bytes.windows(needle.len()).any(|w| w == needle);
            if has(b"/JavaScript") {
                PdfVerdict::ActiveContent
            } else if has(b"ENCRYPTED") {
                PdfVerdict::Unreadable
            } else if has(b"INSPECTOR-DOWN") {
                PdfVerdict::Unavailable
            } else {
                PdfVerdict::Safe
            }
        })
    }
}

/// Pemeriksa PDF yang meniru qpdf: pekerjaan berjalan di thread blocking (tidak ikut dibatalkan bila permintaan
/// dibatalkan) dan mencatat jumlah pemeriksaan serentak tertinggi serta jumlah panggilan.
pub struct CountingInspector {
    delay: std::time::Duration,
    current: Arc<std::sync::atomic::AtomicUsize>,
    pub max_concurrent: Arc<std::sync::atomic::AtomicUsize>,
    pub calls: Arc<std::sync::atomic::AtomicUsize>,
}

impl CountingInspector {
    pub fn new(delay: std::time::Duration) -> Self {
        Self {
            delay,
            current: Arc::default(),
            max_concurrent: Arc::default(),
            calls: Arc::default(),
        }
    }
}

impl PdfInspector for CountingInspector {
    fn inspect<'a>(&'a self, _bytes: &'a [u8]) -> PdfInspectFuture<'a> {
        use std::sync::atomic::Ordering::SeqCst;
        let (delay, current, max, calls) = (
            self.delay,
            Arc::clone(&self.current),
            Arc::clone(&self.max_concurrent),
            Arc::clone(&self.calls),
        );
        Box::pin(async move {
            calls.fetch_add(1, SeqCst);
            let joined = tokio::task::spawn_blocking(move || {
                let now = current.fetch_add(1, SeqCst) + 1;
                max.fetch_max(now, SeqCst);
                std::thread::sleep(delay);
                current.fetch_sub(1, SeqCst);
                PdfVerdict::Safe
            })
            .await;
            joined.unwrap_or(PdfVerdict::Unavailable)
        })
    }
}

/// PDF sintetis kecil yang dianggap aman oleh [`StubInspector`] dan memuat `marker` (untuk uji isi tersimpan).
pub fn pdf(marker: &str) -> Vec<u8> {
    format!("%PDF-1.7\n% {marker}\n1 0 obj\n<< /Type /Catalog >>\nendobj\ntrailer\n<< /Root 1 0 R >>\n%%EOF\n")
        .into_bytes()
}

pub async fn pool_from(var: &str, max: u32) -> PgPool {
    let url = EnvSource::from_process()
        .required_secret(var)
        .unwrap_or_else(|_| {
            panic!(
                "{var} tidak diset: jalankan lewat `bash scripts/db-ephemeral.sh cargo test ...`"
            )
        });
    PgPoolOptions::new()
        .max_connections(max)
        .connect(url.expose())
        .await
        .unwrap()
}

/// Angka acak `n` digit berawalan `99` (NIB/NPWP sintetis, bukan nomor nyata).
pub fn digits(n: usize) -> String {
    let mut b = vec![0u8; n.saturating_sub(2)];
    crypto::fill_random(&mut b).unwrap();
    let tail: String = b.iter().map(|x| char::from(b'0' + x % 10)).collect();
    format!("99{tail}")
}

/// Perusahaan sintetis.
#[derive(Clone)]
pub struct Company {
    pub id: Uuid,
    pub nib: String,
    pub npwp: String,
}

pub async fn insert_client(pool: &PgPool, status: &str, age_minutes: i32) -> Company {
    let id = new_id();
    let (nib, npwp) = (digits(13), digits(16));
    sqlx::query!(
        "INSERT INTO app.corporate_client \
           (id, legal_name, nib, npwp, office_address, office_phone, status, origin, submitted_at) \
         VALUES ($1, $2, $3, $4, 'Jl. Sintetis No. 1', '0210000001', $5, 'self_registered', \
                 now() - make_interval(mins => $6))",
        id,
        format!("PT Dokumen Uji {}", random_hex(4)),
        nib,
        npwp,
        status,
        age_minutes,
    )
    .execute(pool)
    .await
    .unwrap();
    Company { id, nib, npwp }
}

/// Satu HR: akun realm `korporat` dan keanggotaannya.
#[derive(Clone)]
pub struct Hr {
    pub account_id: Uuid,
    pub member_id: Uuid,
    pub subject: String,
    pub email: String,
}

pub async fn insert_hr(pool: &PgPool, client: Uuid, account_status: &str) -> Hr {
    let account_id = new_id();
    let subject = format!("hr-doc-{}", random_hex(8));
    let email = format!("{subject}@example.com");
    sqlx::query!(
        "INSERT INTO app.account (id, account_type, idp_realm, idp_subject, email, status, status_changed_at) \
         VALUES ($1, 'corporate_admin', 'korporat', $2, $3, $4, now())",
        account_id,
        subject,
        email,
        account_status,
    )
    .execute(pool)
    .await
    .unwrap();
    let member_id = new_id();
    sqlx::query!(
        "INSERT INTO app.corporate_member (id, account_id, corporate_client_id) VALUES ($1, $2, $3)",
        member_id,
        account_id,
        client,
    )
    .execute(pool)
    .await
    .unwrap();
    Hr {
        account_id,
        member_id,
        subject,
        email,
    }
}

/// Akun Verifikator baru; mengembalikan `idp_subject`.
pub async fn insert_verifier(pool: &PgPool) -> String {
    let id = new_id();
    let subject = format!("verifikator-doc-{}", random_hex(8));
    sqlx::query!(
        "INSERT INTO app.account (id, account_type, idp_realm, idp_subject, email, status, status_changed_at, full_name) \
         VALUES ($1, 'rs_admin', 'admin', $2, $3, 'active', now(), 'Verifikator Sintetis')",
        id,
        subject,
        format!("{subject}@example.com"),
    )
    .execute(pool)
    .await
    .unwrap();
    sqlx::query!(
        "INSERT INTO app.rs_admin_role (id, account_id, sub_role) VALUES ($1, $2, 'verifier')",
        new_id(),
        id,
    )
    .execute(pool)
    .await
    .unwrap();
    subject
}

/// Dokumen aktif langsung di database (berkas kategori `corporate_document`, bucket `license_document`).
pub async fn seed_document(pool: &PgPool, client: Uuid, kind: &str) -> Uuid {
    let file = new_id();
    sqlx::query!(
        "INSERT INTO app.stored_file \
           (id, bucket, object_key, category, mime_type, size_bytes, sha256, validated_at) \
         VALUES ($1, 'license_document', $2, 'corporate_document', 'image/jpeg', 1024, '\\x00', now())",
        file,
        format!("uji/{}", random_hex(12)),
    )
    .execute(pool)
    .await
    .unwrap();
    sqlx::query!(
        "INSERT INTO app.corporate_client_document (id, corporate_client_id, document_kind, stored_file_id) \
         VALUES ($1, $2, $3, $4)",
        new_id(),
        client,
        kind,
        file,
    )
    .execute(pool)
    .await
    .unwrap();
    file
}

pub async fn seed_all_documents(pool: &PgPool, client: Uuid) {
    for k in KINDS {
        seed_document(pool, client, k).await;
    }
}

/// Jenis dokumen aktif (urut jenis).
pub async fn active_kinds(pool: &PgPool, client: Uuid) -> Vec<String> {
    sqlx::query_scalar!(
        "SELECT document_kind FROM app.corporate_client_document \
         WHERE corporate_client_id = $1 AND deleted_at IS NULL ORDER BY document_kind",
        client
    )
    .fetch_all(pool)
    .await
    .unwrap()
}

pub async fn client_status(pool: &PgPool, id: Uuid) -> (String, i32) {
    let r = sqlx::query!(
        "SELECT status, submission_round FROM app.corporate_client WHERE id = $1",
        id
    )
    .fetch_one(pool)
    .await
    .unwrap();
    (r.status, r.submission_round)
}

/// Proses `api` lengkap.
pub struct World {
    pub addr: SocketAddr,
    pub keys: TestKeys,
    pub app: PgPool,
    pub relay: PgPool,
}

/// `files` = layanan berkas (SeaweedFS atau endpoint yang tidak dipakai), `clock` = jam aplikasi.
pub async fn world(files: Arc<FileStorage>, clock: Arc<dyn Clock>) -> World {
    let app = pool_from("RS_TEST_DB_APP_URL", 10).await;
    let relay = pool_from("RS_TEST_DB_RELAY_URL", 2).await;
    let keys = TestKeys::generate();
    let state = AppState::new(
        common::TYPE_BASE,
        Vec::new(),
        AppServices {
            pool: app.clone(),
            clock: Arc::clone(&clock),
            identity: common::unused_identity_services(app.clone(), Arc::clone(&files)),
            files,
            public_content_limit: common::roomy_limiter(),
            public_image_limit: common::roomy_limiter(),
            consent_text_limit: common::roomy_limiter(),
            corporate_hr: common::corporate_hr(app.clone()),
        },
    );
    // Validasi token memakai jam sistem (token uji dibuat dari jam sistem); jam aplikasi boleh digeser.
    let auth = common::auth_state(&keys, Arc::new(SystemClock));
    let directory = common::account_directory(app.clone());
    let addr = common::spawn_app(state, auth, directory.clone(), directory).await;
    World {
        addr,
        keys,
        app,
        relay,
    }
}

/// `world` dengan jam sistem.
pub async fn world_with(files: Arc<FileStorage>) -> World {
    world(files, Arc::new(SystemClock)).await
}

pub fn code(r: &Resp) -> String {
    r.json()["response_code"]
        .as_str()
        .unwrap_or_default()
        .to_string()
}

impl World {
    pub fn hr_header(&self, hr: &Hr) -> String {
        format!(
            "Authorization: Bearer {}",
            token(&self.keys.korporat, KORPORAT_ISS, KORPORAT_AUD, &hr.subject)
        )
    }

    pub fn verifier_header(&self, subject: &str) -> String {
        format!(
            "Authorization: Bearer {}",
            token(&self.keys.admin, ADMIN_ISS, ADMIN_AUD, subject)
        )
    }

    pub fn patient_header(&self, subject: &str) -> String {
        format!(
            "Authorization: Bearer {}",
            token(&self.keys.pasien, PASIEN_ISS, PASIEN_AUD, subject)
        )
    }

    pub async fn hr_get(&self, hr: &Hr, path: &str) -> Resp {
        request(self.addr, "GET", path, &[&self.hr_header(hr)]).await
    }

    pub async fn hr_post(&self, hr: &Hr, path: &str, body: &Value) -> Resp {
        request_json(self.addr, "POST", path, &[&self.hr_header(hr)], body).await
    }

    pub async fn verifier_get(&self, subject: &str, path: &str) -> Resp {
        request(self.addr, "GET", path, &[&self.verifier_header(subject)]).await
    }

    pub async fn verifier_post(&self, subject: &str, path: &str, body: &Value) -> Resp {
        request_json(
            self.addr,
            "POST",
            path,
            &[&self.verifier_header(subject)],
            body,
        )
        .await
    }

    /// Permintaan slot unggah dokumen.
    pub async fn slot(&self, hr: &Hr, content_type: &str, size: usize) -> Resp {
        self.hr_post(
            hr,
            SLOTS,
            &json!({ "content_type": content_type, "size_bytes": size }),
        )
        .await
    }

    /// Konfirmasi unggahan dokumen.
    pub async fn confirm(&self, hr: &Hr, kind: &str, slot_id: &str) -> Resp {
        self.hr_post(
            hr,
            DOCUMENTS,
            &json!({ "document_kind": kind, "upload_slot_id": slot_id }),
        )
        .await
    }

    /// Slot lalu unggah ke object storage seperti klien (curl); mengembalikan (ID slot, kode HTTP penyedia).
    pub async fn slot_and_upload(&self, hr: &Hr, declared: &str, bytes: &[u8]) -> (String, u16) {
        let r = self.slot(hr, declared, bytes.len()).await;
        assert_eq!(r.status, 201, "{}", r.body);
        let d = &r.json()["data"];
        assert_eq!(d["method"], "post");
        let instruction = UploadInstruction::Post {
            url: d["url"].as_str().unwrap().to_string(),
            fields: d["fields"]
                .as_array()
                .unwrap()
                .iter()
                .map(|f| {
                    (
                        f["name"].as_str().unwrap().to_string(),
                        f["value"].as_str().unwrap().to_string(),
                    )
                })
                .collect(),
        };
        let provider = files::upload(&instruction, bytes);
        (d["upload_slot_id"].as_str().unwrap().to_string(), provider)
    }

    /// Slot, unggah, konfirmasi (tiga langkah); mengembalikan respons konfirmasi.
    pub async fn upload_document(&self, hr: &Hr, kind: &str, declared: &str, bytes: &[u8]) -> Resp {
        let (slot, provider) = self.slot_and_upload(hr, declared, bytes).await;
        assert!(
            (200..300).contains(&provider),
            "unggah ke penyedia {provider}"
        );
        self.confirm(hr, kind, &slot).await
    }

    /// Token versi dokumen dari detail Verifikator (keputusan terikat versi dokumen, F1-T102).
    pub async fn documents_version(&self, verifier: &str, client: Uuid) -> String {
        let r = self
            .verifier_get(verifier, &format!("{QUEUE}/{client}"))
            .await;
        assert_eq!(r.status, 200, "{}", r.body);
        r.json()["data"]["documents_version"]
            .as_str()
            .unwrap()
            .to_string()
    }

    /// Menyetujui (`reason` = None) atau menolak pengajuan dengan token versi terbaru dari detail.
    pub async fn decide(&self, verifier: &str, client: Uuid, reason: Option<&str>) -> Resp {
        let version = self.documents_version(verifier, client).await;
        self.decide_with(verifier, client, reason, &version).await
    }

    /// Seperti [`World::decide`] dengan token yang ditentukan pemanggil.
    pub async fn decide_with(
        &self,
        verifier: &str,
        client: Uuid,
        reason: Option<&str>,
        version: &str,
    ) -> Resp {
        match reason {
            None => {
                self.verifier_post(
                    verifier,
                    &format!("{QUEUE}/{client}/approve"),
                    &json!({ "documents_version": version }),
                )
                .await
            }
            Some(r) => {
                self.verifier_post(
                    verifier,
                    &format!("{QUEUE}/{client}/reject"),
                    &json!({ "reason_code": r, "documents_version": version }),
                )
                .await
            }
        }
    }

    /// Antrean lengkap (`limit=100`) sebagai daftar ID untuk pilihan `readiness` (`None` = bawaan).
    pub async fn queue_ids(&self, verifier: &str, readiness: Option<&str>) -> Vec<String> {
        let path = match readiness {
            Some(r) => format!("{QUEUE}?limit=100&readiness={r}"),
            None => format!("{QUEUE}?limit=100"),
        };
        let r = self.verifier_get(verifier, &path).await;
        assert_eq!(r.status, 200, "{}", r.body);
        r.json()["data"]
            .as_array()
            .unwrap()
            .iter()
            .map(|i| i["corporate_client_id"].as_str().unwrap().to_string())
            .collect()
    }

    /// Payload outbox berkode `event_code` dan aksi `action` dengan `tenant_id` = perusahaan.
    pub async fn tenant_events(&self, event_code: &str, action: &str, tenant: Uuid) -> Vec<Value> {
        sqlx::query_scalar!(
            r#"SELECT payload AS "payload!" FROM app.audit_outbox
               WHERE payload ->> 'event_code' = $1 AND payload ->> 'action_name' = $2
                 AND payload ->> 'tenant_id' = $3
               ORDER BY payload ->> 'occurred_at'"#,
            event_code,
            action,
            tenant.hyphenated().to_string(),
        )
        .fetch_all(&self.relay)
        .await
        .unwrap()
    }

    /// Semua payload outbox perusahaan `tenant`.
    pub async fn all_tenant_events(&self, tenant: Uuid) -> Vec<Value> {
        sqlx::query_scalar!(
            r#"SELECT payload AS "payload!" FROM app.audit_outbox
               WHERE payload ->> 'tenant_id' = $1 ORDER BY payload ->> 'occurred_at'"#,
            tenant.hyphenated().to_string(),
        )
        .fetch_all(&self.relay)
        .await
        .unwrap()
    }

    /// Jumlah baris outbox yang teks payload-nya memuat `needle`.
    pub async fn outbox_containing(&self, needle: &str) -> i64 {
        sqlx::query_scalar!(
            r#"SELECT count(*) AS "n!" FROM app.audit_outbox WHERE payload::text LIKE '%' || $1 || '%'"#,
            needle,
        )
        .fetch_one(&self.relay)
        .await
        .unwrap()
    }

    pub async fn outbox_total(&self) -> i64 {
        sqlx::query_scalar!(r#"SELECT count(*) AS "n!" FROM app.audit_outbox"#)
            .fetch_one(&self.relay)
            .await
            .unwrap()
    }
}
