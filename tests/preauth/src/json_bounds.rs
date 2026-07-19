use super::preauth::Json;

// ADR 0022 freezes the JSON bounds: 1 MiB length, container depth 64, 4,096 object keys,
// 4,096 array items, and 64 KiB strings. Each limit is accepted exactly at the bound and
// rejected one past it by the production parser.

#[test]
fn json_byte_limit_is_exactly_one_mebibyte() {
    let mut document = b"0".to_vec();
    document.resize(1024 * 1024, b' ');
    Json::parse(&document).expect("document at the byte limit");
    document.push(b' ');
    assert_eq!(Json::parse(&document).unwrap_err().code, "json-byte-limit");
}

#[test]
fn json_container_depth_is_exactly_sixty_four() {
    let nested = |depth: usize| format!("{}0{}", "[".repeat(depth), "]".repeat(depth));
    let at_limit = Json::parse(nested(64).as_bytes()).expect("depth at the limit");
    let mut level = &at_limit;
    let mut counted = 0usize;
    while let Ok(children) = level.array() {
        counted += 1;
        match children.first() { Some(child) => level = child, None => break }
    }
    assert_eq!(counted, 64);
    assert_eq!(Json::parse(nested(65).as_bytes()).unwrap_err().code, "json-depth-limit");
}

#[test]
fn json_object_keys_are_bounded_at_four_thousand_ninety_six() {
    let keys = |count: usize| {
        let pairs: Vec<String> = (0..count).map(|index| format!("\"k{index:04}\":0")).collect();
        format!("{{{}}}", pairs.join(","))
    };
    assert_eq!(Json::parse(keys(4096).as_bytes()).expect("keys at the limit").object().unwrap().len(), 4096);
    assert_eq!(Json::parse(keys(4097).as_bytes()).unwrap_err().code, "json-object-keys");
}

#[test]
fn json_array_items_are_bounded_at_four_thousand_ninety_six() {
    let items = |count: usize| format!("[{}]", vec!["0"; count].join(","));
    assert_eq!(Json::parse(items(4096).as_bytes()).expect("items at the limit").array().unwrap().len(), 4096);
    assert_eq!(Json::parse(items(4097).as_bytes()).unwrap_err().code, "json-array-items");
}

#[test]
fn json_strings_are_bounded_at_sixty_four_kibibytes() {
    let text = |length: usize| format!("\"{}\"", "a".repeat(length));
    assert_eq!(Json::parse(text(65536).as_bytes()).expect("string at the limit").string().unwrap().len(), 65536);
    assert_eq!(Json::parse(text(65537).as_bytes()).unwrap_err().code, "json-string-limit");
}
