//! Tipe `ResponseCode`: string 7 digit `HHHMMCC` (ADR-023 butir 1, registry §1).

use http::StatusCode;
use serde::{Serialize, Serializer};

/// Kode respons terdaftar beserta deskripsinya. Deskripsi sama untuk setiap kemunculan kode
/// (registry §1), sehingga keduanya disimpan bersama. `HHH` adalah kode HTTP yang dikirim di
/// header; konsistensinya dijamin karena status HTTP diturunkan dari kode ini.
///
/// Konstruksi hanya lewat [`ResponseCode::new`] (konstanta, divalidasi saat compile). Kode
/// baru wajib ditambahkan ke `docs/05-response-codes.md` pada commit yang sama; test
/// `tests/registry.rs` memeriksanya.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResponseCode {
    code: &'static str,
    description: &'static str,
}

/// Benar bila `s` terdiri dari tepat 7 digit ASCII dan digit pertamanya 2 sampai 5
/// (kategori HTTP 2xx sampai 5xx).
pub(crate) const fn is_valid_shape(s: &str) -> bool {
    let mut rest = s.as_bytes();
    let mut count = 0;
    while let [b, tail @ ..] = rest {
        if !b.is_ascii_digit() || (count == 0 && (*b < b'2' || *b > b'5')) {
            return false;
        }
        count += 1;
        rest = tail;
    }
    count == 7
}

impl ResponseCode {
    /// Membuat kode dari literal. Bentuk salah menggagalkan compile bila dipakai di `const`.
    pub const fn new(code: &'static str, description: &'static str) -> Self {
        // Invarian yang terbukti saat compile bila dipakai di const/static (coding-standards §3.4).
        assert!(
            is_valid_shape(code),
            "response_code harus 7 digit, HTTP 2xx-5xx"
        );
        assert!(!description.is_empty(), "deskripsi tidak boleh kosong");
        Self { code, description }
    }

    /// Kode 7 digit sebagai teks.
    pub const fn as_str(&self) -> &'static str {
        self.code
    }

    /// Deskripsi Bahasa Indonesia (`response_description`).
    pub const fn description(&self) -> &'static str {
        self.description
    }

    /// Kode HTTP dari tiga digit pertama.
    pub fn http_status(&self) -> StatusCode {
        let b = self.code.as_bytes();
        let digit = |i: usize| u16::from(b.get(i).copied().unwrap_or(b'0').wrapping_sub(b'0'));
        let n = digit(0) * 100 + digit(1) * 10 + digit(2);
        StatusCode::from_u16(n).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
    }

    /// Kode modul 2 digit (`MM`).
    pub fn module(&self) -> &'static str {
        self.code.get(3..5).unwrap_or_default()
    }

    /// Kode kasus 2 digit (`CC`).
    pub fn case(&self) -> &'static str {
        self.code.get(5..7).unwrap_or_default()
    }

    /// Benar bila kode ini sukses (HTTP 2xx).
    pub fn is_success(&self) -> bool {
        self.http_status().is_success()
    }
}

impl Serialize for ResponseCode {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(self.code)
    }
}

/// Kode umum modul `00` (registry §3).
pub mod codes {
    use super::ResponseCode;

    pub const OK: ResponseCode = ResponseCode::new("2000000", "Berhasil");
    pub const CREATED: ResponseCode = ResponseCode::new("2010000", "Berhasil dibuat");
    pub const ACCEPTED: ResponseCode = ResponseCode::new("2020000", "Diterima untuk diproses");
    pub const BAD_REQUEST: ResponseCode =
        ResponseCode::new("4000001", "Permintaan tidak dapat dibaca");
    pub const UNAUTHENTICATED: ResponseCode =
        ResponseCode::new("4010001", "Belum login atau sesi berakhir");
    pub const FORBIDDEN: ResponseCode = ResponseCode::new("4030001", "Tidak diizinkan");
    pub const NOT_FOUND: ResponseCode = ResponseCode::new("4040001", "Data tidak ditemukan");
    pub const STATUS_CONFLICT: ResponseCode = ResponseCode::new("4090001", "Konflik status");
    pub const IDEMPOTENCY_KEY_REUSED: ResponseCode = ResponseCode::new(
        "4090002",
        "Kunci idempotensi sudah dipakai dengan isi berbeda",
    );
    pub const PAYLOAD_TOO_LARGE: ResponseCode =
        ResponseCode::new("4130001", "Berkas terlalu besar");
    pub const UNSUPPORTED_MEDIA_TYPE: ResponseCode =
        ResponseCode::new("4150001", "Jenis berkas tidak didukung");
    pub const INVALID: ResponseCode = ResponseCode::new("4220001", "Data tidak valid");
    pub const RATE_LIMITED: ResponseCode =
        ResponseCode::new("4290001", "Terlalu banyak permintaan");
    pub const INTERNAL: ResponseCode =
        ResponseCode::new("5000001", "Terjadi kesalahan pada sistem");
    pub const UPSTREAM_BAD_GATEWAY: ResponseCode = ResponseCode::new(
        "5020001",
        "Layanan pihak ketiga tidak merespons dengan benar",
    );
    pub const UNAVAILABLE: ResponseCode =
        ResponseCode::new("5030001", "Layanan sedang tidak tersedia");
    pub const UPSTREAM_TIMEOUT: ResponseCode =
        ResponseCode::new("5040001", "Layanan pihak ketiga melewati batas waktu");

    /// Semua kode umum; test registry mencocokkannya dengan §3 dokumen (dua arah).
    pub const ALL: [ResponseCode; 17] = [
        OK,
        CREATED,
        ACCEPTED,
        BAD_REQUEST,
        UNAUTHENTICATED,
        FORBIDDEN,
        NOT_FOUND,
        STATUS_CONFLICT,
        IDEMPOTENCY_KEY_REUSED,
        PAYLOAD_TOO_LARGE,
        UNSUPPORTED_MEDIA_TYPE,
        INVALID,
        RATE_LIMITED,
        INTERNAL,
        UPSTREAM_BAD_GATEWAY,
        UNAVAILABLE,
        UPSTREAM_TIMEOUT,
    ];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shape_validation() {
        assert!(is_valid_shape("2000000"));
        assert!(is_valid_shape("5040001"));
        for bad in [
            "",
            "200000",
            "20000000",
            "1000000",
            "6000000",
            "2O00000",
            "200000a",
            " 2000000",
            "2000000\n",
            "４０４0001",
        ] {
            assert!(!is_valid_shape(bad), "{bad:?}");
        }
    }

    #[test]
    fn parts_and_status() {
        let c = codes::UPSTREAM_TIMEOUT;
        assert_eq!(c.as_str(), "5040001");
        assert_eq!(c.http_status(), StatusCode::GATEWAY_TIMEOUT);
        assert_eq!(c.module(), "00");
        assert_eq!(c.case(), "01");
        assert!(!c.is_success());
        assert!(codes::CREATED.is_success());
        assert_eq!(codes::CREATED.http_status(), StatusCode::CREATED);
    }

    #[test]
    fn all_codes_unique_and_consistent() {
        for (i, a) in codes::ALL.iter().enumerate() {
            assert!(is_valid_shape(a.as_str()));
            assert_eq!(a.module(), "00");
            for b in codes::ALL.iter().skip(i + 1) {
                assert_ne!(a.as_str(), b.as_str());
            }
        }
    }

    #[test]
    fn serializes_as_string() {
        let s = serde_json::to_string(&codes::OK).unwrap();
        assert_eq!(s, "\"2000000\"");
    }
}
