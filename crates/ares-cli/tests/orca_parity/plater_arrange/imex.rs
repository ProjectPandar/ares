//! The plater session's arrange and outside check on an IDEX plate (the A2c2 section of
//! `docs/architecture/web-plater-arrange-imex.md`): `ArrangeJob::prepare_imex_zones`
//! (`ArrangeJob.cpp:605-629` at v2.5.0-beta `345efec64`) with the `ImexArranger` around the kernel
//! (`:743-746, 773-776`), and `PartPlate::check_outside`'s IMEX branch (`PartPlate.cpp:4615-4621`).
//! GUI-only: the CLI arrange never builds an `ImexArranger`, and `ensure_imex_zones` returns
//! without a plater (`:828-839`), so there is no Native comparison. The expectations come from the
//! zone layout (A2c1, `test_imex_zones.cpp`) and the arranger (A2a, `test_imex_arrange.cpp`).
//!
//! The Ender-3's 220 mm bed becomes a 1 x 2 IDEX with one parallel mode, T0 Primary and T1
//! Mirror, and 5 mm carriage clearances. Its layout (`compute_imex_zone_layout`) puts the primary
//! zone over X 0..110, the mirror zone over X 110..220, and the collision strip over X 105..110.

use super::wipe_tower::TWO_FILAMENTS;
use super::{Edit, OBJECTS, PROJECT, SINGLE, rewrite_archive, settings};
use crate::runner::repo_root;
use ares_core::{ArrangeScope, PlaterSession};

/// The IDEX printer keys, set first in the project config.
const IDEX: &str = "{\"is_imex\": \"1\", \"imex_gantry_count\": \"1\", \"imex_tools_per_gantry\": \"2\", \"imex_nozzle_clearance_x\": \"5\", \"imex_nozzle_clearance_y\": \"5\", \"imex_mode_names\": [\"primary\", \"parallel\"], \"imex_mode_active_tools\": [\"\", \"0:P,1:M\"],";
/// The process preset's mode (`imex_parallel_mode`, `PrintConfig.cpp:7121-7125`).
const PROCESS_MODE: &str = "\"imex_parallel_mode\": \"parallel\",";
/// The first plate's own mode, as `bbs_3mf.cpp:8496-8498` writes it.
const PLATE_MODE: Edit = (
    OBJECTS,
    "<metadata key=\"plater_id\" value=\"1\"/>",
    "<metadata key=\"plater_id\" value=\"1\"/>\n    <metadata key=\"imex_parallel_mode\" value=\"parallel\"/>",
);
/// The collision strip's left edge: the primary zone less the 5 mm clearance.
const STRIP_MIN_X: f64 = 105.0;

/// `base` with the IDEX keys and `keys` set first in the project config, then `edits`.
fn idex_session(base: &str, keys: &str, edits: &[Edit]) -> PlaterSession {
    let input = rewrite_archive(
        &std::fs::read(repo_root().join(base)).unwrap(),
        |name, text| {
            let mut edited =
                (name == PROJECT).then(|| text.replacen('{', &format!("{IDEX}{keys}"), 1));
            for (entry, from, to) in edits.iter().filter(|(entry, ..)| *entry == name) {
                let current = edited.as_deref().unwrap_or(text);
                assert!(current.contains(from), "{base}: {entry} has no {from:?}");
                edited = Some(current.replace(from, to));
            }
            edited
        },
    );
    PlaterSession::load(input).unwrap()
}

/// "Arrange all", then the 10 mm cube's X range: it must lie in the primary zone, clear of the
/// strip, and stay on the first plate.
fn assert_arranged_in_primary_zone(mut session: PlaterSession) {
    session
        .arrange(ArrangeScope::All, settings(false, false))
        .unwrap();
    let x = session.instance_transform(0, 0).unwrap().to_column_major()[12];
    let (min_x, max_x) = (x - 5.0, x + 5.0);
    assert!(
        (0.0..=STRIP_MIN_X).contains(&min_x) && max_x <= STRIP_MIN_X,
        "the cube spans X {min_x}..{max_x}, outside the primary zone left of the strip"
    );
    assert!(session.plates()[0].instances.contains(&(0, 0)));
}

#[test]
fn plater_arrange_keeps_an_object_in_the_plate_imex_mode_primary_zone() {
    // The plate's own mode (`PartPlate::get_imex_mode`, `PartPlate.cpp:457-465`); the process
    // preset stays on Primary. A plate the arrange adds takes the process preset's mode
    // (`ImexArrangeInput::read_config`, `IMEXArrange.cpp:75-90`), so the beds do not share the
    // zone: the arranger fences the rest of the bed and `finish` packs the plate again inside it.
    assert_arranged_in_primary_zone(idex_session(SINGLE, "", &[PLATE_MODE]));
}

#[test]
fn plater_arrange_keeps_an_object_in_the_process_imex_mode_primary_zone() {
    // The plate stays on Primary, so the process preset's mode applies
    // (`get_effective_imex_mode`, `PartPlate.cpp:467-485`, and the layout's own fallback,
    // `IMEXZones.cpp:28-31`). Every bed then shares the zone, which becomes the bed shape.
    assert_arranged_in_primary_zone(idex_session(SINGLE, PROCESS_MODE, &[]));
}

/// The two-filament cubes (`wipe_tower.rs`) with the filament-2 cube moved into the mirror zone and
/// the tower stored at the bed's back edge, where `reload_scene` re-clamps it by the depth the
/// plate's estimate gives it (`shown_towers`).
const MIRROR_ZONE_CUBE: [Edit; 2] = [
    (
        "3D/3dmodel.model",
        "transform=\"1 0 0 0 1 0 0 0 1 110 104 5\"",
        "transform=\"1 0 0 0 1 0 0 0 1 165 104 5\"",
    ),
    (
        PROJECT,
        "\"wipe_tower_y\": [\"150\"]",
        "\"wipe_tower_y\": [\"215\"]",
    ),
];
/// The moved cube on filament 1 instead.
const ON_FILAMENT_ONE: Edit = (
    OBJECTS,
    "<object id=\"4\">\n    <metadata key=\"name\" value=\"cube10.stl\"/>\n    <metadata key=\"extruder\" value=\"2\"/>",
    "<object id=\"4\">\n    <metadata key=\"name\" value=\"cube10.stl\"/>\n    <metadata key=\"extruder\" value=\"1\"/>",
);

/// "Arrange all", then the stored tower positions after the re-clamp.
fn arranged_tower_positions(mut session: PlaterSession) -> (Vec<f64>, Vec<f64>) {
    session
        .arrange(ArrangeScope::All, settings(false, false))
        .unwrap();
    session.tower_positions()
}

#[test]
fn plater_tower_estimate_leaves_out_an_object_in_the_imex_mirror_zone() {
    // The plate's estimate counts the filaments of the instances `contain_instance_totally` keeps
    // (`PartPlate.cpp:4494-4521`). The filament-2 cube meets the mirror zone, a secondary zone box,
    // so `check_outside` puts it outside (`imex_hull_violates`, `:1174-1178`): the plate sizes the
    // tower for filament 1 alone, as when that cube prints with filament 1.
    let mirror = [MIRROR_ZONE_CUBE.as_slice(), &[PLATE_MODE]].concat();
    let one_filament = [mirror.as_slice(), &[ON_FILAMENT_ONE]].concat();
    let reference = arranged_tower_positions(idex_session(TWO_FILAMENTS, "", &one_filament));
    // On Primary the plate has no zones and counts both filaments, which re-clamps the tower
    // elsewhere: the positions tell the two estimates apart.
    let primary = arranged_tower_positions(idex_session(TWO_FILAMENTS, "", &MIRROR_ZONE_CUBE));
    assert_ne!(
        primary, reference,
        "the two estimates re-clamp the tower alike"
    );
    assert_eq!(
        arranged_tower_positions(idex_session(TWO_FILAMENTS, "", &mirror)),
        reference
    );
}
