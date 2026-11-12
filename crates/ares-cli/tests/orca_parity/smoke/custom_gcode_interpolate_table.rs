//! `interpolate_table` evaluation (`InterpolateTableContext::evaluate`,
//! `PlaceholderParser.cpp:1971-2009`, v2.5.0-beta `345efec64`)
//! (`docs/architecture/placeholder-interpolate-table.md`).
//!
//! - `mk4-interpolate-table-range.3mf`: MK4 with the tower off and the stock
//!   `M201 X{interpolate_table(extruded_weight_total, …)}` line in
//!   `layer_change_gcode`. The first layer change follows the first retract,
//!   so `extruded_weight_total` is below the table (`Extruder.cpp:171-180`)
//!   and upstream rejects the template ("Interpolation value is outside the
//!   table range", `:1999-2008`).
//! - `mk4base-no-tower.3mf`: the same template in `before_layer_change_gcode`,
//!   where every value is inside the table; its `M201` lines must equal
//!   Native's.

#![cfg(target_os = "linux")]

use super::custom_gcode_unknown_function::assert_rejection_matches_native;
use crate::runner::{OrcaRunner, repo_root};

#[test]
fn interpolate_table_outside_range_rejects_like_native() {
    assert_rejection_matches_native(
        "mk4-interpolate-table-range",
        "layer_change_gcode Parsing error at line 3: Interpolation value is outside the table range",
    );
}

#[test]
fn interpolate_table_inside_range_matches_native() {
    let runner = OrcaRunner::from_env()
        .unwrap()
        .expect("official v2.5.0 slicing oracle");
    let project =
        std::fs::read(repo_root().join("tests/parity/mk4-layer-change/mk4base-no-tower.3mf"))
            .unwrap();
    let exported = runner
        .fixture_case("custom-gcode-interpolate-table/inside-range", project)
        .unwrap();
    let sliced = runner.slice_case(&exported).unwrap();
    let ares = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(crate::runner::application::slice_project(
            &sliced.project,
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            ares_core::SlicerContext::new(0),
        ))
        .unwrap();
    let m201 = |gcode: &[u8]| {
        String::from_utf8_lossy(gcode)
            .lines()
            .filter(|line| line.starts_with("M201 "))
            .map(str::to_owned)
            .collect::<Vec<_>>()
    };
    let native = m201(&sliced.reference);
    assert!(native.len() > 2, "the template runs on every layer");
    assert_eq!(m201(&ares), native, "M201 lines");
}
