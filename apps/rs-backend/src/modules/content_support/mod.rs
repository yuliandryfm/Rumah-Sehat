//! Modul `content_support` (kode modul 11, "Konten dan layanan pelanggan", docs/04-arsitektur.md §3).
//! F1-T92: CMS backoffice (US-1101 AC1-AC4, US-1802 AC1-AC5) atas tabel `cms_content` (A15.4): draf,
//! pratinjau, jadwal, arsip, soft delete dan pulihkan (E-16), kebijakan sub-peran Konten.
//! F1-T111: aturan render aman isi Markdown divalidasi saat simpan (`markdown`, M-40).
//! F1-T94: konten situs yang sedang tayang tanpa login (`/v1/public/content`), dengan batas laju.
//! Submodul privat (coding-standards §1.3); modul lain tidak memakai modul ini.

mod domain;
mod dto;
mod error;
mod http;
mod markdown;
mod policy;
mod repo;
mod service;

use crate::app::AppState;
use authz::PolicyRouter;

/// Rute backoffice CMS beserta metadata OpenAPI-nya (`openapi/backoffice.json`), masing-masing dengan
/// kebijakannya (KEPUTUSAN-127).
pub(crate) fn backoffice_router() -> PolicyRouter<AppState> {
    http::router()
}

/// Rute konten publik tanpa login (`openapi/public.json`, F1-T94): hanya konten Tayang kanal situs.
pub(crate) fn public_router() -> PolicyRouter<AppState> {
    http::public_router()
}
