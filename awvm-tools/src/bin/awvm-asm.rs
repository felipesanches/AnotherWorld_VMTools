//! Rust port of `awvm-asm.py`.
//!
//! Usage:
//!     awvm-asm <input.asm>
//!
//! Writes `<input>.bin` next to the input file.  Round-trip
//! property: the bytes produced are byte-identical to those
//! `awvm-disasm` consumed to produce `<input.asm>`.

use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let input: PathBuf = match args.next() {
        Some(s) => s.into(),
        None => {
            eprintln!("usage: awvm-asm <input.asm>");
            return ExitCode::from(2);
        }
    };

    let output = match input.file_name().and_then(|s| s.to_str()) {
        Some(name) if name.ends_with(".asm") => input.with_extension("bin"),
        _ => {
            eprintln!("input filename must end in .asm");
            return ExitCode::from(2);
        }
    };

    println!("\nAssembling '{}' ...", input.display());
    println!("First Pass.");
    println!("Second Pass.");

    if let Err(e) = awvm::asm::assemble(&input, &output) {
        eprintln!("assemble failed: {e}");
        return ExitCode::from(1);
    }
    ExitCode::SUCCESS
}
