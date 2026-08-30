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

#[cfg(test)]
mod tests {
    #[test]
    fn split_key_set_preserves_boundary_membership() {
        assert!(super::contains("accel_to_decel_enable"));
        assert!(super::contains("machine_unload_filament_time"));
        assert!(super::contains("make_overhang_printable"));
        assert!(super::contains("z_hop"));
        assert!(!super::contains("not_an_orca_key"));
    }
}
