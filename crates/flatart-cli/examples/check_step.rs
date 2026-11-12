//! Reads STEP files back and checks them: `cargo run --release -p flatart-cli
//! --example check_step -- FILE.step [FILE.step ...]`.

use flatart_step_truck::check::check;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut failed = false;
    for path in std::env::args().skip(1) {
        let result = std::fs::read_to_string(&path)
            .map_err(|e| e.to_string())
            .and_then(|text| check(&text));
        match result {
            Ok(s) => println!(
                "ok   {path}: {} products, {} bodies, {:.2?}..{:.2?}",
                s.products.len(),
                s.bodies.len(),
                s.bounds.0,
                s.bounds.1
            ),
            Err(e) => {
                failed = true;
                println!("FAIL {path}: {e}");
            }
        }
    }
    if failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
