//! Registri konteks enkripsi, jenis wadah, dan sistem (skema fisik §7.1 AAD bidang 2–3, DD §0.7).
//!
//! Konteks enkripsi adalah pengenal **logis** kolom terenkripsi (misalnya `identity_document.nik_enc`),
//! bukan nama fisik dari katalog, sehingga penggantian nama tabel lewat expand/contract tidak merusak
//! dekripsi. Konteks hanya dapat diambil dari registri ini (field privat), dan setiap konteks terikat
//! ke satu jenis wadah DEK; pemakaian DEK wadah lain ditolak (coding-standards §5.5).

use std::fmt;

/// Sistem pemilik data (AAD bidang 3). Kunci dan DEK tidak pernah dipakai lintas sistem.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum System {
    /// Ruang Sehat (`rs`).
    RuangSehat,
    /// RME minimalis (`rme`).
    Rme,
}

impl System {
    /// Kode ASCII yang masuk AAD.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RuangSehat => "rs",
            Self::Rme => "rme",
        }
    }
}

/// Jenis wadah DEK database aplikasi Ruang Sehat: nilai `data_key.container_type` (DD §0.7, A19.7;
/// CHECK `ck_data_key_container_type` di migrasi F1-T17). Wadah RME ditambahkan bersama task RME.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContainerType {
    /// Percakapan satu sesi konsultasi.
    ConsultationChat,
    /// Dokumentasi klinis satu sesi konsultasi.
    ConsultationClinical,
    /// Satu hasil lab (semua versinya).
    LabResult,
    /// Satu dokumen identitas.
    IdentityDocument,
    /// Satu klaim.
    Claim,
    /// Satu klien integrasi rekanan.
    IntegrationClient,
    /// Satu rekening rekanan.
    PartnerBankAccount,
    /// Satu tiket keluhan.
    ComplaintTicket,
    /// Satu permintaan hak subjek data.
    DataSubjectRequest,
    /// Satu entitas induk riwayat status tanpa wadah sendiri.
    StatusNote,
}

impl ContainerType {
    /// Semua nilai, urut sesuai CHECK di migrasi.
    pub const ALL: [Self; 10] = [
        Self::ConsultationChat,
        Self::ConsultationClinical,
        Self::LabResult,
        Self::IdentityDocument,
        Self::Claim,
        Self::IntegrationClient,
        Self::PartnerBankAccount,
        Self::ComplaintTicket,
        Self::DataSubjectRequest,
        Self::StatusNote,
    ];

    /// Nilai teks di kolom `data_key.container_type`.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ConsultationChat => "consultation_chat",
            Self::ConsultationClinical => "consultation_clinical",
            Self::LabResult => "lab_result",
            Self::IdentityDocument => "identity_document",
            Self::Claim => "claim",
            Self::IntegrationClient => "integration_client",
            Self::PartnerBankAccount => "partner_bank_account",
            Self::ComplaintTicket => "complaint_ticket",
            Self::DataSubjectRequest => "data_subject_request",
            Self::StatusNote => "status_note",
        }
    }

    /// Mengurai nilai teks kolom; nilai di luar daftar menghasilkan `None`.
    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|c| c.as_str() == value)
    }
}

impl fmt::Display for ContainerType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Konteks enkripsi satu kolom `_enc` (AAD bidang 2) beserta jenis wadah DEK yang sah untuknya.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EncryptionContext {
    id: &'static str,
    container_type: ContainerType,
}

impl EncryptionContext {
    /// `identity_document.nik_enc`: NIK di dokumen identitas, wadah `identity_document` (DD §0.7, A2.4).
    pub const IDENTITY_DOCUMENT_NIK: Self = Self {
        id: "identity_document.nik_enc",
        container_type: ContainerType::IdentityDocument,
    };

    /// `identity_verification_request_status_event.note_enc`: catatan riwayat status permintaan
    /// verifikasi, wadah `status_note` (DD §0.7; migrasi F1-T18).
    pub const IDENTITY_VERIFICATION_REQUEST_STATUS_EVENT_NOTE: Self = Self {
        id: "identity_verification_request_status_event.note_enc",
        container_type: ContainerType::StatusNote,
    };

    /// Seluruh registri. Kolom `_enc` baru wajib ditambahkan di sini pada task yang membuat kolomnya.
    pub const ALL: &'static [Self] = &[
        Self::IDENTITY_DOCUMENT_NIK,
        Self::IDENTITY_VERIFICATION_REQUEST_STATUS_EVENT_NOTE,
    ];

    /// Pengenal logis kolom.
    pub const fn id(self) -> &'static str {
        self.id
    }

    /// Jenis wadah DEK yang sah untuk kolom ini.
    pub const fn container_type(self) -> ContainerType {
        self.container_type
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::path::Path;

    #[test]
    fn container_type_round_trips_and_rejects_unknown() {
        for c in ContainerType::ALL {
            assert_eq!(ContainerType::parse(c.as_str()), Some(c));
        }
        assert_eq!(ContainerType::parse("rme_visit"), None);
        assert_eq!(ContainerType::parse("Identity_Document"), None);
        assert_eq!(ContainerType::parse(""), None);
    }

    #[test]
    fn container_types_match_data_key_check_in_migration() {
        // Mencegah drift: daftar enum harus sama persis dengan CHECK ck_data_key_container_type.
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../db/migrations/app/20261119110000_platform_outbox_job_file_key.sql");
        let sql = std::fs::read_to_string(path).unwrap();
        let start = sql
            .find("CONSTRAINT ck_data_key_container_type CHECK (container_type IN")
            .unwrap();
        let rest = &sql[start..];
        let open = rest.find("container_type IN").unwrap() + "container_type IN".len();
        let close = rest.find("))").unwrap();
        let listed: Vec<String> = rest[open..close]
            .split(',')
            .map(|s| {
                s.trim()
                    .trim_start_matches('(')
                    .trim()
                    .trim_matches('\'')
                    .to_string()
            })
            .collect();
        let ours: Vec<String> = ContainerType::ALL
            .iter()
            .map(|c| c.as_str().to_string())
            .collect();
        assert_eq!(listed, ours);
    }

    #[test]
    fn registry_ids_are_unique_logical_enc_columns() {
        let mut seen = HashSet::new();
        for c in EncryptionContext::ALL {
            assert!(seen.insert(c.id()), "konteks ganda: {}", c.id());
            let (table, column) = c.id().split_once('.').unwrap();
            assert!(!table.is_empty());
            assert!(column.ends_with("_enc"), "{}", c.id());
            assert!(
                c.id().bytes().all(|b| b.is_ascii_lowercase()
                    || b.is_ascii_digit()
                    || b == b'_'
                    || b == b'.')
            );
            assert!(c.id().len() <= usize::from(u16::MAX));
        }
    }

    #[test]
    fn registry_columns_exist_as_bytea_in_app_migrations() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../db/migrations/app");
        let mut sql = String::new();
        for entry in std::fs::read_dir(dir).unwrap() {
            let p = entry.unwrap().path();
            if p.extension().is_some_and(|e| e == "sql") {
                sql.push_str(&std::fs::read_to_string(p).unwrap());
            }
        }
        for c in EncryptionContext::ALL {
            let (table, column) = c.id().split_once('.').unwrap();
            let head = format!("CREATE TABLE app.{table} (");
            let start = sql
                .find(&head)
                .unwrap_or_else(|| panic!("tabel {table} tidak ada di migrasi"));
            let block = &sql[start..start + sql[start..].find("\n);").unwrap()];
            let found = block.lines().any(|l| {
                let mut parts = l.split_whitespace();
                parts.next() == Some(column) && parts.next() == Some("bytea")
            });
            assert!(found, "kolom {} bukan bytea di tabel {table}", column);
        }
    }

    #[test]
    fn system_codes_are_distinct() {
        assert_eq!(System::RuangSehat.as_str(), "rs");
        assert_eq!(System::Rme.as_str(), "rme");
    }
}
