//! Alat pengembangan: menulis spesifikasi OpenAPI hasil generate ke direktori yang diberikan.
//!
//! Pemakaian: `cargo run -p rs-backend --bin gen-openapi -- openapi` (memperbarui berkas di-commit)
//! atau dengan direktori sementara (dipakai `scripts/verify.sh` untuk perbandingan). Bukan bagian
//! biner layanan.

use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [dir] = args.as_slice() else {
        eprintln!("pemakaian: gen-openapi <direktori-tujuan>");
        return ExitCode::from(2);
    };
    let specs = match rs_backend::openapi::generate() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("gagal membangkitkan spesifikasi: {e}");
            return ExitCode::FAILURE;
        }
    };
    let dir = PathBuf::from(dir);
    if let Err(e) = std::fs::create_dir_all(&dir) {
        eprintln!("gagal membuat direktori: {e}");
        return ExitCode::FAILURE;
    }
    for (name, json) in specs {
        if let Err(e) = std::fs::write(dir.join(name), json) {
            eprintln!("gagal menulis {name}: {e}");
            return ExitCode::FAILURE;
        }
        println!("ditulis: {name}");
    }
    ExitCode::SUCCESS
}
