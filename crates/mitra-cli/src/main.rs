//! `mitra eval <claims> <evidence> [-o <result.json>]`
//!
//! A thin wrapper: reads the two files, calls mitra-core, writes the result.

use std::process::ExitCode;

const USAGE: &str = "usage: mitra eval <claims.mitra> <evidence.mitra> [-o <result.json>]";

fn main() -> ExitCode {
    match run(std::env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(msg) => {
            eprintln!("mitra: {msg}");
            ExitCode::from(2)
        }
    }
}

fn run(args: Vec<String>) -> Result<(), String> {
    let (claims_path, evidence_path, out) = match args.as_slice() {
        [cmd, c, e] if cmd == "eval" => (c, e, None),
        [cmd, c, e, o, out] if cmd == "eval" && o == "-o" => (c, e, Some(out)),
        _ => return Err(USAGE.into()),
    };
    let read = |p: &String| std::fs::read_to_string(p).map_err(|e| format!("{p}: {e}"));
    let claims = read(claims_path)?;
    let evidence = read(evidence_path)?;
    let report = mitra_core::run(&claims, &evidence, &mitra_core::Policy::default()).map_err(
        |(input, e)| {
            let path = match input {
                mitra_core::Input::Claims => claims_path,
                mitra_core::Input::Evidence => evidence_path,
            };
            format!("{path}: {e}")
        },
    )?;
    let json = report.to_json();
    match out {
        Some(path) => std::fs::write(path, json).map_err(|e| format!("{path}: {e}")),
        None => {
            print!("{json}");
            Ok(())
        }
    }
}
