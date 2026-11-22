//! Redaksi log (ADR-005, ADR-010 butir 1 dan 5, coding-standards §4.1).
//!
//! Dua lapis, keduanya berlaku untuk application log dan security log:
//! 1. Daftar tolak NAMA field ([`is_sensitive_key`]): nilainya diganti `[REDACTED]` apa pun isinya.
//! 2. Pemindaian POLA di setiap nilai teks ([`scrub_text`]): token Bearer/Basic, JWT, pasangan
//!    `kunci=nilai` dengan kunci sensitif (termasuk parameter URL bertanda tangan), email, nomor
//!    kartu/rekening/NIK (deretan 13 digit atau lebih, atau kelompok 4-4-4-4) dan nomor HP Indonesia.
//!
//! Nama field dipecah per token (`_`, `-`, camelCase) dan dicocokkan per token, pasangan token, dan potongan.
//! Untuk kunci sensitif di teks, nilai disamarkan sampai pembatas pada tingkat kurung yang sama, termasuk
//! `Some("x")`, `["a","b"]`, `Nama { .. }`, dan JSON ber-escape.
//!
//! Batas yang diketahui (jaring pengaman, bukan izin):
//! - Skema `Authorization` selain Bearer/Basic/Digest: seluruh nilai header disamarkan (aman, berlebih).
//!   Skema itu sendiri tidak dikenali di teks bebas tanpa kunci `authorization`.
//! - Nilai tanpa kutip yang memuat spasi (`password=abc def`) hanya disamarkan sampai spasi pertama.
//! - Nomor HP bersekat (`0812-3456-7890`) dan rekening 10-12 digit di teks bebas tanpa kunci sensitif
//!   TIDAK dikenali; yang dikenali: deret 13 digit ke atas, kelompok 4-4-4-4, HP `08`/`628` tanpa sekat.
//!   Lindungi lewat nama field (`no_rek`, `phone`) dan `Sensitive<T>`.
//! - UUID kanonik berdiri sendiri (8-4-4-4-12 heksadesimal, tidak menempel pada karakter identifier) disalin
//!   utuh dan tidak dipindai pola digit (TD-21), sehingga angka yang sengaja diformat menyerupai UUID tidak
//!   tersamar. Email dengan bagian lokal berbentuk UUID tetap tersamar.
//! - Lainnya: ini jaring pengaman, bukan izin. Kode tetap dilarang mencatat data terlarang
//!   (nama, alamat, isi klinis, objek domain utuh) dan memakai `Sensitive<T>`; deteksi pola tidak dapat
//!   mengenali nama orang atau alamat dalam teks bebas. Deretan digit sepanjang 13 atau lebih (mis. epoch
//!   milidetik) ikut tersamar: lebih baik berlebihan daripada bocor.

/// Pengganti nilai terlarang.
pub const REDACTED: &str = "[REDACTED]";

/// Token (hasil pemecahan nama field per `_`, `-`, spasi, dan camelCase) yang sendirian sudah sensitif.
const SENSITIVE_TOKENS: &[&str] = &[
    "pin",
    "otp",
    "totp",
    "hotp",
    "mfa",
    "nik",
    "ktp",
    "npwp",
    "bpjs",
    "hp",
    "msisdn",
    "phone",
    "telepon",
    "telp",
    "mobile",
    "pan",
    "cvv",
    "cvc",
    "sandi",
    "pwd",
    "passphrase",
    "passcode",
    "password",
    "passwd",
    "secret",
    "token",
    "cookie",
    "cookies",
    "jwt",
    "sid",
    "sig",
    "email",
    "alamat",
    "lahir",
    "birth",
    "dob",
    "kartu",
    "card",
    "rek",
    "rekening",
    "sex",
    "gender",
    "postal",
    "zip",
    "notes",
    "body",
    "payload",
    "signature",
    "credential",
    "credentials",
    "authorization",
    "bearer",
    "verification",
    "verifier",
    "recovery",
    "dek",
    "bidx",
    "diagnosis",
    "diagnosa",
    "anamnesis",
    "clinical",
    "prescription",
    "ihs",
    "pass",
];

/// Pasangan token yang bersama-sama sensitif: (salah satu dari kiri) dan (salah satu dari kanan).
const TOKEN_PAIRS: &[(&[&str], &[&str])] = &[
    (
        &[
            "account", "document", "dokumen", "bank", "identity", "passport", "paspor", "license",
        ],
        &["number", "no", "num", "nomor", "last4", "holder"],
    ),
    (
        &[
            "api",
            "private",
            "signing",
            "access",
            "encryption",
            "hmac",
            "master",
            "webhook",
            "secret",
        ],
        &["key"],
    ),
    (&["va"], &["number", "no", "nomor", "num"]),
    (&["virtual"], &["account"]),
    (&["auth"], &["code"]),
    (&["medical", "rekam"], &["record", "medis"]),
    (&["ibu"], &["kandung"]),
];

/// Konteks orang: token `name`/`nama` bersama salah satunya menjadi nama orang.
const PERSON_CONTEXT: &[&str] = &[
    "full",
    "first",
    "last",
    "display",
    "user",
    "patient",
    "pasien",
    "recipient",
    "holder",
    "officer",
    "customer",
    "lengkap",
    "doctor",
    "dokter",
    "nakes",
    "member",
    "family",
];

/// Potongan pada nama yang digabung (untuk bentuk tanpa pemisah seperti `nohp`, `apikey`); hanya kata panjang
/// yang aman dari kecocokan tidak sengaja.
const SUBSTRING_KEYS: &[&str] = &[
    "password",
    "passwd",
    "passcode",
    "passphrase",
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
    "nohp",
    "nokartu",
    "nomorkartu",
    "katasandi",
    "sessionid",
    "fullname",
    "firstname",
    "lastname",
    "displayname",
    "username",
    "patientname",
    "namalengkap",
    "namapasien",
    "homeaddress",
    "streetaddress",
    "alamat",
    "requestbody",
    "responsebody",
    "verificationcode",
    "kodeverifikasi",
    "birthdate",
    "tanggallahir",
    "tgllahir",
    "diagnosis",
    "diagnosa",
    "anamnesis",
    "clinical",
    "rekammedis",
    "medicalrecord",
    "prescription",
    "beforevalues",
    "aftervalues",
];

fn key_tokens(key: &str) -> Vec<String> {
    let chars: Vec<char> = key.chars().collect();
    let mut tokens = Vec::new();
    let mut cur = String::new();
    for (j, &c) in chars.iter().enumerate() {
        if !c.is_ascii_alphanumeric() {
            if !cur.is_empty() {
                tokens.push(std::mem::take(&mut cur));
            }
            continue;
        }
        let prev = if j == 0 { None } else { chars.get(j - 1) };
        let next = chars.get(j + 1);
        // Batas camelCase: `userPin` (kecil lalu besar) dan akronim `OTPCode` (besar-besar-kecil).
        let boundary = c.is_ascii_uppercase()
            && !cur.is_empty()
            && (prev.is_some_and(|p| p.is_ascii_lowercase())
                || (prev.is_some_and(|p| p.is_ascii_uppercase())
                    && next.is_some_and(|n| n.is_ascii_lowercase())));
        if boundary {
            tokens.push(std::mem::take(&mut cur));
        }
        cur.push(c.to_ascii_lowercase());
    }
    if !cur.is_empty() {
        tokens.push(cur);
    }
    tokens
}

/// Benar bila nilai field dengan nama ini tidak boleh dicatat. Nama dipecah per token (`_`, `-`, camelCase)
/// dan dicocokkan per token, per pasangan token, dan per potongan nama gabungan. Kolom `*_enc` selalu terlarang.
pub fn is_sensitive_key(key: &str) -> bool {
    let tokens = key_tokens(key);
    let has = |t: &str| tokens.iter().any(|x| x == t);
    let has_any = |set: &[&str]| set.iter().any(|t| has(t));
    if tokens.last().is_some_and(|t| t == "enc") {
        return true;
    }
    if has_any(SENSITIVE_TOKENS) {
        return true;
    }
    if TOKEN_PAIRS.iter().any(|(l, r)| has_any(l) && has_any(r)) {
        return true;
    }
    // Alamat IP boleh dicatat (ADR-010 butir 6); alamat tempat tinggal tidak.
    if has("address") && !has("ip") && !has("mac") {
        return true;
    }
    if (has("name") || has("nama")) && (tokens.len() == 1 || has_any(PERSON_CONTEXT)) {
        return true;
    }
    let joined = tokens.concat();
    matches!(joined.as_str(), "session" | "name" | "nama")
        || SUBSTRING_KEYS.iter().any(|s| joined.contains(s))
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

/// Akhir (eksklusif) UUID kanonik (8-4-4-4-12 heksadesimal) yang berdiri sendiri mulai `at`, bila ada.
/// Pemanggil menjamin `at` di batas token; karakter sesudahnya tidak boleh karakter identifier (termasuk
/// `-`), sehingga deret yang hanya menyerupai UUID di tengah token lain tidak dikecualikan.
fn canonical_uuid_end(chars: &[char], at: usize) -> Option<usize> {
    const GROUPS: [usize; 5] = [8, 4, 4, 4, 12];
    let mut pos = at;
    for (n, len) in GROUPS.iter().enumerate() {
        if n > 0 {
            if chars.get(pos) != Some(&'-') {
                return None;
            }
            pos += 1;
        }
        let end = run_end(chars, pos, |c| c.is_ascii_hexdigit());
        if end - pos != *len {
            return None;
        }
        pos = end;
    }
    if chars.get(pos).is_some_and(|&c| is_ident(c)) {
        return None;
    }
    Some(pos)
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

/// Akhir nilai berkutip. `depth` = jumlah `\` di depan kutip pembuka (0 untuk kutip biasa, 1 untuk JSON di dalam
/// string, dst.). Kutip biasa: `\` melompati karakter berikutnya (kutip ber-escape tidak menutup). Kutip ber-escape:
/// menutup hanya pada kutip yang didahului tepat `depth` garis miring; yang didahului lebih banyak (`\\\"`) adalah
/// kutip di dalam nilai dan dilompati.
fn quoted_value_end(chars: &[char], vstart: usize, q: char, depth: usize) -> usize {
    let mut i = vstart;
    while let Some(&c) = chars.get(i) {
        if c == '\\' {
            let run_end = run_end(chars, i, |x| x == '\\');
            let count = run_end - i;
            if chars.get(run_end) == Some(&q) {
                if depth == 0 {
                    if count % 2 == 1 {
                        i = run_end + 1; // kutip ber-escape di dalam nilai
                        continue;
                    }
                    return run_end; // garis miring berpasangan, kutip menutup
                }
                if count > depth {
                    i = run_end + 1;
                    continue;
                }
                return i;
            }
            i = if depth == 0 && count % 2 == 1 {
                run_end + 1 // `\x`: x dilompati
            } else {
                run_end
            };
            continue;
        }
        if c == q {
            return i;
        }
        i += 1;
    }
    chars.len()
}

/// Akhir (eksklusif) nilai bagi kunci sensitif yang nilainya mulai di `vstart`.
///
/// - Bernilai berkutip: sampai kutip penutup; bila kutip pembuka ber-escape (JSON di dalam string),
///   garis miring sebelum kutip penutup tidak ikut.
/// - Tanpa kutip: sampai pembatas pada tingkat kurung yang sama (`,` `;` `&` spasi, atau kurung penutup
///   yang tidak berpasangan); isi di dalam `(...)`, `[...]`, `{...}` dan string di dalamnya ikut tersamar,
///   sehingga `Some("x")`, `["a","b"]`, dan `Nama { a: "x" }` hilang seluruhnya.
fn sensitive_value_end(
    chars: &[char],
    vstart: usize,
    quote: Option<char>,
    escape_depth: usize,
    key_is_cookie: bool,
    key_is_auth: bool,
) -> usize {
    if key_is_cookie {
        return chars.len();
    }
    if let Some(q) = quote {
        return quoted_value_end(chars, vstart, q, escape_depth);
    }
    if key_is_auth {
        // Skema apa pun (Bearer, Basic, Digest, Token, ...): seluruh nilai header.
        return run_end(chars, vstart, |c| {
            !matches!(c, '\n' | '\r' | '"' | '\'' | ',')
        });
    }
    let word_end = run_end(chars, vstart, |c| !is_value_delim(c));
    let first_word = slice(chars, vstart, word_end);
    if (first_word.eq_ignore_ascii_case("bearer")
        || first_word.eq_ignore_ascii_case("basic")
        || first_word.eq_ignore_ascii_case("digest"))
        && chars.get(word_end).is_some_and(|c| c.is_whitespace())
    {
        return run_end(chars, word_end + 1, is_token_char);
    }
    let mut depth = 0usize;
    let mut in_str: Option<char> = None;
    let mut i = vstart;
    while let Some(&c) = chars.get(i) {
        if let Some(q) = in_str {
            if c == '\\' {
                i += 2;
                continue;
            }
            if c == q {
                in_str = None;
            }
            i += 1;
            continue;
        }
        match c {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => {
                if depth == 0 {
                    break;
                }
                depth -= 1;
            }
            '"' | '\'' => {
                if depth == 0 {
                    break;
                }
                in_str = Some(c);
            }
            '&' | ';' | ',' if depth == 0 => break,
            c if c.is_whitespace() && depth == 0 => {
                // Struct Debug bernama: `Nama { ... }` dilanjutkan.
                let next = chars.iter().skip(i).find(|x| !x.is_whitespace());
                if !(i > vstart && next == Some(&'{')) {
                    break;
                }
            }
            _ => {}
        }
        i += 1;
    }
    i.min(chars.len())
}

/// Menyamarkan pola data terlarang di teks bebas. Lihat dokumentasi modul untuk cakupan dan batasnya.
pub fn scrub_text(input: &str) -> String {
    let chars: Vec<char> = input.chars().collect();
    let mut out = String::with_capacity(input.len());
    let mut i = 0;
    while let Some(&c) = chars.get(i) {
        let at_boundary = i == 0 || chars.get(i - 1).is_some_and(|&p| !is_ident(p));

        // UUID kanonik (ID akun, korelasi, objek) disalin utuh: kelompok heksadesimal yang kebetulan semuanya
        // digit tidak boleh terbaca sebagai kartu 4-4-4-4 atau nomor HP (TD-21). Bentuk 8-4-4-4-12 tidak
        // dapat memuat deret 13 digit maupun kelompok kartu 4-4-4-4 yang lengkap di luar susunan UUID itu.
        if at_boundary && let Some(end) = canonical_uuid_end(&chars, i) {
            out.push_str(&slice(&chars, i, end));
            i = end;
            continue;
        }

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
                && chars.get(end).is_some_and(|c| c.is_whitespace())
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

            // "kunci=nilai" / "kunci: nilai" / JSON ber-escape dengan kunci sensitif.
            // Parameter query OAuth `code` (mis. `?code=...`): hanya di konteks URL, karena `code` sendirian
            // terlalu umum sebagai nama field (`status_code`).
            let url_code =
                ident == "code" && i > 0 && chars.get(i - 1).is_some_and(|&p| p == '?' || p == '&');
            if url_code || is_sensitive_key(&ident) {
                let mut k = end;
                while chars
                    .get(k)
                    .is_some_and(|&x| x == '"' || x == '\'' || x == '\\')
                {
                    k += 1;
                }
                while chars.get(k).is_some_and(|c| c.is_whitespace()) {
                    k += 1;
                }
                // Tuple Debug `("kunci", "nilai")`: kunci berkutip diikuti koma juga pasangan.
                let sep = matches!(chars.get(k), Some(':') | Some('='))
                    || (k > end && chars.get(k) == Some(&','));
                if sep {
                    k += 1;
                    while chars.get(k).is_some_and(|c| c.is_whitespace()) {
                        k += 1;
                    }
                    let mut q_at = k;
                    while chars.get(q_at) == Some(&'\\') {
                        q_at += 1;
                    }
                    let quote = chars.get(q_at).copied().filter(|&x| x == '"' || x == '\'');
                    let escape_depth = if quote.is_some() { q_at - k } else { 0 };
                    let vstart = if quote.is_some() { q_at + 1 } else { k };
                    let lower = ident.to_ascii_lowercase();
                    let vend = sensitive_value_end(
                        &chars,
                        vstart,
                        quote,
                        escape_depth,
                        lower.contains("cookie"),
                        lower.contains("authorization"),
                    );
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
    fn uuids_with_all_digit_groups_are_kept() {
        // TD-21: UUIDv7 acak dengan kelompok ke-3..5 kebetulan semuanya digit (~1% ID) dulu tersamar sebagian
        // sebagai kartu 4-4-4-4 (atau HP 08/628 di kelompok terakhir), merusak account_id di security log.
        for s in [
            "01a20164-f5ec-7123-8456-123456789012",
            "01a20164-f5ec-7123-8456-081234567890",
            "01a20164-f5ec-7123-8456-628123456789",
            "12345678-1234-1234-1234-123456789012",
            "ABCDEF01-1234-7123-9456-123456789012",
        ] {
            assert_eq!(scrub_text(s), s);
            let framed = format!("account_id=\"{s}\" route /x/{s}/y ({s})");
            assert_eq!(scrub_text(&framed), framed);
        }
    }

    #[test]
    fn uuid_exemption_does_not_shield_neighbouring_numbers() {
        for (input, secret) in [
            // Kartu tetap tersamar walau berdampingan dengan UUID.
            (
                "01a20164-f5ec-7123-8456-123456789012 4111-1111-1111-1111",
                "4111-1111",
            ),
            // Bukan UUID kanonik (kelompok terakhir terlalu panjang / disambung): pola kartu tetap berlaku.
            ("01a20164-f5ec-4111-1111-1111111111119", "4111-1111"),
            ("01a20164-f5ec-4111-1111-111111111111-1111", "4111-1111"),
            ("x01a20164-f5ec-4111-1111-111111111111", "4111-1111"),
            // Kelompok kartu tanpa awalan UUID.
            ("f5ec-4111-1111-1111-1111", "4111-1111"),
        ] {
            let out = scrub_text(input);
            assert!(!out.contains(secret), "{input} -> {out}");
            assert!(out.contains(REDACTED), "{input} -> {out}");
        }
        // Email dengan bagian lokal berbentuk UUID tetap tersamar.
        let out = scrub_text("01a20164-f5ec-7123-8456-123456789012@example.com");
        assert!(!out.contains("example.com"), "{out}");
    }

    #[test]
    fn unicode_input_does_not_panic() {
        let s = "日本語 パスワード=abc ✓ é@é.é";
        let _ = scrub_text(s);
    }
}
