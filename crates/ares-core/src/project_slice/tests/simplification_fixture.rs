mod checkpoint;
mod mutations;

use crate::{task22i_browser_input_oracle, task22i_browser_oracle};
use checkpoint::{
    Expected, assert_body_equal_except_magic, assert_changed_layers, assert_checkpoint,
    assert_record, sha256,
};

use super::support::ksr_project;

const FIXTURE_SHA256: &str = "698f40f13c9075b818abedd3d10f022fbb5d8200aed48fbdde651f6bfb21b8a9";
const CHANGED_SLOTS_SHA256: &str =
    "7377acff6b3bea897ad32249b320eeba2bc48091b9618be54d2f3ad44d269514";

const BASE_H: Expected = Expected {
    len: 1_644_681,
    sha256: "e15967c36c0aa47a9a1a3fc31053587777359bedef796053022eaeb36ad49163",
    modes: [460, 0, 0, 0],
    contours: 2_890,
    holes: 395,
    points: 99_212,
};
const BASE_I: Expected = Expected {
    len: 999_721,
    sha256: "0dea485aea9f003db4dbadfd524e82cc2ad33327d3b447a7d985d57d82da72ef",
    modes: [460, 0, 0, 0],
    contours: 2_890,
    holes: 395,
    points: 58_902,
};
const DISABLED_I: Expected = Expected {
    len: 1_644_681,
    sha256: "572688f416497a276540adc57df50742561363a7d0470124ea21759eced591ff",
    ..BASE_H
};
const PRIMARY_H: Expected = Expected {
    len: 427_465,
    sha256: "a0df3397e498306bfcade84b03721fe345d2f4b501e578a5b54df39faff44353",
    modes: [2, 0, 0, 458],
    contours: 470,
    holes: 13,
    points: 25_747,
};
const PRIMARY_I: Expected = Expected {
    len: 275_433,
    sha256: "022cc958a38d5654e0a5fc4e2ca44d5e5ef068b7e57b271cb14151b11005343e",
    modes: [2, 0, 0, 458],
    contours: 470,
    holes: 13,
    points: 16_245,
};
const THRESHOLD_H: Expected = Expected {
    len: 674_201,
    sha256: "4b64a4e70bfceabf414572f6dbe13903245612908cbaf2d12985b6c1ed440214",
    modes: [21, 0, 0, 439],
    contours: 569,
    holes: 127,
    points: 41_012,
};
const THRESHOLD_I: Expected = Expected {
    len: 416_217,
    sha256: "185118681aad5de780a93d6f71f22f497dc7dc7dd82e038ec1feaf32b0f91294",
    modes: [21, 0, 0, 439],
    contours: 569,
    holes: 127,
    points: 24_888,
};

#[test]
fn task22i_committed_archive_matches_complete_h_and_i_checkpoints() {
    assert_eq!(sha256(ksr_project()), FIXTURE_SHA256);
    let (h, i) = repeatable_checkpoints(ksr_project());
    let h_snapshot = assert_checkpoint(&h, b"ARES22H\0", BASE_H);
    let i_snapshot = assert_checkpoint(&i, b"ARES22I\0", BASE_I);

    assert_changed_layers(
        (&h, &h_snapshot),
        (&i, &i_snapshot),
        0..=259,
        CHANGED_SLOTS_SHA256,
    );
    for (slot, len, digest) in [
        (
            0,
            11_681,
            "a9320cf7f76a8a4dc24d394033ae1e53b5245eec5d808d8df26a35a5ac49bc9c",
        ),
        (
            46,
            24_217,
            "0e515d5ebb34e7f06e886956f62b955cc83a7e58e49f2b28ab37374b26f58291",
        ),
        (
            49,
            23_513,
            "c020b4558012a485af5ec1bcc01da9b3785fb448e24e37ee4adcd307deaf0ea8",
        ),
        (
            459,
            737,
            "c8822b67958531cb4b043d338b53f7329e0b00cb4f08108306763e763cd52f80",
        ),
    ] {
        assert_record(&i, &i_snapshot, slot, len, digest);
    }
}

#[test]
fn task22i_resolution_at_threshold_is_exact_marker_only_identity() {
    let project = mutations::resolution("0.001");
    let (h, i) = repeatable_checkpoints(&project);
    assert_checkpoint(&h, b"ARES22H\0", BASE_H);
    assert_checkpoint(&i, b"ARES22I\0", DISABLED_I);
    assert_body_equal_except_magic(&h, &i, b'H', b'I');
}

#[test]
fn task22i_resolution_just_above_threshold_equals_committed_output() {
    let committed = task22i_browser_oracle(ksr_project()).unwrap();
    let project = mutations::resolution("0.0011");
    let (h, i) = repeatable_checkpoints(&project);
    assert_checkpoint(&h, b"ARES22H\0", BASE_H);
    assert_checkpoint(&i, b"ARES22I\0", BASE_I);
    assert_eq!(i, committed);
}

#[test]
fn task22i_runs_after_the_complete_three_option_largest_contour_stage() {
    let project = mutations::primary();
    let (h, i) = repeatable_checkpoints(&project);
    assert_checkpoint(&h, b"ARES22H\0", PRIMARY_H);
    assert_checkpoint(&i, b"ARES22I\0", PRIMARY_I);
}

#[test]
fn task22i_preserves_the_threshold_21_largest_contour_result() {
    let project = mutations::threshold_21();
    let (h, i) = repeatable_checkpoints(&project);
    assert_checkpoint(&h, b"ARES22H\0", THRESHOLD_H);
    assert_checkpoint(&i, b"ARES22I\0", THRESHOLD_I);
}

fn repeatable_checkpoints(project: &[u8]) -> (Vec<u8>, Vec<u8>) {
    let h = task22i_browser_input_oracle(project).unwrap();
    let i = task22i_browser_oracle(project).unwrap();
    assert_eq!(task22i_browser_input_oracle(project).unwrap(), h);
    assert_eq!(task22i_browser_oracle(project).unwrap(), i);
    (h, i)
}
