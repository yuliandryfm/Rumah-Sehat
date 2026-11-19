//! Penelusuran berkas sederhana (tanpa dependensi tambahan).

use std::path::{Path, PathBuf};

/// Daftar berkas berekstensi `ext` di bawah `dir` secara rekursif, terurut, tanpa `target`,
/// `node_modules`, dan direktori tersembunyi. Path hasil absolut (turunan `dir`).
pub fn files_with_ext(dir: &Path, ext: &str) -> Vec<PathBuf> {
    let mut out = Vec::new();
    walk(dir, &mut |p| {
        if p.extension().and_then(|e| e.to_str()) == Some(ext) {
            out.push(p.to_path_buf());
        }
    });
    out.sort();
    out
}

/// Semua berkas bernama `name` di bawah `dir`.
pub fn files_named(dir: &Path, name: &str) -> Vec<PathBuf> {
    let mut out = Vec::new();
    walk(dir, &mut |p| {
        if p.file_name().and_then(|e| e.to_str()) == Some(name) {
            out.push(p.to_path_buf());
        }
    });
    out.sort();
    out
}

fn walk(dir: &Path, f: &mut dyn FnMut(&Path)) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in rd.flatten() {
        let p = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        if p.is_dir() {
            if name == "target" || name == "node_modules" || name.starts_with('.') {
                continue;
            }
            walk(&p, f);
        } else {
            f(&p);
        }
    }
}

/// Path relatif terhadap `root` dengan pemisah `/`.
pub fn rel(root: &Path, p: &Path) -> String {
    p.strip_prefix(root)
        .unwrap_or(p)
        .to_string_lossy()
        .replace('\\', "/")
}

/// Subdirektori langsung (hanya nama) dari `dir`, terurut.
pub fn subdirs(dir: &Path) -> Vec<String> {
    let mut v: Vec<String> = std::fs::read_dir(dir)
        .map(|rd| {
            rd.flatten()
                .filter(|e| e.path().is_dir())
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default();
    v.sort();
    v
}
