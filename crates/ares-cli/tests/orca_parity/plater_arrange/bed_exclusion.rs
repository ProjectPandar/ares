//! "Arrange all" around a bed-exclude volume (the A2b1 section of
//! `docs/architecture/web-plater-arrange-imex.md`): `PartPlateList::preprocess_exclude_areas`'s
//! collision-volume blockers (`PartPlate.cpp:7725-7761` at v2.5.0-beta `345efec64`), which the CLI
//! global arrange adds as well (`OrcaSlicer.cpp:5727`).

use super::{CUBES, Edit, PROJECT, assert_arranged_like_native, settings};

/// A full-height volume over the left half of the 220 mm bed.
const LEFT_HALF_VOLUME: Edit = (
    PROJECT,
    "{\n  \"accel_to_decel_enable\"",
    "{\n  \"bed_exclude_volumes\": \"0..250;0x0,110x0,110x220,0x220\",\n  \"accel_to_decel_enable\"",
);

#[test]
fn plater_arrange_keeps_clear_of_a_bed_exclude_volume_like_native() {
    assert_arranged_like_native(
        "bed-exclude-volume",
        CUBES,
        &[LEFT_HALF_VOLUME],
        settings(false, false),
    );
}
