//! `slowdown_for_curled_perimeters` parity case: the stock matrix C1
//! divergence (`stock/Creality/Creality Sermoon V1 0.4 nozzle` +
//! `cube10.stl`). Ares under-slowed the walls on the layer above the
//! elephant-foot-compensated first layer because `CurlTracker::collect`
//! scored curvature on open emitted-path fragments instead of the closed
//! loop entity upstream hands to `estimate_malformations`
//! (`ExtrusionEntity.hpp:507-512`). The Sermoon V1 preset already sets
//! `slowdown_for_curled_perimeters=1` and `elefant_foot_compensation=0.3`,
//! so the stock case exercises the curled-perimeter slowdown.
//!
//! KNOWN RESIDUAL (owned by the perimeter-vertex-order boundary, not
//! emit): the degenerate `i == 0` CurledLine lands on Ares' natural
//! loop-start vertex, which the oracle's polygon vertex order places at
//! a different vertex — seam-adjacent outer-wall segments on later
//! layers keep ~10% extra slowdown, plus E-ulp and M73-order noise. This
//! test pins the eliminated divergence precisely: the Z0.6 layer section
//! (the layer above the compensated first layer, whose walls emitted
//! ~10-40% too fast) must equal the oracle byte-for-byte modulo `M73`
//! progress lines, whose placement the seeded oracle itself shifts
//! between runs (see `runner::rerun_reference`).

use crate::{CaseInputs, OrcaRunner, VendorProfiles, runner};

fn layer_section<'a>(gcode: &'a str, z_marker: &str, end_marker: &str) -> Vec<&'a str> {
    let start = gcode
        .find(z_marker)
        .unwrap_or_else(|| panic!("missing {z_marker}"));
    let rest = &gcode[start..];
    let end = rest
        .find(end_marker)
        .unwrap_or_else(|| panic!("missing {end_marker}"));
    rest[..end]
        .lines()
        .filter(|line| !line.starts_with("M73 "))
        .collect()
}

#[test]
fn curled_perimeter_slowdown() {
    let Some(runner) = OrcaRunner::from_env().unwrap_or_else(|error| panic!("{error}")) else {
        return;
    };
    let profiles = VendorProfiles::load(
        &runner::repo_root().join("OrcaSlicer/resources/profiles"),
        "Creality",
    )
    .unwrap();
    let selection =
        crate::select_printer(&profiles, "Creality", "Creality Sermoon V1 0.4 nozzle").unwrap();
    let machine = profiles.machine(&selection.printer).unwrap();
    let process = profiles.process(&selection.process).unwrap();
    let filaments = selection
        .filaments
        .iter()
        .map(|name| profiles.filament(name))
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let exported = runner
        .export_case(
            &CaseInputs {
                label: "curled-perimeter/sermoon-v1",
                machine: &machine,
                process: &process,
                filaments: &filaments,
            },
            &serde_json::Map::new(),
            &runner::repo_root().join("tests/parity/cube10.stl"),
        )
        .unwrap();
    runner::application::verify_value(
        &exported.project,
        "slowdown_for_curled_perimeters",
        &serde_json::json!("1"),
    )
    .unwrap();
    let sliced = runner.slice_case(&exported).unwrap();
    let actual_bytes = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(ares_core::slice_project(
            &sliced.project,
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            ares_core::SlicerContext::new(0),
        ))
        .unwrap();
    let actual = std::str::from_utf8(&actual_bytes).unwrap();
    let reference = std::str::from_utf8(&sliced.reference).unwrap();
    // The Z0.6 layer sits above the `elefant_foot_compensation=0.3` first
    // layer: every wall segment's curled-perimeter slowdown must match
    // the oracle byte-for-byte (M73 progress placement aside).
    assert_eq!(
        layer_section(actual, ";Z:0.6", ";Z:0.8"),
        layer_section(reference, ";Z:0.6", ";Z:0.8"),
    );
}
