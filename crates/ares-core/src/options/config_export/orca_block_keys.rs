//! Canonical OrcaSlicer 2.4.2 CONFIG_BLOCK key membership.

mod first;
mod second;

pub(crate) fn contains(key: &str) -> bool {
    let keys = if key < second::KEYS[0] {
        first::KEYS
    } else {
        second::KEYS
    };
    keys.binary_search(&key).is_ok()
}
