//! Geometry-only witnesses for beta `Fill3DHoneycomb.cpp:336–436`.
//! The parent's full-stream assertion remains independent and unchanged.

#[path = "honeycomb_geometry/height_thickness.rs"]
mod height_thickness;
#[path = "honeycomb_geometry/planar_smoothing.rs"]
mod planar_smoothing;

use serde_json::json;

use super::fill_geometry::run;

#[test]
fn three_d_honeycomb_centered_plater_geometry_matches_native() {
    run("honeycomb-centered", json!({}), false);
}

#[test]
fn three_d_honeycomb_moved_plater_geometry_matches_native() {
    run("honeycomb-moved", json!({}), true);
}

#[test]
fn three_d_honeycomb_dense_plater_geometry_matches_native() {
    run(
        "honeycomb-dense",
        json!({"sparse_infill_density": "60%"}),
        false,
    );
}

#[test]
fn three_d_honeycomb_multiline_plater_geometry_matches_native() {
    run("honeycomb-multiline", json!({"fill_multiline": "3"}), false);
}

#[test]
fn three_d_honeycomb_complete_top_plater_geometry_matches_native() {
    run(
        "honeycomb-complete-top",
        json!({"infill_complete_top": "1"}),
        false,
    );
}

#[test]
fn three_d_honeycomb_open_top_plater_geometry_matches_native() {
    run(
        "honeycomb-open-top",
        json!({"infill_complete_top": "0"}),
        false,
    );
}

#[test]
fn three_d_honeycomb_smoothed_plater_geometry_matches_native() {
    run(
        "honeycomb-smoothed",
        json!({"sparse_infill_smooth_factor": "35%"}),
        false,
    );
}
