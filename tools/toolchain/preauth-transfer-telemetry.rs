#![deny(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

const SCHEMA: &str = "schema\trar-apt-transfer-events-v1";
const MAX_FILE: usize = 4 * 1024 * 1024;
const MAX_LINE: usize = 4096;
const MAX_RECORD: usize = 64 * 1024;
const MAX_REQUESTS: usize = 4096;
const MAX_REDIRECTS: usize = 8;
const MAX_CHANNELS: usize = 64;

#[derive(Debug)]
struct Error(&'static str);
type Result<T> = std::result::Result<T, Error>;

fn telemetry_directory_mode(mode: u32) -> bool {
    let expected = if cfg!(target_os = "linux") { 0o2770 } else { 0o770 };
    mode & 0o7777 == expected
}

fn fail(code: &'static str) -> ! {
    eprintln!("preauth-transfer-telemetry:{code}");
    std::process::exit(73)
}

#[derive(Clone, Debug)]
struct Url {
    serialized: String,
    host: String,
}

fn canonical_https_url(raw: &str) -> Result<Url> {
    if raw.len() > MAX_LINE || !raw.is_ascii() || raw.bytes().any(|byte| byte <= b' ' || byte == 0x7f) {
        return Err(Error("url-characters"));
    }
    if !raw.starts_with("https://") || raw.contains('#') || raw.contains('\\') {
        return Err(Error("url-scheme"));
    }
    let rest = &raw[8..];
    let authority_end = rest.find('/').unwrap_or(rest.len());
    let authority = &rest[..authority_end];
    let suffix = &rest[authority_end..];
    if authority.is_empty() || authority.len() > 253 || authority.contains('@') || authority.contains(':')
        || authority.contains('%') || authority.contains('[') || authority.contains(']')
        || authority.ends_with('.') || authority.bytes().any(|byte| byte.is_ascii_uppercase())
    {
        return Err(Error("url-authority"));
    }
    let mut label_count = 0usize;
    let mut ipv4_candidate = true;
    for label in authority.split('.') {
        label_count += 1;
        if label.is_empty() || label.len() > 63 || label.starts_with('-') || label.ends_with('-') || label.starts_with("xn--")
            || !label.bytes().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        {
            return Err(Error("url-host-label"));
        }
        if !label.bytes().all(|byte| byte.is_ascii_digit()) { ipv4_candidate = false; }
    }
    if label_count < 2 || ipv4_candidate { return Err(Error("url-host-literal")); }
    let serialized = format!("https://{authority}{suffix}");
    if serialized != raw { return Err(Error("url-noncanonical")); }
    Ok(Url { serialized, host: authority.to_owned() })
}

#[derive(Debug)]
struct ProtocolRecord {
    code: u16,
    raw: Vec<u8>,
    fields: BTreeMap<String, Vec<String>>,
}

fn read_protocol_record(reader: &mut impl BufRead) -> io::Result<Option<ProtocolRecord>> {
    let mut raw = Vec::new();
    loop {
        let mut line = Vec::new();
        let count = reader.read_until(b'\n', &mut line)?;
        if count == 0 {
            if raw.is_empty() { return Ok(None); }
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "partial method record"));
        }
        if line.len() > MAX_LINE || raw.len().checked_add(line.len()).is_none_or(|size| size > MAX_RECORD) {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "oversized method record"));
        }
        let blank = line == b"\n";
        raw.extend_from_slice(&line);
        if blank { break; }
    }
    let text = std::str::from_utf8(&raw).map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "method utf8"))?;
    let mut lines = text.lines();
    let status = lines.next().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "method status"))?;
    let code_text = status.split_once(' ').map(|pair| pair.0).unwrap_or(status);
    if code_text.len() != 3 || !code_text.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "method code"));
    }
    let code = code_text.parse().map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "method code"))?;
    let mut fields: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for line in lines {
        if line.is_empty() { continue; }
        let (name, value) = line.split_once(": ").ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "method field"))?;
        if name.is_empty() || !name.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'-') {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "method field name"));
        }
        fields.entry(name.to_owned()).or_default().push(value.to_owned());
    }
    Ok(Some(ProtocolRecord { code, raw, fields }))
}

fn one_field<'a>(record: &'a ProtocolRecord, name: &str) -> Result<&'a str> {
    let values = record.fields.get(name).ok_or(Error("method-field-missing"))?;
    if values.len() != 1 || values[0].is_empty() { return Err(Error("method-field-cardinality")); }
    Ok(&values[0])
}

#[derive(Debug)]
struct Active {
    id: u32,
    current: String,
    hops: usize,
    visited: BTreeSet<String>,
    awaiting_redirect_request: bool,
}

#[derive(Debug)]
struct Tracker {
    output: File,
    next_id: u32,
    active: BTreeMap<String, Active>,
    total: usize,
}

impl Tracker {
    fn create(directory: &str) -> Result<Self> {
        Self::create_channel(directory, std::process::id())
    }

    fn create_channel(directory: &str, channel: u32) -> Result<Self> {
        let directory_metadata = fs::symlink_metadata(directory).map_err(|_| Error("telemetry-directory"))?;
        if !directory_metadata.file_type().is_dir() || !telemetry_directory_mode(directory_metadata.permissions().mode()) {
            return Err(Error("telemetry-directory"));
        }
        let path = Path::new(directory).join(format!("channel-{channel:010}.events"));
        let mut output = OpenOptions::new().write(true).create_new(true).mode(0o640).open(&path)
            .map_err(|_| Error("telemetry-create"))?;
        output.set_permissions(fs::Permissions::from_mode(0o640)).map_err(|_| Error("telemetry-mode"))?;
        let output_metadata = output.metadata().map_err(|_| Error("telemetry-file"))?;
        if !output_metadata.file_type().is_file() || output_metadata.nlink() != 1
            || output_metadata.permissions().mode() & 0o777 != 0o640
            || output_metadata.gid() != directory_metadata.gid()
        {
            return Err(Error("telemetry-file"));
        }
        output.write_all(format!("{SCHEMA}\n").as_bytes()).map_err(|_| Error("telemetry-write"))?;
        Ok(Self { output, next_id: 1, active: BTreeMap::new(), total: 0 })
    }

    fn line(&mut self, value: &str) -> Result<()> {
        if value.len() > MAX_LINE || value.contains('\n') || value.contains('\r') {
            return Err(Error("telemetry-line"));
        }
        self.output.write_all(value.as_bytes()).and_then(|_| self.output.write_all(b"\n"))
            .map_err(|_| Error("telemetry-write"))
    }

    fn request(&mut self, record: &ProtocolRecord) -> Result<()> {
        let raw = one_field(record, "URI")?;
        let url = canonical_https_url(raw)?;
        if let Some(active) = self.active.get_mut(&url.serialized) {
            if active.awaiting_redirect_request {
                active.awaiting_redirect_request = false;
                return Ok(());
            }
            return Err(Error("duplicate-active-request"));
        }
        if self.total >= MAX_REQUESTS { return Err(Error("request-bound")); }
        let id = self.next_id;
        self.next_id = self.next_id.checked_add(1).ok_or(Error("request-bound"))?;
        self.total += 1;
        let mut visited = BTreeSet::new();
        visited.insert(url.serialized.clone());
        self.line(&format!("start\t{id:08}\t{}", url.serialized))?;
        self.active.insert(url.serialized.clone(), Active {
            id, current: url.serialized, hops: 0, visited, awaiting_redirect_request: false,
        });
        Ok(())
    }

    fn redirect(&mut self, record: &ProtocolRecord) -> Result<()> {
        let old = canonical_https_url(one_field(record, "URI")?)?;
        let new = canonical_https_url(one_field(record, "New-URI")?)?;
        let mut active = self.active.remove(&old.serialized).ok_or(Error("redirect-unobserved"))?;
        if active.current != old.serialized || active.hops >= MAX_REDIRECTS || !active.visited.insert(new.serialized.clone())
            || self.active.contains_key(&new.serialized)
        {
            return Err(Error("redirect-chain"));
        }
        active.hops += 1;
        self.line(&format!("redirect\t{:08}\t{}\t{}\t{}", active.id, active.hops, old.serialized, new.serialized))?;
        active.current = new.serialized.clone();
        active.awaiting_redirect_request = true;
        self.active.insert(new.serialized, active);
        Ok(())
    }

    fn terminal(&mut self, record: &ProtocolRecord, outcome: &str) -> Result<()> {
        let url = canonical_https_url(one_field(record, "URI")?)?;
        let active = self.active.remove(&url.serialized).ok_or(Error("terminal-unobserved"))?;
        if active.current != url.serialized { return Err(Error("terminal-url")); }
        self.line(&format!("terminal\t{:08}\t{outcome}\t{}", active.id, url.serialized))
    }

    fn finish(&mut self) -> Result<()> {
        if !self.active.is_empty() { return Err(Error("terminal-missing")); }
        self.line(&format!("complete\t{}", self.total))?;
        self.output.sync_all().map_err(|_| Error("telemetry-sync"))
    }
}

fn lock_tracker(tracker: &Arc<Mutex<Tracker>>) -> Result<std::sync::MutexGuard<'_, Tracker>> {
    tracker.lock().map_err(|_| Error("telemetry-lock"))
}

fn proxy(real_method: &str, telemetry: &str) -> Result<()> {
    let tracker = Arc::new(Mutex::new(Tracker::create(telemetry)?));
    let mut child = Command::new(real_method).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::inherit())
        .spawn().map_err(|_| Error("method-spawn"))?;
    let mut child_input = child.stdin.take().ok_or(Error("method-stdin"))?;
    let tracker_input = Arc::clone(&tracker);
    let input_thread = thread::spawn(move || -> Result<()> {
        let stdin = io::stdin();
        let mut reader = stdin.lock();
        while let Some(record) = read_protocol_record(&mut reader).map_err(|_| Error("apt-protocol-input"))? {
            if record.code == 600 { lock_tracker(&tracker_input)?.request(&record)?; }
            child_input.write_all(&record.raw).and_then(|_| child_input.flush()).map_err(|_| Error("method-input-write"))?;
        }
        Ok(())
    });

    let stdout = io::stdout();
    let mut apt_output = stdout.lock();
    let mut child_output = BufReader::new(child.stdout.take().ok_or(Error("method-stdout"))?);
    while let Some(record) = read_protocol_record(&mut child_output).map_err(|_| Error("apt-protocol-output"))? {
        match record.code {
            103 => lock_tracker(&tracker)?.redirect(&record)?,
            201 => lock_tracker(&tracker)?.terminal(&record, "success")?,
            400 => lock_tracker(&tracker)?.terminal(&record, "failure")?,
            100 | 101 | 102 | 104 | 200 | 351 | 401 | 402 | 403 => {},
            _ => return Err(Error("method-event-unknown")),
        }
        apt_output.write_all(&record.raw).and_then(|_| apt_output.flush()).map_err(|_| Error("apt-output-write"))?;
    }
    input_thread.join().map_err(|_| Error("input-thread"))??;
    let status = child.wait().map_err(|_| Error("method-wait"))?;
    if !status.success() { return Err(Error("method-exit")); }
    lock_tracker(&tracker)?.finish()
}

#[derive(Debug)]
struct Policy {
    starts: BTreeSet<String>,
    redirects: BTreeSet<String>,
    maximum_requests: usize,
    maximum_hops: usize,
}

fn parse_policy(path: &str) -> Result<Policy> {
    let text = std::fs::read_to_string(path).map_err(|_| Error("policy-read"))?;
    if !text.ends_with('\n') || text.len() > 64 * 1024 { return Err(Error("policy-framing")); }
    let mut starts = BTreeSet::new();
    let mut redirects = BTreeSet::new();
    let mut maximum_requests = None;
    let mut maximum_hops = None;
    for line in text.lines() {
        if let Some(host) = line.strip_prefix("allowed_apt_origin=") {
            starts.insert(canonical_https_url(&format!("https://{host}/"))?.host);
        } else if let Some(host) = line.strip_prefix("allowed_apt_redirect_origin=") {
            redirects.insert(canonical_https_url(&format!("https://{host}/"))?.host);
        } else if let Some(value) = line.strip_prefix("maximum_transfer_requests=") {
            if maximum_requests.replace(value.parse().map_err(|_| Error("policy-number"))?).is_some() {
                return Err(Error("policy-duplicate"));
            }
        } else if let Some(value) = line.strip_prefix("maximum_redirect_hops=") {
            if maximum_hops.replace(value.parse().map_err(|_| Error("policy-number"))?).is_some() {
                return Err(Error("policy-duplicate"));
            }
        }
    }
    let maximum_requests = maximum_requests.ok_or(Error("policy-missing"))?;
    let maximum_hops = maximum_hops.ok_or(Error("policy-missing"))?;
    if starts.is_empty() || redirects.is_empty() || maximum_requests == 0 || maximum_requests > MAX_REQUESTS
        || maximum_hops == 0 || maximum_hops > MAX_REDIRECTS
    {
        return Err(Error("policy-bound"));
    }
    Ok(Policy { starts, redirects, maximum_requests, maximum_hops })
}

#[derive(Debug)]
struct VerifiedRequest {
    current: String,
    hops: usize,
    visited: BTreeSet<String>,
    terminal: bool,
}

fn verify_channel(path: &Path, policy: &Policy, expected_gid: u32, hosts: &mut BTreeSet<String>) -> Result<usize> {
    let metadata = fs::symlink_metadata(path).map_err(|_| Error("telemetry-missing"))?;
    if !metadata.file_type().is_file() || metadata.len() as usize > MAX_FILE || metadata.nlink() != 1
        || metadata.permissions().mode() & 0o777 != 0o640 || metadata.gid() != expected_gid
    {
        return Err(Error("telemetry-file"));
    }
    let mut bytes = Vec::new();
    File::open(path).and_then(|mut file| file.read_to_end(&mut bytes)).map_err(|_| Error("telemetry-read"))?;
    if bytes.is_empty() || bytes.len() > MAX_FILE || !bytes.ends_with(b"\n") || bytes.contains(&0) {
        return Err(Error("telemetry-framing"));
    }
    let text = std::str::from_utf8(&bytes).map_err(|_| Error("telemetry-utf8"))?;
    let mut lines = text.lines();
    if lines.next() != Some(SCHEMA) { return Err(Error("telemetry-schema")); }
    let mut requests: BTreeMap<u32, VerifiedRequest> = BTreeMap::new();
    let mut completed = None;
    for line in lines {
        if line.len() > MAX_LINE || line.is_empty() || completed.is_some() { return Err(Error("telemetry-record")); }
        let fields: Vec<_> = line.split('\t').collect();
        match fields.as_slice() {
            ["start", id, raw_url] => {
                let id = parse_id(id)?;
                let url = canonical_https_url(raw_url)?;
                if !policy.starts.contains(&url.host) { return Err(Error("start-origin")); }
                if requests.len() >= policy.maximum_requests || requests.contains_key(&id) { return Err(Error("start-cardinality")); }
                hosts.insert(url.host);
                let mut visited = BTreeSet::new(); visited.insert(url.serialized.clone());
                requests.insert(id, VerifiedRequest { current: url.serialized, hops: 0, visited, terminal: false });
            }
            ["redirect", id, hop, raw_from, raw_to] => {
                let id = parse_id(id)?;
                let hop: usize = hop.parse().map_err(|_| Error("redirect-hop"))?;
                let from = canonical_https_url(raw_from)?;
                let to = canonical_https_url(raw_to)?;
                if !policy.redirects.contains(&to.host) { return Err(Error("redirect-origin")); }
                hosts.insert(from.host); hosts.insert(to.host);
                let request = requests.get_mut(&id).ok_or(Error("redirect-unobserved"))?;
                if request.terminal || request.current != from.serialized || hop != request.hops + 1
                    || hop > policy.maximum_hops || !request.visited.insert(to.serialized.clone())
                {
                    return Err(Error("redirect-chain"));
                }
                request.hops = hop; request.current = to.serialized;
            }
            ["terminal", id, outcome @ ("success" | "failure"), raw_url] => {
                let _ = outcome;
                let id = parse_id(id)?;
                let url = canonical_https_url(raw_url)?;
                if !policy.redirects.contains(&url.host) { return Err(Error("terminal-origin")); }
                hosts.insert(url.host);
                let request = requests.get_mut(&id).ok_or(Error("terminal-unobserved"))?;
                if request.terminal || request.current != url.serialized { return Err(Error("terminal-cardinality")); }
                request.terminal = true;
            }
            ["complete", count] => {
                let count: usize = count.parse().map_err(|_| Error("complete-count"))?;
                completed = Some(count);
            }
            _ => return Err(Error("event-unknown")),
        }
    }
    let expected = completed.ok_or(Error("complete-missing"))?;
    if requests.is_empty() || expected != requests.len() || requests.values().any(|request| !request.terminal) {
        return Err(Error("request-cardinality"));
    }
    Ok(requests.len())
}

fn channel_path(directory: &Path, name: &str) -> Result<PathBuf> {
    let digits = name.strip_prefix("channel-").and_then(|value| value.strip_suffix(".events"))
        .ok_or(Error("telemetry-entry"))?;
    if digits.len() != 10 || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(Error("telemetry-entry"));
    }
    Ok(directory.join(name))
}

fn verify(directory: &str, policy_path: &str) -> Result<()> {
    let directory_path = Path::new(directory);
    let metadata = fs::symlink_metadata(directory_path).map_err(|_| Error("telemetry-missing"))?;
    if !metadata.file_type().is_dir() || !telemetry_directory_mode(metadata.permissions().mode()) {
        return Err(Error("telemetry-directory"));
    }
    let policy = parse_policy(policy_path)?;
    let mut paths = Vec::new();
    for entry in fs::read_dir(directory_path).map_err(|_| Error("telemetry-read"))? {
        let entry = entry.map_err(|_| Error("telemetry-read"))?;
        let name = entry.file_name().into_string().map_err(|_| Error("telemetry-entry"))?;
        paths.push(channel_path(directory_path, &name)?);
        if paths.len() > MAX_CHANNELS { return Err(Error("telemetry-channel-bound")); }
    }
    if paths.is_empty() { return Err(Error("telemetry-channel-empty")); }
    paths.sort();
    let channel_count = paths.len();
    let mut hosts = BTreeSet::new();
    let mut total = 0usize;
    for path in paths {
        total = total.checked_add(verify_channel(&path, &policy, metadata.gid(), &mut hosts)?)
            .ok_or(Error("request-bound"))?;
        if total > policy.maximum_requests { return Err(Error("request-bound")); }
    }
    let host_list = hosts.into_iter().collect::<Vec<_>>().join(",");
    eprintln!("preauth-input-producer:transfer-hosts:{host_list}");
    eprintln!("preauth-input-producer:transfer-requests:{total}");
    eprintln!("preauth-input-producer:transfer-channels:{channel_count}");
    Ok(())
}

fn parse_id(raw: &str) -> Result<u32> {
    if raw.len() != 8 || !raw.bytes().all(|byte| byte.is_ascii_digit()) { return Err(Error("request-id")); }
    let id = raw.parse().map_err(|_| Error("request-id"))?;
    if id == 0 { return Err(Error("request-id")); }
    Ok(id)
}

fn main() {
    let arguments: Vec<_> = std::env::args().collect();
    let result = match arguments.as_slice() {
        [_, command, real, telemetry] if command == "--proxy" => proxy(real, telemetry),
        [_, command, telemetry, policy] if command == "--verify" => verify(telemetry, policy),
        [program] => {
            let _ = program;
            let real = std::env::var("RAR_PREAUTH_APT_REAL_METHOD").map_err(|_| Error("proxy-environment"));
            let telemetry = std::env::var("RAR_PREAUTH_APT_TELEMETRY_DIR").map_err(|_| Error("proxy-environment"));
            match (real, telemetry) { (Ok(real), Ok(telemetry)) => proxy(&real, &telemetry), _ => Err(Error("proxy-environment")) }
        }
        [_, command] if command == "--build-root-exec-probe" => Ok(()),
        _ => Err(Error("usage-refused")),
    };
    if let Err(error) = result { fail(error.0); }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strict_url_authority_rejects_ambiguous_forms() {
        assert_eq!(canonical_https_url("https://snapshot.debian.org/x").unwrap().host, "snapshot.debian.org");
        for raw in [
            "http://snapshot.debian.org/x", "https://user@snapshot.debian.org/x",
            "https://snapshot.debian.org:443/x", "https://Snapshot.debian.org/x",
            "https://snapshot.debian.org./x", "https://xn--bcher-kva.example/x",
            "https://bücher.example/x", "https://snapshot%2edebian.org/x",
            "https://127.0.0.1/x", "https://[::1]/x", "https://snapshot.debian.org\\@evil/x",
            "https://snapshot.debian.org/x#fragment", "https://snapshot.debian.org/white space",
        ] { assert!(canonical_https_url(raw).is_err(), "accepted {raw}"); }
    }
}
