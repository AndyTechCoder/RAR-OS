#![deny(unsafe_code)]
#[path="../rar-lab/preauth/src/lib.rs"] mod preauth;

use std::io::Read;
use std::path::Path;

use preauth::{DescriptorDir, parse_input_bundle_v1};

fn refuse(code: &str) -> ! { eprintln!("preauth-transaction:{code}"); std::process::exit(73) }

fn main() {
    let arguments: Vec<_> = std::env::args().collect();
    if arguments.len() != 3 || arguments[1] != "--prepare" { refuse("usage-refused"); }
    if std::env::var("RAR_TRANSACTION_NETWORK").as_deref() != Ok("none") { refuse("network-boundary"); }
    for name in ["ACTIONS_ID_TOKEN_REQUEST_TOKEN", "ACTIONS_ID_TOKEN_REQUEST_URL", "AWS_ACCESS_KEY_ID",
        "AWS_SECRET_ACCESS_KEY", "AWS_SESSION_TOKEN"] {
        if std::env::var_os(name).is_some() { refuse("authority-environment"); }
    }
    let path = &arguments[2];
    if Path::new(path).is_absolute() || path.split('/').any(|part| part.is_empty() || part == "." || part == "..") {
        refuse("input-path");
    }
    let root = DescriptorDir::open_root(Path::new(".")).unwrap_or_else(|_| refuse("repository-descriptor"));
    let mut input = root.open_relative_file(path).unwrap_or_else(|_| refuse("input-open"));
    let maximum = 4u64 * 1024 * 1024 * 1024 + 16 * 1024 * 1024;
    let mut bytes = Vec::new();
    input.by_ref().take(maximum + 1).read_to_end(&mut bytes).unwrap_or_else(|_| refuse("input-read"));
    if bytes.len() as u64 > maximum { refuse("input-bound"); }
    let bundle = parse_input_bundle_v1(&bytes).unwrap_or_else(|error| refuse(error.code));
    if bundle.package_count != 36 { refuse("input-package-count"); }
    println!("input_bundle_schema=rar-preauth-input-bundle-v1");
    println!("input_bundle_sha256={}", bundle.archive_sha256);
    println!("input_object_count={}", bundle.objects.len());
    println!("input_package_count={}", bundle.package_count);
    println!("input_lock_sha256={}", bundle.input_lock_sha256);
    eprintln!("preauth-transaction:m2-incomplete");
    std::process::exit(73);
}
