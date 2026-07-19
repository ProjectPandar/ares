use super::{closing::PostClosingPrintObject, task22g_oracle::encode_with_magic};

pub(super) fn encode(objects: &[PostClosingPrintObject]) -> Vec<u8> {
    encode_with_magic(objects, b"ARES22I\0")
}
