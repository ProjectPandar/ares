//! The filament-to-extruder resolution for bed exclusions, ported from
//! `test_exclusion_volumes.cpp:486-498` through `bed_exclusion::bed_exclusion_extruder_for_filament`.
//! The case is GUI-only upstream (the arrange job's nozzle ids), so the unit test is the oracle.

use ares_core::ProjectFilamentMapMode::{AutoForMatch, Manual};
use ares_core::bed_exclusion::{ExclusionFilamentMap, bed_exclusion_extruder_for_filament};

fn map(
    configured: &[i32],
    mode: ares_core::ProjectFilamentMapMode,
    (is_bambu, automatic_map_resolved): (bool, bool),
) -> ExclusionFilamentMap<'_> {
    ExclusionFilamentMap {
        configured,
        mode,
        is_bambu,
        automatic_map_resolved,
    }
}

#[test]
fn filament_mapping_resolves_physical_exclusion_extruders_conservatively() {
    let extruder = bed_exclusion_extruder_for_filament;
    assert_eq!(extruder(0, &map(&[2, 1], Manual, (false, true)), 2), 1);
    assert_eq!(extruder(1, &map(&[2, 1], Manual, (false, true)), 2), 0);
    // Generic automatic multi-tool printers use matching logical and physical ids.
    assert_eq!(
        extruder(1, &map(&[1, 1], AutoForMatch, (false, false)), 2),
        1
    );
    // Until Bambu automatic grouping has produced a concrete mapping, every nozzle remains possible.
    assert_eq!(extruder(0, &map(&[1], AutoForMatch, (true, false)), 2), -1);
    assert_eq!(extruder(0, &map(&[2], AutoForMatch, (true, true)), 2), 1);
    assert_eq!(extruder(3, &map(&[], Manual, (false, true)), 2), -1);
}
