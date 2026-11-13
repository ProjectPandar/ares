//! `Fill.cpp:1528–1537` standalone fills; `ToolOrdering.cpp:144–164` selectors.

use crate::{
    OrcaRunner,
    runner::native_api_cli::{NativeCase, sha},
};

#[test]
fn standalone_thin_gap_fill_uses_wall_before_sparse() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let project =
        include_bytes!("../../../../tests/parity/standalone-thin-gap-fill/frame-and-block.3mf");
    assert_eq!(
        sha(project),
        "e5f0c53c4d7a864dbe406f52fb05965b27f098dc916628f9eb673a3615b79e35"
    );
    let case = NativeCase::slice(&runner, project, "standalone-thin-gap-fill-");
    for (key, value) in [
        ("wall_generator", "classic"),
        ("outer_wall_filament_id", "1"),
        ("inner_wall_filament_id", "1"),
        ("internal_solid_filament_id", "1"),
        ("top_surface_filament_id", "1"),
        ("bottom_surface_filament_id", "1"),
        ("sparse_infill_filament_id", "2"),
        ("sparse_infill_density", "15%"),
        ("disable_m73", "0"),
    ] {
        assert!(case.native.contains(&format!("; {key} = {value}\n")));
    }
    let mut tool = None;
    let mut gaps = 0;
    let mut sparse = 0;
    for line in case.native.lines() {
        if let Some(id) = line.strip_prefix('T').and_then(|id| id.parse::<u32>().ok()) {
            tool = Some(id);
        } else if line == ";TYPE:Gap infill" {
            assert_eq!(tool, Some(0));
            gaps += 1;
        } else if line == ";TYPE:Sparse infill" {
            assert_eq!(tool, Some(1));
            sparse += 1;
        }
    }
    assert!(gaps > 0 && sparse > 0);
    assert!(case.native.find(";TYPE:Gap infill\n").unwrap() < case.native.find("\nT1\n").unwrap());
    case.compare(ares_core::GenerationMetadata::deterministic(
        2026, 10, 20, 0, 0, 0,
    ));
}
