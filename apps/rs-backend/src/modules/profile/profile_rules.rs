//! Aturan murni profil dan alamat pasien (F1-T56, US-105 AC1a, AC1b, AC3). Tanpa I/O dan tanpa jam sistem
//! (coding-standards §1.2, §5.7): tanggal "hari ini" selalu diberikan pemanggil.
//!
//! Tabel validasi mengikuti PRD §3 US-105 AC1a `[KEPUTUSAN-194]`; setiap pelanggaran menjadi satu
//! `FieldError` (kode galat `invalid_format`/`required`/`out_of_range`) untuk respons `4220001` (KU-01).

use super::domain::IdentityStatus;
use envelope::FieldError;
use time::{Date, Month};

/// US-105 AC1b / PB-22: usia minimum pemilik akun (tahun penuh). `[ASUMSI]` NILAI UJI Fase 1: PRD PB-22
/// menyatakan nilai final `[TIDAK DIKETAHUI]` (menunggu jawaban legal L-08) dan "Fase 1 memakai nilai uji".
/// Angka 17 dipilih sebagai nilai uji dan BUKAN keputusan bisnis; diganti saat PB-22 ditetapkan.
pub(super) const OWNER_MIN_AGE_YEARS: i32 = 17;
/// US-105 AC1a: tanggal lahir tidak lebih dari 120 tahun sebelum hari ini.
pub(super) const MAX_AGE_YEARS: i32 = 120;
/// Jumlah alamat aktif maksimum per pasien. `[ASUMSI]` batas teknis pencegah penyalahgunaan; tidak ada
/// angka di PRD (US-105 AC2 tidak membatasi). Terlampaui = `4220001` pada field `$`.
pub(super) const MAX_ADDRESSES: i64 = 20;
/// Zona waktu alamat bila tidak diisi. `[ASUMSI]` WIB: DD A2.3 mewajibkan `time_zone` (nama IANA,
/// ADR-017 butir 2) tetapi tabel AC1a tidak memuatnya, sehingga isian opsional dengan bawaan ini.
pub(super) const DEFAULT_TIME_ZONE: &str = "Asia/Jakarta";
/// Zona waktu Indonesia yang diterima (nama IANA untuk WIB, WITA, WIT).
const TIME_ZONES: [&str; 3] = ["Asia/Jakarta", "Asia/Makassar", "Asia/Jayapura"];

fn field(name: &str, code: &str, message: &str) -> FieldError {
    FieldError::new(name, code, message)
}

fn required(name: &str, label: &str) -> FieldError {
    field(name, "required", &format!("{label} wajib diisi"))
}

/// Mengumpulkan galat per field; kosong = lolos.
#[derive(Default)]
pub(super) struct Errors(Vec<FieldError>);

impl Errors {
    /// Menyimpan galat bila `r` galat, lalu mengembalikan nilai bila ada.
    pub(super) fn take<T>(&mut self, r: Result<T, FieldError>) -> Option<T> {
        match r {
            Ok(v) => Some(v),
            Err(e) => {
                self.0.push(e);
                None
            }
        }
    }

    pub(super) fn into_result(self) -> Result<(), Vec<FieldError>> {
        if self.0.is_empty() {
            Ok(())
        } else {
            Err(self.0)
        }
    }
}

/// Nama (pasien dan penerima alamat): 1-100 karakter setelah dipangkas; huruf (aksara apa pun), spasi,
/// titik, koma, apostrof, dan tanda hubung.
pub(super) fn normalize_name(name: &str, input: &str, label: &str) -> Result<String, FieldError> {
    let t = input.trim();
    if t.is_empty() {
        return Err(required(name, label));
    }
    if t.chars().count() > 100 {
        return Err(field(
            name,
            "out_of_range",
            &format!("{label} paling banyak 100 karakter"),
        ));
    }
    if !t
        .chars()
        .all(|c| c.is_alphabetic() || matches!(c, ' ' | '.' | ',' | '\'' | '-'))
    {
        return Err(field(
            name,
            "invalid_format",
            &format!(
                "{label} hanya boleh berisi huruf, spasi, titik, koma, apostrof, dan tanda hubung"
            ),
        ));
    }
    Ok(t.to_string())
}

/// Tanggal lahir `YYYY-MM-DD`: tanggal sah, tidak di masa depan menurut tanggal WIB `today_wib`, tidak lebih
/// dari 120 tahun lalu, dan usia minimal [`OWNER_MIN_AGE_YEARS`] (AC1a, AC1b).
pub(super) fn parse_birth_date(input: &str, today_wib: Date) -> Result<Date, FieldError> {
    let format = time::macros::format_description!("[year]-[month]-[day]");
    let date = (input.len() == 10)
        .then(|| Date::parse(input, &format).ok())
        .flatten()
        .ok_or_else(|| {
            field(
                "birth_date",
                "invalid_format",
                "Tanggal lahir harus tanggal sah berformat TTTT-BB-HH",
            )
        })?;
    if date > today_wib {
        return Err(field(
            "birth_date",
            "out_of_range",
            "Tanggal lahir tidak boleh di masa depan",
        ));
    }
    if date < years_before(today_wib, MAX_AGE_YEARS) {
        return Err(field(
            "birth_date",
            "out_of_range",
            "Tanggal lahir tidak boleh lebih dari 120 tahun lalu",
        ));
    }
    if date > years_before(today_wib, OWNER_MIN_AGE_YEARS) {
        return Err(field(
            "birth_date",
            "below_minimum_age",
            &format!("Usia minimum pemilik akun adalah {OWNER_MIN_AGE_YEARS} tahun"),
        ));
    }
    Ok(date)
}

/// Tanggal yang tepat `years` tahun sebelum `d` (29 Februari menjadi 28 Februari pada tahun bukan kabisat).
pub(super) fn years_before(d: Date, years: i32) -> Date {
    let year = d.year() - years;
    Date::from_calendar_date(year, d.month(), d.day()).unwrap_or_else(|_| {
        // Hanya 29 Februari yang tidak ada di tahun target; mundur satu hari. Tanggal di luar rentang
        // `time` mustahil di sini (tahun 1-9999 terjaga oleh tanggal masukan yang sah).
        Date::from_calendar_date(year, Month::February, 28).unwrap_or(d)
    })
}

/// Jenis kelamin: `male` atau `female` (CHECK `ck_care_recipient_sex`).
pub(super) fn parse_sex(input: &str) -> Result<&'static str, FieldError> {
    match input {
        "male" => Ok("male"),
        "female" => Ok("female"),
        _ => Err(field(
            "sex",
            "invalid_format",
            "Jenis kelamin harus male atau female",
        )),
    }
}

/// Nomor HP: spasi dan tanda hubung dibuang; awalan `+62` atau `0`; 8-13 digit setelah kode negara;
/// disimpan sebagai `+62...`. Kepemilikan nomor tidak diverifikasi.
pub(super) fn normalize_phone(name: &str, input: &str, label: &str) -> Result<String, FieldError> {
    let cleaned: String = input.chars().filter(|c| !matches!(c, ' ' | '-')).collect();
    let rest = cleaned
        .strip_prefix("+62")
        .or_else(|| cleaned.strip_prefix('0'))
        .ok_or_else(|| {
            field(
                name,
                "invalid_format",
                &format!("{label} harus diawali +62 atau 0"),
            )
        })?;
    if !rest.chars().all(|c| c.is_ascii_digit()) || !(8..=13).contains(&rest.len()) {
        return Err(field(
            name,
            "invalid_format",
            &format!("{label} harus berisi 8 sampai 13 digit setelah kode negara"),
        ));
    }
    Ok(format!("+62{rest}"))
}

/// Teks wajib dengan batas panjang karakter (setelah dipangkas).
fn bounded(
    name: &str,
    label: &str,
    input: &str,
    min: usize,
    max: usize,
) -> Result<String, FieldError> {
    let t = input.trim();
    if t.is_empty() {
        return Err(required(name, label));
    }
    let n = t.chars().count();
    if n < min || n > max {
        return Err(field(
            name,
            "out_of_range",
            &format!("{label} harus {min} sampai {max} karakter"),
        ));
    }
    Ok(t.to_string())
}

/// Teks opsional: kosong setelah dipangkas = tidak diisi; paling banyak `max` karakter.
fn optional_bounded(
    name: &str,
    label: &str,
    input: Option<&str>,
    max: usize,
) -> Result<Option<String>, FieldError> {
    let Some(t) = input.map(str::trim).filter(|t| !t.is_empty()) else {
        return Ok(None);
    };
    if t.chars().count() > max {
        return Err(field(
            name,
            "out_of_range",
            &format!("{label} paling banyak {max} karakter"),
        ));
    }
    Ok(Some(t.to_string()))
}

/// Masukan alamat mentah dari klien.
pub(super) struct RawAddress<'a> {
    pub(super) label: Option<&'a str>,
    pub(super) recipient_name: &'a str,
    pub(super) recipient_phone: &'a str,
    pub(super) address_line: &'a str,
    pub(super) province_name: &'a str,
    pub(super) city_name: &'a str,
    pub(super) postal_code: Option<&'a str>,
    pub(super) notes: Option<&'a str>,
    pub(super) time_zone: Option<&'a str>,
}

/// Alamat tervalidasi dan ternormalisasi.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ValidAddress {
    pub(super) label: Option<String>,
    pub(super) recipient_name: String,
    pub(super) recipient_phone: String,
    pub(super) address_line: String,
    pub(super) province_name: String,
    pub(super) city_name: String,
    pub(super) postal_code: Option<String>,
    pub(super) notes: Option<String>,
    pub(super) time_zone: &'static str,
}

/// Memvalidasi alamat (AC1a): semua galat dikumpulkan sekaligus supaya isian lain tidak hilang (KU-01).
pub(super) fn validate_address(a: &RawAddress<'_>) -> Result<ValidAddress, Vec<FieldError>> {
    let mut errors = Errors::default();
    let label = errors.take(optional_bounded("label", "Label", a.label, 30));
    let recipient_name = errors.take(normalize_name(
        "recipient_name",
        a.recipient_name,
        "Nama penerima",
    ));
    let recipient_phone = errors.take(normalize_phone(
        "recipient_phone",
        a.recipient_phone,
        "Nomor HP penerima",
    ));
    let address_line = errors.take(bounded(
        "address_line",
        "Alamat lengkap",
        a.address_line,
        5,
        500,
    ));
    let province_name = errors.take(bounded(
        "province_name",
        "Provinsi",
        a.province_name,
        2,
        100,
    ));
    let city_name = errors.take(bounded("city_name", "Kota", a.city_name, 2, 100));
    let postal_code = errors.take(
        optional_bounded("postal_code", "Kode pos", a.postal_code, 10).and_then(|p| match p {
            Some(p) if !p.chars().all(|c| c.is_ascii_digit()) => Err(field(
                "postal_code",
                "invalid_format",
                "Kode pos hanya boleh berisi angka",
            )),
            other => Ok(other),
        }),
    );
    let notes = errors.take(optional_bounded("notes", "Patokan", a.notes, 200));
    let time_zone = errors.take(match a.time_zone {
        None => Ok(DEFAULT_TIME_ZONE),
        Some(z) => TIME_ZONES.iter().copied().find(|t| *t == z).ok_or_else(|| {
            field(
                "time_zone",
                "invalid_format",
                "Zona waktu harus Asia/Jakarta, Asia/Makassar, atau Asia/Jayapura",
            )
        }),
    });
    errors.into_result()?;
    match (
        label,
        recipient_name,
        recipient_phone,
        address_line,
        province_name,
        city_name,
        postal_code,
        notes,
        time_zone,
    ) {
        (
            Some(label),
            Some(recipient_name),
            Some(recipient_phone),
            Some(address_line),
            Some(province_name),
            Some(city_name),
            Some(postal_code),
            Some(notes),
            Some(time_zone),
        ) => Ok(ValidAddress {
            label,
            recipient_name,
            recipient_phone,
            address_line,
            province_name,
            city_name,
            postal_code,
            notes,
            time_zone,
        }),
        // Tidak terjangkau: setiap `None` di atas sudah menjadi galat.
        _ => Err(vec![field("$", "invalid_format", "Isian tidak valid")]),
    }
}

/// US-105 AC3: perubahan nama atau tanggal lahir mengembalikan identitas yang sudah "Disetujui" ke
/// "Menunggu verifikasi". Status lain tidak berubah: "Menunggu verifikasi" sudah diperiksa Verifikator atas
/// data terbaru (detail membaca profil saat ini), "Ditolak" dan "Belum diunggah" memang belum disetujui.
/// Jenis kelamin dan nomor HP tidak menurunkan status (TC-US-105-9).
pub(super) fn requires_reverification(
    status: IdentityStatus,
    name_changed: bool,
    birth_date_changed: bool,
) -> bool {
    status == IdentityStatus::Approved && (name_changed || birth_date_changed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::date;

    const TODAY: Date = date!(2026 - 11 - 22);

    fn msg(r: Result<impl std::fmt::Debug, FieldError>) -> String {
        r.expect_err("harus ditolak").field
    }

    // trace: TC-US-105-2, TC-US-105-15 (US-105 AC1a nama)
    #[test]
    fn name_rules_follow_the_validation_table() {
        let n = |s: &str| normalize_name("full_name", s, "Nama lengkap");
        assert_eq!(n("  Budi Santoso  ").unwrap(), "Budi Santoso");
        assert_eq!(n("A").unwrap(), "A");
        assert_eq!(n(&"a".repeat(100)).unwrap().chars().count(), 100);
        assert_eq!(n("O'Brien-Smith, Jr.").unwrap(), "O'Brien-Smith, Jr.");
        assert!(n("\u{e2}\u{e4}\u{e9}").is_ok());
        assert!(n("山田 太郎").is_ok());
        assert!(n("Мария").is_ok());
        for bad in ["", "   ", "Budi1", "Bu@di", "Budi#", &"a".repeat(101)] {
            assert_eq!(msg(n(bad)), "full_name", "{bad}");
        }
    }

    // trace: TC-US-105-16, TC-US-105-20 (US-105 AC1a tanggal lahir, AC1b usia minimum)
    #[test]
    fn birth_date_boundaries() {
        let p = |s: &str| parse_birth_date(s, TODAY);
        assert!(p("1990-02-28").is_ok());
        assert_eq!(msg(p("1990-02-31")), "birth_date");
        assert_eq!(msg(p("2026-11-23")), "birth_date"); // besok menurut WIB
        assert_eq!(msg(p("bukan tanggal")), "birth_date");
        assert_eq!(msg(p("1990-1-1")), "birth_date");
        // Tepat 120 tahun lalu diterima; kurang satu hari ditolak.
        assert!(p("1906-11-22").is_ok());
        assert_eq!(msg(p("1906-11-21")), "birth_date");
        // Usia minimum: tepat N tahun diterima, N tahun kurang satu hari ditolak.
        assert!(p("2009-11-22").is_ok());
        let e = p("2009-11-23").unwrap_err();
        assert_eq!(e.field, "birth_date");
        assert_eq!(e.code, "below_minimum_age");
    }

    // trace: TC-US-105-16 (hari ini WIB yang dipakai; tanggal WITA yang masih "besok" WIB ditolak)
    #[test]
    fn today_is_judged_by_the_given_wib_date() {
        // WITA sudah 2026-11-23 pukul 00.30, WIB masih 2026-11-22: tanggal lahir 2026-11-23 ditolak.
        assert_eq!(msg(parse_birth_date("2026-11-23", TODAY)), "birth_date");
    }

    #[test]
    fn leap_day_reference_does_not_panic() {
        let today = date!(2028 - 02 - 29);
        assert_eq!(years_before(today, 17), date!(2011 - 02 - 28));
        assert!(parse_birth_date("2011-02-28", today).is_ok());
    }

    // trace: TC-US-105-17 (jenis kelamin dan nomor HP)
    #[test]
    fn sex_and_phone_rules() {
        assert_eq!(parse_sex("male").unwrap(), "male");
        assert_eq!(parse_sex("female").unwrap(), "female");
        for bad in ["", "Male", "other", "Laki-laki"] {
            assert!(parse_sex(bad).is_err(), "{bad}");
        }
        let p = |s: &str| normalize_phone("phone", s, "Nomor HP");
        assert_eq!(p("0812-3456 7890").unwrap(), "+6281234567890");
        assert_eq!(p("+62 812-3456-7890").unwrap(), "+6281234567890");
        assert_eq!(p("+6281234567").unwrap(), "+6281234567");
        assert_eq!(p("0123456789").unwrap(), "+62123456789");
    }

    #[test]
    fn phone_digit_count_and_prefix_limits() {
        let p = |s: &str| normalize_phone("phone", s, "Nomor HP");
        let digits = |n: usize| "8".repeat(n);
        // Batas 8-13 digit setelah kode negara, dengan awalan +62 maupun 0.
        for n in [8, 9, 13] {
            assert!(p(&format!("+62{}", digits(n))).is_ok(), "{n} digit");
            assert!(p(&format!("0{}", digits(n))).is_ok(), "{n} digit");
        }
        for n in [0, 1, 7, 14, 15] {
            assert!(p(&format!("+62{}", digits(n))).is_err(), "{n} digit");
            assert!(p(&format!("0{}", digits(n))).is_err(), "{n} digit");
        }
        assert!(p("+6288888a888").is_err());
        assert!(p("6281234567890").is_err()); // tanpa + dan tanpa 0
        assert!(p("+1234567890").is_err());
        assert!(p("").is_err());
    }

    fn raw<'a>() -> RawAddress<'a> {
        RawAddress {
            label: Some("Rumah"),
            recipient_name: "Siti Aminah",
            recipient_phone: "081234567890",
            address_line: "Jl. Contoh No. 1",
            province_name: "DKI Jakarta",
            city_name: "Jakarta Selatan",
            postal_code: Some("12345"),
            notes: None,
            time_zone: None,
        }
    }

    // trace: TC-US-105-18 (batas alamat)
    #[test]
    fn address_boundaries() {
        let ok = validate_address(&raw()).unwrap();
        assert_eq!(ok.recipient_phone, "+6281234567890");
        assert_eq!(ok.time_zone, "Asia/Jakarta");
        assert_eq!(ok.notes, None);

        let errs = |a: RawAddress<'_>| -> Vec<String> {
            validate_address(&a)
                .unwrap_err()
                .iter()
                .map(|e| e.field.clone())
                .collect()
        };
        let long500 = "a".repeat(500);
        let long501 = "a".repeat(501);
        assert!(
            validate_address(&RawAddress {
                address_line: &long500,
                ..raw()
            })
            .is_ok()
        );
        assert_eq!(
            errs(RawAddress {
                address_line: &long501,
                ..raw()
            }),
            ["address_line"]
        );
        assert_eq!(
            errs(RawAddress {
                address_line: "abcd",
                ..raw()
            }),
            ["address_line"]
        );
        assert!(
            validate_address(&RawAddress {
                address_line: "abcde",
                ..raw()
            })
            .is_ok()
        );
        assert_eq!(
            errs(RawAddress {
                province_name: "J",
                ..raw()
            }),
            ["province_name"]
        );
        assert!(
            validate_address(&RawAddress {
                city_name: "Ab",
                ..raw()
            })
            .is_ok()
        );
        let c101 = "a".repeat(101);
        assert_eq!(
            errs(RawAddress {
                city_name: &c101,
                ..raw()
            }),
            ["city_name"]
        );
        let l30 = "a".repeat(30);
        let l31 = "a".repeat(31);
        assert!(
            validate_address(&RawAddress {
                label: Some(&l30),
                ..raw()
            })
            .is_ok()
        );
        assert_eq!(
            errs(RawAddress {
                label: Some(&l31),
                ..raw()
            }),
            ["label"]
        );
        assert_eq!(
            errs(RawAddress {
                postal_code: Some("12a45"),
                ..raw()
            }),
            ["postal_code"]
        );
        let n200 = "a".repeat(200);
        let n201 = "a".repeat(201);
        assert!(
            validate_address(&RawAddress {
                notes: Some(&n200),
                ..raw()
            })
            .is_ok()
        );
        assert_eq!(
            errs(RawAddress {
                notes: Some(&n201),
                ..raw()
            }),
            ["notes"]
        );
        // Opsional kosong diterima; wajib kosong ditolak per field, semuanya sekaligus.
        let v = validate_address(&RawAddress {
            label: Some("  "),
            postal_code: Some(""),
            notes: Some(" "),
            ..raw()
        })
        .unwrap();
        assert_eq!((v.label, v.postal_code, v.notes), (None, None, None));
        assert_eq!(
            errs(RawAddress {
                recipient_name: "",
                recipient_phone: "",
                address_line: "",
                province_name: "",
                city_name: "",
                ..raw()
            }),
            [
                "recipient_name",
                "recipient_phone",
                "address_line",
                "province_name",
                "city_name"
            ]
        );
        assert_eq!(
            errs(RawAddress {
                time_zone: Some("Asia/Tokyo"),
                ..raw()
            }),
            ["time_zone"]
        );
        assert_eq!(
            validate_address(&RawAddress {
                time_zone: Some("Asia/Makassar"),
                ..raw()
            })
            .unwrap()
            .time_zone,
            "Asia/Makassar"
        );
    }

    // trace: TC-US-105-7, -8, -9 (US-105 AC3)
    #[test]
    fn only_name_or_birth_date_on_approved_identity_requires_reverification() {
        use IdentityStatus::{Approved, NotUploaded, Pending, Rejected};
        assert!(requires_reverification(Approved, true, false));
        assert!(requires_reverification(Approved, false, true));
        assert!(requires_reverification(Approved, true, true));
        assert!(!requires_reverification(Approved, false, false));
        for s in [NotUploaded, Pending, Rejected] {
            assert!(!requires_reverification(s, true, true));
        }
    }
}
