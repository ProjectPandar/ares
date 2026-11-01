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
use super::{CUBES, Edit, PROJECT, assert_arranged_like_native, rewrite_archive, settings};
use crate::runner::{profile_entry, repo_root};
use ares_core::{ArrangeScope, PlaterSession, PresetCatalogHandle};

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

/// A Snapmaker J1 (two extruders, `filament_map` [1, 2] Manual): a 30 x 10 mm object on filament 1,
/// on a 324 x 200 mm bed.
const J1: &str = "tests/parity/closest-lines/single.3mf";
/// A Bambu Lab X2D (two nozzles, `filament_map_mode` Auto For Flush): one object on filament 1 on a
/// 256 mm bed.
const X2D: &str = "tests/parity/generated-printer-ids/input.3mf";
/// Both X2D nozzles reach the whole bed: the arrange of several nozzles' shared area is not ported,
/// and the vendor test reads only `printer_model`.
const X2D_ONE_AREA: Edit = (
    PROJECT,
    "\"extruder_printable_area\": [\n    \"0x0,256x0,256x256,0x256\",\n    \"20.5x0,256x0,256x256,20.5x256\"\n  ]",
    "\"extruder_printable_area\": []",
);

/// `base` in PerExtruder mode with a full-height volume over `(min, max)` for extruder 2 only.
fn extruder_two_volume(
    base: &str,
    (min, max): ([f64; 2], [f64; 2]),
    edits: &[Edit],
) -> PlaterSession {
    let volume = format!(
        "{{\"bed_exclude_volume_mode\": \"per_extruder\", \"extruder_bed_exclude_volumes\": [\"\", \"0..250;{0}x{1},{2}x{1},{2}x{3},{0}x{3}\"],",
        min[0], min[1], max[0], max[1]
    );
    let input = rewrite_archive(
        &std::fs::read(repo_root().join(base)).unwrap(),
        |name, text| {
            (name == PROJECT).then(|| {
                let mut text = text.replacen('{', &volume, 1);
                for (_, from, to) in edits {
                    assert!(text.contains(from), "{base} has no {from:?}");
                    text = text.replace(from, to);
                }
                text
            })
        },
    );
    PlaterSession::load(input).unwrap()
}

/// Arranges all, then whether the first instance moved (packed), with its offset.
fn arrange_packs_first_instance(session: &mut PlaterSession) -> (bool, [f64; 2]) {
    let offset = |session: &PlaterSession| {
        let matrix = session.instance_transform(0, 0).unwrap().to_column_major();
        [matrix[12], matrix[13]]
    };
    let before = offset(session);
    session
        .arrange(ArrangeScope::All, settings(false, false))
        .unwrap();
    let after = offset(session);
    (after != before, after)
}

// The extruder-2 volume covers the whole bed. Upstream's final alignment moves the pile clear of
// every fixed item, whatever its nozzle (`finalAlign`, `nfpplacer.hpp:1150-1176`), and returns
// where no position is clear (`:1166-1167`). So the nozzle ids show when a volume leaves no room:
// an object the volume does not apply to packs on the bed; one it applies to packs on no bed and
// keeps its place (`bed_idx < 0`, `ArrangeJob.cpp:791-799`, reported as unarranged `:933-940`).

#[test]
fn plater_arrange_lets_an_object_into_a_volume_only_another_nozzle_sees() {
    // A2b3, GUI-only (the CLI arrange leaves the nozzle ids empty), so no Native comparison. The
    // object prints with filament 1, which the manual map puts on extruder 1, so the arrange job
    // gives it the nozzle ids {0} (`ArrangeJob.cpp:53-81`). The extruder-2 volume then does not
    // apply to it (`bed_exclusion_applies`, `Arrange.cpp:131-133`).
    let whole_bed = ([-10.0, -10.0], [334.0, 210.0]);
    let (packed, xy) = arrange_packs_first_instance(&mut extruder_two_volume(J1, whole_bed, &[]));
    assert!(
        packed,
        "the object stayed unarranged at {xy:?} under the extruder-2 volume"
    );
}

#[test]
fn plater_arrange_keeps_an_installed_bbl_printer_out_of_every_nozzle_volume() {
    // With BBL installed the X2D's vendor type is Marlin_BBL (`get_current_vendor_type`,
    // `PresetBundle.cpp:992-1024`). Under an automatic filament map the arrange job leaves the ids
    // empty (`ArrangeJob.cpp:73-75`), so every nozzle's volume applies and no bed takes the object.
    let whole_bed = ([-10.0, -10.0], [266.0, 266.0]);
    let mut session = extruder_two_volume(X2D, whole_bed, &[X2D_ONE_AREA]);
    let profiles = repo_root().join("OrcaSlicer/resources/profiles");
    let mut installed = PresetCatalogHandle::new();
    let bbl = profile_entry::vendor(&profiles, "BBL");
    installed
        .add_vendor(&serde_json::to_vec(&bbl).unwrap())
        .unwrap();
    session.set_installed_vendors(&installed);
    let (packed, xy) = arrange_packs_first_instance(&mut session);
    assert!(
        !packed,
        "the BBL object packed to {xy:?} under the extruder-2 volume"
    );
}

#[test]
fn plater_arrange_reads_a_printer_of_no_installed_vendor_as_generic() {
    // No vendor installed: the X2D's vendor type is Unknown. A generic automatic map takes the
    // filament id as the tool id (`PrintConfig.cpp:14201-14202`), so filament 1 is on extruder 1, the
    // ids are {0}, and the object packs under the extruder-2 volume.
    let whole_bed = ([-10.0, -10.0], [266.0, 266.0]);
    let mut session = extruder_two_volume(X2D, whole_bed, &[X2D_ONE_AREA]);
    let (packed, xy) = arrange_packs_first_instance(&mut session);
    assert!(
        packed,
        "the object stayed unarranged at {xy:?} under the extruder-2 volume"
    );
}
