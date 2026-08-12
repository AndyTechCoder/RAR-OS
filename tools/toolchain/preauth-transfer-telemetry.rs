#![deny(unsafe_op_in_unsafe_fn)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Read, Write};
#[cfg(target_os = "linux")]
use std::os::fd::AsRawFd;
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
#[cfg(target_os = "linux")]
use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, SyncSender};
#[cfg(target_os = "linux")]
use std::sync::atomic::{AtomicI32, Ordering};
use std::thread;
use std::time::{Duration, Instant};

const SCHEMA: &str = "schema\trar-apt-transfer-events-v1";
const MAX_FILE: usize = 4 * 1024 * 1024;
const MAX_LINE: usize = 4096;
const MAX_RECORD: usize = 64 * 1024;
const MAX_REQUESTS: usize = 4096;
const MAX_REDIRECTS: usize = 8;
const MAX_CHANNELS: usize = 64;
const LIFECYCLE_SCHEMA: &str = "schema\trar-apt-method-lifecycle-v1";
const SHUTDOWN_GRACE: Duration = Duration::from_millis(250);
const SHUTDOWN_POLL: Duration = Duration::from_millis(10);
const COMPLETION_WAIT: Duration = Duration::from_secs(2);

#[cfg(target_os = "linux")]
static LIFECYCLE_SIGNAL_FD: AtomicI32 = AtomicI32::new(-1);
#[cfg(target_os = "linux")]
static LIFECYCLE_SIGNAL: AtomicI32 = AtomicI32::new(0);
#[cfg(target_os = "linux")]
unsafe extern "C" {
    fn signal(number: i32, handler: extern "C" fn(i32)) -> usize;
    fn write(fd: i32, buffer: *const std::ffi::c_void, count: usize) -> isize;
    fn fsync(fd: i32) -> i32;
    fn kill(pid: i32, signal: i32) -> i32;
}

#[cfg(target_os = "linux")]
extern "C" fn lifecycle_signal_handler(number: i32) {
    if LIFECYCLE_SIGNAL
        .compare_exchange(0, number, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
    {
        return;
    }
    let line: &[u8] = match number {
        1 => b"transition\twrapper-signal-hup\n",
        2 => b"transition\twrapper-signal-int\n",
        15 => b"transition\twrapper-signal-term\n",
        _ => b"transition\twrapper-signal-unknown\n",
    };
    let fd = LIFECYCLE_SIGNAL_FD.load(Ordering::Relaxed);
    if fd >= 0 {
        // SAFETY: the handler uses only the pre-opened append-only trace descriptor and
        // async-signal-safe libc operations. The buffer remains alive for both calls.
        unsafe {
            let _ = write(fd, line.as_ptr().cast(), line.len());
            let _ = fsync(fd);
        }
    }
}

#[derive(Debug)]
struct Error(&'static str);
type Result<T> = std::result::Result<T, Error>;

fn telemetry_directory_mode(mode: u32) -> bool {
    let expected = if cfg!(target_os = "linux") { 0o2770 } else { 0o770 };
    mode & 0o7777 == expected
}

#[derive(Debug)]
struct LifecycleTrace {
    output: File,
    registry: File,
    registry_open: PathBuf,
    registry_complete: PathBuf,
    directory: PathBuf,
    channel: u32,
}

impl LifecycleTrace {
    fn create(directory: &str, channel: u32) -> Result<Self> {
        let directory_metadata = fs::symlink_metadata(directory).map_err(|_| Error("lifecycle-directory"))?;
        if !directory_metadata.file_type().is_dir()
            || !telemetry_directory_mode(directory_metadata.permissions().mode())
        {
            return Err(Error("lifecycle-directory"));
        }
        let directory = Path::new(directory);
        let path = directory.join(format!("trace-{channel:010}.events"));
        let mut output = OpenOptions::new()
            .append(true)
            .create_new(true)
            .mode(0o640)
            .open(path)
            .map_err(|_| Error("lifecycle-create"))?;
        output
            .set_permissions(fs::Permissions::from_mode(0o640))
            .map_err(|_| Error("lifecycle-mode"))?;
        let metadata = output.metadata().map_err(|_| Error("lifecycle-file"))?;
        if !metadata.file_type().is_file()
            || metadata.nlink() != 1
            || metadata.permissions().mode() & 0o777 != 0o640
            || metadata.gid() != directory_metadata.gid()
        {
            return Err(Error("lifecycle-file"));
        }
        output
            .write_all(format!("{LIFECYCLE_SCHEMA}\n").as_bytes())
            .map_err(|_| Error("lifecycle-write"))?;
        let registry_open = directory.join(format!("registry-{channel:010}.open"));
        let registry_complete = directory.join(format!("registry-{channel:010}.complete"));
        let mut registry = OpenOptions::new().write(true).create_new(true).mode(0o640)
            .open(&registry_open).map_err(|_| Error("lifecycle-registry-create"))?;
        registry.set_permissions(fs::Permissions::from_mode(0o640)).map_err(|_| Error("lifecycle-mode"))?;
        let registry_metadata = registry.metadata().map_err(|_| Error("lifecycle-file"))?;
        if !registry_metadata.file_type().is_file() || registry_metadata.nlink() != 1
            || registry_metadata.permissions().mode() & 0o777 != 0o640
            || registry_metadata.gid() != directory_metadata.gid()
        {
            return Err(Error("lifecycle-file"));
        }
        registry.write_all(format!("{LIFECYCLE_SCHEMA}\nchannel\t{channel:010}\n").as_bytes())
            .and_then(|_| registry.flush()).map_err(|_| Error("lifecycle-write"))?;
        registry.sync_all().map_err(|_| Error("lifecycle-sync"))?;
        File::open(directory).and_then(|file| file.sync_all()).map_err(|_| Error("lifecycle-sync"))?;
        #[cfg(target_os = "linux")]
        {
            LIFECYCLE_SIGNAL.store(0, Ordering::Release);
            LIFECYCLE_SIGNAL_FD.store(output.as_raw_fd(), Ordering::Release);
            // SAFETY: the three installed handlers have the exact C ABI and remain
            // valid for the process lifetime. A SIG_ERR result rejects startup.
            unsafe {
                for number in [1, 2, 15] {
                    if signal(number, lifecycle_signal_handler) == usize::MAX {
                        return Err(Error("lifecycle-signal-install"));
                    }
                }
            }
        }
        Ok(Self {
            output, registry, registry_open, registry_complete,
            directory: directory.to_path_buf(), channel,
        })
    }

    fn transition(&mut self, state: &'static str, requests: usize, active: usize) -> Result<()> {
        self.output
            .write_all(format!("transition\t{state}\t{requests}\t{active}\n").as_bytes())
            .map_err(|_| Error("lifecycle-write"))
    }

    fn publish_completion(&mut self, requests: usize) -> Result<()> {
        self.registry.write_all(format!("complete\t{:010}\t{requests}\n", self.channel).as_bytes())
            .and_then(|_| self.registry.flush()).map_err(|_| Error("lifecycle-write"))?;
        self.registry.sync_all().map_err(|_| Error("lifecycle-sync"))?;
        fs::hard_link(&self.registry_open, &self.registry_complete)
            .map_err(|_| Error("lifecycle-publish"))?;
        fs::remove_file(&self.registry_open).map_err(|_| Error("lifecycle-publish"))?;
        File::open(&self.directory).and_then(|file| file.sync_all()).map_err(|_| Error("lifecycle-sync"))
    }

    fn finish(mut self) -> Result<()> {
        self.output.flush().map_err(|_| Error("lifecycle-write"))?;
        self.output.sync_all().map_err(|_| Error("lifecycle-sync"))?;
        #[cfg(target_os = "linux")]
        LIFECYCLE_SIGNAL_FD.store(-1, Ordering::Release);
        Ok(())
    }
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

type RecordMessage = Result<Option<ProtocolRecord>>;

fn record_reader<R: Read>(reader: R, sender: SyncSender<RecordMessage>, code: &'static str) {
    let mut reader = BufReader::new(reader);
    loop {
        let message = read_protocol_record(&mut reader).map_err(|_| Error(code));
        let finished = !matches!(message, Ok(Some(_)));
        if sender.send(message).is_err() || finished { return; }
    }
}

fn pending_signal() -> Option<i32> {
    #[cfg(target_os = "linux")]
    {
        let signal = LIFECYCLE_SIGNAL.load(Ordering::Acquire);
        if signal != 0 { return Some(signal); }
    }
    None
}

enum RecordOrSignal {
    Record(ProtocolRecord),
    Eof,
    Signal(i32),
}

fn resolve_record_message(message: RecordMessage, signal: Option<i32>) -> Result<RecordOrSignal> {
    if let Some(signal) = signal { return Ok(RecordOrSignal::Signal(signal)); }
    match message {
        Ok(Some(record)) => Ok(RecordOrSignal::Record(record)),
        Ok(None) => Ok(RecordOrSignal::Eof),
        Err(error) => Err(error),
    }
}

fn receive_record(receiver: &Receiver<RecordMessage>) -> Result<RecordOrSignal> {
    loop {
        if let Some(signal) = pending_signal() { return Ok(RecordOrSignal::Signal(signal)); }
        match receiver.recv_timeout(SHUTDOWN_POLL) {
            Ok(message) => return resolve_record_message(message, pending_signal()),
            Err(RecvTimeoutError::Timeout) => {},
            Err(RecvTimeoutError::Disconnected) => return Err(Error("protocol-reader")),
        }
    }
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
    pending_path: PathBuf,
    final_path: PathBuf,
    directory: PathBuf,
    channel: u32,
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
        let directory = Path::new(directory);
        let pending_path = directory.join(format!("channel-{channel:010}.pending"));
        let final_path = directory.join(format!("channel-{channel:010}.events"));
        let mut output = OpenOptions::new().write(true).create_new(true).mode(0o640).open(&pending_path)
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
        Ok(Self {
            output, pending_path, final_path, directory: directory.to_path_buf(),
            channel, next_id: 1, active: BTreeMap::new(), total: 0,
        })
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
        self.line(&format!("start\t{:010}-{id:08}\t{}", self.channel, url.serialized))?;
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
        self.line(&format!("redirect\t{:010}-{:08}\t{}\t{}\t{}", self.channel, active.id, active.hops, old.serialized, new.serialized))?;
        active.current = new.serialized.clone();
        active.awaiting_redirect_request = true;
        self.active.insert(new.serialized, active);
        Ok(())
    }

    fn terminal(&mut self, record: &ProtocolRecord, outcome: &str) -> Result<()> {
        let url = canonical_https_url(one_field(record, "URI")?)?;
        let active = self.active.remove(&url.serialized).ok_or(Error("terminal-unobserved"))?;
        if active.current != url.serialized { return Err(Error("terminal-url")); }
        self.line(&format!("terminal\t{:010}-{:08}\t{outcome}\t{}", self.channel, active.id, url.serialized))
    }

    fn finish(mut self) -> Result<()> {
        if !self.active.is_empty() { return Err(Error("terminal-missing")); }
        self.line(&format!("method-complete\t{:010}\t{}", self.channel, self.total))?;
        self.output.flush().map_err(|_| Error("telemetry-write"))?;
        self.output.sync_all().map_err(|_| Error("telemetry-sync"))?;
        fs::hard_link(&self.pending_path, &self.final_path).map_err(|_| Error("telemetry-publish"))?;
        fs::remove_file(&self.pending_path).map_err(|_| Error("telemetry-publish"))?;
        File::open(&self.directory).and_then(|file| file.sync_all()).map_err(|_| Error("telemetry-sync"))
    }
}

enum ProgramOutcome {
    Complete,
    Signaled(i32),
}

fn finish_proxy_shutdown(
    tracker: Tracker,
    mut trace: LifecycleTrace,
    shutdown: ShutdownResult,
) -> Result<ProgramOutcome> {
    let requests = tracker.total;
    let active = tracker.active.len();
    if shutdown.accepted {
        trace.transition("event-channel-sync-start", requests, active)?;
        tracker.finish()?;
        trace.publish_completion(requests)?;
        trace.transition("event-channel-synced-closed", requests, 0)?;
    } else {
        trace.transition("event-channel-completion-refused", requests, active)?;
    }
    let outcome = match shutdown.signal {
        Some(2) if shutdown.accepted => {
            trace.transition("wrapper-exit-success", requests, 0)?;
            ProgramOutcome::Complete
        }
        Some(signal) => {
            trace.transition("wrapper-exit-signal", requests, active)?;
            ProgramOutcome::Signaled(signal)
        }
        None if shutdown.accepted => {
            trace.transition("wrapper-exit-success", requests, 0)?;
            ProgramOutcome::Complete
        }
        None => return Err(Error("method-exit")),
    };
    trace.finish()?;
    Ok(outcome)
}

fn proxy(real_method: &str, telemetry: &str, lifecycle: &str) -> Result<ProgramOutcome> {
    let mut tracker = Tracker::create(telemetry)?;
    let mut trace = LifecycleTrace::create(lifecycle, tracker.channel)?;
    trace.transition("proxy-start", tracker.total, tracker.active.len())?;
    trace.transition("configuration-accepted", tracker.total, tracker.active.len())?;
    let mut child = Command::new(real_method).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::inherit())
        .spawn().map_err(|_| Error("method-spawn"))?;
    trace.transition("child-spawned", tracker.total, tracker.active.len())?;
    let mut child_input = Some(child.stdin.take().ok_or(Error("method-stdin"))?);
    let child_output = child.stdout.take().ok_or(Error("method-stdout"))?;
    let (child_sender, child_receiver) = mpsc::sync_channel(1);
    thread::spawn(move || record_reader(child_output, child_sender, "apt-protocol-output"));
    let stdout = io::stdout();
    let mut apt_output = stdout.lock();
    let startup = match receive_record(&child_receiver)? {
        RecordOrSignal::Record(record) => record,
        RecordOrSignal::Eof => return Err(Error("method-startup")),
        RecordOrSignal::Signal(signal) => {
            let shutdown = shutdown_child(
                ShutdownCause::Signal(signal), &mut child, &mut child_input, &child_receiver,
                &mut tracker, &mut trace, &mut apt_output, false,
            )?;
            return finish_proxy_shutdown(tracker, trace, shutdown);
        }
    };
    if startup.code != 100 { return Err(Error("method-startup")); }
    trace.transition("child-startup", tracker.total, tracker.active.len())?;
    apt_output.write_all(&startup.raw).and_then(|_| apt_output.flush()).map_err(|_| Error("apt-output-write"))?;

    let (apt_sender, apt_receiver) = mpsc::sync_channel(1);
    thread::spawn(move || {
        let stdin = io::stdin();
        let input = stdin.lock();
        record_reader(input, apt_sender, "apt-protocol-input");
    });
    loop {
        let input = match receive_record(&apt_receiver)? {
            RecordOrSignal::Record(record) => record,
            RecordOrSignal::Eof => {
                trace.transition("apt-stdin-eof", tracker.total, tracker.active.len())?;
                let shutdown = shutdown_child(
                    ShutdownCause::InputEof, &mut child, &mut child_input, &child_receiver,
                    &mut tracker, &mut trace, &mut apt_output, false,
                )?;
                return finish_proxy_shutdown(tracker, trace, shutdown);
            }
            RecordOrSignal::Signal(signal) => {
                let shutdown = shutdown_child(
                    ShutdownCause::Signal(signal), &mut child, &mut child_input, &child_receiver,
                    &mut tracker, &mut trace, &mut apt_output, false,
                )?;
                return finish_proxy_shutdown(tracker, trace, shutdown);
            }
        };
        trace.transition("apt-input-record", tracker.total, tracker.active.len())?;
        let acquire = input.code == 600;
        if acquire {
            tracker.request(&input)?;
            trace.transition("request-start", tracker.total, tracker.active.len())?;
        }
        let input_pipe = child_input.as_mut().ok_or(Error("method-stdin"))?;
        input_pipe.write_all(&input.raw).and_then(|_| input_pipe.flush()).map_err(|_| Error("method-input-write"))?;
        if !acquire { continue; }
        loop {
            let record = match receive_record(&child_receiver)? {
                RecordOrSignal::Record(record) => record,
                RecordOrSignal::Eof => {
                    let shutdown = shutdown_child(
                        ShutdownCause::InputEof, &mut child, &mut child_input, &child_receiver,
                        &mut tracker, &mut trace, &mut apt_output, true,
                    )?;
                    if shutdown.accepted { return Err(Error("method-output-eof")); }
                    return Err(Error("method-output-eof"));
                }
                RecordOrSignal::Signal(signal) => {
                    let shutdown = shutdown_child(
                        ShutdownCause::Signal(signal), &mut child, &mut child_input, &child_receiver,
                        &mut tracker, &mut trace, &mut apt_output, false,
                    )?;
                    return finish_proxy_shutdown(tracker, trace, shutdown);
                }
            };
            let handled = handle_child_record(&mut tracker, &record)?;
            if handled.terminal { trace.transition("request-terminal", tracker.total, tracker.active.len())?; }
            apt_output.write_all(&record.raw).and_then(|_| apt_output.flush()).map_err(|_| Error("apt-output-write"))?;
            if handled.response_complete { break; }
        }
    }
}

struct HandledRecord {
    response_complete: bool,
    terminal: bool,
}

fn handle_child_record(tracker: &mut Tracker, record: &ProtocolRecord) -> Result<HandledRecord> {
    match record.code {
        103 => {
            tracker.redirect(record)?;
            Ok(HandledRecord { response_complete: true, terminal: false })
        }
        201 => {
            tracker.terminal(record, "success")?;
            Ok(HandledRecord { response_complete: true, terminal: true })
        }
        400 => {
            tracker.terminal(record, "failure")?;
            Ok(HandledRecord { response_complete: true, terminal: true })
        }
        100 | 101 | 102 | 104 | 200 | 351 | 401 | 402 | 403 => {
            Ok(HandledRecord { response_complete: false, terminal: false })
        }
        _ => Err(Error("method-event-unknown")),
    }
}

#[derive(Clone, Copy)]
enum ShutdownCause {
    InputEof,
    Signal(i32),
}

struct ShutdownResult {
    accepted: bool,
    signal: Option<i32>,
}

fn drain_child_records(
    receiver: &Receiver<RecordMessage>,
    tracker: &mut Tracker,
    trace: &mut LifecycleTrace,
    apt_output: &mut impl Write,
    relay: bool,
    output_eof: &mut bool,
) -> Result<()> {
    loop {
        match receiver.try_recv() {
            Ok(Ok(Some(record))) => {
                let handled = handle_child_record(tracker, &record)?;
                if handled.terminal { trace.transition("request-terminal", tracker.total, tracker.active.len())?; }
                if relay {
                    apt_output.write_all(&record.raw).and_then(|_| apt_output.flush())
                        .map_err(|_| Error("apt-output-write"))?;
                }
            }
            Ok(Ok(None)) => {
                *output_eof = true;
                trace.transition("child-stdout-eof", tracker.total, tracker.active.len())?;
                return Ok(());
            }
            Ok(Err(error)) => return Err(error),
            Err(mpsc::TryRecvError::Empty) => return Ok(()),
            Err(mpsc::TryRecvError::Disconnected) => {
                if *output_eof { return Ok(()); }
                return Err(Error("method-output-uncertain"));
            }
        }
    }
}

fn poll_child_phase(
    child: &mut Child,
    receiver: &Receiver<RecordMessage>,
    tracker: &mut Tracker,
    trace: &mut LifecycleTrace,
    apt_output: &mut impl Write,
    relay: bool,
    output_eof: &mut bool,
    status: &mut Option<ExitStatus>,
) -> Result<bool> {
    let deadline = Instant::now() + SHUTDOWN_GRACE;
    loop {
        drain_child_records(receiver, tracker, trace, apt_output, relay, output_eof)?;
        if status.is_none() {
            *status = child.try_wait().map_err(|_| Error("method-wait"))?;
        }
        if status.is_some() && *output_eof { return Ok(true); }
        if Instant::now() >= deadline { return Ok(false); }
        thread::sleep(SHUTDOWN_POLL);
    }
}

fn signal_owned_child(child: &mut Child, number: i32, status: &mut Option<ExitStatus>) -> Result<bool> {
    if status.is_none() { *status = child.try_wait().map_err(|_| Error("method-wait"))?; }
    if status.is_some() { return Ok(false); }
    #[cfg(target_os = "linux")]
    {
        // SAFETY: `child.id()` names the still-unreaped direct child proven alive by
        // `try_wait`; PID reuse is impossible until it is reaped. No process group is used.
        if unsafe { kill(child.id() as i32, number) } != 0 {
            *status = child.try_wait().map_err(|_| Error("method-wait"))?;
            if status.is_some() { return Ok(false); }
            return Err(Error("method-signal"));
        }
        return Ok(true);
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = number;
        child.kill().map_err(|_| Error("method-signal"))?;
        Ok(true)
    }
}

fn shutdown_child(
    cause: ShutdownCause,
    child: &mut Child,
    child_input: &mut Option<std::process::ChildStdin>,
    receiver: &Receiver<RecordMessage>,
    tracker: &mut Tracker,
    trace: &mut LifecycleTrace,
    apt_output: &mut impl Write,
    mut output_eof: bool,
) -> Result<ShutdownResult> {
    let complete_at_trigger = tracker.active.is_empty();
    drop(child_input.take());
    trace.transition("child-stdin-closed", tracker.total, tracker.active.len())?;
    let relay = matches!(cause, ShutdownCause::InputEof);
    let mut status = child.try_wait().map_err(|_| Error("method-wait"))?;
    let mut controlled_signal = false;

    if let ShutdownCause::Signal(number) = cause {
        trace.transition("child-original-signal", tracker.total, tracker.active.len())?;
        controlled_signal = signal_owned_child(child, number, &mut status)?;
    }
    if !poll_child_phase(child, receiver, tracker, trace, apt_output, relay, &mut output_eof, &mut status)? {
        trace.transition("child-term-escalation", tracker.total, tracker.active.len())?;
        controlled_signal |= signal_owned_child(child, 15, &mut status)?;
        if !poll_child_phase(child, receiver, tracker, trace, apt_output, relay, &mut output_eof, &mut status)? {
            trace.transition("child-kill-escalation", tracker.total, tracker.active.len())?;
            if status.is_none() { status = child.try_wait().map_err(|_| Error("method-wait"))?; }
            if status.is_none() {
                if child.kill().is_err() {
                    status = child.try_wait().map_err(|_| Error("method-wait"))?;
                    if status.is_none() { return Err(Error("method-kill")); }
                }
                controlled_signal = true;
            }
            if !poll_child_phase(child, receiver, tracker, trace, apt_output, relay, &mut output_eof, &mut status)? {
                return Err(Error("method-reap-uncertain"));
            }
        }
    }
    let status = status.ok_or(Error("method-reap-uncertain"))?;
    if status.success() {
        trace.transition("child-exit-success", tracker.total, tracker.active.len())?;
    } else {
        #[cfg(target_os = "linux")]
        let state = if status.signal().is_some() { "child-exit-signal" } else { "child-exit-nonzero" };
        #[cfg(not(target_os = "linux"))]
        let state = "child-exit-nonzero";
        trace.transition(state, tracker.total, tracker.active.len())?;
    }
    let accepted = match cause {
        ShutdownCause::Signal(2) => complete_at_trigger && tracker.active.is_empty()
            && (status.success() || controlled_signal),
        ShutdownCause::Signal(_) => false,
        ShutdownCause::InputEof => tracker.active.is_empty() && (status.success() || controlled_signal),
    };
    Ok(ShutdownResult {
        accepted,
        signal: match cause { ShutdownCause::Signal(number) => Some(number), ShutdownCause::InputEof => None },
    })
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

fn verify_channel(
    path: &Path,
    policy: &Policy,
    expected_gid: u32,
    hosts: &mut BTreeSet<String>,
    global_ids: &mut BTreeSet<String>,
) -> Result<usize> {
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
    let channel = channel_identity(path)?;
    let mut lines = text.lines();
    if lines.next() != Some(SCHEMA) { return Err(Error("telemetry-schema")); }
    let mut requests: BTreeMap<String, VerifiedRequest> = BTreeMap::new();
    let mut completed = None;
    for line in lines {
        if line.len() > MAX_LINE || line.is_empty() || completed.is_some() { return Err(Error("telemetry-record")); }
        let fields: Vec<_> = line.split('\t').collect();
        match fields.as_slice() {
            ["start", id, raw_url] => {
                let id = parse_id(id, &channel)?;
                let url = canonical_https_url(raw_url)?;
                if !policy.starts.contains(&url.host) { return Err(Error("start-origin")); }
                if requests.len() >= policy.maximum_requests || requests.contains_key(&id) || !global_ids.insert(id.clone()) {
                    return Err(Error("start-cardinality"));
                }
                hosts.insert(url.host);
                let mut visited = BTreeSet::new(); visited.insert(url.serialized.clone());
                requests.insert(id, VerifiedRequest { current: url.serialized, hops: 0, visited, terminal: false });
            }
            ["redirect", id, hop, raw_from, raw_to] => {
                let id = parse_id(id, &channel)?;
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
                let id = parse_id(id, &channel)?;
                let url = canonical_https_url(raw_url)?;
                if !policy.redirects.contains(&url.host) { return Err(Error("terminal-origin")); }
                hosts.insert(url.host);
                let request = requests.get_mut(&id).ok_or(Error("terminal-unobserved"))?;
                if request.terminal || request.current != url.serialized { return Err(Error("terminal-cardinality")); }
                request.terminal = true;
            }
            ["method-complete", complete_channel, count] => {
                if *complete_channel != channel { return Err(Error("complete-channel")); }
                let count: usize = count.parse().map_err(|_| Error("complete-count"))?;
                completed = Some(count);
            }
            _ => return Err(Error("event-unknown")),
        }
    }
    let expected = completed.ok_or(Error("complete-missing"))?;
    if expected != requests.len() || requests.values().any(|request| !request.terminal) {
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

fn channel_identity(path: &Path) -> Result<String> {
    let name = path.file_name().and_then(|name| name.to_str()).ok_or(Error("telemetry-entry"))?;
    let digits = name.strip_prefix("channel-").and_then(|value| value.strip_suffix(".events"))
        .ok_or(Error("telemetry-entry"))?;
    if digits.len() != 10 || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(Error("telemetry-entry"));
    }
    Ok(digits.to_owned())
}

fn verify_internal(
    directory: &str,
    policy_path: &str,
    expected_channels: Option<&BTreeMap<String, usize>>,
) -> Result<()> {
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
    if let Some(expected) = expected_channels {
        let observed = paths.iter().map(|path| channel_identity(path)).collect::<Result<BTreeSet<_>>>()?;
        let declared = expected.keys().cloned().collect::<BTreeSet<_>>();
        if observed != declared { return Err(Error("telemetry-channel-cardinality")); }
    }
    let channel_count = paths.len();
    eprintln!("preauth-apt-lifecycle:aggregator:transition=discovery:channels={channel_count}");
    let mut hosts = BTreeSet::new();
    let mut global_ids = BTreeSet::new();
    let mut total = 0usize;
    for path in paths {
        let channel = channel_identity(&path)?;
        let verified = match verify_channel(&path, &policy, metadata.gid(), &mut hosts, &mut global_ids) {
            Ok(count) => {
                eprintln!("preauth-apt-lifecycle:aggregator:channel={channel}:result=accepted:requests={count}");
                count
            }
            Err(error) => {
                eprintln!("preauth-apt-lifecycle:aggregator:channel={channel}:result={}", error.0);
                return Err(error);
            }
        };
        if expected_channels.and_then(|expected| expected.get(&channel)).is_some_and(|count| *count != verified) {
            return Err(Error("lifecycle-request-cardinality"));
        }
        total = total.checked_add(verified).ok_or(Error("request-bound"))?;
        if total > policy.maximum_requests { return Err(Error("request-bound")); }
    }
    if total == 0 { return Err(Error("request-cardinality")); }
    let host_list = hosts.into_iter().collect::<Vec<_>>().join(",");
    eprintln!("preauth-input-producer:transfer-hosts:{host_list}");
    eprintln!("preauth-input-producer:transfer-requests:{total}");
    eprintln!("preauth-input-producer:transfer-channels:{channel_count}");
    Ok(())
}

fn verify(directory: &str, policy_path: &str) -> Result<()> {
    verify_internal(directory, policy_path, None)
}

fn lifecycle_entry_channel(name: &str, suffix: &str) -> Result<String> {
    let digits = name.strip_prefix("registry-").and_then(|value| value.strip_suffix(suffix))
        .ok_or(Error("lifecycle-entry"))?;
    if digits.len() != 10 || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(Error("lifecycle-entry"));
    }
    Ok(digits.to_owned())
}

fn parse_registry(path: &Path, expected_gid: u32, channel: &str, complete: bool) -> Result<Option<usize>> {
    let metadata = fs::symlink_metadata(path).map_err(|_| Error("lifecycle-registry-missing"))?;
    if !metadata.file_type().is_file() || metadata.len() as usize > MAX_LINE
        || metadata.nlink() != 1 || metadata.permissions().mode() & 0o777 != 0o640
        || metadata.gid() != expected_gid
    {
        return Err(Error("lifecycle-registry-file"));
    }
    let text = fs::read_to_string(path).map_err(|_| Error("lifecycle-registry-read"))?;
    if !text.ends_with('\n') || !text.is_ascii() { return Err(Error("lifecycle-registry-framing")); }
    let mut lines = text.lines();
    if lines.next() != Some(LIFECYCLE_SCHEMA)
        || lines.next() != Some(&format!("channel\t{channel}"))
    {
        return Err(Error("lifecycle-registry-record"));
    }
    let count = match (complete, lines.next()) {
        (false, None) => None,
        (true, Some(line)) => {
            let fields = line.split('\t').collect::<Vec<_>>();
            match fields.as_slice() {
                ["complete", record_channel, count] if *record_channel == channel => {
                    Some(count.parse().map_err(|_| Error("lifecycle-registry-record"))?)
                }
                _ => return Err(Error("lifecycle-registry-record")),
            }
        }
        _ => return Err(Error("lifecycle-registry-record")),
    };
    if lines.next().is_some() { return Err(Error("lifecycle-registry-record")); }
    Ok(count)
}

fn discover_registries(directory: &Path) -> Result<BTreeMap<String, Option<usize>>> {
    let metadata = fs::symlink_metadata(directory).map_err(|_| Error("lifecycle-directory"))?;
    if !metadata.file_type().is_dir() || !telemetry_directory_mode(metadata.permissions().mode()) {
        return Err(Error("lifecycle-directory"));
    }
    let mut registries = BTreeMap::new();
    for entry in fs::read_dir(directory).map_err(|_| Error("lifecycle-read"))? {
        let entry = entry.map_err(|_| Error("lifecycle-read"))?;
        let name = entry.file_name().into_string().map_err(|_| Error("lifecycle-entry"))?;
        if let Some(digits) = name.strip_prefix("trace-").and_then(|value| value.strip_suffix(".events")) {
            if digits.len() != 10 || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
                return Err(Error("lifecycle-entry"));
            }
            continue;
        }
        let (channel, complete) = if name.ends_with(".open") {
            (lifecycle_entry_channel(&name, ".open")?, false)
        } else if name.ends_with(".complete") {
            (lifecycle_entry_channel(&name, ".complete")?, true)
        } else {
            return Err(Error("lifecycle-entry"));
        };
        let count = parse_registry(&entry.path(), metadata.gid(), &channel, complete)?;
        if registries.insert(channel, count).is_some() { return Err(Error("lifecycle-registry-duplicate")); }
        if registries.len() > MAX_CHANNELS { return Err(Error("telemetry-channel-bound")); }
    }
    if registries.is_empty() { return Err(Error("lifecycle-registry-empty")); }
    Ok(registries)
}

fn await_verify(telemetry: &str, lifecycle: &str, policy: &str) -> Result<()> {
    let lifecycle_path = Path::new(lifecycle);
    let initial = discover_registries(lifecycle_path)?;
    let declared = initial.keys().cloned().collect::<BTreeSet<_>>();
    let deadline = Instant::now() + COMPLETION_WAIT;
    let completed = loop {
        let observed = discover_registries(lifecycle_path)?;
        if observed.keys().cloned().collect::<BTreeSet<_>>() != declared {
            return Err(Error("lifecycle-registry-reopened"));
        }
        if observed.values().all(Option::is_some) { break observed; }
        if Instant::now() >= deadline { return Err(Error("lifecycle-completion-timeout")); }
        thread::sleep(SHUTDOWN_POLL);
    };
    let expected = completed.into_iter().map(|(channel, count)| (channel, count.unwrap())).collect();
    verify_internal(telemetry, policy, Some(&expected))?;
    let final_state = discover_registries(lifecycle_path)?;
    if final_state != expected.iter().map(|(channel, count)| (channel.clone(), Some(*count))).collect() {
        return Err(Error("lifecycle-registry-reopened"));
    }
    Ok(())
}

fn parse_id(raw: &str, channel: &str) -> Result<String> {
    let (raw_channel, local) = raw.split_once('-').ok_or(Error("request-id"))?;
    if raw_channel != channel || local.len() != 8 || !local.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(Error("request-id"));
    }
    let id: u32 = local.parse().map_err(|_| Error("request-id"))?;
    if id == 0 { return Err(Error("request-id")); }
    Ok(raw.to_owned())
}

fn main() {
    let arguments: Vec<_> = std::env::args().collect();
    let result = match arguments.as_slice() {
        [_, command, real, telemetry, lifecycle] if command == "--proxy" => proxy(real, telemetry, lifecycle),
        [_, command, telemetry, policy] if command == "--verify" => {
            verify(telemetry, policy).map(|_| ProgramOutcome::Complete)
        }
        [_, command, telemetry, lifecycle, policy] if command == "--await-verify" => {
            await_verify(telemetry, lifecycle, policy).map(|_| ProgramOutcome::Complete)
        }
        [program] => {
            let _ = program;
            let real = std::env::var("RAR_PREAUTH_APT_REAL_METHOD").map_err(|_| Error("proxy-environment"));
            let telemetry = std::env::var("RAR_PREAUTH_APT_TELEMETRY_DIR").map_err(|_| Error("proxy-environment"));
            let lifecycle = std::env::var("RAR_PREAUTH_APT_LIFECYCLE_DIR").map_err(|_| Error("proxy-environment"));
            match (real, telemetry, lifecycle) {
                (Ok(real), Ok(telemetry), Ok(lifecycle)) => proxy(&real, &telemetry, &lifecycle),
                _ => Err(Error("proxy-environment")),
            }
        }
        [_, command] if command == "--build-root-exec-probe" => Ok(ProgramOutcome::Complete),
        _ => Err(Error("usage-refused")),
    };
    match result {
        Ok(ProgramOutcome::Complete) => {},
        Ok(ProgramOutcome::Signaled(signal)) => std::process::exit(128 + signal),
        Err(error) => fail(error.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_os = "linux")]
    #[derive(Clone, Copy)]
    enum FixtureChildState {
        Running,
        Exited(i32),
    }

    #[cfg(target_os = "linux")]
    fn shutdown_fixture(
        command: &str,
        active: bool,
        cause: ShutdownCause,
        child_state: FixtureChildState,
    ) -> (ProgramOutcome, String, String) {
        use std::sync::atomic::{AtomicU32, Ordering as AtomicOrdering};
        static NEXT: AtomicU32 = AtomicU32::new(1);
        let sequence = NEXT.fetch_add(1, AtomicOrdering::Relaxed);
        let root = std::env::temp_dir().join(format!("rar-apt-lifecycle-test-{}-{sequence}", std::process::id()));
        let events = root.join("events");
        let lifecycle = root.join("lifecycle");
        fs::create_dir(&root).unwrap();
        fs::create_dir(&events).unwrap();
        fs::create_dir(&lifecycle).unwrap();
        fs::set_permissions(&events, fs::Permissions::from_mode(0o2770)).unwrap();
        fs::set_permissions(&lifecycle, fs::Permissions::from_mode(0o2770)).unwrap();
        let channel = 1000 + sequence;
        let mut tracker = Tracker::create_channel(events.to_str().unwrap(), channel).unwrap();
        if active {
            let mut fields = BTreeMap::new();
            fields.insert("URI".to_owned(), vec!["https://snapshot.debian.org/pending".to_owned()]);
            tracker.request(&ProtocolRecord { code: 600, raw: Vec::new(), fields }).unwrap();
        }
        let mut trace = LifecycleTrace::create(lifecycle.to_str().unwrap(), channel).unwrap();
        let mut child = Command::new("/bin/sh")
            .arg("-c").arg(command)
            .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped())
            .spawn().unwrap();
        let mut child_input = Some(child.stdin.take().unwrap());
        let child_output = child.stdout.take().unwrap();
        let mut child_stderr = BufReader::new(child.stderr.take().unwrap());
        let mut readiness = String::new();
        child_stderr.read_line(&mut readiness).unwrap();
        assert_eq!(readiness, "ready\n");
        if let FixtureChildState::Exited(expected) = child_state {
            let status = child.wait().unwrap();
            assert_eq!(status.code(), Some(expected));
        }
        let (sender, receiver) = mpsc::sync_channel(1);
        thread::spawn(move || record_reader(child_output, sender, "apt-protocol-output"));
        let mut output = Vec::new();
        let shutdown = shutdown_child(
            cause, &mut child, &mut child_input, &receiver, &mut tracker, &mut trace, &mut output, false,
        ).unwrap();
        let outcome = finish_proxy_shutdown(tracker, trace, shutdown).unwrap();
        let final_path = events.join(format!("channel-{channel:010}.events"));
        let pending_path = events.join(format!("channel-{channel:010}.pending"));
        let event_path = if final_path.exists() { final_path } else { pending_path };
        let trace_path = lifecycle.join(format!("trace-{channel:010}.events"));
        let event_text = fs::read_to_string(event_path).unwrap();
        let trace_text = fs::read_to_string(trace_path).unwrap();
        fs::remove_dir_all(root).unwrap();
        (outcome, event_text, trace_text)
    }

    #[test]
    fn captured_signal_precedes_concurrent_input_eof() {
        assert!(matches!(
            resolve_record_message(Ok(None), Some(2)).unwrap(),
            RecordOrSignal::Signal(2)
        ));
    }

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

    #[cfg(target_os = "linux")]
    #[test]
    fn signal_after_terminals_reaps_cooperative_and_escalated_children() {
        for (command, expected_transition) in [
            ("trap 'exit 0' INT TERM; printf 'ready\n' >&2; while :; do read ignored < /dev/zero || :; done", "child-original-signal"),
            ("trap '' INT; trap 'exit 0' TERM; printf 'ready\n' >&2; while :; do read ignored < /dev/zero || :; done", "child-term-escalation"),
            ("trap '' INT TERM; printf 'ready\n' >&2; while :; do read ignored < /dev/zero || :; done", "child-kill-escalation"),
        ] {
            let (outcome, events, trace) = shutdown_fixture(command, false, ShutdownCause::Signal(2), FixtureChildState::Running);
            assert!(matches!(outcome, ProgramOutcome::Complete));
            assert!(events.contains("method-complete\t"));
            assert!(trace.contains(expected_transition));
            assert!(trace.contains("event-channel-synced-closed"));
        }
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn signal_before_terminal_reaps_but_refuses_completion() {
        let command = "trap 'exit 0' INT TERM; printf 'ready\n' >&2; while :; do read ignored < /dev/zero || :; done";
        let (outcome, events, trace) = shutdown_fixture(command, true, ShutdownCause::Signal(2), FixtureChildState::Running);
        assert!(matches!(outcome, ProgramOutcome::Signaled(2)));
        assert!(!events.contains("method-complete"));
        assert!(trace.contains("event-channel-completion-refused"));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn clean_eof_reaps_resident_child_before_completion() {
        let command = "trap '' INT; trap 'exit 0' TERM; printf 'ready\n' >&2; while :; do read ignored < /dev/zero || :; done";
        let (outcome, events, trace) = shutdown_fixture(command, false, ShutdownCause::InputEof, FixtureChildState::Running);
        assert!(matches!(outcome, ProgramOutcome::Complete));
        assert!(events.contains("method-complete\t"));
        assert!(trace.contains("child-term-escalation"));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn nonzero_child_and_unexpected_signals_never_publish_completion() {
        let (outcome, events, _) = shutdown_fixture(
            "printf 'ready\n' >&2; exit 7",
            false,
            ShutdownCause::Signal(2),
            FixtureChildState::Exited(7),
        );
        assert!(matches!(outcome, ProgramOutcome::Signaled(2)));
        assert!(!events.contains("method-complete"));
        for signal in [1, 15] {
            let command = "trap 'exit 0' HUP TERM; printf 'ready\n' >&2; while :; do read ignored < /dev/zero || :; done";
            let (outcome, events, _) = shutdown_fixture(
                command,
                false,
                ShutdownCause::Signal(signal),
                FixtureChildState::Running,
            );
            assert!(matches!(outcome, ProgramOutcome::Signaled(value) if value == signal));
            assert!(!events.contains("method-complete"));
        }
    }
}
