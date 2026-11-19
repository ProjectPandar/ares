#![cfg(target_os = "linux")]

use super::cli_model_inspection::{PROJECT, compare, open_box};

#[test]
fn mesh_inspection_reports_ordered_planes_for_an_open_stl() {
    compare(&["--inspect-mesh"], Some(("open.stl", open_box())), 0);
}

#[test]
fn mesh_inspection_reports_ordered_planes_for_rotated_multipart_objects() {
    compare(
        &["--inspect-mesh", "--rotate", "90"],
        Some(("paint.3mf", PROJECT.to_vec())),
        0,
    );
}
