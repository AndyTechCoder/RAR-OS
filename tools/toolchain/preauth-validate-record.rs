#![deny(unsafe_code)]

#[path = "../rar-lab/preauth/src/lib.rs"]
mod preauth;

use std::env;
use std::fs;

fn read(path: &str) -> String { fs::read_to_string(path).unwrap_or_else(|_| panic!("cannot read {path}")) }

fn main() {
    let args: Vec<_> = env::args().collect();
    let result = match args.get(1).map(String::as_str) {
        Some("prepared") if args.len() == 3 => preauth::PreparedCertification::parse(&read(&args[2])).map(|_| ()),
        Some("host") if args.len() == 3 => preauth::ExecutionHostRecord::parse(&read(&args[2])).map(|_| ()),
        Some("identity") if args.len() == 3 => preauth::IdentityGraph::parse(&read(&args[2])).map(|_| ()),
        Some("closure") if args.len() == 4 => preauth::ClosureLock::parse(&read(&args[2]), &read(&args[3])).map(|_| ()),
        Some("packages") if args.len() == 3 => preauth::PackageManifest::parse(&read(&args[2])).map(|_| ()),
        Some("attestation") if args.len() == 6 => {
            let run = args[5].parse::<u64>().expect("run id");
            preauth::AttestationRecord::parse(&read(&args[2]), &args[3], &args[4], run).map(|_| ())
        }
        _ => panic!("invalid preauth record validator invocation"),
    };
    if let Err(error) = result { eprintln!("{}", error.code); std::process::exit(73); }
}
