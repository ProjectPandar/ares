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
const SCALE_TO_FIT_MODIFIER: &str =
    "scale to build volume on a circular bed needs every volume to be a model part";

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

/// The Ender-3 with a circular bed (`BuildVolume.cpp:46-75`): the 36 integer points exactly on the
/// circle of radius 65 around (110, 110), in angle order, so the RANSAC circle is exactly that one.
/// The largest gap undercuts by 0.65 mm, under the 3 mm limit.
fn circular_bed(base: &str, area: &str) -> PlaterSession {
    let mut points: Vec<(i32, i32)> = (-65..=65)
        .flat_map(|x: i32| [-1, 1].map(|sign| (x, sign * (CIRCLE_RADIUS.pow(2) - x * x).isqrt())))
        .filter(|&(x, y)| x * x + y * y == CIRCLE_RADIUS.pow(2))
        .collect();
    points.sort_by(|a, b| {
        f64::atan2(a.1.into(), a.0.into()).total_cmp(&f64::atan2(b.1.into(), b.0.into()))
    });
    points.dedup();
    assert_eq!(points.len(), 36);
    let area_points: Vec<String> = points
        .iter()
        .map(|(x, y)| format!("\"{}x{}\"", 110 + x, 110 + y))
        .collect();
    let circle = format!("\"printable_area\": [{}]", area_points.join(", "));
    load_edited(base, &[(PROJECT, area, &circle)])
}

const CIRCLE_RADIUS: i32 = 65;

/// A Size edit with the uniform toggle off.
fn set_size(session: &mut PlaterSession, axis: usize, value: f64) {
    session
        .set_size(
            0,
            0,
            ScaleEdit {
                axis,
                value,
                uniform: false,
            },
        )
        .unwrap();
}

#[test]
fn plater_scale_to_fit_fills_the_printable_height_on_a_circular_bed_like_native() {
    // `fit_circle` (`Selection.cpp:1534-1569`): the tall cube's enclosing circle is far inside the
    // bed's, so `s = printable_height / max_z`; its centre goes to the bed circle's, on the bed.
    let mut session = circular_bed(SINGLE, ENDER3_AREA);
    make_tall(&mut session, (0, 0));
    session.scale_to_fit(0, 0).unwrap();
    let s = 250.0 / TALL;
    assert_instance_transform(&session, (0, 0), fitted_at(s, [110.0, 110.0]));
    assert_plate_like_native("scale-to-fit-circle-height", &session, 1);
}

#[test]
fn plater_scale_to_fit_fills_the_bed_circle_with_a_wide_cube_like_native() {
    // A 100 x 100 x 10 mm cube: its 2D hull's enclosing circle (`smallest_enclosing_circle_welzl`,
    // `Circle.hpp:112-154`) is the diagonal's, inflated by `SCALED_EPSILON`, plus 0.01 mm, so
    // `s = 65 / (50 sqrt 2 + 1e-4 + 0.01)`.
    let mut session = circular_bed(SINGLE, ENDER3_AREA);
    set_size(&mut session, 0, 100.0);
    set_size(&mut session, 1, 100.0);
    session.scale_to_fit(0, 0).unwrap();
    let s = f64::from(CIRCLE_RADIUS) / (50.0 * std::f64::consts::SQRT_2 + EPS + 0.01);
    assert_instance_transform(
        &session,
        (0, 0),
        scaled_at([10.0 * s, 10.0 * s, s], [110.0, 110.0, 5.0 * s]),
    );
    assert_plate_like_native("scale-to-fit-circle-radius", &session, 1);
}

#[test]
fn plater_scale_to_fit_centres_on_the_bed_circle_of_plate_two_like_native() {
    // `plate_shift`: the current plate's centre (`PartPlate::get_center_origin`,
    // `PartPlate.cpp:4266-4275`) minus `BuildVolume::bed_center()`, here plate 2's origin. The wide
    // cube of the radius case, fitted on plate 2.
    let mut session = circular_bed(SINGLE, ENDER3_AREA);
    session.add_plate().unwrap();
    let origin = session.plates()[1].origin;
    session
        .translate_instance(0, 0, [origin[0], origin[1], 0.0])
        .unwrap();
    session.select_plate(1).unwrap();
    set_size(&mut session, 0, 100.0);
    set_size(&mut session, 1, 100.0);
    session.scale_to_fit(0, 0).unwrap();
    let s = f64::from(CIRCLE_RADIUS) / (50.0 * std::f64::consts::SQRT_2 + EPS + 0.01);
    assert_instance_transform(
        &session,
        (0, 0),
        scaled_at(
            [10.0 * s, 10.0 * s, s],
            [origin[0] + 110.0, origin[1] + 110.0, 5.0 * s],
        ),
    );
    assert_plate_like_native("scale-to-fit-circle-plate-two", &session, 2);
}

/// A modifier has no `GLVolume` convex hull (`3DScene.cpp:921-927`), which `fit_circle`
/// dereferences: upstream undefined behaviour, refused instead.
#[test]
fn plater_scale_to_fit_refuses_a_modifier_on_a_circular_bed() {
    // A cube with a modifier part (`adaptive-octree-spacing/baseline.3mf`) on the circular bed.
    let mut session = circular_bed(
        "tests/parity/adaptive-octree-spacing/baseline.3mf",
        "\"printable_area\": [\n    \"0x0\",\n    \"256x0\",\n    \"256x256\",\n    \"0x256\"\n  ]",
    );
    let before = export(&session);
    assert!(matches!(
        session.scale_to_fit(0, 0),
        Err(SliceError::InvalidInput(text)) if text == SCALE_TO_FIT_MODIFIER
    ));
    assert!(export(&session) == before, "the session changed");
}
