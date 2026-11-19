//! Redaksi log (ADR-005, ADR-010 butir 1 dan 5, coding-standards §4.1).
//!
//! Dua lapis, keduanya berlaku untuk application log dan security log:
//! 1. Daftar tolak NAMA field ([`is_sensitive_key`]): nilainya diganti `[REDACTED]` apa pun isinya.
//! 2. Pemindaian POLA di setiap nilai teks ([`scrub_text`]): token Bearer/Basic, JWT, pasangan
//!    `kunci=nilai` dengan kunci sensitif (termasuk parameter URL bertanda tangan), email, nomor
//!    kartu/rekening/NIK (deretan 13 digit atau lebih, atau kelompok 4-4-4-4) dan nomor HP Indonesia.
//!
//! Batas yang diketahui: ini jaring pengaman, bukan izin. Kode tetap dilarang mencatat data terlarang
//! (nama, alamat, isi klinis, objek domain utuh) dan memakai `Sensitive<T>`; deteksi pola tidak dapat
//! mengenali nama orang atau alamat dalam teks bebas. Deretan digit sepanjang 13 atau lebih (mis. epoch
//! milidetik) ikut tersamar: lebih baik berlebihan daripada bocor.

/// Pengganti nilai terlarang.
pub const REDACTED: &str = "[REDACTED]";

/// Nama field yang cocok persis (setelah dinormalkan ke huruf kecil alfanumerik).
const EXACT_KEYS: &[&str] = &[
    "pin", "otp", "pan", "nik", "ktp", "cvv", "cvc", "sig", "jwt", "name", "nama", "dob", "hp",
    "sid", "session", "body", "payload", "phone", "email", "cookie", "cookies",
];

/// Nama field yang mengandung salah satu potongan ini (setelah dinormalkan).
const SUBSTRING_KEYS: &[&str] = &[
    "password",
    "passwd",
    "passcode",
    "secret",
    "token",
    "authorization",
    "cookie",
    "apikey",
    "signature",
    "credential",
    "privatekey",
    "cardnumber",
    "creditcard",
    "accountnumber",
    "norek",
    "rekening",
    "email",
    "phonenumber",
    "nomorhp",
    "sessionid",
    "fullname",
    "firstname",
    "lastname",
    "displayname",
    "username",
    "patientname",
    "homeaddress",
    "streetaddress",
    "alamat",
    "requestbody",
    "responsebody",
    "verificationcode",
    "kodeverifikasi",
    "birthdate",
    "tanggallahir",
    "diagnosis",
    "diagnosa",
    "anamnesis",
    "clinical",
    "rekammedis",
    "medicalrecord",
    "prescription",
];

/// Benar bila nilai field dengan nama ini tidak boleh dicatat. Nilai kolom `_enc` selalu terlarang.
pub fn is_sensitive_key(key: &str) -> bool {
    let lower = key.to_ascii_lowercase();
    if lower.ends_with("_enc") {
        return true;
    }
    let norm: String = lower.chars().filter(char::is_ascii_alphanumeric).collect();
    EXACT_KEYS.contains(&norm.as_str()) || SUBSTRING_KEYS.iter().any(|s| norm.contains(s))
}

fn is_ident(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '-'
}

fn is_token_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '-' | '.' | '_' | '~' | '+' | '/' | '=')
}

fn is_email_local(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '%' | '+' | '-')
}

fn is_value_delim(c: char) -> bool {
    c.is_whitespace() || matches!(c, '&' | ';' | ',' | '"' | '\'' | '}' | ')' | ']')
}

fn run_end(chars: &[char], from: usize, pred: impl Fn(char) -> bool) -> usize {
    let mut i = from;
    while chars.get(i).is_some_and(|&c| pred(c)) {
        i += 1;
    }
    i
}

fn slice(chars: &[char], a: usize, b: usize) -> String {
    chars.iter().skip(a).take(b.saturating_sub(a)).collect()
}

/// Panjang deret digit pada `at` (0 bila bukan digit).
fn digit_run(chars: &[char], at: usize) -> usize {
    run_end(chars, at, |c| c.is_ascii_digit()) - at
}

/// Akhir (eksklusif) deret digit yang harus disamarkan mulai `at`, bila ada.
fn sensitive_digits_end(chars: &[char], at: usize, out_ends_plus: bool) -> Option<usize> {
    let first = digit_run(chars, at);
    let end = at + first;
    if first >= 13 {
        return Some(end);
    }
    let head = slice(chars, at, end.min(at + 3));
    // Nomor HP Indonesia: 08xx... (10-13 digit) atau 628xx... (11-14 digit, boleh berawalan '+').
    if (10..=13).contains(&first) && head.starts_with("08") {
        return Some(end);
    }
    if (11..=14).contains(&first) && head.starts_with("628") && (out_ends_plus || first >= 12) {
        return Some(end);
    }
    // Kelompok bergaya kartu: 4-4-4-4 (dipisah satu spasi atau tanda hubung), kelompok akhir 1-7 digit.
    if first == 4 {
        let mut total = 4;
        let mut pos = end;
        loop {
            let sep_ok = chars.get(pos).is_some_and(|&c| c == ' ' || c == '-');
            let len = if sep_ok { digit_run(chars, pos + 1) } else { 0 };
            if len == 0 {
                break;
            }
            total += len;
            pos = pos + 1 + len;
            if len != 4 {
                break;
            }
        }
        if total >= 13 && pos > end {
            return Some(pos);
        }
    }
    None
}

/// Akhir (eksklusif) nilai bagi kunci sensitif yang nilainya mulai di `vstart`.
fn sensitive_value_end(
    chars: &[char],
    vstart: usize,
    quote: Option<char>,
    key_is_cookie: bool,
) -> usize {
    if key_is_cookie {
        return chars.len();
    }
    if let Some(q) = quote {
        let mut i = vstart;
        while chars.get(i).is_some_and(|&c| c != q) {
            i += 1;
        }
        return i;
    }
    let mut end = run_end(chars, vstart, |c| !is_value_delim(c));
    // `Authorization: Bearer <token>`: skema dan kredensial dua kata.
    let first_word = slice(chars, vstart, end);
    if (first_word.eq_ignore_ascii_case("bearer")
        || first_word.eq_ignore_ascii_case("basic")
        || first_word.eq_ignore_ascii_case("digest"))
        && chars.get(end) == Some(&' ')
    {
        end = run_end(chars, end + 1, is_token_char);
    }
    end
}

/// Menyamarkan pola data terlarang di teks bebas. Lihat dokumentasi modul untuk cakupan dan batasnya.
pub fn scrub_text(input: &str) -> String {
    let chars: Vec<char> = input.chars().collect();
    let mut out = String::with_capacity(input.len());
    let mut i = 0;
    while let Some(&c) = chars.get(i) {
        let at_boundary = i == 0 || chars.get(i - 1).is_some_and(|&p| !is_ident(p));

        // JWT: tiga bagian base64url berawalan "eyJ".
        if at_boundary && c == 'e' && slice(&chars, i, i + 3) == "eyJ" {
            let end = run_end(&chars, i, |x| {
                x.is_ascii_alphanumeric() || matches!(x, '-' | '_' | '.')
            });
            if slice(&chars, i, end).matches('.').count() >= 2 {
                out.push_str(REDACTED);
                i = end;
                continue;
            }
        }

        if at_boundary && is_ident(c) {
            let end = run_end(&chars, i, is_ident);
            let ident = slice(&chars, i, end);

            // "Bearer xxx" / "Basic xxx" berdiri sendiri.
            if (ident.eq_ignore_ascii_case("bearer") || ident.eq_ignore_ascii_case("basic"))
                && chars.get(end) == Some(&' ')
            {
                let tok_end = run_end(&chars, end + 1, is_token_char);
                if tok_end > end + 1 {
                    out.push_str(&ident);
                    out.push(' ');
                    out.push_str(REDACTED);
                    i = tok_end;
                    continue;
                }
            }

            // "kunci=nilai" / "kunci: nilai" / "\"kunci\":\"nilai\"" dengan kunci sensitif.
            if is_sensitive_key(&ident) {
                let mut k = end;
                while chars.get(k).is_some_and(|&x| x == '"' || x == '\'') {
                    k += 1;
                }
                while chars.get(k) == Some(&' ') {
                    k += 1;
                }
                if matches!(chars.get(k), Some(':') | Some('=')) {
                    k += 1;
                    while chars.get(k) == Some(&' ') {
                        k += 1;
                    }
                    let quote = chars.get(k).copied().filter(|&x| x == '"' || x == '\'');
                    let vstart = if quote.is_some() { k + 1 } else { k };
                    let is_cookie = ident.to_ascii_lowercase().contains("cookie");
                    let vend = sensitive_value_end(&chars, vstart, quote, is_cookie);
                    if vend > vstart {
                        out.push_str(&slice(&chars, i, vstart));
                        out.push_str(REDACTED);
                        i = vend;
                        continue;
                    }
                }
            }
            // Bukan pola kunci: jatuh ke pemrosesan per karakter di bawah.
        }

        if c == '@' {
            // Email: ambil bagian lokal yang sudah terlanjur ditulis, lalu domain di kanan.
            let mut local = Vec::new();
            while out.chars().next_back().is_some_and(is_email_local) {
                if let Some(x) = out.pop() {
                    local.push(x);
                }
            }
            let dom_end = run_end(&chars, i + 1, |x| {
                x.is_ascii_alphanumeric() || x == '.' || x == '-'
            });
            let domain = slice(&chars, i + 1, dom_end);
            let trimmed = domain.trim_end_matches('.');
            let valid_domain = trimmed.contains('.')
                && !trimmed.starts_with('.')
                && !trimmed.starts_with('-')
                && !trimmed.contains("..");
            if !local.is_empty() && valid_domain {
                out.push_str(REDACTED);
                i = i + 1 + trimmed.chars().count();
                continue;
            }
            out.extend(local.into_iter().rev());
            out.push('@');
            i += 1;
            continue;
        }

        let digit_start =
            c.is_ascii_digit() && (i == 0 || chars.get(i - 1).is_some_and(|p| !p.is_ascii_digit()));
        if digit_start && let Some(end) = sensitive_digits_end(&chars, i, out.ends_with('+')) {
            if out.ends_with('+') {
                out.pop();
            }
            out.push_str(REDACTED);
            i = end;
            continue;
        }

        out.push(c);
        i += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sensitive_keys_are_detected() {
        for k in [
            "password",
            "Password",
            "db_password",
            "pin",
            "otp",
            "token",
            "access_token",
            "refresh-token",
            "authorization",
            "Authorization",
            "cookie",
            "set-cookie",
            "session_id",
            "api_key",
            "client_secret",
            "card_number",
            "account_number",
            "nik",
            "email",
            "phone",
            "name",
            "full_name",
            "nama",
            "alamat",
            "diagnosis",
            "x-amz-signature",
            "nik_enc",
            "body",
            "request_body",
            "mfa_secret",
            "kode_verifikasi",
        ] {
            assert!(is_sensitive_key(k), "{k}");
        }
    }

    #[test]
    fn ordinary_keys_are_not_flagged() {
        for k in [
            "correlation_id",
            "route",
            "status",
            "duration_ms",
            "order_id",
            "actor_id",
            "shipping",
            "mapping",
            "footprint",
            "ip_address",
            "component",
            "message",
            "error_kind",
        ] {
            assert!(!is_sensitive_key(k), "{k}");
        }
    }

    #[test]
    fn scrubs_bearer_basic_and_jwt() {
        let jwt = "eyJhbGciOiJSUzI1NiJ9.eyJzdWIiOiIxMjMifQ.c2lnbmF0dXJl";
        for (input, secret) in [
            ("hdr Authorization: Bearer abc.def-123 end", "abc.def-123"),
            ("got Bearer ZXlKaGJHY2lP trailing", "ZXlKaGJHY2lP"),
            ("got Basic dXNlcjpwYXNz ok", "dXNlcjpwYXNz"),
            (&format!("token was {jwt} here"), jwt),
        ] {
            let out = scrub_text(input);
            assert!(!out.contains(secret), "{out}");
            assert!(out.contains(REDACTED), "{out}");
        }
    }

    #[test]
    fn scrubs_key_value_pairs_and_signed_urls() {
        for (input, secret) in [
            ("login password=hunter2 ok", "hunter2"),
            ("{\"pin\":\"123456\",\"x\":1}", "123456"),
            ("otp: 987654 sent", "987654"),
            (
                "GET /f?X-Amz-Signature=deadbeef01&X-Amz-Expires=60",
                "deadbeef01",
            ),
            ("cookie: sid=abc123; theme=dark", "abc123"),
            ("cookie: sid=abc123; theme=dark", "theme=dark"),
            ("api_key='k-777-secret' x", "k-777-secret"),
        ] {
            let out = scrub_text(input);
            assert!(!out.contains(secret), "{input} -> {out}");
        }
        assert!(scrub_text("X-Amz-Expires=60").contains("60"));
    }

    #[test]
    fn scrubs_email_cards_accounts_nik_and_phone() {
        for (input, secret) in [
            ("user budi.santoso+x@example.co.id gagal", "budi.santoso"),
            ("kartu 4111111111111111 ditolak", "4111111111111111"),
            ("kartu 4111 1111 1111 1111 ditolak", "4111 1111"),
            ("kartu 4111-1111-1111-1111", "4111-1111"),
            ("nik 3174012345678901 x", "3174012345678901"),
            ("rek 1234567890123 x", "1234567890123"),
            ("hp 081234567890 x", "081234567890"),
            ("hp +6281234567890 x", "6281234567890"),
        ] {
            let out = scrub_text(input);
            assert!(!out.contains(secret), "{input} -> {out}");
            assert!(out.contains(REDACTED), "{out}");
        }
    }

    #[test]
    fn keeps_harmless_text_ids_and_uuids() {
        for s in [
            "request_completed",
            "order 0193a5c2-7b1e-7c3a-9d41-2f6e8a1b3c4d paid",
            "status=200 duration_ms=15",
            "at 2026-11-19T10:00:00Z",
            "ip 10.0.0.1 route /patient/v1/orders/{order_id}",
            "price 150000 IDR",
            "mail@ alone and @handle",
        ] {
            assert_eq!(scrub_text(s), s);
        }
    }

    #[test]
    fn unicode_input_does_not_panic() {
        let s = "日本語 パスワード=abc ✓ é@é.é";
        let _ = scrub_text(s);
    }
}
