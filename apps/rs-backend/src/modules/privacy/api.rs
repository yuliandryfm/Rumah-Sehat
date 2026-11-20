//! Antarmuka publik modul `privacy` untuk modul lain (satu-satunya jalan; tanpa SQL lintas modul).
//!
//! - Teks: [`active_consent_text`] membaca versi berlaku per jenis ([`ConsentType`]) menurut `Clock` aplikasi;
//!   [`check_selection`] memeriksa versi dan butir yang dipilih (US-101 AC4, US-104 AC3, AC5), tidak
//!   bergantung pada jenis pihak.
//! - Bukti: [`record_granted_consent`] menulis `consent_record` di transaksi pemanggil dan mengembalikan
//!   event E-13 untuk ditulis pemanggil di transaksi yang sama (US-104 AC2). Pihak diberikan lewat
//!   [`ConsentSubject`]; kini hanya pasien (skema A3.3), varian lain menunggu perluasan skema.
//! - Gate production (KEPUTUSAN-289): [`ConsentTextReadiness`] membuat proses `api` dengan
//!   `DEPLOY_ENV=production` tidak siap selama versi teks pasien yang berlaku masih draf atau tidak ada.

pub use super::domain::{
    ConsentContact, ConsentContent, ConsentItem, ConsentSection, ConsentType, ContentProblem,
    REQUIRED_SECTIONS, SelectionCheck, check_selection, parse_content,
};
pub use super::error::PrivacyError;
pub use super::service::{
    ActiveConsentText, ConsentSubject, ConsentTextReadiness, active_consent_text, is_final_text,
    record_granted_consent,
};
