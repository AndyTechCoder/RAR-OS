#![deny(unsafe_code)]
#[path="../rar-lab/preauth/src/lib.rs"] mod preauth;

use std::io::{Read, Write};
use std::path::Path;

use preauth::{DescriptorDir, canonicalize_base_oci};

fn refuse(code: &str) -> ! { eprintln!("preauth-base-oci:{code}"); std::process::exit(73) }

fn checked_relative(path: &str) -> &str {
    if Path::new(path).is_absolute() || path.split('/').any(|part| part.is_empty() || part == "." || part == "..") {
        refuse("input-path");
    }
    path
}

fn main() {
    let arguments: Vec<_> = std::env::args().collect();
    if arguments.len() != 4 || arguments[1] != "--canonicalize" { refuse("usage-refused"); }
    for name in ["ACTIONS_ID_TOKEN_REQUEST_TOKEN", "ACTIONS_ID_TOKEN_REQUEST_URL", "AWS_ACCESS_KEY_ID",
        "AWS_SECRET_ACCESS_KEY", "AWS_SESSION_TOKEN", "GH_TOKEN", "GITHUB_TOKEN"] {
        if std::env::var_os(name).is_some() { refuse("authority-environment"); }
    }
    let raw_path = checked_relative(&arguments[2]);
    let out_path = checked_relative(&arguments[3]);
    let root = DescriptorDir::open_root(Path::new(".")).unwrap_or_else(|_| refuse("repository-descriptor"));
    let mut input = root.open_relative_file(raw_path).unwrap_or_else(|_| refuse("input-open"));
    let maximum = 4u64 * 1024 * 1024 * 1024;
    let mut bytes = Vec::new();
    input.by_ref().take(maximum + 1).read_to_end(&mut bytes).unwrap_or_else(|_| refuse("input-read"));
    if bytes.len() as u64 > maximum { refuse("input-bound"); }
    let result = canonicalize_base_oci(&bytes).unwrap_or_else(|error| refuse(error.code));
    let mut output = std::fs::OpenOptions::new().write(true).create_new(true).open(out_path)
        .unwrap_or_else(|_| refuse("output-create"));
    output.write_all(&result.canonical).unwrap_or_else(|_| refuse("output-write"));
    output.sync_all().unwrap_or_else(|_| refuse("output-sync"));
    eprintln!("preauth-base-oci:layers={} manifest_sha256={}", result.layer_count, result.manifest_sha256);
    println!("base_oci_config_sha256={}", result.config_sha256);
}
