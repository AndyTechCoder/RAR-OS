//! Bounded RAR-owned JSON parser for OCI metadata documents.

pub const MAX_JSON_BYTES: usize = 1_048_576;
const MAX_JSON_DEPTH: usize = 32;
const MAX_JSON_CONTAINER_ITEMS: usize = 4_096;
const MAX_JSON_STRING_BYTES: usize = 262_144;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Number(String),
    String(String),
    Array(Vec<Value>),
    Object(Vec<(String, Value)>),
}

impl Value {
    pub fn member(&self, name: &str) -> Option<&Value> {
        match self {
            Value::Object(entries) => entries
                .iter()
                .find(|(key, _)| key == name)
                .map(|(_, value)| value),
            _ => None,
        }
    }

    pub fn string(&self) -> Option<&str> {
        match self {
            Value::String(value) => Some(value),
            _ => None,
        }
    }

    pub fn array(&self) -> Option<&[Value]> {
        match self {
            Value::Array(values) => Some(values),
            _ => None,
        }
    }

    pub fn unsigned_integer(&self) -> Option<u64> {
        match self {
            Value::Number(value)
                if !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit()) =>
            {
                value.parse().ok()
            }
            _ => None,
        }
    }
}

pub fn parse(input: &[u8]) -> Result<Value, Error> {
    if input.len() > MAX_JSON_BYTES {
        return Err(Error::DocumentTooLarge);
    }
    let mut parser = Parser { input, offset: 0 };
    let value = parser.value(0)?;
    parser.space();
    if parser.offset != input.len() {
        return Err(Error::TrailingData);
    }
    Ok(value)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    DocumentTooLarge,
    NestingTooDeep,
    ContainerTooLarge,
    StringTooLarge,
    InvalidValue,
    InvalidLiteral,
    InvalidArray,
    InvalidObject,
    DuplicateKey,
    InvalidString,
    InvalidEscape,
    InvalidUnicode,
    InvalidUtf8,
    InvalidNumber,
    TrailingData,
    UnexpectedEnd,
}

struct Parser<'a> {
    input: &'a [u8],
    offset: usize,
}

impl Parser<'_> {
    fn value(&mut self, depth: usize) -> Result<Value, Error> {
        if depth > MAX_JSON_DEPTH {
            return Err(Error::NestingTooDeep);
        }
        self.space();
        match self.peek() {
            Some(b'n') => {
                self.literal(b"null")?;
                Ok(Value::Null)
            }
            Some(b't') => {
                self.literal(b"true")?;
                Ok(Value::Bool(true))
            }
            Some(b'f') => {
                self.literal(b"false")?;
                Ok(Value::Bool(false))
            }
            Some(b'"') => Ok(Value::String(self.string()?)),
            Some(b'[') => self.array(depth + 1),
            Some(b'{') => self.object(depth + 1),
            Some(b'-' | b'0'..=b'9') => Ok(Value::Number(self.number()?)),
            None => Err(Error::UnexpectedEnd),
            _ => Err(Error::InvalidValue),
        }
    }

    fn array(&mut self, depth: usize) -> Result<Value, Error> {
        self.take(b'[')?;
        self.space();
        let mut values = Vec::new();
        if self.peek() == Some(b']') {
            self.offset += 1;
            return Ok(Value::Array(values));
        }
        loop {
            if values.len() >= MAX_JSON_CONTAINER_ITEMS {
                return Err(Error::ContainerTooLarge);
            }
            values.push(self.value(depth)?);
            self.space();
            match self.peek() {
                Some(b',') => self.offset += 1,
                Some(b']') => {
                    self.offset += 1;
                    break;
                }
                _ => return Err(Error::InvalidArray),
            }
        }
        Ok(Value::Array(values))
    }

    fn object(&mut self, depth: usize) -> Result<Value, Error> {
        self.take(b'{')?;
        self.space();
        let mut entries: Vec<(String, Value)> = Vec::new();
        if self.peek() == Some(b'}') {
            self.offset += 1;
            return Ok(Value::Object(entries));
        }
        loop {
            if entries.len() >= MAX_JSON_CONTAINER_ITEMS {
                return Err(Error::ContainerTooLarge);
            }
            self.space();
            let key = self.string()?;
            if entries.iter().any(|(existing, _)| existing == &key) {
                return Err(Error::DuplicateKey);
            }
            self.space();
            self.take(b':')?;
            let value = self.value(depth)?;
            entries.push((key, value));
            self.space();
            match self.peek() {
                Some(b',') => self.offset += 1,
                Some(b'}') => {
                    self.offset += 1;
                    break;
                }
                _ => return Err(Error::InvalidObject),
            }
        }
        Ok(Value::Object(entries))
    }

    fn string(&mut self) -> Result<String, Error> {
        self.take(b'"')?;
        let mut output = String::new();
        loop {
            let byte = self.peek().ok_or(Error::UnexpectedEnd)?;
            self.offset += 1;
            match byte {
                b'"' => return Ok(output),
                0x00..=0x1f => return Err(Error::InvalidString),
                b'\\' => {
                    let escape = self.peek().ok_or(Error::UnexpectedEnd)?;
                    self.offset += 1;
                    match escape {
                        b'"' => output.push('"'),
                        b'\\' => output.push('\\'),
                        b'/' => output.push('/'),
                        b'b' => output.push('\u{0008}'),
                        b'f' => output.push('\u{000c}'),
                        b'n' => output.push('\n'),
                        b'r' => output.push('\r'),
                        b't' => output.push('\t'),
                        b'u' => self.unicode_escape(&mut output)?,
                        _ => return Err(Error::InvalidEscape),
                    }
                }
                0x20..=0x7f => output.push(byte as char),
                _ => {
                    let start = self.offset - 1;
                    let remaining = std::str::from_utf8(&self.input[start..])
                        .map_err(|_| Error::InvalidUtf8)?;
                    let character = remaining.chars().next().ok_or(Error::InvalidUtf8)?;
                    output.push(character);
                    self.offset = start
                        .checked_add(character.len_utf8())
                        .ok_or(Error::InvalidUtf8)?;
                }
            }
            if output.len() > MAX_JSON_STRING_BYTES {
                return Err(Error::StringTooLarge);
            }
        }
    }

    fn unicode_escape(&mut self, output: &mut String) -> Result<(), Error> {
        let first = self.hex_quad()?;
        let code = if (0xd800..=0xdbff).contains(&first) {
            if self.peek() != Some(b'\\') || self.input.get(self.offset + 1) != Some(&b'u') {
                return Err(Error::InvalidUnicode);
            }
            self.offset += 2;
            let second = self.hex_quad()?;
            if !(0xdc00..=0xdfff).contains(&second) {
                return Err(Error::InvalidUnicode);
            }
            0x10000 + (((first - 0xd800) as u32) << 10) + u32::from(second - 0xdc00)
        } else if (0xdc00..=0xdfff).contains(&first) {
            return Err(Error::InvalidUnicode);
        } else {
            u32::from(first)
        };
        output.push(char::from_u32(code).ok_or(Error::InvalidUnicode)?);
        Ok(())
    }

    fn hex_quad(&mut self) -> Result<u16, Error> {
        let mut value = 0u16;
        for _ in 0..4 {
            let byte = self.peek().ok_or(Error::UnexpectedEnd)?;
            self.offset += 1;
            value = value.checked_mul(16).ok_or(Error::InvalidUnicode)?;
            value = value
                .checked_add(match byte {
                    b'0'..=b'9' => u16::from(byte - b'0'),
                    b'a'..=b'f' => u16::from(byte - b'a' + 10),
                    b'A'..=b'F' => u16::from(byte - b'A' + 10),
                    _ => return Err(Error::InvalidUnicode),
                })
                .ok_or(Error::InvalidUnicode)?;
        }
        Ok(value)
    }

    fn number(&mut self) -> Result<String, Error> {
        let start = self.offset;
        if self.peek() == Some(b'-') {
            self.offset += 1;
        }
        match self.peek() {
            Some(b'0') => {
                self.offset += 1;
                if matches!(self.peek(), Some(b'0'..=b'9')) {
                    return Err(Error::InvalidNumber);
                }
            }
            Some(b'1'..=b'9') => {
                self.offset += 1;
                while matches!(self.peek(), Some(b'0'..=b'9')) {
                    self.offset += 1;
                }
            }
            _ => return Err(Error::InvalidNumber),
        }
        if self.peek() == Some(b'.') {
            self.offset += 1;
            if !matches!(self.peek(), Some(b'0'..=b'9')) {
                return Err(Error::InvalidNumber);
            }
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.offset += 1;
            }
        }
        if matches!(self.peek(), Some(b'e' | b'E')) {
            self.offset += 1;
            if matches!(self.peek(), Some(b'+' | b'-')) {
                self.offset += 1;
            }
            if !matches!(self.peek(), Some(b'0'..=b'9')) {
                return Err(Error::InvalidNumber);
            }
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.offset += 1;
            }
        }
        String::from_utf8(self.input[start..self.offset].to_vec())
            .map_err(|_| Error::InvalidNumber)
    }

    fn literal(&mut self, expected: &[u8]) -> Result<(), Error> {
        if !self.input[self.offset..].starts_with(expected) {
            return Err(Error::InvalidLiteral);
        }
        self.offset += expected.len();
        Ok(())
    }

    fn take(&mut self, expected: u8) -> Result<(), Error> {
        if self.peek() != Some(expected) {
            return Err(Error::InvalidValue);
        }
        self.offset += 1;
        Ok(())
    }

    fn space(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t' | b'\r' | b'\n')) {
            self.offset += 1;
        }
    }

    fn peek(&self) -> Option<u8> {
        self.input.get(self.offset).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_utf8_unicode_escapes_and_unsigned_integers() {
        let value = parse("{\"name\":\"RAR 🚀 \\uD83D\\uDD10\",\"size\":42}".as_bytes())
            .unwrap();
        assert_eq!(value.member("name").and_then(Value::string), Some("RAR 🚀 🔐"));
        assert_eq!(value.member("size").and_then(Value::unsigned_integer), Some(42));
    }

    #[test]
    fn rejects_duplicates_invalid_numbers_and_trailing_data() {
        for input in [
            r#"{"a":1,"a":2}"#,
            "01",
            "1.",
            "1e",
            "{}{}",
            r#""\uDC00""#,
        ] {
            assert!(parse(input.as_bytes()).is_err(), "accepted {input}");
        }
    }
}
