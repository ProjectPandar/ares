//! Loaded-config rejections where the filament only carries the machine through
//! validation, so it comes from the stock profiles at the upstream pin.
use serde_json::json;

use super::{OverrideCase, run_override_case};

/// First stock filament the vendor's profiles offer for `machine`.
fn stock_filament(vendor_name: &str, machine: &str) -> String {
    let profiles_root = super::super::super::repo_root().join("OrcaSlicer/resources/profiles");
    crate::presets::VendorProfiles::load(&profiles_root, vendor_name)
        .unwrap()
        .compatible_filament(machine)
        .unwrap()
}

/// `bridge_line_width` > min_nozzle_diameter — the separate
/// `Slic3r::validate` line-width bound (PrintConfig.cpp:10504-10517), not
/// the generic `is_value_valid` loop.
#[tokio::test]
async fn bridge_line_width_over_nozzle_loaded_config() {
    let machine = "Creality Ender-3 0.4 nozzle";
    run_override_case(&OverrideCase {
        vendor_name: "Creality",
        machine,
        process: "0.20mm Standard @Creality Ender3 0.4",
        filament: &stock_filament("Creality", machine),
        kind: "process",
        key: "bridge_line_width",
        value: json!("1.0"),
        error_lines:
            "bridge_line_width: Bridge line width must not exceed nozzle diameter: 1.000000",
        prefix: "o3-bridge-",
    })
    .await;
}
