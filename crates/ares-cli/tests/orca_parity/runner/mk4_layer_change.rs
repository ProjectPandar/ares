//! `GCode.cpp:4631-4641` + `placeholder_parser_process`
//! (`GCode.cpp:3896-3930`): the layer-change templates read the
//! `PlaceholderParserIntegration::update_from_gcodewriter` totals
//! (`GCode.cpp:1663-1697`) — the sum of every registered extruder's
//! `Extruder::extruded_volume()` (`Extruder.cpp:125-135`)
//! tachometer. The MK4 profile's `before_layer_change_gcode` exposes
//! the divergence directly: `M201 X{interpolate_table(
//! extruded_weight_total, ...)}` and `M74 W[extruded_weight_total]`
//! diverge at layer 2 (native W0.297065 vs an active-extruder-only
//! W0.0809351), which then shifts the GCodeProcessor time model
//! (per-axis `M201` acceleration clamps) and moves the first
//! `M73 P32 R61` progress marker one tower-wipe extrusion segment
//! early (line 1816).

use crate::{
    OrcaRunner, golden,
    runner::native_api_cli::{NativeCase, sha},
};
use std::process::Command;

const MK4_BASE: &[u8] = include_bytes!("../../../../../tests/parity/mk4-layer-change/mk4base.3mf");
const MK4_BASE_SHA: &str = "d0a4dfdb7f89120719b380a2995687a81a7e699025b8f3734349480bdebdc046";

#[test]
fn mk4_layer_change_weight_total() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    assert_eq!(sha(MK4_BASE), MK4_BASE_SHA);
    let case = NativeCase::slice(&runner, MK4_BASE, "mk4-layer-change-");
    // Layer 2 pin: the totals template sees ALL extruders' tachometers
    // (native W0.297065), not just the active extruder (W0.0809351).
    assert!(case.native.contains("\nM74 W0.297065\n"));

    let api = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(ares_core::slice_project_plate_validated(
            &MK4_BASE,
            ares_core::GenerationMetadata::deterministic(2026, 10, 28, 0, 0, 0),
            None,
            &ares_core::ConfigOverrides::default(),
            ares_core::SlicerContext::new(0),
        ))
        .unwrap();
    std::fs::write(case.root.join("ares.api.gcode"), &api).unwrap();
    let cli_path = case.root.join("ares.cli.gcode");
    let cli = Command::new(env!("CARGO_BIN_EXE_ares"))
        .args(["slice", "-o"])
        .arg(&cli_path)
        .arg(case.root.join("input.3mf"))
        .output()
        .unwrap();
    assert!(cli.status.success(), "CLI at {:?}", case.root);
    let cli_bytes = std::fs::read(&cli_path).unwrap();
    assert_eq!(api, cli_bytes, "API/CLI raw equality");
    assert!(cli_bytes.windows(15).any(|w| w == b"\nM74 W0.297065\n"));

    golden::compare_ordered_bytes_generator_only(case.native.as_bytes(), &api)
        .unwrap_or_else(|error| panic!("API at {:?}: {error}", case.root));
}
