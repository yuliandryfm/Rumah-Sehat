//! Handler profil dan alamat pasien (`/patient/v1`, F1-T56, US-105). Setiap rute berkebijakan akun pasien
//! (`policy.rs`); token, status akun, dan sesi ditegakkan middleware crate `authz` sebelum handler.
//! Handler hanya mengurai DTO, memanggil satu fungsi `api`, dan memetakan hasil ke envelope ADR-023.
//! Profil selalu milik akun pemanggil (tanpa ID di path); alamat dicari per akun pemanggil, sehingga
//! ID alamat orang lain = `404` yang sama dengan ID yang tidak ada (KU-03).

use super::api::{
    self as profile, ProfileError, create_address, delete_address, get_profile, list_addresses,
    update_address, update_profile,
};
use super::http::{body_error, care_recipient_id as parse_id, fail};
use super::policy;
use super::profile_dto::{
    AddressListCreatedEnvelope, AddressListDto, AddressListEnvelope, AddressRequest, ProfileDto,
    ProfileEnvelope, UpdateProfileRequest,
};
use crate::app::AppState;
use crate::http_support::ApiSuccess;
use crate::openapi::ErrorEnvelope;
use authz::{PolicyRouter, Principal};
use axum::Extension;
use axum::extract::rejection::JsonRejection;
use axum::extract::{Json, Path, State};
use axum::response::{IntoResponse, Response};
use envelope::Success;
use foundation::CorrelationId;
use utoipa_axum::routes;

/// Pola rute untuk security log dan audit (bukan URL berisi ID).
const PROFILE_ROUTE: &str = "/patient/v1/profile";
const ADDRESSES_ROUTE: &str = "/patient/v1/addresses";
const ADDRESS_ROUTE: &str = "/patient/v1/addresses/{address_id}";

pub(super) fn router() -> PolicyRouter<AppState> {
    PolicyRouter::new()
        .routes(policy::patient(), routes!(profile_get, profile_update))
        .routes(policy::patient(), routes!(addresses_list, address_create))
        .routes(policy::patient(), routes!(address_update, address_delete))
}

fn profile_response(
    state: &AppState,
    cid: CorrelationId,
    r: Result<profile::ProfileView, ProfileError>,
) -> Response {
    let dto =
        r.and_then(|v| ProfileDto::from_view(v).map_err(|_| ProfileError::Internal("profile_dto")));
    match dto {
        Ok(d) => ApiSuccess(Success::ok(d).with_correlation_id(cid)).into_response(),
        Err(e) => fail(state, cid, e),
    }
}

fn list_response(
    state: &AppState,
    cid: CorrelationId,
    created: bool,
    r: Result<Vec<profile::AddressView>, ProfileError>,
) -> Response {
    match r {
        Ok(v) => {
            let dto = AddressListDto::from_views(v);
            let success = if created {
                Success::created(dto)
            } else {
                Success::ok(dto)
            };
            ApiSuccess(success.with_correlation_id(cid)).into_response()
        }
        Err(e) => fail(state, cid, e),
    }
}

/// Profil data diri pemilik akun (US-105 AC1) beserta status identitas (AB-05).
#[utoipa::path(
    get,
    path = "/patient/v1/profile",
    tag = "profile",
    operation_id = "profile_get",
    security(("bearer" = [])),
    responses(
        (status = 200, description = "Profil.", body = ProfileEnvelope, content_type = "application/json"),
        (status = 401, description = "Belum login, token tidak sah, sesi dicabut, atau akun nonaktif.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 404, description = "Akun belum memiliki profil.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 503, description = "Layanan tidak tersedia.", body = ErrorEnvelope, content_type = "application/problem+json")
    )
)]
async fn profile_get(
    State(state): State<AppState>,
    Extension(cid): Extension<CorrelationId>,
    principal: Principal,
) -> Response {
    let r = get_profile(state.identity(), &principal).await;
    profile_response(&state, cid, r)
}

/// Simpan profil (US-105 AC1, AC1a, AC1b, AC3). Mengubah nama atau tanggal lahir saat identitas
/// `approved` mengembalikan status ke `pending` (menunggu verifikasi ulang); perubahan lain tidak
/// memengaruhi status. Simpan tanpa perubahan tidak mencatat event.
#[utoipa::path(
    put,
    path = "/patient/v1/profile",
    tag = "profile",
    operation_id = "profile_update",
    security(("bearer" = [])),
    request_body(content = UpdateProfileRequest, content_type = "application/json"),
    responses(
        (status = 200, description = "Profil tersimpan; status identitas sesudahnya.", body = ProfileEnvelope, content_type = "application/json"),
        (status = 400, description = "Badan permintaan tidak dapat dibaca.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 401, description = "Belum login, token tidak sah, sesi dicabut, atau akun nonaktif.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 404, description = "Akun belum memiliki profil.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 409, description = "Transisi status identitas kalah oleh keputusan lain yang tersimpan lebih dulu (4090001).", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 415, description = "Badan bukan JSON.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 422, description = "Isian tidak valid per field (4220001): nama, tanggal lahir (sah, rentang, usia minimum), jenis kelamin, nomor HP.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 503, description = "Layanan tidak tersedia.", body = ErrorEnvelope, content_type = "application/problem+json")
    )
)]
async fn profile_update(
    State(state): State<AppState>,
    Extension(cid): Extension<CorrelationId>,
    principal: Principal,
    body: Result<Json<UpdateProfileRequest>, JsonRejection>,
) -> Response {
    let Json(req) = match body {
        Ok(b) => b,
        Err(e) => return fail(&state, cid, body_error(&e)),
    };
    let r = update_profile(state.identity(), &principal, PROFILE_ROUTE, &req.input()).await;
    profile_response(&state, cid, r)
}

/// Daftar alamat aktif (US-105 AC2): alamat utama pertama; alamat terhapus tidak tampil (KU-04).
#[utoipa::path(
    get,
    path = "/patient/v1/addresses",
    tag = "profile",
    operation_id = "address_list",
    security(("bearer" = [])),
    responses(
        (status = 200, description = "Daftar alamat.", body = AddressListEnvelope, content_type = "application/json"),
        (status = 401, description = "Belum login, token tidak sah, sesi dicabut, atau akun nonaktif.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 404, description = "Akun belum memiliki profil.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 503, description = "Layanan tidak tersedia.", body = ErrorEnvelope, content_type = "application/problem+json")
    )
)]
async fn addresses_list(
    State(state): State<AppState>,
    Extension(cid): Extension<CorrelationId>,
    principal: Principal,
) -> Response {
    let r = list_addresses(state.identity(), &principal).await;
    list_response(&state, cid, false, r)
}

/// Tambah alamat (US-105 AC1a, AC2). Alamat pertama otomatis utama; `is_default = true` melepas utama
/// yang lama. Jumlah alamat aktif paling banyak 20 (batas teknis awal). Respons memuat daftar alamat
/// sesudahnya.
#[utoipa::path(
    post,
    path = "/patient/v1/addresses",
    tag = "profile",
    operation_id = "address_create",
    security(("bearer" = [])),
    request_body(content = AddressRequest, content_type = "application/json"),
    responses(
        (status = 201, description = "Alamat ditambahkan; daftar alamat sesudahnya.", body = AddressListCreatedEnvelope, content_type = "application/json"),
        (status = 400, description = "Badan permintaan tidak dapat dibaca.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 401, description = "Belum login, token tidak sah, sesi dicabut, atau akun nonaktif.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 404, description = "Akun belum memiliki profil.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 415, description = "Badan bukan JSON.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 422, description = "Isian tidak valid per field (4220001), atau jumlah alamat melebihi batas.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 503, description = "Layanan tidak tersedia.", body = ErrorEnvelope, content_type = "application/problem+json")
    )
)]
async fn address_create(
    State(state): State<AppState>,
    Extension(cid): Extension<CorrelationId>,
    principal: Principal,
    body: Result<Json<AddressRequest>, JsonRejection>,
) -> Response {
    let Json(req) = match body {
        Ok(b) => b,
        Err(e) => return fail(&state, cid, body_error(&e)),
    };
    let r = create_address(state.identity(), &principal, ADDRESSES_ROUTE, &req.input()).await;
    list_response(&state, cid, true, r)
}

/// Ubah alamat milik pemanggil (penggantian penuh). `is_default = true` menjadikannya utama; `false` pada
/// alamat utama ditolak (`4220001`, field `is_default`). ID alamat yang tidak ada dan milik pasien lain
/// dijawab `404` yang sama (KU-03) dan dicatat sebagai event E-10 gagal.
#[utoipa::path(
    put,
    path = "/patient/v1/addresses/{address_id}",
    tag = "profile",
    operation_id = "address_update",
    security(("bearer" = [])),
    params(("address_id" = String, Path, description = "ID alamat (UUID).")),
    request_body(content = AddressRequest, content_type = "application/json"),
    responses(
        (status = 200, description = "Alamat tersimpan; daftar alamat sesudahnya.", body = AddressListEnvelope, content_type = "application/json"),
        (status = 400, description = "Badan permintaan tidak dapat dibaca.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 401, description = "Belum login, token tidak sah, sesi dicabut, atau akun nonaktif.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 404, description = "Alamat tidak ada atau bukan milik akun ini (KU-03).", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 415, description = "Badan bukan JSON.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 422, description = "Isian tidak valid per field (4220001).", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 503, description = "Layanan tidak tersedia.", body = ErrorEnvelope, content_type = "application/problem+json")
    )
)]
async fn address_update(
    State(state): State<AppState>,
    Extension(cid): Extension<CorrelationId>,
    principal: Principal,
    Path(raw): Path<String>,
    body: Result<Json<AddressRequest>, JsonRejection>,
) -> Response {
    let id = match parse_id(&raw) {
        Ok(id) => id,
        Err(e) => return fail(&state, cid, e),
    };
    let Json(req) = match body {
        Ok(b) => b,
        Err(e) => return fail(&state, cid, body_error(&e)),
    };
    let r = update_address(
        state.identity(),
        &principal,
        ADDRESS_ROUTE,
        id,
        &req.input(),
    )
    .await;
    list_response(&state, cid, false, r)
}

/// Hapus alamat milik pemanggil (soft delete, ADR-003). Bila yang dihapus alamat utama, alamat aktif
/// tertua berikutnya menjadi utama. Respons memuat daftar alamat sesudahnya.
#[utoipa::path(
    delete,
    path = "/patient/v1/addresses/{address_id}",
    tag = "profile",
    operation_id = "address_delete",
    security(("bearer" = [])),
    params(("address_id" = String, Path, description = "ID alamat (UUID).")),
    responses(
        (status = 200, description = "Alamat dihapus; daftar alamat sesudahnya.", body = AddressListEnvelope, content_type = "application/json"),
        (status = 401, description = "Belum login, token tidak sah, sesi dicabut, atau akun nonaktif.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 404, description = "Alamat tidak ada atau bukan milik akun ini (KU-03).", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 503, description = "Layanan tidak tersedia.", body = ErrorEnvelope, content_type = "application/problem+json")
    )
)]
async fn address_delete(
    State(state): State<AppState>,
    Extension(cid): Extension<CorrelationId>,
    principal: Principal,
    Path(raw): Path<String>,
) -> Response {
    let id = match parse_id(&raw) {
        Ok(id) => id,
        Err(e) => return fail(&state, cid, e),
    };
    let r = delete_address(state.identity(), &principal, ADDRESS_ROUTE, id).await;
    list_response(&state, cid, false, r)
}
