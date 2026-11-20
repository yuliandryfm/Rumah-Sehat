//! Kode event PRD §7.1 dan kode teks tertutup (snake_case) untuk aksi, jenis objek, peran, dan alasan.

use crate::error::{AuditError, AuditErrorKind};
use std::fmt;

/// Kode event PRD v1.3 §7.1. Disimpan sebagai `e_01` … `e_18` (DD B1 `event_code`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventCode {
    /// E-01 Login berhasil.
    E01,
    /// E-02 Login gagal, termasuk akun terkunci.
    E02,
    /// E-03 Logout.
    E03,
    /// E-04 Pembuatan akun dan perubahan kredensial (nilai tidak dicatat).
    E04,
    /// E-05 Akses ke data pasien sensitif oleh pihak selain subjek data.
    E05,
    /// E-06 Pembuatan dan perubahan catatan tenaga kesehatan, resep, rekam medis RME, hasil lab.
    E06,
    /// E-07 Verifikasi identitas disetujui atau ditolak.
    E07,
    /// E-08 Pembayaran, pembagian dana, refund, dan keputusan klaim.
    E08,
    /// E-09 Perubahan status konsultasi, pendaftaran, home visit, dan pesanan.
    E09,
    /// E-10 Penambahan, perubahan, verifikasi, dan penonaktifan akun.
    E10,
    /// E-11 Perubahan hak akses atau peran.
    E11,
    /// E-12 Ekspor atau unduh data.
    E12,
    /// E-13 Persetujuan diberikan/ditarik; permintaan hak subjek data.
    E13,
    /// E-14 Perubahan pengaturan yang memengaruhi uang atau aturan.
    E14,
    /// E-15 Anggota keluarga beserta pernyataan persetujuannya.
    E15,
    /// E-16 Soft delete dan pemulihan data master; penghapusan dan anonimisasi.
    E16,
    /// E-17 Pengiriman data pribadi ke pihak luar (jenis data dan tujuan, tanpa isi).
    E17,
    /// E-18 Impor data peserta korporat dan pengiriman undangan (CR-021, PRD v1.3).
    E18,
}

impl EventCode {
    /// Semua kode, urut.
    pub const ALL: [EventCode; 18] = [
        Self::E01,
        Self::E02,
        Self::E03,
        Self::E04,
        Self::E05,
        Self::E06,
        Self::E07,
        Self::E08,
        Self::E09,
        Self::E10,
        Self::E11,
        Self::E12,
        Self::E13,
        Self::E14,
        Self::E15,
        Self::E16,
        Self::E17,
        Self::E18,
    ];

    /// Bentuk simpan (`e_01` … `e_18`).
    pub fn as_str(self) -> &'static str {
        match self {
            Self::E01 => "e_01",
            Self::E02 => "e_02",
            Self::E03 => "e_03",
            Self::E04 => "e_04",
            Self::E05 => "e_05",
            Self::E06 => "e_06",
            Self::E07 => "e_07",
            Self::E08 => "e_08",
            Self::E09 => "e_09",
            Self::E10 => "e_10",
            Self::E11 => "e_11",
            Self::E12 => "e_12",
            Self::E13 => "e_13",
            Self::E14 => "e_14",
            Self::E15 => "e_15",
            Self::E16 => "e_16",
            Self::E17 => "e_17",
            Self::E18 => "e_18",
        }
    }
}

impl fmt::Display for EventCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Panjang maksimum kode.
pub const CODE_MAX_LEN: usize = 64;
/// Jumlah digit SELURUHNYA (bukan per deret) yang boleh ada di satu kode. Lebih dari ini ditolak supaya
/// NIK, nomor kartu, nomor rekening, atau nomor HP tidak dapat diselipkan lewat kode, termasuk yang
/// dipecah pemisah (`n_31740_12345_...`) (PRD §7.3, review keamanan F1-T22 BLOKIR-1).
pub const CODE_MAX_DIGITS: usize = 4;

/// Kode teks tertutup: huruf kecil ASCII, digit, dan `_`; diawali huruf; 1–64 karakter; paling banyak 4
/// digit seluruhnya. Dipakai untuk nama aksi, jenis objek, jenis akun, peran, kode alasan, tujuan dan
/// jenis data E-17, serta nilai berupa kode (mis. status). Teks bebas tidak dapat masuk lewat tipe ini.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Code(String);

impl Code {
    /// Memvalidasi dan membungkus kode.
    pub fn new(value: &str) -> Result<Self, AuditError> {
        if is_valid_code(value) {
            Ok(Self(value.to_string()))
        } else {
            Err(AuditError::new(AuditErrorKind::InvalidCode, "code"))
        }
    }

    /// Nilai kode.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Code {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

pub(crate) fn is_valid_code(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if value.len() > CODE_MAX_LEN || !first.is_ascii_lowercase() {
        return false;
    }
    let mut digits = 0usize;
    for c in value.chars() {
        if c.is_ascii_digit() {
            digits += 1;
            if digits > CODE_MAX_DIGITS {
                return false;
            }
        } else if !(c.is_ascii_lowercase() || c == '_') {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_codes_are_unique_and_match_dd_format() {
        let mut seen = std::collections::BTreeSet::new();
        for (i, c) in EventCode::ALL.iter().enumerate() {
            assert_eq!(c.as_str(), format!("e_{:02}", i + 1));
            assert!(seen.insert(c.as_str()));
            assert!(is_valid_code(c.as_str()));
        }
        assert_eq!(seen.len(), 18);
    }

    #[test]
    fn code_accepts_snake_case() {
        for ok in [
            "status",
            "order_status_changed",
            "e_01",
            "rs_admin",
            "v2",
            "a1234",
            "http_4_0_4",
        ] {
            assert!(Code::new(ok).is_ok(), "{ok}");
        }
    }

    #[test]
    fn code_rejects_free_text_and_numbers() {
        let long = "a".repeat(65);
        for bad in [
            "",
            "Status",
            "1abc",
            "_x",
            "has space",
            "budi@example.com",
            "x-y",
            "x.y",
            "a12345",
            "a123456",
            // Probe review keamanan: digit dipecah pemisah tetap dihitung seluruhnya.
            "n_31740_12345_67890_1",
            "n_3_1_7_4_0",
            "a12_34_5",
            "nik_3174012345678901",
            "card_4111111111111111",
            long.as_str(),
            "naïve",
        ] {
            assert_eq!(
                Code::new(bad).map_err(|e| e.kind()),
                Err(AuditErrorKind::InvalidCode),
                "{bad}"
            );
        }
    }
}
