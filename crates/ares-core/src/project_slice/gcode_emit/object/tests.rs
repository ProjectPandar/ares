use super::encode_base64;

#[test]
fn sole_label_uses_orca_little_endian_bitset_encoding() {
    assert_eq!(encode_base64(1_u64.to_le_bytes()), *b"AQAAAAAAAAA=");
}
