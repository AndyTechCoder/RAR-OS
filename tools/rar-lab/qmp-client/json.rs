// RAR-owned bounded JSON parser for the host-only QMP client.
// It intentionally implements only JSON, not JSON5 or a permissive superset.

#[derive(Clone, Debug, PartialEq)]
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
            Value::Object(entries) => entries.iter().find(|(key, _)| key == name).map(|(_, value)| value),
            _ => None,
        }
    }

    pub fn string(&self) -> Option<&str> {
        match self {
            Value::String(value) => Some(value),
            _ => None,
        }
    }

    pub fn is_object(&self) -> bool { matches!(self, Value::Object(_)) }

    pub fn is_number(&self) -> bool { matches!(self, Value::Number(_)) }

    pub fn is_string_array(&self) -> bool {
        matches!(self, Value::Array(values) if values.iter().all(|value| matches!(value, Value::String(_))))
    }

    pub fn is_empty_object(&self) -> bool {
        matches!(self, Value::Object(entries) if entries.is_empty())
    }
}

pub fn parse(input: &[u8]) -> Result<Value, &'static str> {
    if input.len() > 65_536 {
        return Err("JSON message exceeds bound");
    }
    let mut parser = Parser { input, offset: 0 };
    let value = parser.value(0)?;
    parser.space();
    if parser.offset != input.len() {
        return Err("trailing JSON data");
    }
    Ok(value)
}

struct Parser<'a> {
    input: &'a [u8],
    offset: usize,
}

impl Parser<'_> {
    fn value(&mut self, depth: usize) -> Result<Value, &'static str> {
        if depth > 32 {
            return Err("JSON nesting exceeds bound");
        }
        self.space();
        match self.peek() {
            Some(b'n') => { self.literal(b"null")?; Ok(Value::Null) }
            Some(b't') => { self.literal(b"true")?; Ok(Value::Bool(true)) }
            Some(b'f') => { self.literal(b"false")?; Ok(Value::Bool(false)) }
            Some(b'"') => Ok(Value::String(self.string()?)),
            Some(b'[') => self.array(depth + 1),
            Some(b'{') => self.object(depth + 1),
            Some(b'-' | b'0'..=b'9') => Ok(Value::Number(self.number()?)),
            _ => Err("invalid JSON value"),
        }
    }

    fn array(&mut self, depth: usize) -> Result<Value, &'static str> {
        self.take(b'[')?;
        self.space();
        let mut values = Vec::new();
        if self.peek() == Some(b']') {
            self.offset += 1;
            return Ok(Value::Array(values));
        }
        loop {
            if values.len() == 1024 {
                return Err("JSON array exceeds bound");
            }
            values.push(self.value(depth)?);
            self.space();
            match self.peek() {
                Some(b',') => { self.offset += 1; }
                Some(b']') => { self.offset += 1; break; }
                _ => return Err("invalid JSON array"),
            }
        }
        Ok(Value::Array(values))
    }

    fn object(&mut self, depth: usize) -> Result<Value, &'static str> {
        self.take(b'{')?;
        self.space();
        let mut entries: Vec<(String, Value)> = Vec::new();
        if self.peek() == Some(b'}') {
            self.offset += 1;
            return Ok(Value::Object(entries));
        }
        loop {
            if entries.len() == 1024 {
                return Err("JSON object exceeds bound");
            }
            self.space();
            let key = self.string()?;
            if entries.iter().any(|(existing, _)| existing == &key) {
                return Err("duplicate JSON object key");
            }
            self.space();
            self.take(b':')?;
            let value = self.value(depth)?;
            entries.push((key, value));
            self.space();
            match self.peek() {
                Some(b',') => { self.offset += 1; }
                Some(b'}') => { self.offset += 1; break; }
                _ => return Err("invalid JSON object"),
            }
        }
        Ok(Value::Object(entries))
    }

    fn string(&mut self) -> Result<String, &'static str> {
        self.take(b'"')?;
        let mut output = String::new();
        while let Some(byte) = self.peek() {
            self.offset += 1;
            match byte {
                b'"' => return Ok(output),
                0x00..=0x1f => return Err("control byte in JSON string"),
                b'\\' => {
                    let escape = self.peek().ok_or("truncated JSON escape")?;
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
                        _ => return Err("invalid JSON escape"),
                    }
                }
                0x20..=0x7f => output.push(byte as char),
                _ => return Err("non-ASCII JSON transport byte"),
            }
            if output.len() > 65_536 {
                return Err("JSON string exceeds bound");
            }
        }
        Err("unterminated JSON string")
    }

    fn unicode_escape(&mut self, output: &mut String) -> Result<(), &'static str> {
        let first = self.hex_quad()?;
        let code = if (0xd800..=0xdbff).contains(&first) {
            if self.peek() != Some(b'\\') || self.input.get(self.offset + 1) != Some(&b'u') {
                return Err("missing low surrogate");
            }
            self.offset += 2;
            let second = self.hex_quad()?;
            if !(0xdc00..=0xdfff).contains(&second) {
                return Err("invalid low surrogate");
            }
            0x10000 + (((first - 0xd800) as u32) << 10) + (second - 0xdc00) as u32
        } else if (0xdc00..=0xdfff).contains(&first) {
            return Err("unpaired low surrogate");
        } else {
            first as u32
        };
        output.push(char::from_u32(code).ok_or("invalid Unicode scalar")?);
        Ok(())
    }

    fn hex_quad(&mut self) -> Result<u16, &'static str> {
        let mut value = 0u16;
        for _ in 0..4 {
            let byte = self.peek().ok_or("truncated Unicode escape")?;
            self.offset += 1;
            value = value.checked_mul(16).ok_or("Unicode escape overflow")?;
            value += match byte {
                b'0'..=b'9' => (byte - b'0') as u16,
                b'a'..=b'f' => (byte - b'a' + 10) as u16,
                b'A'..=b'F' => (byte - b'A' + 10) as u16,
                _ => return Err("invalid Unicode escape"),
            };
        }
        Ok(value)
    }

    fn number(&mut self) -> Result<String, &'static str> {
        let start = self.offset;
        if self.peek() == Some(b'-') { self.offset += 1; }
        match self.peek() {
            Some(b'0') => { self.offset += 1; if matches!(self.peek(), Some(b'0'..=b'9')) { return Err("leading zero in JSON number"); } }
            Some(b'1'..=b'9') => { self.offset += 1; while matches!(self.peek(), Some(b'0'..=b'9')) { self.offset += 1; } }
            _ => return Err("invalid JSON number"),
        }
        if self.peek() == Some(b'.') {
            self.offset += 1;
            if !matches!(self.peek(), Some(b'0'..=b'9')) { return Err("invalid JSON fraction"); }
            while matches!(self.peek(), Some(b'0'..=b'9')) { self.offset += 1; }
        }
        if matches!(self.peek(), Some(b'e' | b'E')) {
            self.offset += 1;
            if matches!(self.peek(), Some(b'+' | b'-')) { self.offset += 1; }
            if !matches!(self.peek(), Some(b'0'..=b'9')) { return Err("invalid JSON exponent"); }
            while matches!(self.peek(), Some(b'0'..=b'9')) { self.offset += 1; }
        }
        String::from_utf8(self.input[start..self.offset].to_vec()).map_err(|_| "invalid number encoding")
    }

    fn literal(&mut self, expected: &[u8]) -> Result<(), &'static str> {
        if self.input.get(self.offset..self.offset + expected.len()) != Some(expected) {
            return Err("invalid JSON literal");
        }
        self.offset += expected.len();
        Ok(())
    }

    fn take(&mut self, expected: u8) -> Result<(), &'static str> {
        if self.peek() != Some(expected) { return Err("unexpected JSON byte"); }
        self.offset += 1;
        Ok(())
    }

    fn space(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t' | b'\r' | b'\n')) { self.offset += 1; }
    }

    fn peek(&self) -> Option<u8> { self.input.get(self.offset).copied() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_qmp_shapes_and_unicode() {
        let value = parse(br#"{"QMP":{"capabilities":[]},"text":"\uD83D\uDE80"}"#).unwrap();
        assert!(value.member("QMP").is_some());
        assert_eq!(value.member("text").and_then(Value::string), Some("🚀"));
    }

    #[test]
    fn rejects_duplicates_trailing_and_depth() {
        assert!(parse(br#"{"id":"a","id":"b"}"#).is_err());
        assert!(parse(br#"{}{}"#).is_err());
        let deep = format!("{}0{}", "[".repeat(34), "]".repeat(34));
        assert!(parse(deep.as_bytes()).is_err());
    }

    #[test]
    fn rejects_invalid_numbers_and_surrogates() {
        for input in ["01", "1.", "1e", "\"\\uDC00\"", "\"\\uD800x\""] {
            assert!(parse(input.as_bytes()).is_err(), "accepted {input}");
        }
    }
}
