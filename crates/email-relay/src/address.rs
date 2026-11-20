//! Alamat email: bentuk sempit yang cukup untuk Keycloak dan Alertmanager (ADR-026 butir 4).
//!
//! - Hanya ASCII (SES tidak mendukung SMTPUTF8 [SRC-580]); bagian lokal dot-atom (RFC 5322 §3.2.3,
//!   tanpa quoted-string), domain berupa label `[A-Za-z0-9-]` minimal dua label.
//! - Perbandingan tanpa membedakan huruf besar/kecil ([`Mailbox::key`]).
//! - Daftar alamat di header (`From`, `To`, `Cc`, `Bcc`): nama tampilan, quoted-string, komentar, dan
//!   `<addr-spec>` dipahami; grup (`nama: a, b;`) dan bentuk lain ditolak.

use std::fmt;

/// Panjang maksimum alamat (RFC 5321 §4.5.3.1.3: path 256 termasuk `<>`).
pub const MAX_ADDRESS_LEN: usize = 254;

/// Alamat yang sudah divalidasi. Tidak pernah muncul di `Debug` (data pribadi, ADR-005).
#[derive(Clone, PartialEq, Eq)]
pub struct Mailbox(String);

impl fmt::Debug for Mailbox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Mailbox([REDACTED])")
    }
}

fn atext(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b"!#$%&'*+-/=?^_`{|}~".contains(&b)
}

impl Mailbox {
    /// Memvalidasi `local@domain`.
    pub fn parse(raw: &str) -> Option<Self> {
        if raw.len() < 3 || raw.len() > MAX_ADDRESS_LEN || !raw.is_ascii() {
            return None;
        }
        let (local, domain) = raw.split_once('@')?;
        if local.is_empty()
            || local.len() > 64
            || local.starts_with('.')
            || local.ends_with('.')
            || local.contains("..")
            || !local.bytes().all(|b| atext(b) || b == b'.')
        {
            return None;
        }
        let labels: Vec<&str> = domain.split('.').collect();
        if labels.len() < 2
            || !labels.iter().all(|l| {
                !l.is_empty()
                    && l.len() <= 63
                    && !l.starts_with('-')
                    && !l.ends_with('-')
                    && l.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
            })
        {
            return None;
        }
        Some(Self(raw.to_string()))
    }

    /// Nilai apa adanya.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Kunci perbandingan (huruf kecil).
    pub fn key(&self) -> String {
        self.0.to_ascii_lowercase()
    }

    /// Sama tanpa membedakan huruf besar/kecil.
    pub fn same(&self, other: &Mailbox) -> bool {
        self.0.eq_ignore_ascii_case(&other.0)
    }
}

/// Mengurai daftar alamat nilai header (sudah di-unfold). `None` bila ada elemen yang tidak dipahami.
pub fn parse_address_list(value: &str) -> Option<Vec<Mailbox>> {
    let mut out = Vec::new();
    for element in split_top_level(value)? {
        let element = element.trim();
        if element.is_empty() {
            return None;
        }
        out.push(parse_element(element)?);
    }
    if out.is_empty() { None } else { Some(out) }
}

/// Memecah pada koma tingkat atas; menolak `:`/`;` (grup) dan kutip/komentar/kurung siku tak seimbang.
fn split_top_level(value: &str) -> Option<Vec<String>> {
    let mut parts = Vec::new();
    let mut cur = String::new();
    let mut in_quote = false;
    let mut depth = 0usize;
    let mut in_angle = false;
    let mut chars = value.chars();
    while let Some(c) = chars.next() {
        if in_quote {
            match c {
                '\\' => {
                    cur.push(c);
                    cur.push(chars.next()?);
                }
                '"' => {
                    in_quote = false;
                    cur.push(c);
                }
                _ => cur.push(c),
            }
            continue;
        }
        if depth > 0 {
            match c {
                '\\' => {
                    chars.next()?;
                }
                '(' => depth += 1,
                ')' => depth -= 1,
                _ => {}
            }
            continue;
        }
        match c {
            '"' if !in_angle => {
                in_quote = true;
                cur.push(c);
            }
            '(' if !in_angle => depth = 1,
            '<' if !in_angle => {
                in_angle = true;
                cur.push(c);
            }
            '<' => return None,
            '>' if in_angle => {
                in_angle = false;
                cur.push(c);
            }
            '>' => return None,
            ':' | ';' if !in_angle => return None,
            ',' if !in_angle => parts.push(std::mem::take(&mut cur)),
            _ => cur.push(c),
        }
    }
    if in_quote || depth > 0 || in_angle {
        return None;
    }
    parts.push(cur);
    Some(parts)
}

/// Satu elemen: `addr-spec` atau `[nama] <addr-spec>` (komentar sudah dibuang).
fn parse_element(element: &str) -> Option<Mailbox> {
    match element.find('<') {
        None => Mailbox::parse(element),
        Some(open) => {
            let close = element.rfind('>')?;
            let tail = element.get(close + 1..)?;
            if close < open || !tail.trim().is_empty() || element.matches('<').count() != 1 {
                return None;
            }
            Mailbox::parse(element.get(open + 1..close)?.trim())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mailbox_accepts_plain_ascii_and_rejects_others() {
        for ok in ["a@b.c", "no-reply+x@uji.invalid", "A.B@Sub.Example.Test"] {
            assert!(Mailbox::parse(ok).is_some(), "{ok}");
        }
        for bad in [
            "",
            "a@b",
            "@b.c",
            "a@",
            "a@@b.c",
            "a@b..c",
            ".a@b.c",
            "a.@b.c",
            "a..b@b.c",
            "a b@c.d",
            "\"a\"@b.c",
            "a@-b.c",
            "a@b_c.d",
            "pàsien@b.c",
            "a@b.c\r\nRCPT TO:<x@y.z>",
            "a@b.c>",
        ] {
            assert!(Mailbox::parse(bad).is_none(), "{bad:?}");
        }
        assert!(Mailbox::parse(&format!("{}@b.c", "a".repeat(65))).is_none());
        assert!(!format!("{:?}", Mailbox::parse("rahasia@b.c").unwrap()).contains("rahasia"));
    }

    #[test]
    fn address_list_handles_display_names_quotes_and_comments() {
        let one =
            |v: &str| parse_address_list(v).map(|l| l.iter().map(Mailbox::key).collect::<Vec<_>>());
        assert_eq!(one("a@b.c"), Some(vec!["a@b.c".to_string()]));
        assert_eq!(
            one("Ruang Sehat <NoReply@Uji.Invalid>"),
            Some(vec!["noreply@uji.invalid".to_string()])
        );
        assert_eq!(
            one("=?UTF-8?Q?Ruang_Sehat?= <n@u.i>"),
            Some(vec!["n@u.i".to_string()])
        );
        assert_eq!(
            one("\"Sehat, Ruang\" <n@u.i>"),
            Some(vec!["n@u.i".to_string()])
        );
        assert_eq!(
            one("n@u.i (komentar, dengan koma)"),
            Some(vec!["n@u.i".to_string()])
        );
        assert_eq!(
            one("a@b.c, \"x\" <d@e.f>"),
            Some(vec!["a@b.c".to_string(), "d@e.f".to_string()])
        );
        for bad in [
            "",
            "a@b.c,",
            ", a@b.c",
            "grup: a@b.c;",
            "<a@b.c",
            "a@b.c>",
            "<a@b.c> <d@e.f>",
            "\"tak tertutup <a@b.c>",
            "(tak tertutup a@b.c",
            "<a@b.c> ekor",
            "nama <bukan-alamat>",
        ] {
            assert!(parse_address_list(bad).is_none(), "{bad:?}");
        }
    }
}
