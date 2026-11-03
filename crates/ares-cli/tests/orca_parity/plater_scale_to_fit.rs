//! "Scale to build volume" (`docs/architecture/web-plater-scale-to-fit.md`):
//! `Plater::priv::scale_selection_to_fit_print_volume` (`Plater.cpp:11092-11112` at v2.5.0-beta
//! `345efec64`) and `Selection::scale_to_fit_print_volume`'s rectangle fit (`Selection.cpp:1481-1532`).
//! Each object is first made 200 mm tall, so the fit is limited by the printable height and the
//! slices stay small. The expected transforms follow the upstream math from the fixture's values.

use ares_core::{PlaterSession, ScaleEdit, SliceError};

use super::super::super::export;
use super::super::{SINGLE, assert_plate_like_native, load, rewrite_archive};
use super::{SIBLINGS, assert_instance_transform, scaled_at};
use crate::runner::repo_root;

const PROJECT: &str = "Metadata/project_settings.config";
/// `BuildVolume::SceneEpsilon` (`BuildVolume.hpp:110`).
const EPS: f64 = 1e-4;
/// The tall cube's size before the fit: 10 x 10 x 200 mm.
const TALL: f64 = 200.0;
/// `fit_rectangle`'s padding: 1/100 mm on every side (`Selection.cpp:1513`).
const PADDING: f64 = 0.02;
const ENDER3_AREA: &str =
    "\"printable_area\": [\n\t\t\"0x0\",\n\t\t\"220x0\",\n\t\t\"220x220\",\n\t\t\"0x220\"\n\t]";
const SCALE_TO_FIT_CIRCLE: &str = "scale to build volume is not ported for a circular bed";

/// `base` with each `(entry, text, replacement)` applied.
fn load_edited(base: &str, edits: &[(&str, &str, &str)]) -> PlaterSession {
    let input = rewrite_archive(
        &std::fs::read(repo_root().join(base)).unwrap(),
        |name, text| {
            let mut edited = None::<String>;
            for (_, from, to) in edits.iter().filter(|(entry, ..)| *entry == name) {
                let current = edited.as_deref().unwrap_or(text);
                assert!(current.contains(from), "{name} has no {from:?}");
                edited = Some(current.replace(from, to));
            }
            edited
        },
    );
    PlaterSession::load(input).unwrap()
}

/// The ported Size Z edit with the uniform toggle off: the instance becomes 200 mm tall.
fn make_tall(session: &mut PlaterSession, (object, instance): (usize, usize)) {
    session
        .set_size(
            object,
            instance,
            ScaleEdit {
                axis: 2,
                value: TALL,
                uniform: false,
            },
        )
        .unwrap();
}

/// `fit`'s factor of the tall cube in a volume `height` mm high: the Z ratio of the bed box
/// (`get_build_volume`, `-eps..height + eps`) to the padded selection box.
fn z_limited_factor(height: f64) -> f64 {
    ((height + EPS) - -EPS) / (TALL + PADDING)
}

/// The tall cube fitted at `(x, y)`: scaled by `s` (Z by `20 s`), standing on the bed.
fn fitted_at(s: f64, [x, y]: [f64; 2]) -> [f64; 16] {
    scaled_at([s, s, 20.0 * s], [x, y, 0.5 * TALL * s])
}

#[test]
fn plater_scale_to_fit_fills_the_printable_height_on_the_plate_centre_like_native() {
    // The tall cube at (110, 110) on the 220 x 220 x 250 mm Ender-3 bed: the bed box's centre is
    // (110, 110), so it only grows; its bottom stays on the bed.
    let mut session = load(SINGLE);
    make_tall(&mut session, (0, 0));
    session.scale_to_fit(0, 0).unwrap();
    let s = z_limited_factor(250.0);
    assert_instance_transform(&session, (0, 0), fitted_at(s, [110.0, 110.0]));
    assert_plate_like_native("scale-to-fit-plain", &session, 1);
}

#[test]
fn plater_scale_to_fit_centres_in_the_first_free_volume_beside_a_bed_exclude_area_like_native() {
    // `bed_exclude_area` over X 150..220: `free_print_volumes` (`Plater.cpp:11049-11090`) makes the
    // edges X {-eps, 150, 220, 220 + eps} and Y {-eps, 0, 220, 220 + eps}. Y -eps..0 is a sliver,
    // so the first volume is X -eps..150, Y -eps..220; every volume gives the same Z-limited
    // factor, and only a strictly larger one replaces the first.
    let mut session = load_edited(
        SINGLE,
        &[(
            PROJECT,
            "\"bed_exclude_area\": [\n\t\t\"0x0\"\n\t]",
            "\"bed_exclude_area\": [\"150x0\", \"220x0\", \"220x220\", \"150x220\"]",
        )],
    );
    make_tall(&mut session, (0, 0));
    session.scale_to_fit(0, 0).unwrap();
    let s = z_limited_factor(250.0);
    let centre = [0.5 * (-EPS + 150.0), 0.5 * (-EPS + 220.0)];
    assert_instance_transform(&session, (0, 0), fitted_at(s, centre));
    assert_plate_like_native("scale-to-fit-exclude-area", &session, 1);
}

#[test]
fn plater_scale_to_fit_centres_on_the_area_every_nozzle_reaches_like_native() {
    // `get_build_volume(true)` (`PartPlate.cpp:5263-5282`) with a nozzle area: the box of
    // `get_shared_poly` of the one area X 20..220, Y 0..220, widened by eps.
    let mut session = load_edited(
        SINGLE,
        &[(
            PROJECT,
            "\"extruder_printable_area\": [],",
            "\"extruder_printable_area\": [\"20x0,220x0,220x220,20x220\"],",
        )],
    );
    make_tall(&mut session, (0, 0));
    session.scale_to_fit(0, 0).unwrap();
    let s = z_limited_factor(250.0);
    let centre = [
        0.5 * ((20.0 - EPS) + (220.0 + EPS)),
        0.5 * (-EPS + (220.0 + EPS)),
    ];
    assert_instance_transform(&session, (0, 0), fitted_at(s, centre));
    assert_plate_like_native("scale-to-fit-nozzle-area", &session, 1);
}

#[test]
fn plater_scale_to_fit_scales_the_sibling_instance_but_moves_only_the_fitted_one_like_native() {
    // The ported scale synchronizes the other instance's linear part (`Selection.cpp:3045`) and
    // both drop onto the bed; the translate moves the fitted instance alone.
    let mut session = load(SIBLINGS);
    make_tall(&mut session, (0, 0));
    session.scale_to_fit(0, 0).unwrap();
    let s = z_limited_factor(250.0);
    assert_instance_transform(&session, (0, 0), fitted_at(s, [110.0, 110.0]));
    assert_instance_transform(&session, (0, 1), fitted_at(s, [130.0, 110.0]));
    assert_plate_like_native("scale-to-fit-siblings", &session, 1);
}

/// Preservation control, no parity claim beyond plate 1: a Convex bed (`BuildVolume.cpp:77-80`)
/// takes no fit (`Selection.cpp:1574-1579`).
#[test]
fn plater_scale_to_fit_on_a_convex_bed_changes_nothing() {
    let mut session = load_edited(
        SINGLE,
        &[(
            PROJECT,
            ENDER3_AREA,
            "\"printable_area\": [\"55x0\", \"165x0\", \"220x110\", \"165x220\", \"55x220\", \"0x110\"]",
        )],
    );
    let before = export(&session);
    session.scale_to_fit(0, 0).unwrap();
    assert!(export(&session) == before, "the session changed");
    assert_plate_like_native("scale-to-fit-convex-bed", &session, 1);
}

/// `fit_circle` (`Selection.cpp:1534-1569`) is deferred: a 72-gon bed (`BuildVolume.cpp:46-75`)
/// is refused.
#[test]
fn plater_scale_to_fit_refuses_a_circular_bed() {
    let points: Vec<String> = (0..72)
        .map(|index| {
            let angle = f64::from(index) * std::f64::consts::TAU / 72.0;
            format!(
                "\"{:.4}x{:.4}\"",
                110.0 + 100.0 * angle.cos(),
                110.0 + 100.0 * angle.sin()
            )
        })
        .collect();
    let area = format!("\"printable_area\": [{}]", points.join(", "));
    let mut session = load_edited(SINGLE, &[(PROJECT, ENDER3_AREA, &area)]);
    assert!(matches!(
        session.scale_to_fit(0, 0),
        Err(SliceError::InvalidInput(text)) if text == SCALE_TO_FIT_CIRCLE
    ));
}
