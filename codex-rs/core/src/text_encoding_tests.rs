use super::bytes_to_string_smart;

#[test]
fn bytes_to_string_smart_returns_valid_utf8_verbatim() {
    assert_eq!(bytes_to_string_smart("hello".as_bytes()), "hello");
}

#[test]
fn bytes_to_string_smart_decodes_windows_1252_punctuation() {
    let bytes = [0x93, b't', b'e', b's', b't', 0x94];

    assert_eq!(bytes_to_string_smart(&bytes), "“test”");
}

#[test]
fn bytes_to_string_smart_falls_back_for_invalid_utf8() {
    let bytes = [0xff, 0xfe, b'a'];

    assert!(bytes_to_string_smart(&bytes).contains('a'));
}
