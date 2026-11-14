use super::{check_project, fixtures};

#[test]
fn coplanar_truncated_tetrahedron_orients_like_native() {
    check_project(Some("1"), &fixtures::geometry(true), None, "coplanar", true);
}

#[test]
fn nonconvex_disconnected_mesh_orients_like_native() {
    check_project(
        Some("1"),
        &fixtures::geometry(false),
        None,
        "nonconvex",
        true,
    );
}

#[test]
fn anisotropic_rotated_instance_retains_upright_like_native() {
    check_project(
        Some("1"),
        &fixtures::transformed(false, false),
        None,
        "instance-scale",
        false,
    );
}

#[test]
fn mirrored_scaled_instance_retains_upright_like_native() {
    check_project(
        Some("1"),
        &fixtures::transformed(false, true),
        None,
        "instance-mirror",
        false,
    );
}

#[test]
fn anisotropic_rotated_volume_retains_upright_like_native() {
    check_project(
        Some("1"),
        &fixtures::transformed(true, false),
        None,
        "volume-scale",
        false,
    );
}

#[test]
fn mirrored_scaled_volume_orients_like_native() {
    check_project(
        Some("1"),
        &fixtures::transformed(true, true),
        None,
        "volume-mirror",
        true,
    );
}

#[test]
fn multiple_volumes_are_oriented_as_one_object_like_native() {
    check_project(
        Some("1"),
        &fixtures::multiple("volumes"),
        None,
        "volumes",
        true,
    );
}

#[test]
fn multiple_instances_are_scored_together_like_native() {
    check_project(
        Some("1"),
        &fixtures::multiple("instances"),
        None,
        "instances",
        true,
    );
}

#[test]
fn independent_objects_preserve_association_like_native() {
    check_project(
        Some("1"),
        &fixtures::multiple("objects"),
        None,
        "objects",
        true,
    );
}

#[test]
fn explicit_plate_keeps_orientation_after_import_selection_like_native() {
    check_project(
        Some("1"),
        &fixtures::multiple("plates"),
        Some(1),
        "explicit-plate",
        true,
    );
}
