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

// The `1e308%` overlap sentinels these cases used are now rejected by the
// O3 loaded-config range loop (`PrintConfig.cpp:10519-10559` is_value_valid)
// while the source is being prepared, long before the internal
// infill-boundary overlap preflight runs. The expected error is therefore the
// out-of-range text, not the unreachable downstream RANGE_ERROR.
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
    // `1e308%` overflows the float bound, so `is_value_valid` reports the key
    // out of `[-FLT_MAX, FLT_MAX]` during `prepare_post_classic_gap_extrusion`.
    assert!(matches!(
        prepare_post_classic_gap_extrusion(&archive.bytes()),
        Err(SliceError::InvalidInput(message)) if message
            == format!("{key}: 1e+308% not in range [-340282346638528859811704183484516925440.000000,340282346638528859811704183484516925440.000000]")
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
