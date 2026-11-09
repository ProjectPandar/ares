//! Extruder.cpp:244-252: toolchange retraction read from the overlaid filament column.

use super::check_filament_columns;
use serde_json::json;

const SOURCES: &[&str] = &[
    "Print.cpp:4726-4749 retract overlay",
    "Extruder.cpp:244-252 toolchange getters",
    "GCodeWriter.cpp:1468-1476 retract_for_toolchange",
];

#[test]
fn identity_map_toolchange_retract_overrides() {
    check_filament_columns(
        "identity-map-toolchange-retract-overrides",
        SOURCES,
        &[
            "; filament_map = 1,2",
            "; filament_retract_length_toolchange = 1,3",
            "; filament_retract_restart_extra_toolchange = 0.1,0.3",
        ],
        json!({
            "filament_retract_length_toolchange":["1","3"],
            "filament_retract_restart_extra_toolchange":["0.1","0.3"]
        }),
    );
}

#[test]
fn swapped_filament_map_toolchange_retract_overrides() {
    check_filament_columns(
        "swapped-filament-map-toolchange-retract-overrides",
        SOURCES,
        &[
            "; filament_map = 2,1",
            "; filament_retract_length_toolchange = 1,3",
            "; filament_retract_restart_extra_toolchange = 0.1,0.3",
        ],
        json!({
            "filament_map":["2","1"],
            "filament_retract_length_toolchange":["1","3"],
            "filament_retract_restart_extra_toolchange":["0.1","0.3"]
        }),
    );
}

/// The start template reads `retract_length_toolchange[0]` from the overlaid vector.
#[test]
fn swapped_machine_columns_toolchange_retract() {
    check_filament_columns(
        "swapped-machine-columns-toolchange-retract",
        SOURCES,
        &["; filament_map = 2,1", "; retract_length_toolchange = 1,3"],
        json!({
            "filament_map":["2","1"],
            "printer_extruder_variant":["Direct Drive Standard","Direct Drive Standard"],
            "printer_extruder_id":["1","2"],
            "retract_length_toolchange":["1","3"]
        }),
    );
}
