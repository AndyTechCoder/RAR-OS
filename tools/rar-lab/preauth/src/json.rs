//! Repository-owned bounded JSON parser for pre-authorization trust inputs.
//!
//! This intentionally implements only JSON, not a permissive JavaScript dialect. It rejects
//! duplicate object keys, non-canonical number forms, invalid Unicode escapes, trailing data,
//! and inputs outside the frozen ADR 0022 bounds (1 MiB length, container depth 64, 4,096 object keys, 4,096 array items, 64 KiB strings). `canonical()` emits deterministic RFC 8259 JSON
//! with lexicographically ordered object keys.

use std::collections::BTreeMap;

use super::{PreauthError, Result};

const MAX_BYTES: usize = 1024 * 1024;
const MAX_DEPTH: usize = 64;
const MAX_CONTAINER_ITEMS: usize = 4096;
const MAX_STRING: usize = 64 * 1024;
const MAX_DIAGNOSTIC_KEYS: usize = 32;
const MAX_DIAGNOSTIC_KEY_BYTES: usize = 96;
const MAX_DIAGNOSTIC_PATH_BYTES: usize = 192;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Number(u64),
    String(String),
    Array(Vec<Json>),
    Object(BTreeMap<String, Json>),
}

impl Json {
    pub fn parse(input: &[u8]) -> Result<Self> {
        if input.len() > MAX_BYTES {
            return Err(PreauthError::new("json-byte-limit"));
        }
        let text = std::str::from_utf8(input).map_err(|_| PreauthError::new("json-utf8"))?;
        let mut parser = Parser { bytes: text.as_bytes(), at: 0 };
        parser.ws();
        let value = parser.value(0)?;
        parser.ws();
        if parser.at != parser.bytes.len() {
            return Err(PreauthError::new("json-trailing-data"));
        }
        Ok(value)
    }

    pub fn object(&self) -> Result<&BTreeMap<String, Json>> {
        match self { Self::Object(value) => Ok(value), _ => Err(PreauthError::new("json-type")) }
    }
    pub fn array(&self) -> Result<&[Json]> {
        match self { Self::Array(value) => Ok(value), _ => Err(PreauthError::new("json-type")) }
    }
    pub fn string(&self) -> Result<&str> {
        match self { Self::String(value) => Ok(value), _ => Err(PreauthError::new("json-type")) }
    }
    pub fn number(&self) -> Result<u64> {
        match self { Self::Number(value) => Ok(*value), _ => Err(PreauthError::new("json-type")) }
    }
    pub fn exact_keys(&self, required: &[&str], optional: &[&str]) -> Result<()> {
        let object = self.object()?;
        if required.iter().any(|key| !object.contains_key(*key))
            || object.keys().any(|key| !required.contains(&key.as_str()) && !optional.contains(&key.as_str()))
        {
            return Err(PreauthError::new("json-key-set"));
        }
        Ok(())
    }
    /// Returns a value-redacted, deterministic key-set diagnostic for a failed exact-key check.
    ///
    /// `document` and `path` are caller-owned schema labels, never input-derived filesystem paths.
    /// Actual keys are already bounded by the parser; this report adds smaller count and byte caps
    /// so even adversarial Unicode/control-key inputs cannot create unbounded logs.
    pub fn key_set_diagnostic(
        &self, document: &str, path: &str, required: &[&str], optional: &[&str],
    ) -> Result<Option<String>> {
        let object = self.object()?;
        let mut allowed = required.iter().chain(optional).copied().collect::<Vec<_>>();
        allowed.sort_unstable();
        allowed.dedup();
        let mut missing = required.iter().copied().filter(|key| !object.contains_key(*key)).collect::<Vec<_>>();
        missing.sort_unstable();
        let unknown = object.keys().filter(|key| !allowed.contains(&key.as_str())).map(String::as_str).collect::<Vec<_>>();
        if missing.is_empty() && unknown.is_empty() { return Ok(None); }
        if document.is_empty() || document.len() > 64
            || !document.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
            || path.is_empty() || path.len() > MAX_DIAGNOSTIC_PATH_BYTES || !path.starts_with('/')
        { return Err(PreauthError::new("json-diagnostic-label")); }
        let actual = object.keys().map(String::as_str).collect::<Vec<_>>();
        Ok(Some(format!(
            "json_key_set document={} path={} actual_count={} actual_reported={} actual={:?} allowed_count={} allowed_reported={} allowed={:?} missing_count={} missing_reported={} missing={:?} unknown_count={} unknown_reported={} unknown={:?} key_cap={} key_byte_cap={}",
            document, escape_path(path), actual.len(), actual.len().min(MAX_DIAGNOSTIC_KEYS), diagnostic_keys(&actual),
            allowed.len(), allowed.len().min(MAX_DIAGNOSTIC_KEYS), diagnostic_keys(&allowed),
            missing.len(), missing.len().min(MAX_DIAGNOSTIC_KEYS), diagnostic_keys(&missing),
            unknown.len(), unknown.len().min(MAX_DIAGNOSTIC_KEYS), diagnostic_keys(&unknown),
            MAX_DIAGNOSTIC_KEYS, MAX_DIAGNOSTIC_KEY_BYTES,
        )))
    }
    pub fn get(&self, key: &str) -> Result<&Json> {
        self.object()?.get(key).ok_or_else(|| PreauthError::new("json-missing-key"))
    }

    pub fn canonical(&self) -> String {
        let mut output = String::new();
        self.write(&mut output);
        output
    }
    fn write(&self, output: &mut String) {
        match self {
            Self::Null => output.push_str("null"),
            Self::Bool(value) => output.push_str(if *value { "true" } else { "false" }),
            Self::Number(value) => output.push_str(&value.to_string()),
            Self::String(value) => write_string(output, value),
            Self::Array(values) => {
                output.push('[');
                for (index, value) in values.iter().enumerate() {
                    if index != 0 { output.push(','); }
                    value.write(output);
                }
                output.push(']');
            }
            Self::Object(values) => {
                output.push('{');
                for (index, (key, value)) in values.iter().enumerate() {
                    if index != 0 { output.push(','); }
                    write_string(output, key);
                    output.push(':');
                    value.write(output);
                }
                output.push('}');
            }
        }
    }
}

fn diagnostic_keys(values: &[&str]) -> Vec<String> {
    values.iter().take(MAX_DIAGNOSTIC_KEYS).map(|value| escape_key(value)).collect()
}

fn escape_key(value: &str) -> String {
    let mut output = String::new();
    for byte in value.as_bytes().iter().take(MAX_DIAGNOSTIC_KEY_BYTES) {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-') {
            output.push(char::from(*byte));
        } else {
            output.push_str(&format!("\\x{byte:02x}"));
        }
    }
    if value.len() > MAX_DIAGNOSTIC_KEY_BYTES { output.push_str("..."); }
    output
}

fn escape_path(value: &str) -> String {
    value.split('/').map(|component| component.replace('~', "~0").replace('/', "~1")).collect::<Vec<_>>().join("/")
}

fn write_string(output: &mut String, value: &str) {
    output.push('"');
    for character in value.chars() {
        match character {
            '"' => output.push_str("\\\""), '\\' => output.push_str("\\\\"),
            '\u{08}' => output.push_str("\\b"), '\u{0c}' => output.push_str("\\f"),
            '\n' => output.push_str("\\n"), '\r' => output.push_str("\\r"), '\t' => output.push_str("\\t"),
            c if c <= '\u{1f}' => output.push_str(&format!("\\u{:04x}", c as u32)),
            c => output.push(c),
        }
    }
    output.push('"');
}

struct Parser<'a> { bytes: &'a [u8], at: usize }
impl Parser<'_> {
    fn ws(&mut self) { while self.bytes.get(self.at).is_some_and(u8::is_ascii_whitespace) { self.at += 1; } }
    fn bump(&mut self) -> Result<u8> {
        let byte = *self.bytes.get(self.at).ok_or_else(|| PreauthError::new("json-truncated"))?;
        self.at += 1; Ok(byte)
    }
    fn value(&mut self, depth: usize) -> Result<Json> {
        self.ws();
        match self.bytes.get(self.at).copied() {
            Some(b'{') => {
                if depth >= MAX_DEPTH { return Err(PreauthError::new("json-depth-limit")); }
                self.object_value(depth + 1)
            }
            Some(b'[') => {
                if depth >= MAX_DEPTH { return Err(PreauthError::new("json-depth-limit")); }
                self.array_value(depth + 1)
            }
            Some(b'"') => Ok(Json::String(self.string_value()?)),
            Some(b't') => { self.literal(b"true")?; Ok(Json::Bool(true)) }
            Some(b'f') => { self.literal(b"false")?; Ok(Json::Bool(false)) }
            Some(b'n') => { self.literal(b"null")?; Ok(Json::Null) }
            Some(b'0'..=b'9') => self.number_value().map(Json::Number),
            _ => Err(PreauthError::new("json-syntax")),
        }
    }
    fn literal(&mut self, expected: &[u8]) -> Result<()> {
        if self.bytes.get(self.at..self.at + expected.len()) != Some(expected) { return Err(PreauthError::new("json-syntax")); }
        self.at += expected.len(); Ok(())
    }
    fn number_value(&mut self) -> Result<u64> {
        let start = self.at;
        if self.bump()? == b'0' {
            if self.bytes.get(self.at).is_some_and(u8::is_ascii_digit) { return Err(PreauthError::new("json-number-form")); }
        } else { while self.bytes.get(self.at).is_some_and(u8::is_ascii_digit) { self.at += 1; } }
        if self.bytes.get(self.at).is_some_and(|b| matches!(b, b'.' | b'e' | b'E' | b'+' | b'-')) {
            return Err(PreauthError::new("json-number-form"));
        }
        std::str::from_utf8(&self.bytes[start..self.at]).unwrap().parse().map_err(|_| PreauthError::new("json-number-overflow"))
    }
    fn string_value(&mut self) -> Result<String> {
        if self.bump()? != b'"' { return Err(PreauthError::new("json-syntax")); }
        let mut output = String::new();
        loop {
            let byte = self.bump()?;
            match byte {
                b'"' => break,
                0..=0x1f => return Err(PreauthError::new("json-control-character")),
                b'\\' => {
                    match self.bump()? {
                        b'"' => output.push('"'), b'\\' => output.push('\\'), b'/' => output.push('/'),
                        b'b' => output.push('\u{08}'), b'f' => output.push('\u{0c}'), b'n' => output.push('\n'),
                        b'r' => output.push('\r'), b't' => output.push('\t'),
                        b'u' => {
                            let first = self.hex4()?;
                            let scalar = if (0xd800..=0xdbff).contains(&first) {
                                if self.bump()? != b'\\' || self.bump()? != b'u' { return Err(PreauthError::new("json-surrogate")); }
                                let second = self.hex4()?;
                                if !(0xdc00..=0xdfff).contains(&second) { return Err(PreauthError::new("json-surrogate")); }
                                0x10000 + ((first - 0xd800) << 10) + (second - 0xdc00)
                            } else if (0xdc00..=0xdfff).contains(&first) { return Err(PreauthError::new("json-surrogate")); }
                            else { first };
                            output.push(char::from_u32(scalar).ok_or_else(|| PreauthError::new("json-unicode"))?);
                        }
                        _ => return Err(PreauthError::new("json-escape")),
                    }
                }
                _ => {
                    self.at -= 1;
                    let rest = std::str::from_utf8(&self.bytes[self.at..]).map_err(|_| PreauthError::new("json-utf8"))?;
                    let character = rest.chars().next().ok_or_else(|| PreauthError::new("json-truncated"))?;
                    output.push(character); self.at += character.len_utf8();
                }
            }
            if output.len() > MAX_STRING { return Err(PreauthError::new("json-string-limit")); }
        }
        Ok(output)
    }
    fn hex4(&mut self) -> Result<u32> {
        let mut value = 0_u32;
        for _ in 0..4 {
            value = value * 16 + match self.bump()? { b'0'..=b'9' => u32::from(self.bytes[self.at-1]-b'0'), b'a'..=b'f' => u32::from(self.bytes[self.at-1]-b'a'+10), b'A'..=b'F' => u32::from(self.bytes[self.at-1]-b'A'+10), _ => return Err(PreauthError::new("json-unicode")) };
        }
        Ok(value)
    }
    fn array_value(&mut self, depth: usize) -> Result<Json> {
        self.at += 1; self.ws(); let mut values = Vec::new();
        if self.bytes.get(self.at) == Some(&b']') { self.at += 1; return Ok(Json::Array(values)); }
        loop {
            values.push(self.value(depth)?);
            if values.len() > MAX_CONTAINER_ITEMS { return Err(PreauthError::new("json-array-items")); }
            self.ws();
            match self.bump()? { b',' => self.ws(), b']' => break, _ => return Err(PreauthError::new("json-syntax")) }
        }
        Ok(Json::Array(values))
    }
    fn object_value(&mut self, depth: usize) -> Result<Json> {
        self.at += 1; self.ws(); let mut values = BTreeMap::new();
        if self.bytes.get(self.at) == Some(&b'}') { self.at += 1; return Ok(Json::Object(values)); }
        loop {
            if self.bytes.get(self.at) != Some(&b'"') { return Err(PreauthError::new("json-syntax")); }
            let key = self.string_value()?; self.ws(); if self.bump()? != b':' { return Err(PreauthError::new("json-syntax")); }
            self.ws(); let value = self.value(depth)?;
            if values.insert(key, value).is_some() { return Err(PreauthError::new("json-duplicate-key")); }
            if values.len() > MAX_CONTAINER_ITEMS { return Err(PreauthError::new("json-object-keys")); }
            self.ws(); match self.bump()? { b',' => self.ws(), b'}' => break, _ => return Err(PreauthError::new("json-syntax")) }
        }
        Ok(Json::Object(values))
    }
}
