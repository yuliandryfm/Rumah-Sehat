//! Pemeriksaan header pesan sebelum diteruskan ke SES (ADR-026 butir 4, revisi C-01, M-34 (c)).
//!
//! SES membaca header khusus dari isi pesan Raw (`X-SES-*`) [SRC-365], sehingga relay menolak pesan bila:
//! - ada header berawalan `X-SES-`;
//! - header `From` tidak tepat satu, tidak memuat tepat satu alamat, atau alamatnya bukan pengirim amplop;
//! - `To`/`Cc`/`Bcc` memuat alamat selain penerima amplop (atau `To` lebih dari satu baris);
//! - bagian header bukan ASCII, baris header rusak, atau tidak ada baris kosong pemisah header dan isi.
//!
//! Isi pesan (badan) tidak diperiksa dan tidak pernah dicatat.

use crate::address::{Mailbox, parse_address_list};

/// Alasan penolakan (label tetap untuk log; tanpa nilai header).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeaderViolation {
    /// Tidak ada baris kosong pemisah header dan isi.
    NoBody,
    /// Byte non-ASCII atau kendali di bagian header.
    NonAscii,
    /// Baris header tanpa nama yang sah atau tanpa `:`.
    Malformed,
    /// Header `X-SES-*`.
    SesHeader,
    /// `From` hilang, ganda, atau memuat lebih dari satu alamat.
    FromCount,
    /// Alamat `From` bukan pengirim amplop.
    FromMismatch,
    /// `To`/`Cc`/`Bcc` memuat alamat selain penerima amplop, tidak dapat diurai, atau `To` ganda.
    ExtraRecipient,
}

impl HeaderViolation {
    /// Label log.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NoBody => "no_body",
            Self::NonAscii => "non_ascii_header",
            Self::Malformed => "malformed_header",
            Self::SesHeader => "x_ses_header",
            Self::FromCount => "from_count",
            Self::FromMismatch => "from_mismatch",
            Self::ExtraRecipient => "extra_recipient",
        }
    }
}

/// Header yang sudah di-unfold: (nama, nilai).
fn headers(raw: &[u8]) -> Result<Vec<(String, String)>, HeaderViolation> {
    // Pesan disimpan dengan pemisah CRLF (relay menolak LF/CR tunggal saat DATA).
    let end = raw
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .ok_or(HeaderViolation::NoBody)?;
    let block = raw.get(..end).ok_or(HeaderViolation::NoBody)?;
    if !block
        .iter()
        .all(|&b| b == b'\t' || b == b'\r' || b == b'\n' || (0x20..0x7f).contains(&b))
    {
        return Err(HeaderViolation::NonAscii);
    }
    let text = std::str::from_utf8(block).map_err(|_| HeaderViolation::NonAscii)?;
    let mut out: Vec<(String, String)> = Vec::new();
    for line in text.split("\r\n") {
        if line.starts_with(' ') || line.starts_with('\t') {
            let last = out.last_mut().ok_or(HeaderViolation::Malformed)?;
            last.1.push(' ');
            last.1.push_str(line.trim());
            continue;
        }
        let (name, value) = line.split_once(':').ok_or(HeaderViolation::Malformed)?;
        if name.is_empty() || !name.bytes().all(|b| (33..=126).contains(&b) && b != b':') {
            return Err(HeaderViolation::Malformed);
        }
        out.push((name.to_ascii_lowercase(), value.trim().to_string()));
    }
    Ok(out)
}

/// Memeriksa header terhadap amplop. `from` sudah ada di daftar izin.
pub fn check_headers(raw: &[u8], from: &Mailbox, rcpt: &Mailbox) -> Result<(), HeaderViolation> {
    let hs = headers(raw)?;
    if hs.iter().any(|(n, _)| n.starts_with("x-ses-")) {
        return Err(HeaderViolation::SesHeader);
    }
    let froms: Vec<&String> = hs
        .iter()
        .filter(|(n, _)| n == "from")
        .map(|(_, v)| v)
        .collect();
    let [from_value] = froms.as_slice() else {
        return Err(HeaderViolation::FromCount);
    };
    let from_list = parse_address_list(from_value).ok_or(HeaderViolation::FromCount)?;
    let [header_from] = from_list.as_slice() else {
        return Err(HeaderViolation::FromCount);
    };
    if !header_from.same(from) {
        return Err(HeaderViolation::FromMismatch);
    }
    if hs.iter().filter(|(n, _)| n == "to").count() > 1 {
        return Err(HeaderViolation::ExtraRecipient);
    }
    for (_, value) in hs
        .iter()
        .filter(|(n, _)| matches!(n.as_str(), "to" | "cc" | "bcc"))
    {
        if value.is_empty() {
            continue;
        }
        let list = parse_address_list(value).ok_or(HeaderViolation::ExtraRecipient)?;
        if !list.iter().all(|m| m.same(rcpt)) {
            return Err(HeaderViolation::ExtraRecipient);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mb(s: &str) -> Mailbox {
        Mailbox::parse(s).unwrap()
    }

    fn check(headers: &str) -> Result<(), HeaderViolation> {
        let raw = format!("{headers}\r\n\r\nisi\r\n");
        check_headers(
            raw.as_bytes(),
            &mb("noreply@uji.invalid"),
            &mb("pasien@uji.invalid"),
        )
    }

    const OK: &str =
        "From: Ruang Sehat <noreply@uji.invalid>\r\nTo: pasien@uji.invalid\r\nSubject: Verifikasi";

    #[test]
    fn keycloak_like_headers_pass() {
        assert_eq!(check(OK), Ok(()));
        assert_eq!(
            check("from: NOREPLY@uji.invalid\r\nto: Pasien@Uji.Invalid"),
            Ok(())
        );
        assert_eq!(
            check("From: noreply@uji.invalid\r\nTo:\r\n pasien@uji.invalid\r\nSubject: a\r\n\tb"),
            Ok(())
        );
    }

    #[test]
    fn ses_headers_are_rejected() {
        for h in [
            "X-SES-CONFIGURATION-SET: lain",
            "x-ses-source-arn: arn",
            "X-Ses-From-Arn: x",
        ] {
            assert_eq!(
                check(&format!("{OK}\r\n{h}")),
                Err(HeaderViolation::SesHeader),
                "{h}"
            );
        }
    }

    #[test]
    fn from_must_be_single_and_match_envelope() {
        assert_eq!(
            check("To: pasien@uji.invalid"),
            Err(HeaderViolation::FromCount)
        );
        assert_eq!(
            check(&format!("{OK}\r\nFrom: noreply@uji.invalid")),
            Err(HeaderViolation::FromCount)
        );
        assert_eq!(
            check("From: noreply@uji.invalid, lain@uji.invalid\r\nTo: pasien@uji.invalid"),
            Err(HeaderViolation::FromCount)
        );
        assert_eq!(
            check("From: penyerang@uji.invalid\r\nTo: pasien@uji.invalid"),
            Err(HeaderViolation::FromMismatch)
        );
    }

    #[test]
    fn extra_recipients_in_headers_are_rejected() {
        for h in [
            "From: noreply@uji.invalid\r\nTo: pasien@uji.invalid, lain@uji.invalid",
            "From: noreply@uji.invalid\r\nTo: lain@uji.invalid",
            "From: noreply@uji.invalid\r\nTo: pasien@uji.invalid\r\nCc: lain@uji.invalid",
            "From: noreply@uji.invalid\r\nTo: pasien@uji.invalid\r\nBcc: lain@uji.invalid",
            "From: noreply@uji.invalid\r\nTo: pasien@uji.invalid\r\nTo: pasien@uji.invalid",
            "From: noreply@uji.invalid\r\nTo: grup: lain@uji.invalid;",
        ] {
            assert_eq!(check(h), Err(HeaderViolation::ExtraRecipient), "{h:?}");
        }
    }

    #[test]
    fn malformed_or_non_ascii_headers_are_rejected() {
        assert_eq!(
            check(&format!("{OK}\r\nSubject: Verifikasi \u{e9}")),
            Err(HeaderViolation::NonAscii)
        );
        assert_eq!(
            check(&format!("{OK}\r\nbaris tanpa titik dua")),
            Err(HeaderViolation::Malformed)
        );
        assert_eq!(
            check(&format!(" lanjutan\r\n{OK}")),
            Err(HeaderViolation::Malformed)
        );
        assert_eq!(
            check(&format!("{OK}\r\nNa me: x")),
            Err(HeaderViolation::Malformed)
        );
        let no_body = format!("{OK}\r\n");
        assert_eq!(
            check_headers(
                no_body.as_bytes(),
                &mb("noreply@uji.invalid"),
                &mb("pasien@uji.invalid")
            ),
            Err(HeaderViolation::NoBody)
        );
    }
}
