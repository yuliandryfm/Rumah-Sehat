//! Harness isolasi tenant perusahaan (F1-T100, T1; ADR-028 butir 1, KEPUTUSAN-254, threat model KOR-07
//! dan H-18, NFR-SEC-06 diperluas, US-1904 AC3–AC5). Dijalankan `scripts/verify.sh` lewat
//! `bash scripts/db-ephemeral.sh cargo test --workspace` (PostgreSQL sekali pakai, role runtime `rs_api`);
//! tanpa database, test GAGAL.
//!
//! **Setiap endpoint `/v1/corporate/*` wajib punya satu [`Case`] di [`PRODUCTION_CASES`].** Test
//! [`every_corporate_endpoint_is_registered_in_harness`] membandingkan daftar itu dengan registri router
//! proses `api` (`rs_backend::app::corporate_endpoints`) dan gagal bila ada endpoint tanpa kasus, kasus tanpa
//! endpoint, atau jenis kasus yang tidak cocok dengan kebijakannya.
//!
//! Mesin harness ([`run_harness`]) untuk dua perusahaan A (penyerang) dan B (korban), per kasus HR:
//! 1. token realm lain (pasien, admin) ke endpoint HR → `401`;
//! 2. HR A memanggil endpoint dengan **semua** parameter path diisi ID objek milik B, ditambah
//!    `corporate_client_id`/`corporate_member_id` milik B di query dan isi JSON → endpoint ber-ID wajib `404`
//!    `4040001`, endpoint tanpa ID tidak boleh `5xx`;
//! 3. respons ke A (isi dan header) tidak boleh memuat penanda B (ID, nama badan usaha, NIB, NPWP, email);
//! 4. data B di semua tabel ber-`corporate_client_id` (dicari dinamis dari `information_schema`, sehingga
//!    tabel korporat baru ikut terjaga) dan baris `corporate_client` B tidak berubah oleh percobaan A;
//! 5. pembanding positif: HR B atas objeknya sendiri tidak `401`/`403`/`404`/`5xx` (bukti percobaan A
//!    mengenai objek yang benar-benar ada), dan responsnya tidak memuat penanda A.
//!
//! Mesin dibuktikan dengan router uji (endpoint contoh di atas `corporate_member`): versi benar lolos,
//! versi bocor (daftar tanpa filter, baca dan ubah tanpa pemeriksaan pemilik) GAGAL di setiap jenis
//! kebocoran. Rute handler BFF `web/korporat` dan job asinkron pembangkit berkas (H-18) belum ada; keduanya
//! wajib ditambahkan ke harness oleh task pembuatnya (F1-T90, F1-T108, Fase 2).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)] // berkas test

mod common;

use authz::{
    CorporateScope, DenialSink, EndpointKey, Enforcement, Policy, PolicyRouter, Principal,
    PublicReason, corporate_object,
};
use axum::Json;
use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use common::{
    ADMIN_AUD, ADMIN_ISS, KORPORAT_AUD, KORPORAT_ISS, PASIEN_AUD, PASIEN_ISS, Resp, TestKeys, token,
};
use envelope::CommonError;
use foundation::{SystemClock, Uuid, new_id};
use observability::EnvSource;
use rs_backend::app::{
    AppServices, AppState, corporate_endpoints, non_corporate_authenticated_endpoints,
};
use rs_backend::http_support::EnvelopeRenderer;
use rs_backend::modules::corporate::api::CorporateDirectory;
use serde::Deserialize;
use serde_json::{Value, json};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::collections::{BTreeSet, HashMap};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use utoipa_axum::routes;

// ===== daftar kasus endpoint produksi (H-18) =====

/// Pembentuk isi JSON sah untuk tenant pemanggil.
type BodyFn = fn(&Tenant) -> Value;

/// Cara harness menguji satu endpoint di namespace korporat.
#[derive(Clone, Copy)]
enum Probe {
    /// Endpoint publik tanpa data tenant (misalnya pendaftaran perusahaan, US-1901). Alasan wajib supaya
    /// pengecualian terlihat di review.
    Public { reason: &'static str },
    /// Endpoint HR: dijalankan mesin harness. `body` = isi JSON sah untuk tenant pemanggil (endpoint
    /// `POST`/`PUT`/`PATCH`); harness menyelipkan ID perusahaan korban ke dalamnya.
    Hr { body: Option<BodyFn> },
}

/// Satu kasus harness: metode, pola path (sama persis dengan registri router), dan cara uji.
#[derive(Clone, Copy)]
struct Case {
    method: &'static str,
    path: &'static str,
    probe: Probe,
}

/// Semua endpoint `/v1/corporate/*` proses `api`. Task yang menambah endpoint WAJIB menambah kasusnya di
/// sini (dan objek fixture untuk parameter path barunya di [`seed_tenant`]), atau test cakupan di bawah gagal.
///
/// F1-T107: anggota HR. Pembanding positif penonaktifan (HR B atas keanggotaannya sendiri) dijawab `409`
/// (diri sendiri ditolak), cukup membuktikan objek B ada tanpa mengubah data B.
const PRODUCTION_CASES: &[Case] = &[
    Case {
        method: "GET",
        path: "/v1/corporate/members",
        probe: Probe::Hr { body: None },
    },
    Case {
        method: "POST",
        path: "/v1/corporate/members/{corporate_member_id}/deactivate",
        probe: Probe::Hr { body: None },
    },
];

/// Pelanggaran cakupan harness terhadap endpoint terdaftar.
fn coverage_violations(registered: &[(EndpointKey, Policy)], cases: &[Case]) -> Vec<String> {
    let mut out = Vec::new();
    let mut seen = BTreeSet::new();
    for c in cases {
        let key = (c.method.to_string(), c.path.to_string());
        if !seen.insert(key.clone()) {
            out.push(format!("kasus ganda {} {}", c.method, c.path));
        }
        match registered.iter().find(|(k, _)| *k == key) {
            None => out.push(format!(
                "kasus {} {} tanpa endpoint terdaftar",
                c.method, c.path
            )),
            Some((_, policy)) => match (c.probe, policy) {
                (Probe::Public { reason }, Policy::Public(_)) if !reason.trim().is_empty() => {}
                (Probe::Hr { .. }, Policy::Require(r)) if r.is_corporate() => {}
                _ => out.push(format!(
                    "jenis kasus {} {} tidak cocok dengan kebijakannya {policy:?}",
                    c.method, c.path
                )),
            },
        }
    }
    for ((method, path), _) in registered {
        if !seen.contains(&(method.clone(), path.clone())) {
            out.push(format!(
                "endpoint {method} {path} belum terdaftar di harness isolasi (PRODUCTION_CASES)"
            ));
        }
    }
    out
}

// ===== fixture dua perusahaan =====

async fn pool(var: &str) -> PgPool {
    let url = EnvSource::from_process()
        .required_secret(var)
        .unwrap_or_else(|_| {
            panic!(
                "{var} tidak diset: harness isolasi wajib dijalankan lewat \
                 `bash scripts/db-ephemeral.sh cargo test ...` (dipakai scripts/verify.sh)"
            )
        });
    PgPoolOptions::new()
        .max_connections(4)
        .connect(url.expose())
        .await
        .unwrap_or_else(|_| panic!("koneksi {var} gagal"))
}

fn random_hex(n: usize) -> String {
    let mut b = vec![0u8; n];
    crypto::fill_random(&mut b).unwrap();
    b.iter().map(|x| format!("{x:02x}")).collect()
}

/// Satu perusahaan sintetis dengan satu HR aktif dan objek per jenis parameter path.
struct Tenant {
    client_id: Uuid,
    member_id: Uuid,
    account_id: Uuid,
    subject: String,
    email: String,
    legal_name: String,
    nib: String,
    npwp: String,
    /// Objek milik perusahaan ini per nama parameter path (`corporate_member_id`, ...).
    ids: HashMap<&'static str, Uuid>,
}

impl Tenant {
    /// Penanda yang tidak boleh pernah muncul di respons untuk perusahaan lain.
    fn markers(&self) -> Vec<String> {
        let mut m = vec![
            self.client_id.to_string(),
            self.member_id.to_string(),
            self.account_id.to_string(),
            self.subject.clone(),
            self.email.clone(),
            self.legal_name.clone(),
            self.nib.clone(),
            self.npwp.clone(),
        ];
        m.extend(self.ids.values().map(Uuid::to_string));
        m
    }
}

async fn insert_client(pool: &PgPool, status: &str, tag: &str) -> (Uuid, String, String, String) {
    let id = new_id();
    let legal_name = format!("PT Uji Isolasi {tag} {}", random_hex(4));
    let nib = format!("NIB{}", random_hex(8));
    let npwp = format!("NPWP{}", random_hex(8));
    sqlx::query!(
        "INSERT INTO app.corporate_client (id, legal_name, nib, npwp, office_address, office_phone, status, origin) \
         VALUES ($1, $2, $3, $4, 'Jl. Uji No. 1', '0210000000', $5, 'self_registered')",
        id,
        legal_name,
        nib,
        npwp,
        status,
    )
    .execute(pool)
    .await
    .unwrap();
    (id, legal_name, nib, npwp)
}

/// Akun HR aktif di realm `korporat` dengan keanggotaan aktif di `client_id`; mengembalikan
/// (akun, subjek, email, anggota).
async fn insert_hr(pool: &PgPool, client_id: Uuid) -> (Uuid, String, String, Uuid) {
    let account_id = new_id();
    let subject = format!("hr-{}", random_hex(8));
    let email = format!("{subject}@example.com");
    sqlx::query!(
        "INSERT INTO app.account (id, account_type, idp_realm, idp_subject, email, status, status_changed_at) \
         VALUES ($1, 'corporate_admin', 'korporat', $2, $3, 'active', now())",
        account_id,
        subject,
        email,
    )
    .execute(pool)
    .await
    .unwrap();
    let member_id = new_id();
    sqlx::query!(
        "INSERT INTO app.corporate_member (id, account_id, corporate_client_id, job_title) \
         VALUES ($1, $2, $3, 'HR Uji')",
        member_id,
        account_id,
        client_id,
    )
    .execute(pool)
    .await
    .unwrap();
    (account_id, subject, email, member_id)
}

/// Perusahaan Aktif dengan satu HR dan objek fixture untuk setiap jenis parameter path yang dikenal
/// harness. Parameter path baru di endpoint baru = tambahkan objeknya di sini.
async fn seed_tenant(pool: &PgPool, tag: &str) -> Tenant {
    let (client_id, legal_name, nib, npwp) = insert_client(pool, "active", tag).await;
    let (account_id, subject, email, member_id) = insert_hr(pool, client_id).await;
    let ids = HashMap::from([("corporate_member_id", member_id)]);
    Tenant {
        client_id,
        member_id,
        account_id,
        subject,
        email,
        legal_name,
        nib,
        npwp,
        ids,
    }
}

// ===== mesin harness =====

struct Harness<'a> {
    addr: SocketAddr,
    keys: &'a TestKeys,
    pool: &'a PgPool,
    attacker: &'a Tenant,
    victim: &'a Tenant,
}

fn bearer(t: &str) -> String {
    format!("Authorization: Bearer {t}")
}

fn hr_token(keys: &TestKeys, tenant: &Tenant) -> String {
    token(&keys.korporat, KORPORAT_ISS, KORPORAT_AUD, &tenant.subject)
}

/// Mengisi parameter path `{nama}` dengan objek `tenant`; parameter tanpa objek fixture = galat harness.
fn fill_path(path: &str, tenant: &Tenant) -> Result<(String, bool), String> {
    let mut out = String::new();
    let mut rest = path;
    let mut has_params = false;
    while let Some(start) = rest.find('{') {
        let end = rest[start..]
            .find('}')
            .map(|e| start + e)
            .ok_or_else(|| format!("pola path rusak: {path}"))?;
        let name = &rest[start + 1..end];
        let id = tenant
            .ids
            .get(name)
            .ok_or_else(|| format!("fixture tanpa objek untuk parameter {{{name}}} di {path}"))?;
        out.push_str(&rest[..start]);
        out.push_str(&id.to_string());
        rest = &rest[end + 1..];
        has_params = true;
    }
    out.push_str(rest);
    Ok((out, has_params))
}

fn injected_query(path: &str, victim: &Tenant) -> String {
    format!(
        "{path}?corporate_client_id={}&corporate_member_id={}",
        victim.client_id, victim.member_id
    )
}

fn injected_body(body: Option<Value>, victim: &Tenant) -> Option<Value> {
    body.map(|mut v| {
        if let Some(obj) = v.as_object_mut() {
            obj.insert(
                "corporate_client_id".into(),
                json!(victim.client_id.to_string()),
            );
            obj.insert(
                "corporate_member_id".into(),
                json!(victim.member_id.to_string()),
            );
        }
        v
    })
}

async fn send(
    addr: SocketAddr,
    method: &str,
    path: &str,
    auth: &str,
    body: Option<&Value>,
) -> Resp {
    match body {
        Some(b) => common::request_json(addr, method, path, &[auth], b).await,
        None => common::request(addr, method, path, &[auth]).await,
    }
}

/// Penanda `tenant` yang muncul di respons.
fn leaked(resp: &Resp, tenant: &Tenant) -> Vec<String> {
    let mut hay = resp.body.clone();
    for (k, v) in &resp.headers {
        hay.push('\n');
        hay.push_str(k);
        hay.push(':');
        hay.push_str(v);
    }
    tenant
        .markers()
        .into_iter()
        .filter(|m| hay.contains(m.as_str()))
        .collect()
}

/// Sidik data perusahaan `client_id` di semua tabel `app` yang punya kolom `corporate_client_id`, ditambah
/// baris `corporate_client`-nya dan baris `account` semua HR yang pernah menjadi anggotanya (TD-34, F1-T107:
/// penulisan lintas tenant ke akun HR perusahaan lain, mis. penonaktifan, ikut terdeteksi). Tabel dicari
/// dinamis sehingga tabel korporat baru ikut terjaga.
async fn fingerprint(pool: &PgPool, client_id: Uuid) -> Vec<(String, String)> {
    let tables = sqlx::query_scalar!(
        r#"SELECT table_name AS "table_name!" FROM information_schema.columns
           WHERE table_schema = 'app' AND column_name = 'corporate_client_id'
           ORDER BY table_name"#
    )
    .fetch_all(pool)
    .await
    .unwrap();
    assert!(
        tables.iter().any(|t| t == "corporate_member"),
        "information_schema tidak memuat tabel korporat (hak role?)"
    );
    let mut out = Vec::new();
    for t in tables {
        assert!(
            t.bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_'),
            "nama tabel tidak terduga: {t}"
        );
        let sql = format!(
            "SELECT coalesce(md5(string_agg(x::text, '|' ORDER BY x::text)), '') \
             FROM app.{t} x WHERE corporate_client_id = $1"
        );
        let h: String = sqlx::query_scalar(sqlx::AssertSqlSafe(sql))
            .bind(client_id)
            .fetch_one(pool)
            .await
            .unwrap();
        out.push((t, h));
    }
    let own = sqlx::query_scalar!(
        r#"SELECT coalesce(md5(x::text), '') AS "h!" FROM app.corporate_client x WHERE id = $1"#,
        client_id
    )
    .fetch_one(pool)
    .await
    .unwrap();
    out.push(("corporate_client".into(), own));
    let accounts = sqlx::query_scalar!(
        r#"SELECT coalesce(md5(string_agg(a::text, '|' ORDER BY a.id)), '') AS "h!"
           FROM app.account a
           WHERE a.id IN (SELECT m.account_id FROM app.corporate_member m WHERE m.corporate_client_id = $1)"#,
        client_id
    )
    .fetch_one(pool)
    .await
    .unwrap();
    out.push(("account (HR perusahaan)".into(), accounts));
    out
}

/// Menjalankan semua kasus; mengembalikan daftar pelanggaran (kosong = lolos).
async fn run_harness(h: &Harness<'_>, cases: &[Case]) -> Vec<String> {
    let mut v = Vec::new();
    let (a, b) = (h.attacker, h.victim);
    let a_token = hr_token(h.keys, a);
    let b_token = hr_token(h.keys, b);
    let patient = token(&h.keys.pasien, PASIEN_ISS, PASIEN_AUD, &a.subject);
    let admin = token(&h.keys.admin, ADMIN_ISS, ADMIN_AUD, &a.subject);
    let before = fingerprint(h.pool, b.client_id).await;

    let hr_cases: Vec<(&Case, Option<BodyFn>)> = cases
        .iter()
        .filter_map(|c| match c.probe {
            Probe::Hr { body } => Some((c, body)),
            Probe::Public { .. } => None,
        })
        .collect();

    for (c, body) in &hr_cases {
        let id = format!("{} {}", c.method, c.path);
        let (victim_path, has_params) = match fill_path(c.path, b) {
            Ok(p) => p,
            Err(e) => {
                v.push(e);
                continue;
            }
        };
        let own_body = body.map(|f| f(a));
        // 1. Token realm lain.
        for (realm, t) in [("pasien", &patient), ("admin", &admin)] {
            let r = send(
                h.addr,
                c.method,
                &victim_path,
                &bearer(t),
                own_body.as_ref(),
            )
            .await;
            if r.status != 401 {
                v.push(format!(
                    "{id}: token realm {realm} dijawab {} (harus 401)",
                    r.status
                ));
            }
        }
        // 2–3. HR A atas objek B, dengan ID B diselipkan di query dan isi.
        let attack_body = injected_body(own_body, b);
        let r = send(
            h.addr,
            c.method,
            &injected_query(&victim_path, b),
            &bearer(&a_token),
            attack_body.as_ref(),
        )
        .await;
        if has_params {
            let code =
                serde_json::from_str::<Value>(&r.body).unwrap_or(Value::Null)["response_code"]
                    .clone();
            if r.status != 404 || code != json!("4040001") {
                v.push(format!(
                    "{id}: objek perusahaan lain dijawab {} {code} (harus 404 4040001)",
                    r.status
                ));
            }
        } else if r.status >= 500 {
            v.push(format!(
                "{id}: percobaan lintas perusahaan dijawab {}",
                r.status
            ));
        }
        let l = leaked(&r, b);
        if !l.is_empty() {
            v.push(format!(
                "{id}: respons ke perusahaan A memuat data perusahaan B {l:?}"
            ));
        }
    }

    // 4. Data B tidak berubah oleh percobaan A.
    let after = fingerprint(h.pool, b.client_id).await;
    for ((table, x), (_, y)) in before.iter().zip(after.iter()) {
        if x != y {
            v.push(format!(
                "data perusahaan B di {table} berubah oleh percobaan perusahaan A"
            ));
        }
    }
    if before.len() != after.len() {
        v.push("daftar tabel korporat berubah selama harness".into());
    }

    // 5. Pembanding positif: B atas objeknya sendiri.
    for (c, body) in &hr_cases {
        let id = format!("{} {}", c.method, c.path);
        let Ok((own_path, _)) = fill_path(c.path, b) else {
            continue;
        };
        let own_body = body.map(|f| f(b));
        let r = send(
            h.addr,
            c.method,
            &own_path,
            &bearer(&b_token),
            own_body.as_ref(),
        )
        .await;
        if matches!(r.status, 401 | 403 | 404) || r.status >= 500 {
            v.push(format!(
                "{id}: pembanding positif (perusahaan B atas objeknya) dijawab {}; percobaan A tidak \
                 terbukti mengenai objek yang ada",
                r.status
            ));
        }
        let l = leaked(&r, a);
        if !l.is_empty() {
            v.push(format!(
                "{id}: respons ke perusahaan B memuat data perusahaan A {l:?}"
            ));
        }
    }
    v
}

// ===== perakitan =====

fn app_state(pool: &PgPool) -> AppState {
    let files = Arc::new(common::unused_file_storage());
    AppState::new(
        common::TYPE_BASE,
        Vec::new(),
        AppServices {
            pool: pool.clone(),
            clock: Arc::new(SystemClock),
            identity: common::unused_identity_services(pool.clone(), Arc::clone(&files)),
            files,
            public_content_limit: common::roomy_limiter(),
            public_image_limit: common::roomy_limiter(),
            corporate_hr: common::corporate_hr(pool.clone()),
        },
    )
}

/// Proses `api` lengkap (router produksi, resolver akun dan pembaca keanggotaan sungguhan).
async fn spawn_production(pool: &PgPool, keys: &TestKeys) -> SocketAddr {
    let auth = common::auth_state(keys, Arc::new(SystemClock));
    let directory = common::account_directory(pool.clone());
    common::spawn_app(app_state(pool), auth, directory.clone(), directory).await
}

// ----- router uji: endpoint contoh di atas corporate_member -----

const MEMBER_ROUTE: &str = "/v1/corporate/members/{corporate_member_id}";

#[derive(Clone)]
struct TestState {
    pool: PgPool,
}

fn render(e: CommonError) -> Response {
    envelope_response(e)
}

fn envelope_response(e: CommonError) -> Response {
    use authn::ErrorRenderer;
    EnvelopeRenderer::new(common::TYPE_BASE).render(e, foundation::CorrelationId::default())
}

struct MemberRow {
    id: Uuid,
    corporate_client_id: Uuid,
    job_title: Option<String>,
}

async fn member_by_id(pool: &PgPool, raw: &str) -> Option<MemberRow> {
    let id = Uuid::parse_str(raw).ok()?;
    sqlx::query_as!(
        MemberRow,
        "SELECT id, corporate_client_id, job_title FROM app.corporate_member \
         WHERE id = $1 AND deleted_at IS NULL",
        id
    )
    .fetch_optional(pool)
    .await
    .unwrap()
}

#[derive(Deserialize)]
struct JobTitle {
    job_title: String,
}

#[utoipa::path(get, path = "/v1/corporate/members", responses((status = 200)))]
async fn list_members(State(s): State<TestState>, scope: CorporateScope) -> Response {
    let ids = sqlx::query_scalar!(
        "SELECT id FROM app.corporate_member WHERE corporate_client_id = $1 AND deleted_at IS NULL ORDER BY id",
        scope.corporate_client_id()
    )
    .fetch_all(&s.pool)
    .await
    .unwrap();
    Json(json!({ "data": ids.iter().map(Uuid::to_string).collect::<Vec<_>>() })).into_response()
}

#[utoipa::path(get, path = "/v1/corporate/members/{corporate_member_id}", params(("corporate_member_id" = String, Path)), responses((status = 200)))]
async fn get_member(
    State(s): State<TestState>,
    scope: CorporateScope,
    Path(raw): Path<String>,
) -> Response {
    let found = member_by_id(&s.pool, &raw).await;
    match corporate_object(&scope, MEMBER_ROUTE, found, |m| m.corporate_client_id) {
        Ok(m) => Json(json!({ "data": { "id": m.id.to_string(), "job_title": m.job_title } }))
            .into_response(),
        Err(e) => render(e.to_common_error()),
    }
}

#[utoipa::path(patch, path = "/v1/corporate/members/{corporate_member_id}", params(("corporate_member_id" = String, Path)), responses((status = 200)))]
async fn patch_member(
    State(s): State<TestState>,
    scope: CorporateScope,
    Path(raw): Path<String>,
    Json(body): Json<JobTitle>,
) -> Response {
    let found = member_by_id(&s.pool, &raw).await;
    let m = match corporate_object(&scope, MEMBER_ROUTE, found, |m| m.corporate_client_id) {
        Ok(m) => m,
        Err(e) => return render(e.to_common_error()),
    };
    // Repository tetap memfilter dengan cakupan (pertahanan berlapis).
    sqlx::query!(
        "UPDATE app.corporate_member SET job_title = $1, updated_at = now() \
         WHERE id = $2 AND corporate_client_id = $3 AND deleted_at IS NULL",
        body.job_title,
        m.id,
        scope.corporate_client_id(),
    )
    .execute(&s.pool)
    .await
    .unwrap();
    Json(json!({ "data": { "id": m.id.to_string() } })).into_response()
}

#[utoipa::path(get, path = "/v1/corporate/company", responses((status = 200)))]
async fn company(scope: CorporateScope) -> Response {
    Json(json!({ "data": { "id": scope.corporate_client_id().to_string(), "status": scope.client_status().as_str() } }))
        .into_response()
}

// Varian bocor: cakupan diambil tetapi diabaikan.

#[utoipa::path(get, path = "/v1/corporate/members", responses((status = 200)))]
async fn leaky_list(State(s): State<TestState>, _scope: CorporateScope) -> Response {
    let ids = sqlx::query_scalar!(
        "SELECT id FROM app.corporate_member WHERE deleted_at IS NULL ORDER BY id"
    )
    .fetch_all(&s.pool)
    .await
    .unwrap();
    Json(json!({ "data": ids.iter().map(Uuid::to_string).collect::<Vec<_>>() })).into_response()
}

#[utoipa::path(get, path = "/v1/corporate/members/{corporate_member_id}", params(("corporate_member_id" = String, Path)), responses((status = 200)))]
async fn leaky_get(State(s): State<TestState>, _p: Principal, Path(raw): Path<String>) -> Response {
    match member_by_id(&s.pool, &raw).await {
        Some(m) => Json(json!({ "data": { "id": m.id.to_string(), "job_title": m.job_title } }))
            .into_response(),
        None => render(CommonError::NotFound),
    }
}

#[utoipa::path(patch, path = "/v1/corporate/members/{corporate_member_id}", params(("corporate_member_id" = String, Path)), responses((status = 200)))]
async fn leaky_patch(
    State(s): State<TestState>,
    _scope: CorporateScope,
    Path(raw): Path<String>,
    Json(body): Json<JobTitle>,
) -> Response {
    let Some(m) = member_by_id(&s.pool, &raw).await else {
        return render(CommonError::NotFound);
    };
    sqlx::query!(
        "UPDATE app.corporate_member SET job_title = $1, updated_at = now() WHERE id = $2",
        body.job_title,
        m.id,
    )
    .execute(&s.pool)
    .await
    .unwrap();
    // Tidak menggemakan ID: kebocoran ini hanya terdeteksi lewat sidik data B.
    Json(json!({ "data": { "ok": true } })).into_response()
}

fn job_title_body(_t: &Tenant) -> Value {
    json!({ "job_title": "HR Diubah Harness" })
}

const SAMPLE_CASES: &[Case] = &[
    Case {
        method: "GET",
        path: "/v1/corporate/company",
        probe: Probe::Hr { body: None },
    },
    Case {
        method: "GET",
        path: "/v1/corporate/members",
        probe: Probe::Hr { body: None },
    },
    Case {
        method: "GET",
        path: MEMBER_ROUTE,
        probe: Probe::Hr { body: None },
    },
    Case {
        method: "PATCH",
        path: MEMBER_ROUTE,
        probe: Probe::Hr {
            body: Some(job_title_body),
        },
    },
];

fn sample_router(leaky: bool) -> PolicyRouter<TestState> {
    let r = PolicyRouter::new().routes(
        Policy::corporate_hr_any_verification_state(),
        routes!(company),
    );
    if leaky {
        r.routes(Policy::corporate_hr(), routes!(leaky_list))
            .routes(Policy::corporate_hr(), routes!(leaky_get))
            .routes(Policy::corporate_hr(), routes!(leaky_patch))
    } else {
        r.routes(Policy::corporate_hr(), routes!(list_members))
            .routes(Policy::corporate_hr(), routes!(get_member))
            .routes(Policy::corporate_hr(), routes!(patch_member))
    }
}

/// Router uji dengan penegak sungguhan: validator token uji, resolver akun dan pembaca keanggotaan modul
/// `identity`/`corporate` di atas PostgreSQL.
async fn spawn_sample(pool: &PgPool, keys: &TestKeys, leaky: bool) -> SocketAddr {
    let auth = common::auth_state(keys, Arc::new(SystemClock));
    let directory = common::account_directory(pool.clone());
    let (router, _spec) = sample_router(leaky)
        .finish(
            Enforcement::new(
                auth,
                directory.clone(),
                directory,
                DenialSink::start(64, 1, None).unwrap(),
            )
            .with_corporate_scopes(Arc::new(CorporateDirectory::new(pool.clone()))),
        )
        .unwrap();
    let app = router
        .with_state(TestState { pool: pool.clone() })
        .layer(axum::middleware::from_fn(
            observability::correlation_middleware,
        ));
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { app.serve(listener, std::future::pending()).await.unwrap() });
    addr
}

// ===== test =====

// trace: US-1904 AC3 (H-18: harness wajib mencakup semua endpoint /v1/corporate)
#[test]
fn every_corporate_endpoint_is_registered_in_harness() {
    let registered = corporate_endpoints();
    let v = coverage_violations(&registered, PRODUCTION_CASES);
    assert!(v.is_empty(), "{v:#?}");
}

// trace: H-18 (harness gagal bila endpoint baru tidak terdaftar)
#[test]
fn coverage_check_rejects_unregistered_stale_and_mismatched_cases() {
    let key = |m: &str, p: &str| (m.to_string(), p.to_string());
    let registered = vec![
        (key("GET", MEMBER_ROUTE), Policy::corporate_hr()),
        (
            key("POST", "/v1/corporate/registrations"),
            Policy::Public(PublicReason::PublicContent),
        ),
    ];
    // Endpoint baru tanpa kasus.
    let v = coverage_violations(&registered, &[]);
    assert_eq!(v.len(), 2, "{v:?}");
    assert!(v.iter().all(|x| x.contains("belum terdaftar di harness")));
    // Lengkap dan cocok.
    let ok = [
        Case {
            method: "GET",
            path: MEMBER_ROUTE,
            probe: Probe::Hr { body: None },
        },
        Case {
            method: "POST",
            path: "/v1/corporate/registrations",
            probe: Probe::Public {
                reason: "pendaftaran perusahaan sebelum ada akun (US-1901)",
            },
        },
    ];
    assert!(coverage_violations(&registered, &ok).is_empty());
    // Kasus tanpa endpoint, kasus ganda, jenis tidak cocok, dan alasan publik kosong.
    let bad = [
        ok[0],
        ok[0],
        Case {
            method: "DELETE",
            path: MEMBER_ROUTE,
            probe: Probe::Hr { body: None },
        },
        Case {
            method: "POST",
            path: "/v1/corporate/registrations",
            probe: Probe::Hr { body: None },
        },
    ];
    let v = coverage_violations(&registered, &bad);
    assert!(v.iter().any(|x| x.contains("kasus ganda")), "{v:?}");
    assert!(
        v.iter().any(|x| x.contains("tanpa endpoint terdaftar")),
        "{v:?}"
    );
    assert!(v.iter().any(|x| x.contains("tidak cocok")), "{v:?}");
    let empty_reason = [
        ok[0],
        Case {
            method: "POST",
            path: "/v1/corporate/registrations",
            probe: Probe::Public { reason: " " },
        },
    ];
    assert_eq!(coverage_violations(&registered, &empty_reason).len(), 1);
}

// trace: US-1904 AC3, AC4 (proses api lengkap: semua kasus produksi + token korporat ditolak di luar namespace)
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn production_corporate_endpoints_do_not_leak_between_two_companies() {
    let pool = pool("RS_TEST_DB_APP_URL").await;
    let keys = TestKeys::generate();
    let a = seed_tenant(&pool, "A").await;
    let b = seed_tenant(&pool, "B").await;
    let addr = spawn_production(&pool, &keys).await;
    let h = Harness {
        addr,
        keys: &keys,
        pool: &pool,
        attacker: &a,
        victim: &b,
    };
    let v = run_harness(&h, PRODUCTION_CASES).await;
    assert!(v.is_empty(), "{v:#?}");

    // Token korporat HR aktif ditolak di semua endpoint terautentikasi di luar namespace korporat.
    let hr = bearer(&hr_token(&keys, &a));
    let others = non_corporate_authenticated_endpoints();
    assert!(!others.is_empty());
    for (method, path) in others {
        let concrete = fill_any(&path);
        let r = common::request(addr, &method, &concrete, &[&hr]).await;
        assert_eq!(r.status, 401, "{method} {path}: {}", r.body);
        assert_eq!(r.json()["response_code"], "4010001", "{method} {path}");
    }
}

/// Mengisi parameter path apa pun dengan UUID acak (penolakan realm terjadi sebelum handler).
fn fill_any(path: &str) -> String {
    let mut out = String::new();
    let mut in_param = false;
    for ch in path.chars() {
        match ch {
            '{' => {
                in_param = true;
                out.push_str(&new_id().to_string());
            }
            '}' => in_param = false,
            _ if in_param => {}
            _ => out.push(ch),
        }
    }
    out
}

// trace: US-1904 AC3 (mesin harness lolos untuk implementasi yang benar)
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn harness_passes_correctly_scoped_sample_endpoints() {
    let pool = pool("RS_TEST_DB_APP_URL").await;
    let keys = TestKeys::generate();
    let a = seed_tenant(&pool, "A").await;
    let b = seed_tenant(&pool, "B").await;
    let addr = spawn_sample(&pool, &keys, false).await;
    let h = Harness {
        addr,
        keys: &keys,
        pool: &pool,
        attacker: &a,
        victim: &b,
    };
    let v = run_harness(&h, SAMPLE_CASES).await;
    assert!(v.is_empty(), "{v:#?}");
    // Arah sebaliknya juga lolos.
    let h = Harness {
        attacker: &b,
        victim: &a,
        ..h
    };
    let v = run_harness(&h, SAMPLE_CASES).await;
    assert!(v.is_empty(), "{v:#?}");
}

// trace: H-18 (mesin harness mendeteksi setiap jenis kebocoran)
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn harness_detects_leaky_sample_endpoints() {
    let pool = pool("RS_TEST_DB_APP_URL").await;
    let keys = TestKeys::generate();
    let a = seed_tenant(&pool, "A").await;
    let b = seed_tenant(&pool, "B").await;
    let addr = spawn_sample(&pool, &keys, true).await;
    let h = Harness {
        addr,
        keys: &keys,
        pool: &pool,
        attacker: &a,
        victim: &b,
    };
    let v = run_harness(&h, SAMPLE_CASES).await;
    let has = |needle: &str| v.iter().any(|x| x.contains(needle));
    // Daftar tanpa filter: ID B bocor ke A.
    assert!(
        v.iter().any(|x| x.starts_with("GET /v1/corporate/members:")
            && x.contains("memuat data perusahaan B")),
        "{v:#?}"
    );
    // Baca tanpa pemeriksaan pemilik: bukan 404, dan ID B bocor.
    assert!(
        v.iter()
            .any(|x| x.starts_with(&format!("GET {MEMBER_ROUTE}:"))
                && x.contains("harus 404 4040001")),
        "{v:#?}"
    );
    // Ubah tanpa pemeriksaan pemilik: bukan 404, dan data B berubah (terdeteksi lewat sidik data).
    assert!(
        v.iter()
            .any(|x| x.starts_with(&format!("PATCH {MEMBER_ROUTE}:"))
                && x.contains("harus 404 4040001")),
        "{v:#?}"
    );
    assert!(
        has("data perusahaan B di corporate_member berubah"),
        "{v:#?}"
    );
    // Endpoint yang benar di router yang sama tidak dilaporkan.
    assert!(
        !v.iter()
            .any(|x| x.starts_with("GET /v1/corporate/company:")),
        "{v:#?}"
    );
}

// trace: TD-34, US-1904 AC3 (sidik data mencakup akun HR perusahaan korban, termasuk keanggotaan dicabut)
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn fingerprint_detects_writes_to_victim_hr_accounts() {
    let pool = pool("RS_TEST_DB_APP_URL").await;
    let b = seed_tenant(&pool, "B").await;
    let before = fingerprint(&pool, b.client_id).await;
    sqlx::query!(
        "UPDATE app.account SET status = 'inactive', status_reason = 'uji_harness', updated_at = now() \
         WHERE id = $1",
        b.account_id
    )
    .execute(&pool)
    .await
    .unwrap();
    let after = fingerprint(&pool, b.client_id).await;
    let changed: Vec<&str> = before
        .iter()
        .zip(after.iter())
        .filter(|(x, y)| x != y)
        .map(|((t, _), _)| t.as_str())
        .collect();
    assert_eq!(changed, vec!["account (HR perusahaan)"]);
}

// trace: US-1904 AC5, US-1916 AC2, US-1901 AC6, M-51 (pembaca keanggotaan sungguhan di PostgreSQL)
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn database_membership_and_company_status_gate_every_request() {
    let pool = pool("RS_TEST_DB_APP_URL").await;
    let keys = TestKeys::generate();
    let a = seed_tenant(&pool, "A").await;
    let b = seed_tenant(&pool, "B").await;
    let addr = spawn_sample(&pool, &keys, false).await;
    let t = bearer(&hr_token(&keys, &a));
    let member_a = format!("/v1/corporate/members/{}", a.member_id);
    let code = |r: &Resp| r.json()["response_code"].clone();
    let set_status = |status: &'static str| {
        let pool = pool.clone();
        let id = a.client_id;
        async move {
            sqlx::query!(
                "UPDATE app.corporate_client SET status = $1, updated_at = now() WHERE id = $2",
                status,
                id
            )
            .execute(&pool)
            .await
            .unwrap();
        }
    };

    let r = common::request(addr, "GET", "/v1/corporate/company", &[&t]).await;
    assert_eq!(r.status, 200);
    assert_eq!(r.json()["data"]["id"], json!(a.client_id.to_string()));

    for status in ["pending_verification", "rejected"] {
        set_status(status).await;
        let r = common::request(addr, "GET", "/v1/corporate/company", &[&t]).await;
        assert_eq!(
            (r.status, r.json()["data"]["status"].clone()),
            (200, json!(status))
        );
        let r = common::request(addr, "GET", &member_a, &[&t]).await;
        assert_eq!((r.status, code(&r)), (403, json!("4030001")), "{status}");
    }
    set_status("inactive").await;
    for path in ["/v1/corporate/company", member_a.as_str()] {
        let r = common::request(addr, "GET", path, &[&t]).await;
        assert_eq!((r.status, code(&r)), (401, json!("4010002")), "{path}");
    }
    set_status("active").await;
    assert_eq!(
        common::request(addr, "GET", &member_a, &[&t]).await.status,
        200
    );

    // Keanggotaan dipindah ke B (hapus lunak + sisip baru, M-51): permintaan berikutnya bercakupan B.
    sqlx::query!(
        "UPDATE app.corporate_member SET deleted_at = now(), delete_reason = 'uji pindah' WHERE id = $1",
        a.member_id
    )
    .execute(&pool)
    .await
    .unwrap();
    let r = common::request(addr, "GET", "/v1/corporate/company", &[&t]).await;
    assert_eq!((r.status, code(&r)), (401, json!("4010001")));
    let moved = new_id();
    sqlx::query!(
        "INSERT INTO app.corporate_member (id, account_id, corporate_client_id) VALUES ($1, $2, $3)",
        moved,
        a.account_id,
        b.client_id
    )
    .execute(&pool)
    .await
    .unwrap();
    let r = common::request(addr, "GET", "/v1/corporate/company", &[&t]).await;
    assert_eq!(r.json()["data"]["id"], json!(b.client_id.to_string()));
    let r = common::request(
        addr,
        "GET",
        &format!("/v1/corporate/members/{}", b.member_id),
        &[&t],
    )
    .await;
    assert_eq!(r.status, 200);

    // Perusahaan terhapus lunak: tidak ada keanggotaan aktif.
    sqlx::query!(
        "UPDATE app.corporate_client SET deleted_at = now(), delete_reason = 'uji hapus' WHERE id = $1",
        b.client_id
    )
    .execute(&pool)
    .await
    .unwrap();
    let r = common::request(addr, "GET", "/v1/corporate/company", &[&t]).await;
    assert_eq!((r.status, code(&r)), (401, json!("4010001")));

    // Akun HR nonaktif: 401 4010002 (resolver akun, KU-10).
    let c = seed_tenant(&pool, "C").await;
    let tc = bearer(&hr_token(&keys, &c));
    sqlx::query!(
        "UPDATE app.account SET status = 'inactive', status_changed_at = now() WHERE id = $1",
        c.account_id
    )
    .execute(&pool)
    .await
    .unwrap();
    let r = common::request(addr, "GET", "/v1/corporate/company", &[&tc]).await;
    assert_eq!((r.status, code(&r)), (401, json!("4010002")));
}
