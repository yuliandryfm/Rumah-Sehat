//! Kepemilikan tabel per modul (coding-standards §1.3, KEPUTUSAN-113, ADR-006).
//!
//! `apps/<app>/module-tables.toml` mencatat tabel milik tiap modul (`[modules.<nama>]`) dan milik
//! crate bersama yang sah (`[crates.<nama>]`: `audit_outbox` lewat `audit`, `data_key` lewat `crypto`).
//! Pemeriksa membaca literal SQL di kode (`apps/<app>/src`, `crates/*/src`) dan berkas `.sql` di sana,
//! lalu menolak tabel yang tidak dimiliki pemilik berkas itu. Nama tabel tanpa awalan skema.
//!
//! Batas yang diketahui: SQL yang dirakit saat runtime dari potongan non-literal tidak terlihat
//! (nama tabel dinamis `{x}` di dalam literal SQL ditolak sebagai `table-dynamic`); test integrasi
//! di `tests/` tidak diperiksa.

use crate::lexer::{line_of, scan};
use crate::{Finding, fs};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

#[derive(Debug, Deserialize, Default)]
pub struct Owned {
    #[serde(default)]
    pub tables: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct ModuleTables {
    /// Nama database; migrasi di `db/migrations/<database>/`.
    pub database: String,
    #[serde(default)]
    pub modules: BTreeMap<String, Owned>,
    #[serde(default)]
    pub crates: BTreeMap<String, Owned>,
}

/// Mengurai isi `module-tables.toml`.
pub fn parse(text: &str) -> Result<ModuleTables, String> {
    toml::from_str(text).map_err(|e| format!("module-tables.toml tidak valid: {e}"))
}

fn f(rule: &'static str, file: &str, line: usize, detail: &str, msg: impl Into<String>) -> Finding {
    Finding::new(rule, file, line, detail, msg)
}

/// Memeriksa setiap `apps/*/module-tables.toml` beserta kode crate bersama.
pub fn check_all_apps(root: &Path) -> Vec<Finding> {
    let mut out = Vec::new();
    let mut crate_owned: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut apps = Vec::new();
    for app in fs::subdirs(&root.join("apps")) {
        let rel = format!("apps/{app}/module-tables.toml");
        let path = root.join(&rel);
        if !path.exists() {
            if root.join("apps").join(&app).join("src/modules").is_dir() {
                out.push(f(
                    "table-owner",
                    &rel,
                    0,
                    "module-tables.toml",
                    "app dengan src/modules wajib punya module-tables.toml",
                ));
            }
            continue;
        }
        let parsed = std::fs::read_to_string(&path)
            .map_err(|e| e.to_string())
            .and_then(|t| parse(&t));
        match parsed {
            Ok(mt) => {
                for (c, o) in &mt.crates {
                    crate_owned
                        .entry(c.clone())
                        .or_default()
                        .extend(o.tables.iter().cloned());
                }
                apps.push((app, mt));
            }
            Err(e) => out.push(f("table-owner", &rel, 0, "parse", e)),
        }
    }
    for (app, mt) in &apps {
        out.extend(check_app(root, app, mt));
    }
    if !apps.is_empty() {
        out.extend(check_crates(root, &crate_owned));
    }
    out
}

fn duplicate_owners(file: &str, mt: &ModuleTables) -> Vec<Finding> {
    let mut seen: BTreeMap<&str, String> = BTreeMap::new();
    let mut out = Vec::new();
    let all = mt
        .modules
        .iter()
        .map(|(n, o)| (format!("modul {n}"), o))
        .chain(mt.crates.iter().map(|(n, o)| (format!("crate {n}"), o)));
    for (owner, o) in all {
        for t in &o.tables {
            if let Some(prev) = seen.insert(t.as_str(), owner.clone()) {
                out.push(f(
                    "table-owner",
                    file,
                    0,
                    t,
                    format!("tabel dimiliki dua pihak: {prev} dan {owner}"),
                ));
            }
        }
    }
    out
}

/// Memeriksa satu app terhadap catatannya (modul, migrasi).
pub fn check_app(root: &Path, app: &str, mt: &ModuleTables) -> Vec<Finding> {
    let toml_file = format!("apps/{app}/module-tables.toml");
    let mut out = duplicate_owners(&toml_file, mt);
    let modules_dir = root.join("apps").join(app).join("src/modules");
    for m in fs::subdirs(&modules_dir) {
        if !mt.modules.contains_key(&m) {
            out.push(f(
                "table-owner",
                &toml_file,
                0,
                &m,
                format!(
                    "modul `{m}` ada di src/modules tetapi belum terdaftar di module-tables.toml"
                ),
            ));
        }
    }
    let src = root.join("apps").join(app).join("src");
    for p in fs::files_with_ext(&src, "rs")
        .into_iter()
        .chain(fs::files_with_ext(&src, "sql"))
    {
        let file = fs::rel(root, &p);
        let Ok(text) = std::fs::read_to_string(&p) else {
            continue;
        };
        let prefix = format!("apps/{app}/src/modules/");
        let owner = file
            .strip_prefix(&prefix)
            .and_then(|r| r.split('/').next())
            .map(str::to_string);
        let owned: Option<&Owned> = owner.as_ref().and_then(|m| mt.modules.get(m));
        let label = owner
            .as_deref()
            .map_or("(di luar modul)".to_string(), |m| format!("modul {m}"));
        out.extend(check_text(&file, &text, &label, owned, mt));
    }
    out.extend(check_migrations(root, app, mt));
    out
}

fn check_crates(root: &Path, crate_owned: &BTreeMap<String, BTreeSet<String>>) -> Vec<Finding> {
    let mut out = Vec::new();
    for c in fs::subdirs(&root.join("crates")) {
        if c == "arch-check" {
            continue;
        }
        let src = root.join("crates").join(&c).join("src");
        let owned = Owned {
            tables: crate_owned
                .get(&c)
                .map(|s| s.iter().cloned().collect())
                .unwrap_or_default(),
        };
        let mt_all = ModuleTables {
            database: String::new(),
            modules: BTreeMap::new(),
            crates: crate_owned
                .iter()
                .map(|(k, v)| {
                    (
                        k.clone(),
                        Owned {
                            tables: v.iter().cloned().collect(),
                        },
                    )
                })
                .collect(),
        };
        for p in fs::files_with_ext(&src, "rs")
            .into_iter()
            .chain(fs::files_with_ext(&src, "sql"))
        {
            let file = fs::rel(root, &p);
            if let Ok(text) = std::fs::read_to_string(&p) {
                out.extend(check_text(
                    &file,
                    &text,
                    &format!("crate {c}"),
                    Some(&owned),
                    &mt_all,
                ));
            }
        }
    }
    out
}

/// Memeriksa satu berkas: tiap tabel di SQL harus dimiliki `owned` (None = tanpa pemilik).
pub fn check_text(
    file: &str,
    text: &str,
    label: &str,
    owned: Option<&Owned>,
    mt: &ModuleTables,
) -> Vec<Finding> {
    let mut out = Vec::new();
    for (line, table) in sql_tables_in_file(file, text) {
        if table.starts_with('{') {
            out.push(f("table-dynamic", file, line, &table, "nama tabel dinamis di literal SQL tidak dapat diperiksa kepemilikannya; tulis nama tabel literal"));
            continue;
        }
        if owned.is_some_and(|o| o.tables.contains(&table)) {
            continue;
        }
        let actual = mt
            .modules
            .iter()
            .map(|(n, o)| (format!("modul {n}"), o))
            .chain(mt.crates.iter().map(|(n, o)| (format!("crate {n}"), o)))
            .find(|(_, o)| o.tables.contains(&table))
            .map(|(n, _)| n);
        let msg = match actual {
            Some(a) => format!(
                "{label} mengakses tabel `{table}` milik {a}; pakai api.rs pemiliknya (coding-standards §1.3)"
            ),
            None => format!(
                "{label} mengakses tabel `{table}` yang tidak terdaftar di module-tables.toml"
            ),
        };
        out.push(f("table-owner", file, line, &table, msg));
    }
    out
}

fn check_migrations(root: &Path, app: &str, mt: &ModuleTables) -> Vec<Finding> {
    let dir = root.join("db/migrations").join(&mt.database);
    let mut out = Vec::new();
    let owned: BTreeSet<&str> = mt
        .modules
        .values()
        .chain(mt.crates.values())
        .flat_map(|o| o.tables.iter().map(String::as_str))
        .collect();
    for p in fs::files_with_ext(&dir, "sql") {
        let file = fs::rel(root, &p);
        let Ok(text) = std::fs::read_to_string(&p) else {
            continue;
        };
        for t in created_tables(&text) {
            if !owned.contains(t.as_str()) {
                out.push(f("table-owner", &file, 0, &t, format!("tabel `{t}` dibuat di migrasi tetapi belum dimiliki siapa pun di apps/{app}/module-tables.toml")));
            }
        }
    }
    out
}

// ---------------------------------------------------------------- SQL

#[derive(Debug)]
struct Tok {
    text: String,
    word: bool,
    /// Karakter tepat setelah token adalah `(` (tanpa spasi).
    paren_next: bool,
}

fn tokenize(sql: &str) -> Vec<Tok> {
    let chars: Vec<char> = sql.chars().collect();
    let mut toks = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars.get(i).copied().unwrap_or(' ');
        if c == '-' && chars.get(i + 1) == Some(&'-') {
            while i < chars.len() && chars.get(i) != Some(&'\n') {
                i += 1;
            }
        } else if c == '/' && chars.get(i + 1) == Some(&'*') {
            i += 2;
            while i < chars.len() && !(chars.get(i) == Some(&'*') && chars.get(i + 1) == Some(&'/'))
            {
                i += 1;
            }
            i += 2;
        } else if c == '\'' {
            i += 1;
            while i < chars.len() && chars.get(i) != Some(&'\'') {
                i += 1;
            }
            i += 1;
        } else if c.is_alphanumeric() || matches!(c, '_' | '.' | '"' | '$' | '{' | '}') {
            let mut w = String::new();
            while let Some(&d) = chars.get(i) {
                if d.is_alphanumeric() || matches!(d, '_' | '.' | '$' | '{' | '}') {
                    w.push(d.to_ascii_lowercase());
                } else if d != '"' {
                    break;
                }
                i += 1;
            }
            let paren_next = chars.get(i) == Some(&'(');
            toks.push(Tok {
                text: w,
                word: true,
                paren_next,
            });
        } else {
            if !c.is_whitespace() {
                toks.push(Tok {
                    text: c.to_string(),
                    word: false,
                    paren_next: false,
                });
            }
            i += 1;
        }
    }
    toks
}

const NOT_ALIAS: &[&str] = &[
    "where",
    "join",
    "left",
    "right",
    "inner",
    "outer",
    "full",
    "cross",
    "natural",
    "on",
    "using",
    "group",
    "order",
    "limit",
    "offset",
    "having",
    "union",
    "intersect",
    "except",
    "set",
    "returning",
    "for",
    "window",
    "values",
    "select",
    "fetch",
    "tablesample",
    "lateral",
    "from",
    "do",
    "when",
    "then",
];
const FROM_IN_FUNCS: &[&str] = &[
    "extract",
    "trim",
    "substring",
    "overlay",
    "position",
    "date_part",
];

fn tx(toks: &[Tok], i: usize) -> &str {
    toks.get(i).map_or("", |t| t.text.as_str())
}

/// Nama tabel (tanpa skema, huruf kecil) yang diakses oleh pernyataan SQL `sql`.
pub fn tables_in_sql(sql: &str) -> Vec<String> {
    let toks = tokenize(sql);
    let mut ctes: BTreeSet<String> = BTreeSet::new();
    for (i, t) in toks.iter().enumerate() {
        let mut j = i + 1;
        if t.word && tx(&toks, j) == "as" {
            j += 1;
            if tx(&toks, j) == "not" {
                j += 1;
            }
            if tx(&toks, j) == "materialized" {
                j += 1;
            }
            if tx(&toks, j) == "(" {
                ctes.insert(t.text.clone());
            }
        }
    }
    let mut found = Vec::new();
    let mut parens: Vec<String> = Vec::new();
    let mut i = 0;
    while i < toks.len() {
        let t = toks.get(i);
        let text = tx(&toks, i);
        let prev = if i == 0 { "" } else { tx(&toks, i - 1) };
        if text == "(" {
            parens.push(prev.to_string());
        } else if text == ")" {
            parens.pop();
        }
        if t.is_some_and(|t| t.word) {
            match text {
                "from" => {
                    let in_func = parens
                        .last()
                        .is_some_and(|p| FROM_IN_FUNCS.contains(&p.as_str()));
                    if prev != "distinct" && !in_func {
                        i = table_refs(&toks, i + 1, true, true, &mut found);
                        continue;
                    }
                }
                "join" => {
                    i = table_refs(&toks, i + 1, false, true, &mut found);
                    continue;
                }
                "into" if prev == "insert" => {
                    i = table_refs(&toks, i + 1, false, false, &mut found);
                    continue;
                }
                "update" if !matches!(prev, "for" | "do" | "key" | "share") => {
                    i = table_refs(&toks, i + 1, false, false, &mut found);
                    continue;
                }
                "truncate" => {
                    let next = if tx(&toks, i + 1) == "table" {
                        i + 2
                    } else {
                        i + 1
                    };
                    i = table_refs(&toks, next, false, false, &mut found);
                    continue;
                }
                _ => {}
            }
        }
        i += 1;
    }
    found.into_iter().filter(|n| !ctes.contains(n)).collect()
}

/// Mengurai referensi tabel mulai `i`; `list` mengizinkan daftar dipisah koma (FROM a, b);
/// `func_check` menganggap `nama(` sebagai fungsi (FROM/JOIN), bukan tabel (INSERT INTO t(kolom)).
fn table_refs(
    toks: &[Tok],
    mut i: usize,
    list: bool,
    func_check: bool,
    found: &mut Vec<String>,
) -> usize {
    loop {
        while matches!(tx(toks, i), "only" | "lateral") {
            i += 1;
        }
        let Some(t) = toks.get(i) else {
            return i;
        };
        if !t.word {
            return i; // subquery "(" atau lainnya
        }
        i += 1;
        let is_func = func_check && t.paren_next;
        if !is_func && let Some(n) = normalize(&t.text) {
            found.push(n);
        }
        // alias opsional
        if tx(toks, i) == "as" {
            i += 2;
        } else if toks.get(i).is_some_and(|a| a.word) && !NOT_ALIAS.contains(&tx(toks, i)) {
            i += 1;
        }
        if list && tx(toks, i) == "," {
            i += 1;
            continue;
        }
        return i;
    }
}

fn normalize(raw: &str) -> Option<String> {
    if raw.starts_with('{') {
        return Some(raw.to_string());
    }
    if !raw.starts_with(|c: char| c.is_alphabetic() || c == '_') {
        return None;
    }
    if raw.starts_with("information_schema.")
        || raw.starts_with("pg_catalog.")
        || raw.starts_with("pg_")
    {
        return None;
    }
    let name = raw.rsplit('.').next().unwrap_or(raw);
    if name.is_empty() {
        None
    } else {
        Some(name.to_string())
    }
}

/// Tabel yang dibuat oleh `CREATE TABLE` di teks migrasi (partisi `PARTITION OF` dilewati).
pub fn created_tables(sql: &str) -> Vec<String> {
    let toks = tokenize(sql);
    let mut out = Vec::new();
    let mut i = 0;
    while i < toks.len() {
        if tx(&toks, i) == "create" {
            let mut j = i + 1;
            while matches!(
                tx(&toks, j),
                "unlogged" | "temp" | "temporary" | "global" | "local"
            ) {
                j += 1;
            }
            if tx(&toks, j) == "table" {
                j += 1;
                if tx(&toks, j) == "if" {
                    j += 3; // if not exists
                }
                let name = tx(&toks, j);
                let is_partition = tx(&toks, j + 1) == "partition";
                if !is_partition && let Some(n) = normalize(name) {
                    out.push(n);
                }
            }
        }
        i += 1;
    }
    out
}

fn looks_like_sql(lit: &str) -> bool {
    let body: String = lit
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with("--"))
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase();
    let words: Vec<&str> = body.split_whitespace().collect();
    let has = |w: &str| words.contains(&w);
    match words.first().copied() {
        Some("select") => has("from"),
        Some("insert") => has("into"),
        Some("update") => has("set"),
        Some("delete") => has("from"),
        Some("with") => {
            has("as") && (has("select") || has("insert") || has("update") || has("delete"))
        }
        Some("truncate") => true,
        _ => false,
    }
}

/// (nomor baris, tabel) untuk semua SQL di berkas `.rs` (literal) atau `.sql` (seluruh isi).
pub fn sql_tables_in_file(file: &str, text: &str) -> Vec<(usize, String)> {
    if file.ends_with(".sql") {
        return tables_in_sql(text).into_iter().map(|t| (0, t)).collect();
    }
    let sc = scan(text);
    let mut out = Vec::new();
    for (off, lit) in &sc.literals {
        if looks_like_sql(lit) {
            let line = line_of(&sc.code, *off);
            out.extend(tables_in_sql(lit).into_iter().map(|t| (line, t)));
        }
    }
    out
}
