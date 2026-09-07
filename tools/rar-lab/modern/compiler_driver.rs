//! RAR-owned host-only compiler driver; never executes the compiled adapter.
//! Inactive source candidate. Parent must enforce and inspect the compiler role.
#![forbid(unsafe_code)]
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
use std::path::Path;
use std::process::{Command, Stdio};

const DRIVER: &str = "/rar-compile-driver";
const SYSROOT: &str = "/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu";
const RUSTC: &str = "/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustc";
const LLD: &str = "/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/lib/rustlib/x86_64-unknown-linux-gnu/bin/rust-lld";
const OUTPUT: &str = "/build/rar-reference";
const SOURCE: &str = "/source/tools/rar-lab/modern/target_reference.rs";
const MAX_OUTPUT: u64 = 8 * 1024 * 1024;
const SOURCES: [&str; 5] = [
    "/source/tools/rar-lab/modern/target_reference.rs",
    "/source/core/crypto/sha256.rs",
    "/source/core/crypto/sha512.rs",
    "/source/core/crypto/ed25519.rs",
    "/source/core/crypto/chacha20poly1305.rs",
];
#[derive(Debug)]
enum Error { Guard, Source, Compiler, Output, Io }
impl From<std::io::Error> for Error {
    fn from(_: std::io::Error) -> Self { Self::Io }
}
fn read_bounded(path: &str, maximum: u64) -> Result<Vec<u8>, Error> {
    let mut value = Vec::new();
    fs::File::open(path)?.take(maximum + 1).read_to_end(&mut value)?;
    if value.len() as u64 > maximum { return Err(Error::Guard); }
    Ok(value)
}
fn process_status(raw: &[u8]) -> Result<(), Error> {
    let text = std::str::from_utf8(raw).map_err(|_| Error::Guard)?;
    for (key, expected) in [
        ("Uid:", "65532 65532 65532 65532"),
        ("Gid:", "65532 65532 65532 65532"),
        ("CapInh:", "0000000000000000"), ("CapPrm:", "0000000000000000"),
        ("CapEff:", "0000000000000000"), ("CapBnd:", "0000000000000000"),
        ("CapAmb:", "0000000000000000"), ("NoNewPrivs:", "1"), ("Seccomp:", "2"),
    ] {
        let matches: Vec<_> = text.lines().filter_map(|line| line.strip_prefix(key)).collect();
        if matches.len() != 1 || matches[0].split_whitespace().collect::<Vec<_>>()
                != expected.split_whitespace().collect::<Vec<_>>() {
            return Err(Error::Guard);
        }
    }
    Ok(())
}
fn guard() -> Result<(), Error> {
    if std::env::consts::OS != "linux" || std::env::consts::ARCH != "x86_64" ||
        std::env::args_os().count() != 1 ||
        std::env::var("RAR_COMPILER_ROLE").ok().as_deref() != Some("modern-v0") {
        return Err(Error::Guard);
    }
    if fs::read_link("/proc/self/exe")? != Path::new(DRIVER) { return Err(Error::Guard); }
    process_status(&read_bounded("/proc/self/status", 65536)?)?;
    let build = fs::symlink_metadata("/build")?;
    if !build.is_dir() || build.uid() != 65532 || build.gid() != 65532 ||
        build.mode() & 0o7777 != 0o700 || fs::read_dir("/build")?.next().is_some() {
        return Err(Error::Guard);
    }
    Ok(())
}
fn source_inventory() -> Result<(), Error> {
    let mut pending = vec![Path::new("/source").to_path_buf()];
    let mut found = Vec::new();
    let mut count = 0usize;
    let mut bytes = 0u64;
    while let Some(path) = pending.pop() {
        count += 1;
        if count > 64 { return Err(Error::Source); }
        let info = fs::symlink_metadata(&path)?;
        if info.uid() != 0 || info.gid() != 0 || info.mode() & 0o7022 != 0 ||
            info.file_type().is_symlink() || fs::canonicalize(&path)? != path {
            return Err(Error::Source);
        }
        if info.is_dir() {
            if info.mode() & 0o777 != 0o555 { return Err(Error::Source); }
            for child in fs::read_dir(&path)? {
                pending.push(child?.path());
                if pending.len() + count > 64 { return Err(Error::Source); }
            }
        } else if info.is_file() {
            if info.mode() & 0o777 != 0o444 || info.len() == 0 ||
                info.len() > 256 * 1024 || info.nlink() != 1 {
                return Err(Error::Source);
            }
            let name = path.to_str().ok_or(Error::Source)?;
            if !SOURCES.contains(&name) { return Err(Error::Source); }
            bytes = bytes.checked_add(info.len()).ok_or(Error::Source)?;
            if bytes > 512 * 1024 { return Err(Error::Source); }
            found.push(name.to_owned());
        } else { return Err(Error::Source); }
    }
    found.sort();
    let mut expected = SOURCES.map(str::to_owned); expected.sort();
    if found != expected { return Err(Error::Source); }
    Ok(())
}
fn compiler_args() -> Vec<String> {
    [
        "--edition=2024", "--crate-name=rar_modern_reference", "--crate-type=bin",
        "--target=x86_64-unknown-linux-musl", "--sysroot", SYSROOT,
        "-C", "opt-level=2", "-C", "panic=abort", "-C", "codegen-units=1",
        "-C", "strip=symbols", "-C", "debuginfo=0", "-C", "target-cpu=x86-64",
        "-C", "target-feature=+crt-static", "-C", "link-self-contained=yes",
        "-C", "relocation-model=static", "-C", "metadata=rar-modern-reference-v0",
        "--remap-path-prefix=/source=/rar-source", "-o", OUTPUT, SOURCE,
    ].into_iter().map(str::to_owned)
     .chain(["-C".to_owned(), format!("linker={LLD}")]).collect()
}
fn compiler_command() -> Command {
    let mut command=Command::new(RUSTC);
    command.args(compiler_args()).env_clear()
        .env("PATH", "/nonexistent").env("LC_ALL", "C").env("LANG", "C")
        .env("TMPDIR", "/build")
        .env("LD_LIBRARY_PATH", format!("{SYSROOT}/lib"))
        .current_dir("/source").stdin(Stdio::null()).stdout(Stdio::null())
        .stderr(Stdio::inherit());
    command
}
fn main_inner() -> Result<(), Error> {
    guard()?;
    source_inventory()?;
    // Exact tools/arguments only. No shell, Cargo, plugins, inherited variables,
    // input-driven output path or reference library is introduced here.
    let status = compiler_command().status()?;
    if !status.success() { return Err(Error::Compiler); }
    // Linux O_NOFOLLOW. Parent independently validates the returned ELF bytes.
    let mut file = OpenOptions::new().read(true).custom_flags(0x20000).open(OUTPUT)?;
    let before = file.metadata()?;
    if !before.is_file() || before.nlink() != 1 || before.uid() != 65532 ||
        before.gid() != 65532 || before.mode() & 0o6000 != 0 ||
        !(64..=MAX_OUTPUT).contains(&before.len()) { return Err(Error::Output); }
    let mut value = Vec::new();
    (&mut file).take(MAX_OUTPUT + 1).read_to_end(&mut value)?;
    let after = file.metadata()?;
    if value.len() as u64 != before.len() || after.len() != before.len() ||
        after.mtime() != before.mtime() || after.mtime_nsec() != before.mtime_nsec() ||
        after.ctime() != before.ctime() || after.ctime_nsec() != before.ctime_nsec() ||
        value.get(..4) != Some(&b"\x7fELF"[..]) { return Err(Error::Output); }
    let stdout = std::io::stdout();
    let mut output = stdout.lock();
    output.write_all(&value)?; output.flush()?;
    Ok(())
}
fn main() {
    if let Err(error) = main_inner() {
        eprintln!("compiler driver refused/failed: {error:?}");
        std::process::exit(70);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    const STATUS: &str = "Uid:\t65532\t65532\t65532\t65532\nGid:\t65532\t65532\t65532\t65532\nCapInh:\t0000000000000000\nCapPrm:\t0000000000000000\nCapEff:\t0000000000000000\nCapBnd:\t0000000000000000\nCapAmb:\t0000000000000000\nNoNewPrivs:\t1\nSeccomp:\t2\n";
    #[test] fn exact_compiler_contract() {
        let args = compiler_args();
        assert!(args.contains(&format!("linker={LLD}")));
        assert!(args.contains(&SOURCE.to_owned()));
        assert!(args.contains(&OUTPUT.to_owned()));
        assert!(args.contains(&"--target=x86_64-unknown-linux-musl".to_owned()));
        assert_eq!(args.iter().filter(|x| x.as_str() == "-o").count(), 1);
        assert_eq!(args.iter().filter(|x| x.as_str() == "--sysroot").count(), 1);
        assert!(!args.iter().any(|x| x.contains("openssl") || x.contains("sodium")));
    }
    #[test] fn process_must_be_unprivileged_and_confined() {
        assert!(process_status(STATUS.as_bytes()).is_ok());
        for bad in [STATUS.replace("65532", "0"), STATUS.replace("Seccomp:\t2", "Seccomp:\t0"),
                    STATUS.replace("NoNewPrivs:\t1", "NoNewPrivs:\t0"),
                    STATUS.replace("CapEff:\t0000000000000000", "CapEff:\t0000000000000001"),
                    STATUS.replace("CapBnd:\t0000000000000000", "CapBnd:\t0000000000000001"),
                    format!("{STATUS}Uid:\t65532 65532 65532 65532\n")] {
            assert!(process_status(bad.as_bytes()).is_err());
        }
        assert!(process_status(&[255]).is_err());
    }
    #[test] fn separate_rar_identity_and_fixed_compiler_environment() {
        assert_eq!(DRIVER,"/rar-compile-driver");
        let command=compiler_command();
        assert_eq!(command.get_program(),std::ffi::OsStr::new(RUSTC));
        assert_eq!(command.get_current_dir(),Some(Path::new("/source")));
        let environment=command.get_envs().map(|(key,value)|
            (key.to_str().unwrap(),value.unwrap().to_str().unwrap()))
            .collect::<std::collections::BTreeMap<_,_>>();
        let library=format!("{SYSROOT}/lib");
        assert_eq!(environment,std::collections::BTreeMap::from([
            ("PATH","/nonexistent"),("LC_ALL","C"),("LANG","C"),
            ("TMPDIR","/build"),("LD_LIBRARY_PATH",library.as_str())]));
    }

}
