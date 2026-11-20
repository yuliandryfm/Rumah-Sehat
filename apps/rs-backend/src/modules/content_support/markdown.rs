//! Aturan render aman isi CMS (F1-T111, M-40, ADR-027 butir 5, US-1802 AC6, KEPUTUSAN-264), divalidasi
//! backend saat konten disimpan. Bagian yang melanggar **ditolak** (bukan dinetralkan diam-diam), dengan
//! galat per baris tanpa menggemakan isi berbahaya (wireframe C14):
//!
//! 1. Markdown CommonMark tanpa ekstensi, diurai pulldown-cmark 0.13.6 (`Options::empty()`, [SRC-600]).
//!    HTML mentah dalam bentuk apa pun (blok HTML maupun HTML sebaris, termasuk komentar dan atribut `on*`)
//!    ditolak.
//! 2. Tautan (inline, rujukan, autolink) hanya berskema `https` atau `mailto`, atau relatif terhadap situs
//!    (`/jalur`, `jalur`, `#jangkar`, `?q`). Tujuan yang memuat spasi atau karakter kendali (browser
//!    membuang tab/baris baru di dalam URL sehingga `java\tscript:` menjadi `javascript:`), garis miring
//!    terbalik, atau diawali `//` (host lain tanpa skema) ditolak.
//! 3. Gambar hanya dari endpoint gambar publik backend: tepat `/v1/public/content/images/<uuid>`
//!    ([`PUBLIC_IMAGE_PATH`]), dan wajib bertekst alternatif (US-1802 AC5).
//!
//! Pemeriksaan ini satu lapis dari beberapa: situs dan pratinjau backoffice tetap merender tanpa HTML mentah
//! dengan daftar izin skema yang sama, pratinjau di iframe `sandbox`, dan CSP tanpa skrip inline (F1-T95,
//! F1-T96, ADR-027 butir 5).

use envelope::FieldError;
use foundation::Uuid;
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

/// Awalan jalur endpoint gambar konten publik (F1-T93); satu-satunya sumber gambar yang diizinkan.
pub(crate) const PUBLIC_IMAGE_PATH: &str = "/v1/public/content/images/";

/// Pelanggaran aturan render aman.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Violation {
    /// HTML mentah (blok atau sebaris).
    RawHtml,
    /// Tujuan tautan di luar daftar izin skema.
    LinkScheme,
    /// Sumber gambar bukan endpoint gambar publik backend.
    ImageSource,
    /// Gambar tanpa teks alternatif.
    ImageAlt,
}

impl Violation {
    fn code(self) -> &'static str {
        match self {
            Self::RawHtml => "raw_html_not_allowed",
            Self::LinkScheme => "link_scheme_not_allowed",
            Self::ImageSource => "image_source_not_allowed",
            Self::ImageAlt => "image_alt_missing",
        }
    }

    fn message(self, line: usize) -> String {
        let what = match self {
            Self::RawHtml => "HTML mentah tidak diizinkan",
            Self::LinkScheme => "tautan harus berskema https, mailto, atau relatif",
            Self::ImageSource => "gambar harus dari pustaka gambar Ruang Sehat",
            Self::ImageAlt => "gambar tanpa teks alternatif",
        };
        format!("{what} (baris {line})")
    }
}

/// Tujuan tautan yang diizinkan (aturan 2).
pub(super) fn allowed_link(dest: &str) -> bool {
    if dest.is_empty() {
        return true;
    }
    if dest
        .chars()
        .any(|c| c.is_control() || c.is_whitespace() || c == '\\')
    {
        return false;
    }
    if dest.starts_with("//") {
        return false;
    }
    // Skema = bagian sebelum ':' bila ':' muncul sebelum '/', '?', atau '#' (RFC 3986 §3, §4.2).
    let boundary = dest.find(['/', '?', '#']).unwrap_or(dest.len());
    match dest.get(..boundary).and_then(|head| head.find(':')) {
        None => true,
        Some(colon) => {
            let scheme = dest.get(..colon).unwrap_or_default().to_ascii_lowercase();
            let rest = dest.get(colon + 1..).unwrap_or_default();
            match scheme.as_str() {
                "https" => rest.starts_with("//") && rest.len() > 2,
                "mailto" => !rest.is_empty(),
                _ => false,
            }
        }
    }
}

/// Sumber gambar yang diizinkan (aturan 3): tepat `/v1/public/content/images/<uuid huruf kecil>`.
pub(super) fn allowed_image(dest: &str) -> bool {
    dest.strip_prefix(PUBLIC_IMAGE_PATH)
        .and_then(|id| Uuid::parse_str(id).ok().map(|u| (id, u)))
        .is_some_and(|(id, u)| u.hyphenated().to_string() == id)
}

fn line_of(text: &str, offset: usize) -> usize {
    text.get(..offset)
        .map_or(1, |head| head.bytes().filter(|b| *b == b'\n').count() + 1)
}

/// Semua pelanggaran beserta nomor baris (1-based), urut kemunculan.
pub(super) fn violations(markdown: &str) -> Vec<(Violation, usize)> {
    let mut out = Vec::new();
    // (baris awal gambar, teks alternatif terkumpul) selama berada di dalam tag gambar.
    let mut image: Option<(usize, String)> = None;
    for (event, range) in Parser::new_ext(markdown, Options::empty()).into_offset_iter() {
        let line = line_of(markdown, range.start);
        match event {
            Event::Html(_) | Event::InlineHtml(_) => out.push((Violation::RawHtml, line)),
            Event::Start(Tag::Link { dest_url, .. }) => {
                if !allowed_link(&dest_url) {
                    out.push((Violation::LinkScheme, line));
                }
            }
            Event::Start(Tag::Image { dest_url, .. }) => {
                if !allowed_image(&dest_url) {
                    out.push((Violation::ImageSource, line));
                }
                image = Some((line, String::new()));
            }
            Event::End(TagEnd::Image) => {
                if let Some((start, alt)) = image.take()
                    && alt.trim().is_empty()
                {
                    out.push((Violation::ImageAlt, start));
                }
            }
            Event::Text(t) | Event::Code(t) => {
                if let Some((_, alt)) = image.as_mut() {
                    alt.push_str(&t);
                }
            }
            _ => {}
        }
    }
    out
}

/// Galat per baris untuk field `field` (KU-01); kosong = aman disimpan.
pub(super) fn field_errors(field: &str, markdown: &str) -> Vec<FieldError> {
    violations(markdown)
        .into_iter()
        .map(|(v, line)| FieldError::new(field, v.code(), v.message(line)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const IMG: &str = "/v1/public/content/images/0193a1b2-c3d4-7e5f-8a6b-7c8d9e0f1a2b";

    fn kinds(md: &str) -> Vec<Violation> {
        violations(md).into_iter().map(|(v, _)| v).collect()
    }

    // trace: US-1802 AC6
    #[test]
    fn safe_markdown_passes() {
        let md = format!(
            "## Siapa kami\n\nTeks **tebal**, *miring*, `kode`.\n\n- [Layanan](/layanan)\n- [Kontak](mailto:halo@example.com)\n- [Mitra](https://example.com/mitra)\n- [Atas](#atas)\n\n![Foto klinik]({IMG})\n\n> kutipan\n\n```\n<script>dalam blok kode = teks</script>\n```\n"
        );
        assert_eq!(kinds(&md), []);
        assert!(field_errors("body", &md).is_empty());
    }

    // trace: US-1802 AC6
    #[test]
    fn raw_html_in_every_form_is_rejected() {
        let payloads = [
            "<script>alert(1)</script>",
            "teks <script>alert(1)</script> teks",
            "<iframe src=\"https://example.com\"></iframe>",
            "<img src=x onerror=alert(1)>",
            "<a href=\"javascript:alert(1)\">x</a>",
            "<div onclick=\"alert(1)\">x</div>",
            "<svg><script>alert(1)</script></svg>",
            "<!-- komentar -->",
            "<style>body{}</style>",
            "<details open ontoggle=alert(1)>",
            "teks <b onmouseover=alert(1)>b</b>",
            "<math><mtext></mtext></math>",
        ];
        for p in payloads {
            assert!(kinds(p).contains(&Violation::RawHtml), "{p}");
        }
    }

    // trace: US-1802 AC6
    #[test]
    fn dangerous_link_schemes_are_rejected() {
        let payloads = [
            "[x](javascript:alert(1))",
            "[x](JaVaScRiPt:alert(1))",
            "[x](jav&#x61;script:alert(1))",
            "[x](jav&#97;script:alert(1))",
            "[x](<javascript:alert(1)>)",
            "[x](<java\tscript:alert(1)>)",
            "[x](data:text/html;base64,PHNjcmlwdD4=)",
            "[x](vbscript:msgbox(1))",
            "[x](file:///etc/passwd)",
            "[x](http://example.com)",
            "[x](//example.com/jalur)",
            "[x](\\\\example.com)",
            "[x](https:example.com)",
            "<javascript:alert(1)>",
            "<http://example.com>",
            "[x][r]\n\n[r]: javascript:alert(1)",
            "[x](mailto:)",
        ];
        for p in payloads {
            assert!(kinds(p).contains(&Violation::LinkScheme), "{p:?}");
        }
        assert!(allowed_link("https://example.com"));
        assert!(allowed_link("relatif/jalur?x=a:b"));
        assert!(allowed_link("?q=1"));
        assert!(!allowed_link("javascript:"));
    }

    // trace: US-1802 AC6
    #[test]
    fn images_only_from_public_endpoint_with_alt_text() {
        let payloads = [
            "![a](https://img.example.test/x.png)",
            "![a](/v1/public/content/images/bukan-uuid)",
            "![a](/v1/public/content/images/0193A1B2-C3D4-7E5F-8A6B-7C8D9E0F1A2B)",
            "![a](/v1/public/content/images/0193a1b2-c3d4-7e5f-8a6b-7c8d9e0f1a2b/../../x)",
            "![a](javascript:alert(1))",
            "![a](data:image/png;base64,AAAA)",
            "![a](/v1/backoffice/x.png)",
        ];
        for p in payloads {
            assert!(kinds(p).contains(&Violation::ImageSource), "{p}");
        }
        assert_eq!(kinds(&format!("![]({IMG})")), [Violation::ImageAlt]);
        assert_eq!(kinds(&format!("![  ]({IMG})")), [Violation::ImageAlt]);
        assert_eq!(kinds(&format!("![*Foto* `x`]({IMG})")), []);
    }

    #[test]
    fn errors_name_line_without_echoing_payload() {
        let md = "## Siapa kami\n[Teks]\n\n![ ](https://img.example.test/x.png)\n<script>alert(1)</script>\n[klik](javascript:alert(1))\n";
        let e = field_errors("body", md);
        let got: Vec<(&str, &str)> = e
            .iter()
            .map(|x| (x.code.as_str(), x.message.as_str()))
            .collect();
        assert_eq!(
            got,
            [
                (
                    "image_source_not_allowed",
                    "gambar harus dari pustaka gambar Ruang Sehat (baris 4)"
                ),
                (
                    "image_alt_missing",
                    "gambar tanpa teks alternatif (baris 4)"
                ),
                (
                    "raw_html_not_allowed",
                    "HTML mentah tidak diizinkan (baris 5)"
                ),
                (
                    "link_scheme_not_allowed",
                    "tautan harus berskema https, mailto, atau relatif (baris 6)"
                ),
            ]
        );
        for x in &e {
            assert_eq!(x.field, "body");
            assert!(!x.message.contains("script") && !x.message.contains("javascript"));
        }
    }
}
