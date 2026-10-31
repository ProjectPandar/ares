//! Brim.cpp:504-510: AutoBrim consumes EFC-clipped per-group polygons.

use crate::efc_outline_witness_case::run;

const RAW_EFC: &[u8] = include_bytes!(
    "../../../../tests/parity/auto-brim-efc-clipped-groups/auto-efc0p3-outline0.3mf"
);

#[test]
fn auto_brim_consumes_efc_clipped_groups_active() {
    run(
        "auto-brim-efc-active-",
        include_bytes!(
            "../../../../tests/parity/auto-brim-efc-clipped-groups/auto-efc0p3-outline1.3mf"
        ),
        "bfe65cf39ea2c485c016aa6563e788679826ff17ae4fdfa68cd69b3d17617b93",
        Some((RAW_EFC, "auto-brim-efc-native-raw-")),
        |_| {},
    );
}

#[test]
fn auto_brim_consumes_efc_clipped_groups_zero_disabled_control() {
    run(
        "auto-brim-efc-zero-off-",
        include_bytes!(
            "../../../../tests/parity/auto-brim-efc-clipped-groups/auto-efc0-outline0.3mf"
        ),
        "1faebd4419d3310ad0d43c73a10bc7d85cda391b5c8b2faa128ec7e250833fd0",
        None,
        |_| {},
    );
}

#[test]
fn auto_brim_consumes_efc_clipped_groups_zero_enabled_control() {
    run(
        "auto-brim-efc-zero-on-",
        include_bytes!(
            "../../../../tests/parity/auto-brim-efc-clipped-groups/auto-efc0-outline1.3mf"
        ),
        "895dfab6a6438a91d475b1fb2fd8cecfbc64ac5dc672dad9b0bd88d12b2f07a0",
        None,
        |_| {},
    );
}

#[test]
fn auto_brim_consumes_efc_clipped_groups_flag_disabled_control() {
    run(
        "auto-brim-efc-flag-off-",
        RAW_EFC,
        "0e4066dedce3940dde14653f8c081b6c0cdd771b6f3b8e5f2ee851e12691ed61",
        None,
        |_| {},
    );
}
