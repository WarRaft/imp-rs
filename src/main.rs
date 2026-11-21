//! `imp-rs`: checks import lists and lists their files. See `usage`.

use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    let (list, path) = match args.as_slice() {
        ["--list", p] => (true, *p),
        [p] if !p.starts_with('-') => (false, *p),
        _ => {
            eprintln!("usage: imp-rs <war3map.imp>          check: read and write back");
            eprintln!("       imp-rs --list <war3map.imp>   flags and paths, tab-separated");
            return ExitCode::FAILURE;
        }
    };
    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("{path}: {e}");
            return ExitCode::FAILURE;
        }
    };
    let imports = match imp_rs::parse(&bytes) {
        Ok(i) => i,
        Err(e) => {
            eprintln!("{path}: {e}");
            return ExitCode::FAILURE;
        }
    };
    if list {
        for f in &imports.files {
            println!("{}\t{}\t{}", f.flags, f.path, f.path_in_map());
        }
    } else if imports.to_bytes().ok() != Some(bytes) {
        eprintln!("{path}: differs after writing");
        return ExitCode::FAILURE;
    } else {
        println!("{path}: version {}, {} files", imports.version, imports.files.len());
    }
    ExitCode::SUCCESS
}
