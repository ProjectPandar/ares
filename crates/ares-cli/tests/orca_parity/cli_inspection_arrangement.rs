#![cfg(target_os = "linux")]

use super::cli_model_inspection::{compare, open_box};

#[test]
fn mesh_inspection_preserves_stl_xy_when_arrangement_is_disabled() {
    compare_stl("--inspect-mesh", "0");
}

#[test]
fn mesh_inspection_reports_arranged_stl_placement_when_forced() {
    compare_stl("--inspect-mesh", "1");
}

#[test]
fn info_reports_object_bounds_with_and_without_arrangement() {
    for arrange in ["0", "1"] {
        compare_stl("--info", arrange);
    }
}

#[test]
fn paint_inspection_reports_mesh_local_bounds_with_and_without_arrangement() {
    for arrange in ["0", "1"] {
        compare_stl("--inspect-paint", arrange);
    }
}

fn compare_stl(action: &str, arrange: &str) {
    compare(
        &[action, "--arrange", arrange],
        Some(("placement.stl", open_box())),
        0,
    );
}
