//! "Arrange all" around a bed-exclude volume (the A2b1 section of
//! `docs/architecture/web-plater-arrange-imex.md`): `PartPlateList::preprocess_exclude_areas`'s
//! collision-volume blockers (`PartPlate.cpp:7725-7761` at v2.5.0-beta `345efec64`), which the CLI
//! global arrange adds as well (`OrcaSlicer.cpp:5727`). Beside the prime tower, the A2b2 section:
//! the plate's tower estimate counts the instances `PartPlate::check_outside` keeps inside
//! (`PartPlate.cpp:4562-4612`). In these cases no instance starts in an exclusion, so the GUI path
//! agrees with the CLI's `get_extruders_under_cli` count (`OrcaSlicer.cpp:5735`).

use super::wipe_tower::{
    NO_TOWER_BRIM, PRIME_VOLUME_PURGE, RECTANGLE_TOWER, TWO_FILAMENTS, TWO_FILAMENTS_CORNER,
};
use super::{CUBES, Edit, PROJECT, assert_arranged_like_native, settings};

/// The two-filament cubes with the 20 x 11.25 mm rectangle tower at (1, 1) (`wipe_tower.rs`).
const CORNER_TOWER: [Edit; 6] = [
    NO_TOWER_BRIM,
    RECTANGLE_TOWER[0],
    RECTANGLE_TOWER[1],
    PRIME_VOLUME_PURGE,
    TWO_FILAMENTS_CORNER[0],
    TWO_FILAMENTS_CORNER[1],
];

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

#[test]
fn plater_arrange_keeps_clear_of_the_prime_tower_and_a_bed_exclude_area_like_native() {
    let right_strip = (
        PROJECT,
        "\"bed_exclude_area\": [\"0x0\"]",
        "\"bed_exclude_area\": [\"150x0\", \"220x0\", \"220x220\", \"150x220\"]",
    );
    assert_arranged_like_native(
        "tower-bed-exclude-area",
        TWO_FILAMENTS,
        &[CORNER_TOWER.as_slice(), &[right_strip]].concat(),
        settings(false, false),
    );
}

#[test]
fn plater_arrange_keeps_clear_of_the_prime_tower_and_a_bed_exclude_volume_like_native() {
    let right_volume = (
        PROJECT,
        "{\"absolute_correction\"",
        "{\"bed_exclude_volumes\": \"0..250;150x0,220x0,220x220,150x220\", \"absolute_correction\"",
    );
    assert_arranged_like_native(
        "tower-bed-exclude-volume",
        TWO_FILAMENTS,
        &[CORNER_TOWER.as_slice(), &[right_volume]].concat(),
        settings(false, false),
    );
}
