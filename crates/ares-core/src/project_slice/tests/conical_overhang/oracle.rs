use crate::{
    ProjectVolumeType, SliceError, Transform3d, geometry::CoordinateScale, mesh_slicer::SlicingMode,
};

use super::super::super::{
    closing::{PostClosingLayer, PostClosingPrintObject, PostClosingVolume},
    region_slices::{RegionSurface, prepare_region_slices},
    task22j_oracle,
    top_empty_layers::remove_project_top_empty_layers,
    volume_bounds::build_volume_bounds,
    volume_regions::VolumeRegionGraph,
};
use super::super::{
    region_fixture::checkpoint,
    region_slices::complex,
    support::{object as project_object, plan, project_volume, resolved, resolved_object},
};
use super::{
    apply_objects, apply_resolved, expolygon, geometry_error, geometry_snapshot, identity_snapshot,
    layer_geometry, object_options, post_region, print_object, rectangle, region_options,
    sidecar_snapshot, square,
};

#[test]
fn task22l_phase_a_ten_object_released_k_to_expected_l_is_exact() {
    let (_, k) = released_ten_object_k();
    assert_eq!(k.len(), 5_848);
    assert_eq!(
        checkpoint::sha256(&k),
        "037b5e1b5aa9eb2f5c9c38f00a8d7a23768217fd7cc7ec13bb71f21d9edb3b07"
    );

    let l = checkpoint::encode_with_magic(&checkpoint::parse_k(&k).stream, b"ARES22L\0");
    assert_eq!(l.len(), 5_848);
    assert_eq!(
        checkpoint::sha256(&l),
        "fe46d60251dcf95590c71a3e55cafdf81e0fc6af5b3cb95d58d6c39ea693b264"
    );
    assert_eq!(
        checkpoint::parse_l(&l).stream,
        checkpoint::parse_k(&k).stream
    );
    assert_eq!(&l[8..], &k[8..]);
}

#[test]
fn task22l_ten_object_transition_is_exact() {
    let (mut outputs, k) = released_ten_object_k();
    let resolved = (0..outputs.len())
        .map(|index| resolved(index, object_options(55.0, 0.0, 0.2), Vec::new()))
        .collect::<Vec<_>>();
    apply_resolved(&mut outputs, &resolved, CoordinateScale::Normal).unwrap();
    let actual = task22j_oracle::encode_with_magic(&outputs, b"ARES22L\0");
    let expected = checkpoint::encode_with_magic(&checkpoint::parse_k(&k).stream, b"ARES22L\0");
    assert_eq!(actual, expected);
    assert_eq!(
        checkpoint::sha256(&actual),
        "fe46d60251dcf95590c71a3e55cafdf81e0fc6af5b3cb95d58d6c39ea693b264"
    );
}

fn released_ten_object_k() -> (Vec<super::PostRegionPrintObject>, Vec<u8>) {
    let mut outputs = complex::synthetic_outputs();
    remove_project_top_empty_layers(&mut outputs);
    let k = task22j_oracle::encode_with_magic(&outputs, b"ARES22K\0");
    (outputs, k)
}

#[test]
fn task22l_oracle_fixed_normal_and_large_projection_vectors() {
    let cases = [
        (
            CoordinateScale::Normal,
            2_000_000,
            1_500_000,
            3_500_000,
            expolygon(&[
                (2_000_000, 285_530),
                (3_214_470, 285_530),
                (3_214_470, 1_714_470),
                (2_000_000, 1_714_470),
                (2_000_000, 2_000_000),
                (0, 2_000_000),
                (0, 0),
                (2_000_000, 0),
            ]),
        ),
        (
            CoordinateScale::LargeBed,
            200_000,
            150_000,
            350_000,
            expolygon(&[
                (200_000, 28_553),
                (321_447, 28_553),
                (321_447, 171_447),
                (200_000, 171_447),
                (200_000, 200_000),
                (0, 200_000),
                (0, 0),
                (200_000, 0),
            ]),
        ),
    ];

    for (scale, size, upper_min, upper_max, expected) in cases {
        let region = post_region(
            0,
            region_options(true, 1, 0, 0.0, 0),
            vec![
                vec![rectangle(0, 0, size, size)],
                vec![rectangle(upper_min, 0, upper_max, size)],
            ],
        );
        let mut object = print_object(0, 0, &[0.2, 0.2], vec![region]);
        apply_objects(
            std::slice::from_mut(&mut object),
            vec![object_options(55.0, 0.0, 0.2)],
            scale,
        )
        .unwrap();
        assert_eq!(layer_geometry(&object, 0, 0), vec![expected]);
    }
}

#[test]
fn task22l_oracle_safety_offset_is_ten_coordinates_at_both_scales() {
    let normal = ownership_object(1);
    let large = ownership_object(10);
    let cases = [
        (CoordinateScale::Normal, normal, 1_000_000, 800_010, 199_990),
        (CoordinateScale::LargeBed, large, 100_000, 80_010, 19_990),
    ];

    for (scale, mut object, split, upper_y, lower_y) in cases {
        apply_objects(
            std::slice::from_mut(&mut object),
            vec![object_options(0.0, 0.0, 0.2)],
            scale,
        )
        .unwrap();
        assert_eq!(
            layer_geometry(&object, 1, 0),
            vec![
                expolygon(&[
                    (2 * split, split),
                    (split, split),
                    (split, upper_y),
                    (2 * split, upper_y),
                ]),
                expolygon(&[
                    (2 * split, lower_y),
                    (split, lower_y),
                    (split, 0),
                    (2 * split, 0),
                ]),
            ]
        );
    }
}

#[test]
fn task22l_oracle_validates_all_contexts_before_any_mutation() {
    let gated = [
        print_object(0, 0, &[], vec![]),
        print_object(0, 0, &[0.2, 0.2], vec![]),
        print_object(
            0,
            0,
            &[0.2, 0.2],
            vec![post_region(
                0,
                region_options(false, 1, 0, 0.0, 0),
                vec![vec![square(0, 1_000_000)], vec![square(1, 1_000_001)]],
            )],
        ),
    ];
    for mut object in gated {
        assert_eq!(
            apply_objects(
                std::slice::from_mut(&mut object),
                vec![object_options(-0.1, 0.0, 0.2)],
                CoordinateScale::Normal,
            ),
            Err(SliceError::InvalidInput(
                "invalid Orca option make_overhang_printable_angle".to_owned()
            ))
        );
    }

    let mut ninety = print_object(0, 0, &[0.2], vec![]);
    assert_eq!(
        apply_objects(
            std::slice::from_mut(&mut ninety),
            vec![object_options(90.0, -0.1, 0.2)],
            CoordinateScale::Normal,
        ),
        Err(SliceError::InvalidInput(
            "invalid Orca option make_overhang_printable_hole_size".to_owned()
        ))
    );

    let mut objects = vec![projection_object(0), print_object(1, 0, &[], vec![])];
    let before = geometry_snapshot(&objects);
    assert_eq!(
        apply_objects(
            &mut objects,
            vec![
                object_options(0.0, 0.0, 0.2),
                object_options(55.0, -0.1, 0.2),
            ],
            CoordinateScale::Normal,
        ),
        Err(SliceError::InvalidInput(
            "invalid Orca option make_overhang_printable_hole_size".to_owned()
        ))
    );
    assert_eq!(geometry_snapshot(&objects), before);
}

#[test]
fn task22l_oracle_maps_clipper_range_error_without_identity_fallback() {
    const HI: i64 = 0x3fff_ffff_ffff_ffff;
    let shape = expolygon(&[(HI - 80, 0), (HI - 40, 0), (HI - 40, 40), (HI - 80, 40)]);
    let region = post_region(
        0,
        region_options(true, 1, 0, 0.0, 0),
        vec![vec![shape.clone()], vec![shape]],
    );
    let mut object = print_object(0, 0, &[0.2, 0.2], vec![region]);
    let before = geometry_snapshot(std::slice::from_ref(&object));
    assert_eq!(
        apply_objects(
            std::slice::from_mut(&mut object),
            vec![object_options(0.0, 0.0, 0.2)],
            CoordinateScale::Normal,
        ),
        Err(geometry_error())
    );
    assert_eq!(geometry_snapshot(std::slice::from_ref(&object)), before);
}

#[test]
fn task22l_oracle_preserves_plan_ids_options_order_and_complete_sidecar() {
    let mut objects = vec![object_with_sidecar(), projection_object(1)];
    let before = identity_snapshot(&objects);
    let sidecar_before = sidecar_snapshot(&objects[0]);
    apply_objects(
        &mut objects,
        vec![object_options(0.0, 0.0, 0.2), object_options(0.0, 0.0, 0.2)],
        CoordinateScale::Normal,
    )
    .unwrap();

    assert_eq!(identity_snapshot(&objects), before);
    assert_eq!(sidecar_snapshot(&objects[0]), sidecar_before);

    let first = geometry_snapshot(&objects);
    let mut repeated = vec![object_with_sidecar(), projection_object(1)];
    apply_objects(
        &mut repeated,
        vec![object_options(0.0, 0.0, 0.2), object_options(0.0, 0.0, 0.2)],
        CoordinateScale::Normal,
    )
    .unwrap();
    assert_eq!(geometry_snapshot(&repeated), first);
}

fn projection_object(source_object_index: usize) -> super::PostRegionPrintObject {
    print_object(
        source_object_index,
        source_object_index + 10,
        &[0.2, 0.2],
        vec![post_region(
            source_object_index + 30,
            region_options(true, 1, 0, 0.0, 0),
            vec![
                vec![square(0, 1_000_000)],
                vec![rectangle(800_000, 0, 1_800_000, 1_000_000)],
            ],
        )],
    )
}

fn ownership_object(divisor: i64) -> super::PostRegionPrintObject {
    let u = 100_000 / divisor;
    print_object(
        0,
        divisor as usize,
        &[0.2, 0.2],
        vec![
            post_region(
                10,
                region_options(true, 1, 0, 0.0, 0),
                vec![
                    vec![rectangle(0, 0, 10 * u, 10 * u)],
                    vec![rectangle(9 * u, 2 * u, 25 * u, 8 * u)],
                ],
            ),
            post_region(
                20,
                region_options(false, 1, 0, 0.0, 0),
                vec![vec![rectangle(10 * u, 0, 20 * u, 10 * u)], vec![]],
            ),
        ],
    )
}

fn object_with_sidecar() -> super::PostRegionPrintObject {
    let source = project_object(
        "task22l.model",
        1,
        vec![project_volume(
            "task22l.model",
            1,
            ProjectVolumeType::ModelPart,
            true,
            false,
        )],
        &[Transform3d::IDENTITY],
    );
    let resolved = resolved_object(0, &[Transform3d::IDENTITY]);
    let bounded = build_volume_bounds(
        &source,
        &resolved,
        PostClosingPrintObject::new(
            plan(0, 0, 2),
            vec![PostClosingVolume::new(
                0,
                17,
                ProjectVolumeType::ModelPart,
                vec![
                    PostClosingLayer::new(SlicingMode::Regular, vec![square(0, 1_000_000)]),
                    PostClosingLayer::new(
                        SlicingMode::Regular,
                        vec![rectangle(800_000, 0, 1_800_000, 1_000_000)],
                    ),
                ],
            )],
        ),
    );
    let (mut object, ..) = prepare_region_slices(
        bounded,
        VolumeRegionGraph {
            all_regions: vec![region_options(true, 1, 0, 0.0, 0)],
            volume_regions: Vec::new(),
        },
    )
    .into_parts();
    object.regions[0].layers[0]
        .surfaces
        .push(RegionSurface::internal(square(0, 1_000_000)));
    object.regions[0].layers[1]
        .surfaces
        .push(RegionSurface::internal(rectangle(
            800_000, 0, 1_800_000, 1_000_000,
        )));
    object
}
