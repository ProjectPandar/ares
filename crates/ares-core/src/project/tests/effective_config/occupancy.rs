use crate::{
    Point3d, ProjectMesh, ProjectVolume, ProjectVolumeType,
    project::{
        effective_config::{layers::LayerCandidateRange, occupancy::model_part_occupies_range},
        transform::Transform3d,
    },
};

#[test]
fn single_range_admits_any_nonempty_mesh_but_not_an_empty_triangle_set() {
    let far_away = volume(
        vec![
            Point3d::new(0.0, 0.0, 100.0),
            Point3d::new(1.0, 0.0, 100.0),
            Point3d::new(0.0, 1.0, 100.0),
        ],
        vec![[0, 1, 2]],
        Transform3d::IDENTITY,
    );
    let empty = volume(
        vec![Point3d::new(0.0, 0.0, 0.5)],
        Vec::new(),
        Transform3d::IDENTITY,
    );
    let slab = range(0.0, 1.0);

    let admitted: bool = model_part_occupies_range(Transform3d::IDENTITY, &far_away, 1, slab);
    let empty_admitted: bool = model_part_occupies_range(Transform3d::IDENTITY, &empty, 1, slab);

    assert!(admitted);
    assert!(!empty_admitted);
}

#[test]
fn multi_range_expands_by_epsilon_and_uses_strict_edge_bounds() {
    let slab = range(1.0001, 1.9999);
    let lower_equal = edge_volume(0.5, 1.0);
    let lower_just_inside = edge_volume(0.5, f64::from(1.0_f32.next_up()));
    let upper_equal = edge_volume(2.0, 2.5);
    let upper_just_inside = edge_volume(f64::from(2.0_f32.next_down()), 2.5);

    assert!(!model_part_occupies_range(
        Transform3d::IDENTITY,
        &lower_equal,
        2,
        slab
    ));
    assert!(model_part_occupies_range(
        Transform3d::IDENTITY,
        &lower_just_inside,
        2,
        slab
    ));
    assert!(!model_part_occupies_range(
        Transform3d::IDENTITY,
        &upper_equal,
        2,
        slab
    ));
    assert!(model_part_occupies_range(
        Transform3d::IDENTITY,
        &upper_just_inside,
        2,
        slab
    ));
}

#[test]
fn crossing_edge_occupies_slab_when_no_vertex_is_inside() {
    let crossing = edge_volume(-1.0, 2.0);

    assert!(model_part_occupies_range(
        Transform3d::IDENTITY,
        &crossing,
        2,
        range(0.25, 0.75)
    ));
}

#[test]
fn print_object_then_volume_composition_is_noncommutative() {
    let print_object_without_xy = transform_3mf("1 0 0 0 1 0 0 0 2 0 0 0");
    let volume = volume(
        vec![
            Point3d::new(0.0, 0.0, 0.0),
            Point3d::new(1.0, 0.0, 0.0),
            Point3d::new(0.0, 1.0, 0.0),
        ],
        vec![[0, 1, 2]],
        transform_3mf("1 0 0 0 1 0 0 0 1 10 20 1"),
    );
    let point = volume.mesh().vertices()[0];
    let correct = print_object_without_xy
        .then(volume.transform())
        .without_xy_translation()
        .transform_point(point);
    let reversed = volume
        .transform()
        .then(print_object_without_xy)
        .without_xy_translation()
        .transform_point(point);

    assert_eq!(correct.z, 2.0);
    assert_eq!(reversed.z, 1.0);
    assert!(model_part_occupies_range(
        print_object_without_xy,
        &volume,
        2,
        range(1.9, 2.1)
    ));
}

#[test]
fn occupancy_casts_transform_and_vertex_operands_before_arithmetic() {
    let volume = volume(
        vec![
            Point3d::new(1.0, 0.0, 0.0),
            Point3d::new(1.0, 1.0, 0.0),
            Point3d::new(1.0, 0.0, 1.0),
        ],
        vec![[0, 1, 2]],
        transform_3mf("1 0 16777217 0 1 0 0 0 0 0 0 -16777216"),
    );

    let occupied: bool =
        model_part_occupies_range(Transform3d::IDENTITY, &volume, 2, range(0.0, 0.1));

    assert!(occupied);
}

fn edge_volume(lower_z: f64, upper_z: f64) -> ProjectVolume {
    volume(
        vec![
            Point3d::new(0.0, 0.0, lower_z),
            Point3d::new(1.0, 0.0, upper_z),
            Point3d::new(0.0, 1.0, upper_z),
        ],
        vec![[0, 1, 2]],
        Transform3d::IDENTITY,
    )
}

fn volume(
    vertices: Vec<Point3d>,
    triangles: Vec<[u32; 3]>,
    transform: Transform3d,
) -> ProjectVolume {
    ProjectVolume::new(
        "synthetic.model".to_owned(),
        1,
        ProjectMesh::new(vertices, triangles),
        transform,
        (
            "model-part".to_owned(),
            ProjectVolumeType::ModelPart,
            Default::default(),
            Transform3d::IDENTITY,
        ),
    )
}

fn range(min_z: f64, max_z: f64) -> LayerCandidateRange {
    LayerCandidateRange {
        min_z,
        max_z,
        source_range_index: None,
    }
}

fn transform_3mf(value: &str) -> Transform3d {
    Transform3d::parse_3mf(value).unwrap()
}
