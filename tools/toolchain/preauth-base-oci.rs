#![deny(unsafe_code)]
#[path="../rar-lab/preauth/src/lib.rs"] mod preauth;

use std::io::{Read, Write};
use std::path::Path;

use preauth::{DescriptorDir, canonicalize_base_oci, describe_base_oci, sha256_hex};

fn refuse(code: &str) -> ! { eprintln!("preauth-base-oci:{code}"); std::process::exit(73) }

fn checked_relative(path: &str) -> &str {
    if Path::new(path).is_absolute() || path.split('/').any(|part| part.is_empty() || part == "." || part == "..") {
        refuse("input-path");
    }
    path
}

fn main() {
    let arguments: Vec<_> = std::env::args().collect();
    if arguments.len() != 6 || arguments[1] != "--canonicalize" { refuse("usage-refused"); }
    for name in ["ACTIONS_ID_TOKEN_REQUEST_TOKEN", "ACTIONS_ID_TOKEN_REQUEST_URL", "AWS_ACCESS_KEY_ID",
        "AWS_SECRET_ACCESS_KEY", "AWS_SESSION_TOKEN", "GH_TOKEN", "GITHUB_TOKEN"] {
        if std::env::var_os(name).is_some() { refuse("authority-environment"); }
    }
    let raw_path = checked_relative(&arguments[2]);
    let out_path = checked_relative(&arguments[3]);
    let expected_image_name = &arguments[4];
    let expected_ref_name = &arguments[5];
    let root = DescriptorDir::open_root(Path::new(".")).unwrap_or_else(|_| refuse("repository-descriptor"));
    let input = root.open_relative_file(raw_path).unwrap_or_else(|_| refuse("input-open"));
    let maximum = 4u64 * 1024 * 1024 * 1024;
    let mut bytes = Vec::new();
    Read::take(input, maximum + 1).read_to_end(&mut bytes).unwrap_or_else(|_| refuse("input-read"));
    if bytes.len() as u64 > maximum { refuse("input-bound"); }
    let result = canonicalize_base_oci(&bytes, expected_image_name, expected_ref_name).unwrap_or_else(|error| {
        for line in describe_base_oci(&bytes) { eprintln!("preauth-base-oci:observed:{line}"); }
        refuse(error.code)
    });
    // The canonical bytes are written through held directory descriptors and never reopened by
    // this process; the emitted digest lets the assembling host detect any later substitution.
    let mut components: Vec<&str> = out_path.split('/').collect();
    let leaf = components.pop().unwrap_or_else(|| refuse("output-path"));
    let mut directory = root;
    for component in components {
        directory = directory.open_dir(component).unwrap_or_else(|_| refuse("output-parent"));
    }
    let mut output = directory.create_exclusive_file(leaf).unwrap_or_else(|_| refuse("output-create"));
    output.write_all(&result.canonical).unwrap_or_else(|_| refuse("output-write"));
    output.sync_all().unwrap_or_else(|_| refuse("output-sync"));
    directory.sync().unwrap_or_else(|_| refuse("output-parent-sync"));
    eprintln!("preauth-base-oci:layers={} manifest_sha256={}", result.layer_count, result.manifest_sha256);
    println!("base_oci_canonical_sha256={}", sha256_hex(&result.canonical));
    println!("base_oci_config_sha256={}", result.config_sha256);
}
