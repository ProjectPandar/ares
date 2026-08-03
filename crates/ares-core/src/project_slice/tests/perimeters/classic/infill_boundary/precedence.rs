use crate::{
    SliceError,
    project_slice::perimeters::{
        classic::{
            gap_extrusion::PreparedPostClassicGapExtrusion,
            infill_boundary::{self, GeometryStep},
        },
        prepare_post_classic_gap_extrusion,
    },
};

use super::super::super::super::support::KsrArchive;

const CONFIG: &str = "Metadata/project_settings.config";
const RANGE_ERROR: &str =
    "Classic infill-boundary overlap is outside the supported coordinate range";

pub(super) const FAILURE_CANDIDATES: [GeometryStep; 10] = [
    GeometryStep::Simplify,
    GeometryStep::AggregateUnion,
    GeometryStep::OrdinaryOffset,
    GeometryStep::TopOffset,
    GeometryStep::TopIntersection,
    GeometryStep::TopOverlapOffset,
    GeometryStep::TopUnion,
    GeometryStep::NoOverlapTwo,
    GeometryStep::NoOverlapOne,
    GeometryStep::FinalTopUnion,
];

#[test]
fn task22o15_ordinary_overlap_preflight_beats_every_geometry_candidate() {
    assert_conversion_precedence("infill_wall_overlap", "15%", "1e308%");
}

#[test]
fn task22o15_top_overlap_preflight_beats_every_geometry_candidate() {
    assert_conversion_precedence("top_bottom_infill_wall_overlap", "25%", "1e308%");
}

fn assert_conversion_precedence(key: &str, old: &str, new: &str) {
    let mut archive = KsrArchive::new();
    archive.replace_unique(
        CONFIG,
        &format!("\"{key}\": \"{old}\""),
        &format!("\"{key}\": \"{new}\""),
    );
    let source = prepare_post_classic_gap_extrusion(archive.bytes()).unwrap();
    assert_numeric_precedence(&source);
    assert!(matches!(
        infill_boundary::finish(source),
        Err(SliceError::InvalidInput(message)) if message == RANGE_ERROR
    ));
}

pub(super) fn assert_numeric_precedence(source: &PreparedPostClassicGapExtrusion) {
    for candidate in FAILURE_CANDIDATES {
        infill_boundary::reset_geometry_hooks();
        infill_boundary::fail_geometry_at(candidate);
        assert!(matches!(
            infill_boundary::validate_numeric_preflight_for_test(source),
            Err(SliceError::InvalidInput(message)) if message == RANGE_ERROR
        ));
        assert!(
            infill_boundary::geometry_events().is_empty(),
            "numeric preflight reached {candidate:?}"
        );
    }
}
