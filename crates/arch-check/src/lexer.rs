//! Pemindai Rust ringan: membuang komentar dan mengosongkan isi literal string supaya pencarian
//! teks tidak tertipu komentar atau pesan. Bukan parser penuh; cukup untuk aturan arsitektur.

/// Hasil pemindaian satu berkas.
pub struct Scanned {
    /// Kode dengan komentar dan isi literal diganti spasi (newline dipertahankan, panjang sama).
    pub code: String,
    /// Isi mentah tiap literal string beserta offset byte awalnya di `code`.
    pub literals: Vec<(usize, String)>,
}

fn at(b: &[u8], i: usize) -> u8 {
    b.get(i).copied().unwrap_or(0)
}

/// Memindai `src`. Escape dan raw string (`r#"..."#`) ditangani; `'a` (lifetime) dibedakan dari `'a'`.
pub fn scan(src: &str) -> Scanned {
    let b = src.as_bytes();
    let n = b.len();
    let mut out: Vec<u8> = Vec::with_capacity(n);
    let mut literals = Vec::new();
    let blank = |c: u8| if c == b'\n' { b'\n' } else { b' ' };
    let mut i = 0;
    while i < n {
        let c = at(b, i);
        if c == b'/' && at(b, i + 1) == b'/' {
            while i < n && at(b, i) != b'\n' {
                out.push(b' ');
                i += 1;
            }
        } else if c == b'/' && at(b, i + 1) == b'*' {
            let mut depth = 0usize;
            while i < n {
                if at(b, i) == b'/' && at(b, i + 1) == b'*' {
                    depth += 1;
                    out.extend_from_slice(b"  ");
                    i += 2;
                } else if at(b, i) == b'*' && at(b, i + 1) == b'/' {
                    depth = depth.saturating_sub(1);
                    out.extend_from_slice(b"  ");
                    i += 2;
                    if depth == 0 {
                        break;
                    }
                } else {
                    out.push(blank(at(b, i)));
                    i += 1;
                }
            }
        } else if c == b'"'
            || (c == b'r' && is_raw_start(b, i))
            || (c == b'b' && (at(b, i + 1) == b'"' || at(b, i + 1) == b'r') && !prev_is_ident(b, i))
        {
            let start = out.len();
            let (end, raw) = string_end(b, i);
            let lit = src.get(i..end).unwrap_or("");
            literals.push((start, lit_body(lit, raw)));
            for k in i..end {
                let ch = at(b, k);
                out.push(if ch == b'"' || ch == b'\n' { ch } else { b' ' });
            }
            i = end;
        } else if c == b'\'' {
            let is_char = at(b, i + 1) == b'\\'
                || at(b, i + 2) == b'\''
                || (at(b, i + 1) >= 0x80 && (i + 2..i + 6).any(|k| at(b, k) == b'\''));
            if is_char {
                let mut k = i + 1;
                if at(b, k) == b'\\' {
                    k += 1;
                }
                k += 1;
                while k < n && at(b, k) != b'\'' {
                    k += 1;
                }
                out.extend(std::iter::repeat_n(
                    b' ',
                    k.min(n.saturating_sub(1)) + 1 - i,
                ));
                i = k + 1;
            } else {
                out.push(c);
                i += 1;
            }
        } else {
            out.push(c);
            i += 1;
        }
    }
    out.truncate(n);
    Scanned {
        code: String::from_utf8_lossy(&out).into_owned(),
        literals,
    }
}

fn prev_is_ident(b: &[u8], i: usize) -> bool {
    i > 0 && is_ident(at(b, i - 1))
}

fn is_raw_start(b: &[u8], i: usize) -> bool {
    // r"..." atau r#"..."#; tidak boleh bagian dari identifier (misalnya `for"`).
    if prev_is_ident(b, i) {
        return false;
    }
    let mut k = i + 1;
    while at(b, k) == b'#' {
        k += 1;
    }
    at(b, k) == b'"'
}

/// Mengembalikan (indeks setelah literal, apakah raw).
fn string_end(b: &[u8], i: usize) -> (usize, bool) {
    let n = b.len();
    let mut k = i;
    if at(b, k) == b'b' {
        k += 1;
    }
    let raw = at(b, k) == b'r';
    if raw {
        k += 1;
        let mut hashes = 0;
        while at(b, k) == b'#' {
            hashes += 1;
            k += 1;
        }
        k += 1; // tanda kutip pembuka
        while k < n {
            if at(b, k) == b'"' && (1..=hashes).all(|h| at(b, k + h) == b'#') {
                return (k + 1 + hashes, true);
            }
            k += 1;
        }
        return (n, true);
    }
    k += 1;
    while k < n {
        match at(b, k) {
            b'\\' => k += 2,
            b'"' => return (k + 1, false),
            _ => k += 1,
        }
    }
    (n, false)
}

fn lit_body(lit: &str, raw: bool) -> String {
    let start = lit.find('"').map_or(0, |p| p + 1);
    let end = if raw {
        lit.rfind('"').unwrap_or(lit.len())
    } else {
        lit.len().saturating_sub(1)
    };
    lit.get(start..end.max(start)).unwrap_or("").to_string()
}

/// Nomor baris (1-based) untuk offset byte.
pub fn line_of(code: &str, offset: usize) -> usize {
    code.as_bytes()
        .iter()
        .take(offset)
        .filter(|&&c| c == b'\n')
        .count()
        + 1
}

/// Indeks penutup untuk pembuka di `open` (salah satu `([{`), atau `None`.
pub fn matching_close(code: &str, open: usize) -> Option<usize> {
    let mut depth = 0usize;
    for (k, &c) in code.as_bytes().iter().enumerate().skip(open) {
        match c {
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(k);
                }
            }
            _ => {}
        }
    }
    None
}

/// Benar bila `c` karakter identifier.
pub fn is_ident(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_'
}

/// Semua kemunculan `needle` di `code` sebagai token utuh (tidak menempel pada identifier).
pub fn find_tokens(code: &str, needle: &str) -> Vec<usize> {
    let b = code.as_bytes();
    let mut res = Vec::new();
    let mut from = 0;
    while let Some(p) = code.get(from..).and_then(|s| s.find(needle)) {
        let s = from + p;
        let e = s + needle.len();
        let before_ok = s == 0 || !is_ident(at(b, s - 1));
        let after_ok =
            !is_ident(at(b, e)) || !is_ident(at(needle.as_bytes(), needle.len().saturating_sub(1)));
        if before_ok && after_ok {
            res.push(s);
        }
        from = e;
    }
    res
}

/// Nama fungsi terdekat: `after = false` mencari `fn x` sebelum `offset`, `true` sesudahnya.
pub fn enclosing_fn(code: &str, offset: usize, after: bool) -> Option<String> {
    let pos = if after {
        let s = offset;
        code.get(s..)?.find("fn ").map(|p| s + p)?
    } else {
        code.get(..offset)?.rfind("fn ")?
    };
    let rest = code.get(pos + 3..)?;
    let name: String = rest
        .trim_start()
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
        .collect();
    if name.is_empty() { None } else { Some(name) }
}
