//! Pengurai minimal respons `ListObjectVersions` (`ListVersionsResult`) [SRC-553]: hanya elemen yang
//! dibutuhkan untuk menghapus semua versi satu kunci (`Version`/`DeleteMarker` → `Key`, `VersionId`;
//! `IsTruncated`, `NextKeyMarker`, `NextVersionIdMarker`). Bukan pengurai XML umum: elemen dicari sebagai
//! teks tag tanpa atribut, entitas XML dasar di-unescape, dan bentuk lain dianggap respons tidak valid.

use crate::error::ObjectStorageError;

/// Satu versi objek atau penanda hapus.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct VersionEntry {
    pub(crate) key: String,
    pub(crate) version_id: String,
}

/// Satu halaman hasil daftar versi.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct VersionPage {
    pub(crate) entries: Vec<VersionEntry>,
    pub(crate) is_truncated: bool,
    pub(crate) next_key_marker: Option<String>,
    pub(crate) next_version_id_marker: Option<String>,
}

fn unescape(raw: &str) -> Result<String, ObjectStorageError> {
    let mut out = String::with_capacity(raw.len());
    let mut rest = raw;
    while let Some(pos) = rest.find('&') {
        out.push_str(rest.get(..pos).ok_or(ObjectStorageError::InvalidResponse)?);
        let tail = rest.get(pos..).ok_or(ObjectStorageError::InvalidResponse)?;
        let (ch, len) = [
            ("&amp;", '&'),
            ("&lt;", '<'),
            ("&gt;", '>'),
            ("&quot;", '"'),
            ("&apos;", '\''),
        ]
        .iter()
        .find(|(ent, _)| tail.starts_with(ent))
        .map(|(ent, c)| (*c, ent.len()))
        .ok_or(ObjectStorageError::InvalidResponse)?;
        out.push(ch);
        rest = tail.get(len..).ok_or(ObjectStorageError::InvalidResponse)?;
    }
    out.push_str(rest);
    Ok(out)
}

/// Isi teks elemen `<tag>...</tag>` pertama di `xml`.
fn element<'a>(xml: &'a str, tag: &str) -> Option<&'a str> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let start = xml.find(&open)? + open.len();
    let len = xml.get(start..)?.find(&close)?;
    xml.get(start..start + len)
}

/// Semua blok `<tag>...</tag>` (isi tanpa tag pembungkus).
fn blocks<'a>(xml: &'a str, tag: &str) -> Vec<&'a str> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let mut out = Vec::new();
    let mut rest = xml;
    while let Some(s) = rest.find(&open) {
        let Some(after) = rest.get(s + open.len()..) else {
            break;
        };
        let Some(e) = after.find(&close) else {
            break;
        };
        if let Some(inner) = after.get(..e) {
            out.push(inner);
        }
        rest = after.get(e + close.len()..).unwrap_or("");
    }
    out
}

/// Mengurai satu halaman `ListVersionsResult`.
pub(crate) fn parse_version_page(xml: &str) -> Result<VersionPage, ObjectStorageError> {
    if !xml.contains("<ListVersionsResult") {
        return Err(ObjectStorageError::InvalidResponse);
    }
    let mut page = VersionPage {
        is_truncated: element(xml, "IsTruncated").map(str::trim) == Some("true"),
        next_key_marker: element(xml, "NextKeyMarker")
            .map(unescape)
            .transpose()?
            .filter(|s| !s.is_empty()),
        next_version_id_marker: element(xml, "NextVersionIdMarker")
            .map(unescape)
            .transpose()?
            .filter(|s| !s.is_empty()),
        ..VersionPage::default()
    };
    for tag in ["Version", "DeleteMarker"] {
        for block in blocks(xml, tag) {
            let key = element(block, "Key").ok_or(ObjectStorageError::InvalidResponse)?;
            let version_id =
                element(block, "VersionId").ok_or(ObjectStorageError::InvalidResponse)?;
            page.entries.push(VersionEntry {
                key: unescape(key)?,
                version_id: unescape(version_id)?,
            });
        }
    }
    Ok(page)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_versions_delete_markers_and_pagination() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ListVersionsResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/">
  <Name>b</Name><Prefix>k</Prefix><IsTruncated>true</IsTruncated>
  <NextKeyMarker>k</NextKeyMarker><NextVersionIdMarker>v&amp;2</NextVersionIdMarker>
  <Version><Key>k</Key><VersionId>v1</VersionId><IsLatest>false</IsLatest></Version>
  <DeleteMarker><Key>k</Key><VersionId>dm1</VersionId></DeleteMarker>
  <Version><Key>k2</Key><VersionId>null</VersionId></Version>
</ListVersionsResult>"#;
        let page = parse_version_page(xml).unwrap();
        assert!(page.is_truncated);
        assert_eq!(page.next_key_marker.as_deref(), Some("k"));
        assert_eq!(page.next_version_id_marker.as_deref(), Some("v&2"));
        assert_eq!(
            page.entries,
            vec![
                VersionEntry {
                    key: "k".into(),
                    version_id: "v1".into()
                },
                VersionEntry {
                    key: "k2".into(),
                    version_id: "null".into()
                },
                VersionEntry {
                    key: "k".into(),
                    version_id: "dm1".into()
                },
            ]
        );
    }

    #[test]
    fn malformed_responses_are_rejected() {
        assert!(parse_version_page("<Error><Code>AccessDenied</Code></Error>").is_err());
        assert!(
            parse_version_page(
                "<ListVersionsResult><Version><Key>k</Key></Version></ListVersionsResult>"
            )
            .is_err()
        );
        assert!(parse_version_page(
            "<ListVersionsResult><Version><Key>a&bogus;</Key><VersionId>1</VersionId></Version></ListVersionsResult>"
        )
        .is_err());
        let empty = parse_version_page(
            "<ListVersionsResult><IsTruncated>false</IsTruncated></ListVersionsResult>",
        )
        .unwrap();
        assert!(empty.entries.is_empty() && !empty.is_truncated);
    }
}
