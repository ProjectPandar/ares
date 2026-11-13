//! `Arrange.cpp:150-246` at `40ecc66f7`: short-object spacing and sequential bed inflation.

use std::fs;

use ares_core::{ArrangeScope, PlaterSession};

use super::{
    Arranged, BY_LAYER, CUBES, Edit, MODEL, NAMED, PROJECT, edited_input, native_arranged, settings,
};
use crate::runner::OrcaRunner;

const SHORT: [Edit; 3] = [
    (
        PROJECT,
        "\"nozzle_height\": \"2.5\"",
        "\"nozzle_height\": \"20\"",
    ),
    (PROJECT, "\"skirt_loops\": \"2\"", "\"skirt_loops\": \"0\""),
    (PROJECT, "\"brim_width\": \"8\"", "\"brim_width\": \"0\""),
];

fn check(label: &str, edits: &[Edit]) {
    let edits = [NAMED.as_slice(), SHORT.as_slice(), edits].concat();
    let input = edited_input(label, (CUBES, &edits));
    let native = native_arranged(label, &input, settings(false, false), (&[], None));
    let mut session = PlaterSession::load(input.clone()).unwrap();
    session
        .arrange(ArrangeScope::All, settings(false, false))
        .unwrap();
    let actual = Arranged::from_session(&session, None);
    let runner = OrcaRunner::from_env().unwrap().unwrap();
    let directory = runner.work.join(label);
    fs::create_dir_all(&directory).unwrap();
    fs::write(directory.join("input.3mf"), input).unwrap();
    fs::write(
        directory.join("native-arranged.txt"),
        format!("{native:#?}"),
    )
    .unwrap();
    fs::write(directory.join("ares-arranged.txt"), format!("{actual:#?}")).unwrap();
    fs::write(
        directory.join("arrangement-witness.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "case": label,
            "matches_native": actual == native,
        }))
        .unwrap(),
    )
    .unwrap();
    println!("{label}: artifacts {directory:?}");
    assert_eq!(actual, native, "{label}");
}

#[test]
fn short_objects_use_nozzle_clearance_like_native() {
    check("short-nozzle-clearance", &[]);
}

#[test]
fn crowded_short_objects_stay_on_the_bed_like_native() {
    let larger = [
        (
            MODEL,
            "1 0 0 0 1 0 0 0 1 50 110 2\"",
            "8 0 0 0 8 0 0 0 1 50 110 2\"",
        ),
        (
            MODEL,
            "1 0 0 0 1 0 0 0 1 125 110 2\"",
            "8 0 0 0 8 0 0 0 1 125 110 2\"",
        ),
        (
            MODEL,
            "1 0 0 0 1 0 0 0 1 200 110 2\"",
            "8 0 0 0 8 0 0 0 1 200 110 2\"",
        ),
    ];
    check("short-crowded-bed", &larger);
}

#[test]
fn short_objects_keep_exclusions_in_place_when_the_bed_shrinks_like_native() {
    let left_half = (
        PROJECT,
        "{\n  \"accel_to_decel_enable\"",
        "{\n  \"bed_exclude_volumes\": \"0..250;0x0,110x0,110x220,0x220\",\n  \"accel_to_decel_enable\"",
    );
    check("short-positive-margin-exclusion", &[left_half]);
}

#[test]
fn turned_sequential_objects_fit_their_inflated_outline_like_native() {
    let turned = (
        MODEL,
        "transform=\"1 0 0 0 1 0 0 0 1 50 110 2\"",
        "transform=\"10.6066017 10.6066017 0 -10.6066017 10.6066017 0 0 0 1 50 110 2\"",
    );
    check("short-turned-bed-margin", &[turned]);
}

#[test]
fn bed_filling_sequential_objects_get_the_margin_back_like_native() {
    let wide = (
        MODEL,
        "transform=\"1 0 0 0 1 0 0 0 1 50 110 2\"",
        "transform=\"22 0 0 0 1 0 0 0 1 110 110 2\"",
    );
    check("short-full-bed-margin", &[wide]);
}

#[test]
fn by_layer_object_spacing_matches_native() {
    check("short-by-layer-control", &[BY_LAYER]);
}
