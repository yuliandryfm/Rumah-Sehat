//! Titik masuk `rs-backend`. Logika ada di pustaka (`src/lib.rs`) supaya dapat dites.

use observability::EnvSource;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    rs_backend::cli::run(&args, &EnvSource::from_process())
}
