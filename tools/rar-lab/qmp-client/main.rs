mod json;

use json::Value;
use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::os::unix::fs::{FileTypeExt, MetadataExt};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process;
use std::thread;
use std::time::{Duration, Instant};

const SOCKET: &str = "/tmp/rar-qmp.sock";
const SERIAL: &str = "/evidence/serial.log";
const MAX_MESSAGE: usize = 65_536;
const MAX_SERIAL: u64 = 8 * 1024 * 1024;
const MAX_CAPTURE: u64 = 64 * 1024 * 1024;
#[cfg(not(test))]
const IO_TIMEOUT: Duration = Duration::from_secs(10);
#[cfg(test)]
const IO_TIMEOUT: Duration = Duration::from_millis(250);

fn main() {
    if let Err(error) = run() {
        eprintln!("rar-qmp-client: {error}");
        process::exit(70);
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    if args.len() == 2 && args[1] == "--version" {
        println!("rar-qmp-client 1");
        return Ok(());
    }
    let verb = args.get(1).map(String::as_str).ok_or("missing verb")?;
    match verb {
        "wait-ready" => {
            exact_args(&args, 4)?;
            let timeout = milliseconds(&args[3], 1, 60_000)?;
            Qmp::connect(&args[2], Instant::now() + timeout)?;
        }
        "continue" => {
            exact_args(&args, 3)?;
            Qmp::connect(&args[2], Instant::now() + IO_TIMEOUT)?.command("cont", None, "continue")?;
        }
        "key-chord" => {
            exact_args(&args, 4)?;
            let events = key_events(&args[3])?;
            Qmp::connect(&args[2], Instant::now() + IO_TIMEOUT)?.command("input-send-event", Some(&events), "key-chord")?;
        }
        "pointer" => {
            exact_args(&args, 6)?;
            let x = number(&args[3], 0, 0x7fff)?;
            let y = number(&args[4], 0, 0x7fff)?;
            let buttons = number(&args[5], 0, 7)?;
            let events = pointer_events(x, y, buttons);
            Qmp::connect(&args[2], Instant::now() + IO_TIMEOUT)?.command("input-send-event", Some(&events), "pointer")?;
        }
        "serial-offset" => {
            exact_args(&args, 3)?;
            if args[2] != SERIAL { return Err("file path is not allowlisted".into()); }
            let (_, metadata) = open_bounded_regular(Path::new(SERIAL), MAX_SERIAL)?;
            println!("{}", metadata.len());
        }
        "wait-trace" => {
            exact_args(&args, 7)?;
            let lower = number(&args[5], 0, MAX_SERIAL)?;
            let timeout = milliseconds(&args[6], 1, 60_000)?;
            wait_trace(&args[2], &args[3], &args[4], lower, timeout)?;
        }
        "capture" => {
            exact_args(&args, 4)?;
            capture(&args[2], &args[3])?;
        }
        "quit" => {
            exact_args(&args, 3)?;
            Qmp::connect(&args[2], Instant::now() + IO_TIMEOUT)?.command("quit", None, "quit")?;
        }
        _ => return Err("unknown verb".into()),
    }
    Ok(())
}

fn exact_args(args: &[String], expected: usize) -> Result<(), String> {
    if args.len() != expected { return Err("wrong argument count".into()); }
    Ok(())
}

fn number(text: &str, minimum: u64, maximum: u64) -> Result<u64, String> {
    if text.is_empty() || text.bytes().any(|byte| !byte.is_ascii_digit()) {
        return Err("numeric argument is malformed".into());
    }
    let value = text.parse::<u64>().map_err(|_| "numeric argument overflows")?;
    if value < minimum || value > maximum { return Err("numeric argument is outside bound".into()); }
    Ok(value)
}

fn milliseconds(text: &str, minimum: u64, maximum: u64) -> Result<Duration, String> {
    Ok(Duration::from_millis(number(text, minimum, maximum)?))
}

struct Qmp {
    stream: UnixStream,
}

impl Qmp {
    fn connect(path: &str, deadline: Instant) -> Result<Self, String> {
        if path != SOCKET { return Err("QMP socket is not the allowlisted path".into()); }
        let stream = loop {
            match fs::symlink_metadata(path) {
                Ok(metadata) if metadata.file_type().is_socket() => {}
                Ok(_) => return Err("QMP path is not a Unix socket".into()),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    if Instant::now() >= deadline { return Err("QMP socket readiness timed out".into()); }
                    thread::sleep(Duration::from_millis(10));
                    continue;
                }
                Err(_) => return Err("QMP socket metadata unavailable".into()),
            }
            match UnixStream::connect(path) {
                Ok(stream) => break stream,
                Err(_) if Instant::now() < deadline => thread::sleep(Duration::from_millis(10)),
                Err(_) => return Err("QMP connection timed out".into()),
            }
        };
        stream.set_read_timeout(Some(IO_TIMEOUT)).map_err(|_| "cannot set QMP read timeout")?;
        stream.set_write_timeout(Some(IO_TIMEOUT)).map_err(|_| "cannot set QMP write timeout")?;
        let mut qmp = Qmp { stream };
        let greeting = qmp.read_message()?;
        if greeting.member("id").is_some() || greeting.member("return").is_some() || greeting.member("error").is_some() || greeting.member("event").is_some() {
            return Err("QMP greeting contains response fields".into());
        }
        let qmp_member = greeting.member("QMP").ok_or("QMP greeting missing")?;
        let version = qmp_member.member("version").ok_or("QMP greeting version missing")?;
        let qemu = version.member("qemu").ok_or("QMP greeting QEMU version missing")?;
        if !qmp_member.is_object() || !version.is_object() || !qemu.is_object()
            || !qemu.member("major").is_some_and(Value::is_number)
            || !qemu.member("minor").is_some_and(Value::is_number)
            || !qemu.member("micro").is_some_and(Value::is_number)
            || version.member("package").and_then(Value::string).is_none()
            || !qmp_member.member("capabilities").is_some_and(Value::is_string_array) {
            return Err("QMP greeting malformed".into());
        }
        qmp.command("qmp_capabilities", None, "capabilities")?;
        Ok(qmp)
    }

    fn command(&mut self, execute: &str, arguments: Option<&str>, id: &str) -> Result<(), String> {
        if !safe_token(execute) || !safe_token(id) { return Err("unsafe QMP command token".into()); }
        let request = match arguments {
            Some(arguments) => format!("{{\"execute\":\"{execute}\",\"arguments\":{arguments},\"id\":\"{id}\"}}\n"),
            None => format!("{{\"execute\":\"{execute}\",\"id\":\"{id}\"}}\n"),
        };
        if request.len() > MAX_MESSAGE { return Err("QMP request exceeds bound".into()); }
        self.stream.write_all(request.as_bytes()).map_err(|_| "QMP request write failed")?;
        self.stream.flush().map_err(|_| "QMP request flush failed")?;
        for _ in 0..64 {
            let response = self.read_message()?;
            if let Some(event) = response.member("event") {
                if event.string().is_none() || response.member("id").is_some() || response.member("return").is_some() || response.member("error").is_some() {
                    return Err("malformed asynchronous QMP event".into());
                }
                continue;
            }
            let response_id = response.member("id").and_then(Value::string).ok_or("QMP response id missing")?;
            if response_id != id { return Err("QMP response id is unknown or out of order".into()); }
            if response.member("error").is_some() { return Err("QMP command returned error".into()); }
            if !response.member("return").is_some_and(Value::is_empty_object) { return Err("QMP success payload is not an empty object".into()); }
            return Ok(());
        }
        Err("too many asynchronous QMP events".into())
    }

    fn read_message(&mut self) -> Result<Value, String> {
        let mut bytes = Vec::new();
        let mut byte = [0u8; 1];
        loop {
            if bytes.len() == MAX_MESSAGE { return Err("QMP message exceeds bound".into()); }
            let count = self.stream.read(&mut byte).map_err(|_| "QMP response read failed")?;
            if count == 0 { return Err("QMP connection closed before response".into()); }
            if byte[0] == b'\n' { break; }
            bytes.push(byte[0]);
        }
        if bytes.last() == Some(&b'\r') { bytes.pop(); }
        json::parse(&bytes).map_err(str::to_string)
    }
}

fn safe_token(value: &str) -> bool {
    !value.is_empty() && value.len() <= 64 && value.bytes().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-' || byte == b'_')
}

fn key_events(chord: &str) -> Result<String, String> {
    let keys: &[&str] = match chord {
        "ctrl-alt-b" => &["ctrl", "alt", "b"],
        "ctrl-alt-c" => &["ctrl", "alt", "c"],
        "ctrl-alt-d" => &["ctrl", "alt", "d"],
        "ctrl-alt-f" => &["ctrl", "alt", "f"],
        "ctrl-alt-g" => &["ctrl", "alt", "g"],
        "meta-l" => &["meta_l", "l"],
        "meta-t" => &["meta_l", "t"],
        "meta-s" => &["meta_l", "s"],
        "meta-1" => &["meta_l", "1"],
        "meta-2" => &["meta_l", "2"],
        _ => return Err("keyboard chord is not allowlisted".into()),
    };
    let mut events = Vec::new();
    for key in keys { events.push(key_event(key, true)); }
    for key in keys.iter().rev() { events.push(key_event(key, false)); }
    Ok(format!("{{\"events\":[{}]}}", events.join(",")))
}

fn key_event(key: &str, down: bool) -> String {
    format!("{{\"type\":\"key\",\"data\":{{\"down\":{down},\"key\":{{\"type\":\"qcode\",\"data\":\"{key}\"}}}}}}")
}

fn pointer_events(x: u64, y: u64, buttons: u64) -> String {
    let mut events = vec![
        format!("{{\"type\":\"abs\",\"data\":{{\"axis\":\"x\",\"value\":{x}}}}}"),
        format!("{{\"type\":\"abs\",\"data\":{{\"axis\":\"y\",\"value\":{y}}}}}"),
    ];
    for (mask, name) in [(1, "left"), (2, "middle"), (4, "right")] {
        if buttons & mask != 0 {
            events.push(format!("{{\"type\":\"btn\",\"data\":{{\"down\":true,\"button\":\"{name}\"}}}}"));
            events.push(format!("{{\"type\":\"btn\",\"data\":{{\"down\":false,\"button\":\"{name}\"}}}}"));
        }
    }
    format!("{{\"events\":[{}]}}", events.join(","))
}

fn same_file(left: &fs::Metadata, right: &fs::Metadata) -> bool {
    left.dev() == right.dev() && left.ino() == right.ino()
}

fn open_bounded_regular(path: &Path, maximum: u64) -> Result<(File, fs::Metadata), String> {
    let path_metadata = fs::symlink_metadata(path).map_err(|_| "file metadata unavailable")?;
    if !path_metadata.file_type().is_file() || path_metadata.file_type().is_symlink() || path_metadata.len() > maximum {
        return Err("path is not a bounded regular file".into());
    }
    let file = OpenOptions::new().read(true).write(false).open(path).map_err(|_| "file cannot open")?;
    let descriptor_metadata = file.metadata().map_err(|_| "file descriptor metadata unavailable")?;
    if !descriptor_metadata.file_type().is_file() || descriptor_metadata.len() > maximum || !same_file(&path_metadata, &descriptor_metadata) {
        return Err("file changed during open".into());
    }
    Ok((file, descriptor_metadata))
}

fn wait_trace(socket: &str, serial: &str, marker: &str, lower: u64, timeout: Duration) -> Result<(), String> {
    if serial != SERIAL { return Err("serial path is not allowlisted".into()); }
    if marker.is_empty() || marker.len() > 160 || !marker.bytes().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b':' | b'-')) {
        return Err("trace marker is invalid".into());
    }
    let deadline = Instant::now() + timeout;
    let _qmp = Qmp::connect(socket, deadline)?;
    let (mut file, identity) = open_bounded_regular(Path::new(serial), MAX_SERIAL)?;
    loop {
        let path_metadata = fs::symlink_metadata(serial).map_err(|_| "serial log metadata unavailable")?;
        let descriptor_metadata = file.metadata().map_err(|_| "serial descriptor metadata unavailable")?;
        if path_metadata.file_type().is_symlink() || !same_file(&identity, &path_metadata) || !same_file(&identity, &descriptor_metadata) {
            return Err("serial log identity changed".into());
        }
        let size = descriptor_metadata.len();
        if size > MAX_SERIAL { return Err("serial log exceeds bound".into()); }
        if size < lower { return Err("serial log shrank below lower bound".into()); }
        if size > lower {
            file.seek(SeekFrom::Start(lower)).map_err(|_| "serial seek failed")?;
            let capacity = usize::try_from(size - lower).map_err(|_| "serial span overflows")?;
            let mut bytes = Vec::with_capacity(capacity);
            (&mut file).take(size - lower).read_to_end(&mut bytes).map_err(|_| "serial read failed")?;
            if bytes.len() as u64 != size - lower { return Err("serial changed during bounded read".into()); }
            let mut start = 0usize;
            let mut match_offset = None;
            for (index, byte) in bytes.iter().enumerate() {
                if *byte == b'\n' {
                    let mut end = index;
                    if end > start && bytes[end - 1] == b'\r' { end -= 1; }
                    let line = &bytes[start..end];
                    if line.len() > 256 { return Err("serial line exceeds bound".into()); }
                    if line == marker.as_bytes() {
                        if match_offset.is_some() { return Err("trace marker is duplicated after lower bound".into()); }
                        match_offset = Some(lower + index as u64 + 1);
                    }
                    start = index + 1;
                }
            }
            if let Some(offset) = match_offset {
                println!("{offset}");
                return Ok(());
            }
        }
        if Instant::now() >= deadline { return Err("trace marker timed out".into()); }
        thread::sleep(Duration::from_millis(10));
    }
}

fn capture(socket: &str, output: &str) -> Result<(), String> {
    let path = capture_path(output)?;
    if fs::symlink_metadata(&path).is_ok() { return Err("capture output already exists".into()); }
    let parent = path.parent().ok_or("capture parent missing")?;
    let parent_metadata = fs::symlink_metadata(parent).map_err(|_| "capture parent unavailable")?;
    if !parent_metadata.is_dir() || parent_metadata.file_type().is_symlink() { return Err("capture parent is unsafe".into()); }
    let name = path.file_name().and_then(|name| name.to_str()).ok_or("capture filename invalid")?;
    let temporary = path.with_file_name(format!(".{name}.tmp"));
    if fs::symlink_metadata(&temporary).is_ok() { return Err("capture temporary output already exists".into()); }
    let temporary_text = temporary.to_str().ok_or("capture temporary path encoding invalid")?;
    let arguments = format!("{{\"filename\":\"{}\",\"format\":\"ppm\"}}", temporary_text);
    Qmp::connect(socket, Instant::now() + IO_TIMEOUT)?.command("screendump", Some(&arguments), "capture")?;
    let (mut source, source_metadata) = open_bounded_regular(&temporary, MAX_CAPTURE)?;
    validate_ppm_file(&mut source, &source_metadata)?;
    fs::hard_link(&temporary, &path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::AlreadyExists { "capture output appeared before commit" } else { "capture atomic no-replace commit failed" }
    })?;
    let committed_metadata = fs::symlink_metadata(&path).map_err(|_| "capture commit disappeared")?;
    if committed_metadata.file_type().is_symlink() || !same_file(&source_metadata, &committed_metadata) {
        return Err("capture commit identity mismatch".into());
    }
    let (mut committed, reopened_metadata) = open_bounded_regular(&path, MAX_CAPTURE)?;
    if !same_file(&source_metadata, &reopened_metadata) { return Err("capture changed after commit".into()); }
    validate_ppm_file(&mut committed, &reopened_metadata)?;
    fs::remove_file(&temporary).map_err(|_| "capture temporary cleanup failed")?;
    Ok(())
}

fn capture_path(output: &str) -> Result<PathBuf, String> {
    let path = Path::new(output);
    if path.parent() != Some(Path::new("/evidence")) { return Err("capture path escapes evidence root".into()); }
    let name = path.file_name().and_then(|name| name.to_str()).ok_or("capture filename invalid")?;
    if !name.ends_with(".ppm") || name.len() > 80 || !name.bytes().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'.')) {
        return Err("capture filename is not allowlisted".into());
    }
    Ok(path.to_path_buf())
}

fn validate_ppm(path: &Path) -> Result<(), String> {
    let (mut file, metadata) = open_bounded_regular(path, MAX_CAPTURE)?;
    validate_ppm_file(&mut file, &metadata)
}

fn validate_ppm_file(file: &mut File, metadata: &fs::Metadata) -> Result<(), String> {
    if metadata.len() == 0 || metadata.len() > MAX_CAPTURE {
        return Err("capture file type or size invalid".into());
    }
    let mut header = [0u8; 128];
    file.seek(SeekFrom::Start(0)).map_err(|_| "capture header seek failed")?;
    let count = file.read(&mut header).map_err(|_| "capture header read failed")?;
    let positions: Vec<usize> = header[..count].iter().enumerate().filter_map(|(index, byte)| (*byte == b'\n').then_some(index)).take(3).collect();
    if positions.len() != 3 { return Err("capture header is incomplete".into()); }
    let magic = &header[..positions[0]];
    let dimensions = std::str::from_utf8(&header[positions[0] + 1..positions[1]]).map_err(|_| "capture dimensions encoding invalid")?;
    let maximum = &header[positions[1] + 1..positions[2]];
    if magic != b"P6" || maximum != b"255" { return Err("capture header is invalid".into()); }
    let mut parts = dimensions.split(' ');
    let width = parts.next().ok_or("capture width missing")?.parse::<u64>().map_err(|_| "capture width invalid")?;
    let height = parts.next().ok_or("capture height missing")?.parse::<u64>().map_err(|_| "capture height invalid")?;
    if parts.next().is_some() || width == 0 || width > 4096 || height == 0 || height > 2160 { return Err("capture dimensions outside bound".into()); }
    let pixels = width.checked_mul(height).and_then(|value| value.checked_mul(3)).ok_or("capture size overflows")?;
    let expected = positions[2] as u64 + 1 + pixels;
    if metadata.len() != expected { return Err("capture payload size mismatch".into()); }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::net::UnixListener;
    use std::sync::Mutex;

    static QMP_TEST_LOCK: Mutex<()> = Mutex::new(());

    fn send_line(stream: &mut UnixStream, line: &str) {
        stream.write_all(line.as_bytes()).unwrap();
        stream.write_all(b"\n").unwrap();
        stream.flush().unwrap();
    }

    fn read_line(stream: &mut UnixStream) -> String {
        let mut output = Vec::new();
        let mut byte = [0u8; 1];
        loop {
            match stream.read(&mut byte) {
                Ok(0) => break,
                Ok(_) if byte[0] == b'\n' => break,
                Ok(_) => output.push(byte[0]),
                Err(_) => break,
            }
        }
        String::from_utf8(output).unwrap()
    }

    fn greeting() -> &'static str {
        r#"{"QMP":{"version":{"qemu":{"major":7,"minor":2,"micro":0},"package":""},"capabilities":[]}}"#
    }

    fn with_server<R>(handler: impl FnOnce(UnixStream) + Send + 'static, client: impl FnOnce() -> R) -> R {
        let _guard = QMP_TEST_LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        let _ = fs::remove_file(SOCKET);
        let listener = UnixListener::bind(SOCKET).unwrap();
        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            handler(stream);
        });
        let result = client();
        server.join().unwrap();
        let _ = fs::remove_file(SOCKET);
        result
    }

    #[test]
    fn keyboard_is_allowlisted_and_released() {
        let events = key_events("ctrl-alt-b").unwrap();
        assert!(events.contains("\"data\":\"ctrl\""));
        assert!(events.contains("\"down\":false"));
        assert!(key_events("ctrl-alt-delete").is_err());
    }

    #[test]
    fn pointer_is_bounded_and_ordered() {
        let events = pointer_events(32, 24, 1);
        assert!(events.find("\"axis\":\"x\"").unwrap() < events.find("\"button\":\"left\"").unwrap());
        assert!(events.contains("\"down\":true"));
        assert!(events.contains("\"down\":false"));
        assert!(number("32767", 0, 0x7fff).is_ok());
        assert!(number("32768", 0, 0x7fff).is_err());
    }

    #[test]
    fn numeric_and_capture_names_fail_closed() {
        assert_eq!(number("000", 0, 1).unwrap(), 0);
        assert!(number("-1", 0, 1).is_err());
        assert!(number("2", 0, 1).is_err());
        assert!(capture_path("/evidence/boot.ppm").is_ok());
        assert!(capture_path("/tmp/boot.ppm").is_err());
        assert!(capture_path("/evidence/../boot.ppm").is_err());
    }

    #[test]
    fn qmp_protocol_accepts_only_strict_transcripts() {
        let valid = with_server(|mut stream| {
            send_line(&mut stream, greeting());
            assert!(read_line(&mut stream).contains("qmp_capabilities"));
            send_line(&mut stream, r#"{"return":{},"id":"capabilities"}"#);
            assert!(read_line(&mut stream).contains("\"execute\":\"cont\""));
            send_line(&mut stream, r#"{"return":{},"id":"continue"}"#);
        }, || {
            let mut qmp = Qmp::connect(SOCKET, Instant::now() + IO_TIMEOUT).unwrap();
            qmp.command("cont", None, "continue")
        });
        assert!(valid.is_ok());

        let malformed_greeting = with_server(|mut stream| {
            send_line(&mut stream, r#"{"QMP":{}}"#);
        }, || Qmp::connect(SOCKET, Instant::now() + IO_TIMEOUT));
        assert!(malformed_greeting.is_err());

        let malformed_capabilities = with_server(|mut stream| {
            send_line(&mut stream, r#"{"QMP":{"version":{"qemu":{"major":7,"minor":2,"micro":0},"package":""},"capabilities":[1]}}"#);
        }, || Qmp::connect(SOCKET, Instant::now() + IO_TIMEOUT));
        assert!(malformed_capabilities.is_err());

        let null_return = with_server(|mut stream| {
            send_line(&mut stream, greeting());
            read_line(&mut stream);
            send_line(&mut stream, r#"{"return":null,"id":"capabilities"}"#);
        }, || Qmp::connect(SOCKET, Instant::now() + IO_TIMEOUT));
        assert!(null_return.is_err());

        let wrong_id = with_server(|mut stream| {
            send_line(&mut stream, greeting());
            read_line(&mut stream);
            send_line(&mut stream, r#"{"return":{},"id":"wrong"}"#);
        }, || Qmp::connect(SOCKET, Instant::now() + IO_TIMEOUT));
        assert!(wrong_id.is_err());

        let error_and_return = with_server(|mut stream| {
            send_line(&mut stream, greeting());
            read_line(&mut stream);
            send_line(&mut stream, r#"{"return":{},"error":{"class":"GenericError","desc":"no"},"id":"capabilities"}"#);
        }, || Qmp::connect(SOCKET, Instant::now() + IO_TIMEOUT));
        assert!(error_and_return.is_err());

        let truncated = with_server(|mut stream| {
            stream.write_all(b"{\"QMP\":").unwrap();
        }, || Qmp::connect(SOCKET, Instant::now() + IO_TIMEOUT));
        assert!(truncated.is_err());

        let slow = with_server(|mut stream| {
            send_line(&mut stream, greeting());
            read_line(&mut stream);
            thread::sleep(IO_TIMEOUT + IO_TIMEOUT);
            let _ = stream.write_all(b"{\"return\":{},\"id\":\"capabilities\"}\n");
        }, || Qmp::connect(SOCKET, Instant::now() + IO_TIMEOUT));
        assert!(slow.is_err());

        let _guard = QMP_TEST_LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        let _ = fs::remove_file(SOCKET);
        let alternate = "/tmp/rar-qmp-test-target.sock";
        let _ = fs::remove_file(alternate);
        let listener = UnixListener::bind(alternate).unwrap();
        std::os::unix::fs::symlink(alternate, SOCKET).unwrap();
        assert!(Qmp::connect(SOCKET, Instant::now() + IO_TIMEOUT).is_err());
        drop(listener);
        fs::remove_file(SOCKET).unwrap();
        fs::remove_file(alternate).unwrap();
    }

    #[test]
    fn qmp_events_cannot_masquerade_or_flood() {
        let masquerade = with_server(|mut stream| {
            send_line(&mut stream, greeting());
            read_line(&mut stream);
            send_line(&mut stream, r#"{"return":{},"id":"capabilities"}"#);
            read_line(&mut stream);
            send_line(&mut stream, r#"{"event":"STOP","id":"continue"}"#);
        }, || {
            let mut qmp = Qmp::connect(SOCKET, Instant::now() + IO_TIMEOUT).unwrap();
            qmp.command("cont", None, "continue")
        });
        assert!(masquerade.is_err());

        let flood = with_server(|mut stream| {
            send_line(&mut stream, greeting());
            read_line(&mut stream);
            send_line(&mut stream, r#"{"return":{},"id":"capabilities"}"#);
            read_line(&mut stream);
            for _ in 0..64 { send_line(&mut stream, r#"{"event":"STOP"}"#); }
        }, || {
            let mut qmp = Qmp::connect(SOCKET, Instant::now() + IO_TIMEOUT).unwrap();
            qmp.command("cont", None, "continue")
        });
        assert!(flood.is_err());
    }

    #[test]
    fn capture_commit_is_no_replace_and_ppm_is_exact() {
        let success_path = Path::new("/evidence/success.ppm");
        let success_temporary = Path::new("/evidence/.success.ppm.tmp");
        let _ = fs::remove_file(success_path);
        let _ = fs::remove_file(success_temporary);
        let success = with_server(move |mut stream| {
            send_line(&mut stream, greeting());
            read_line(&mut stream);
            send_line(&mut stream, r#"{"return":{},"id":"capabilities"}"#);
            assert!(read_line(&mut stream).contains("screendump"));
            fs::write(success_temporary, b"P6\n1 1\n255\n\0\0\0").unwrap();
            send_line(&mut stream, r#"{"return":{},"id":"capture"}"#);
        }, || capture(SOCKET, "/evidence/success.ppm"));
        assert!(success.is_ok());
        assert_eq!(fs::read(success_path).unwrap(), b"P6\n1 1\n255\n\0\0\0");
        assert!(!success_temporary.exists());
        fs::remove_file(success_path).unwrap();

        let final_path = Path::new("/evidence/collision.ppm");
        let temporary = Path::new("/evidence/.collision.ppm.tmp");
        let _ = fs::remove_file(final_path);
        let _ = fs::remove_file(temporary);
        let collision = with_server(move |mut stream| {
            send_line(&mut stream, greeting());
            read_line(&mut stream);
            send_line(&mut stream, r#"{"return":{},"id":"capabilities"}"#);
            assert!(read_line(&mut stream).contains("screendump"));
            fs::write(temporary, b"P6\n1 1\n255\n\0\0\0").unwrap();
            fs::write(final_path, b"competing writer").unwrap();
            send_line(&mut stream, r#"{"return":{},"id":"capture"}"#);
        }, || capture(SOCKET, "/evidence/collision.ppm"));
        assert!(collision.is_err());
        assert_eq!(fs::read(final_path).unwrap(), b"competing writer");
        let _ = fs::remove_file(final_path);
        let _ = fs::remove_file(temporary);

        let valid = Path::new("/build/valid.ppm");
        let oversized = Path::new("/build/oversized.ppm");
        fs::write(valid, b"P6\n1 1\n255\n\0\0\0").unwrap();
        fs::write(oversized, b"P6\n1 1\n255\n\0\0\0x").unwrap();
        assert!(validate_ppm(valid).is_ok());
        assert!(validate_ppm(oversized).is_err());
        fs::remove_file(valid).unwrap();
        fs::remove_file(oversized).unwrap();
    }

    #[test]
    fn trace_wait_rejects_duplicates_and_file_replacement() {
        let serial = Path::new(SERIAL);
        fs::write(serial, b"prefix\n").unwrap();
        let lower = fs::metadata(serial).unwrap().len();
        let delayed = with_server(move |mut stream| {
            send_line(&mut stream, greeting());
            read_line(&mut stream);
            send_line(&mut stream, r#"{"return":{},"id":"capabilities"}"#);
            thread::sleep(Duration::from_millis(40));
            OpenOptions::new().append(true).open(SERIAL).unwrap()
                .write_all(b"surface:ready\n").unwrap();
        }, || wait_trace(SOCKET, SERIAL, "surface:ready", lower, IO_TIMEOUT));
        assert!(delayed.is_ok());

        fs::write(serial, b"surface:ready\nsurface:ready\n").unwrap();
        let duplicate = with_server(|mut stream| {
            send_line(&mut stream, greeting());
            read_line(&mut stream);
            send_line(&mut stream, r#"{"return":{},"id":"capabilities"}"#);
            thread::sleep(IO_TIMEOUT);
        }, || wait_trace(SOCKET, SERIAL, "surface:ready", 0, IO_TIMEOUT));
        assert!(duplicate.is_err());

        fs::write(serial, b"prefix\n").unwrap();
        let lower = fs::metadata(serial).unwrap().len();
        let replaced = with_server(move |mut stream| {
            send_line(&mut stream, greeting());
            read_line(&mut stream);
            send_line(&mut stream, r#"{"return":{},"id":"capabilities"}"#);
            thread::sleep(Duration::from_millis(50));
            fs::rename(SERIAL, "/evidence/replaced.log").unwrap();
            fs::write(SERIAL, b"prefix\nsurface:ready\n").unwrap();
            thread::sleep(IO_TIMEOUT);
        }, || wait_trace(SOCKET, SERIAL, "surface:ready", lower, IO_TIMEOUT + IO_TIMEOUT));
        assert!(replaced.is_err());
        let _ = fs::remove_file(SERIAL);
        let _ = fs::remove_file("/evidence/replaced.log");
    }
}
