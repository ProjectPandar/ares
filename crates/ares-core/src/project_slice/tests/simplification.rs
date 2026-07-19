use crate::{
    ProjectVolumeType, SliceError,
    geometry::{CoordinateScale, ExPolygon, Point, Polygon},
    mesh_slicer::SlicingMode,
};

use super::super::{
    closing::{PostClosingLayer, PostClosingPrintObject, PostClosingVolume},
    layers::{PlannedLayer, PlannedPrintObject},
    simplification::{apply_project_simplification, simplification_tolerance},
};

const MODES: [SlicingMode; 4] = [
    SlicingMode::Regular,
    SlicingMode::EvenOdd,
    SlicingMode::Positive,
    SlicingMode::PositiveLargestContour,
];

#[test]
fn task22i_project_resolution_threshold_maps_to_exact_scaled_float() {
    for resolution in [0.0, 0.001] {
        assert_eq!(
            simplification_tolerance(resolution, CoordinateScale::Normal),
            None
        );
        assert_eq!(
            simplification_tolerance(resolution, CoordinateScale::LargeBed),
            None
        );
    }
    for resolution in [0.0011, 0.002, 0.012, 1.0] {
        assert_eq!(
            simplification_tolerance(resolution, CoordinateScale::Normal),
            Some(2500.0)
        );
        assert_eq!(
            simplification_tolerance(resolution, CoordinateScale::LargeBed),
            Some(250.0)
        );
    }
}

#[test]
fn task22i_project_all_modes_use_exact_normal_and_large_bed_tolerances() {
    assert_scale(
        CoordinateScale::Normal,
        expolygon(&[(0, 0), (5000, 0), (2500, 2500)]),
        expolygon(&[(10_000, 0), (15_000, 0), (12_500, 2501)]),
        expolygon(&[(12_500, 2501), (10_000, 0), (15_000, 0)]),
    );
    assert_scale(
        CoordinateScale::LargeBed,
        expolygon(&[(0, 0), (500, 0), (250, 250)]),
        expolygon(&[(1000, 0), (1500, 0), (1250, 251)]),
        expolygon(&[(1250, 251), (1000, 0), (1500, 0)]),
    );
}

#[test]
fn task22i_project_preserves_records_and_processes_each_expolygon_independently() {
    let input = expolygon(&[(0, 0), (10_000, 0), (10_000, 10_000), (0, 10_000)]);
    let expected = expolygon(&[(10_000, 10_000), (0, 10_000), (0, 0), (10_000, 0)]);
    let first_plan = plan(17, 19, 2);
    let second_plan = plan(23, 29, 1);
    let mut objects = vec![
        PostClosingPrintObject::new(
            first_plan.clone(),
            vec![
                PostClosingVolume::new(
                    31,
                    37,
                    ProjectVolumeType::ModelPart,
                    vec![layer(
                        SlicingMode::Regular,
                        vec![input.clone(), input.clone()],
                    )],
                ),
                PostClosingVolume::new(
                    41,
                    43,
                    ProjectVolumeType::NegativeVolume,
                    vec![layer(SlicingMode::EvenOdd, Vec::new())],
                ),
            ],
        ),
        PostClosingPrintObject::new(
            second_plan.clone(),
            vec![PostClosingVolume::new(
                47,
                53,
                ProjectVolumeType::ParameterModifier,
                vec![layer(SlicingMode::Positive, vec![input])],
            )],
        ),
    ];

    apply_project_simplification(&mut objects, 1.0, CoordinateScale::Normal).unwrap();

    assert_eq!(objects[0].plan(), &first_plan);
    assert_eq!(objects[1].plan(), &second_plan);
    assert_volume(
        &objects[0].volumes()[0],
        31,
        37,
        ProjectVolumeType::ModelPart,
    );
    assert_volume(
        &objects[0].volumes()[1],
        41,
        43,
        ProjectVolumeType::NegativeVolume,
    );
    assert_volume(
        &objects[1].volumes()[0],
        47,
        53,
        ProjectVolumeType::ParameterModifier,
    );
    assert_eq!(
        objects[0].volumes()[0].layers()[0].mode(),
        SlicingMode::Regular
    );
    assert_eq!(
        objects[0].volumes()[0].layers()[0].expolygons(),
        &[expected.clone(), expected.clone()]
    );
    assert!(objects[0].volumes()[1].layers()[0].expolygons().is_empty());
    assert_eq!(
        objects[1].volumes()[0].layers()[0].expolygons(),
        &[expected]
    );
}

#[test]
fn task22i_project_disabled_stage_skips_geometry_and_enabled_range_error_is_exact() {
    const HI: i64 = 0x3fff_ffff_ffff_ffff;
    let invalid = expolygon(&[
        (HI + 1, 0),
        (HI + 10_001, 0),
        (HI + 10_001, 10_000),
        (HI + 1, 10_000),
    ]);
    let mut objects = single_object(vec![invalid.clone()]);
    let expolygons_ptr = objects[0].volumes()[0].layers()[0].expolygons().as_ptr();
    let points_ptr = objects[0].volumes()[0].layers()[0].expolygons()[0]
        .contour()
        .points()
        .as_ptr();

    for resolution in [0.0, 0.001] {
        assert_eq!(
            apply_project_simplification(&mut objects, resolution, CoordinateScale::Normal),
            Ok(())
        );
        assert_eq!(
            objects[0].volumes()[0].layers()[0].expolygons().as_ptr(),
            expolygons_ptr
        );
        assert_eq!(
            objects[0].volumes()[0].layers()[0].expolygons()[0]
                .contour()
                .points()
                .as_ptr(),
            points_ptr
        );
    }
    assert_eq!(objects[0].volumes()[0].layers()[0].expolygons(), &[invalid]);
    assert_eq!(
        apply_project_simplification(&mut objects, 0.0011, CoordinateScale::Normal),
        Err(SliceError::InvalidInput(
            "project simplification polygon coordinate is outside the supported Clipper range"
                .to_owned()
        ))
    );
}

fn assert_scale(
    scale: CoordinateScale,
    equality: ExPolygon,
    above: ExPolygon,
    expected: ExPolygon,
) {
    let object_plan = plan(3, 5, MODES.len());
    let mut objects = vec![PostClosingPrintObject::new(
        object_plan.clone(),
        vec![PostClosingVolume::new(
            7,
            11,
            ProjectVolumeType::SupportEnforcer,
            MODES
                .into_iter()
                .map(|mode| layer(mode, vec![equality.clone(), above.clone()]))
                .collect(),
        )],
    )];
    apply_project_simplification(&mut objects, 0.0011, scale).unwrap();
    assert_eq!(objects[0].plan(), &object_plan);
    for (layer, mode) in objects[0].volumes()[0].layers().iter().zip(MODES) {
        assert_eq!(layer.mode(), mode);
        assert_eq!(layer.expolygons(), std::slice::from_ref(&expected));
    }
}

fn single_object(expolygons: Vec<ExPolygon>) -> Vec<PostClosingPrintObject> {
    vec![PostClosingPrintObject::new(
        plan(1, 2, 1),
        vec![PostClosingVolume::new(
            3,
            4,
            ProjectVolumeType::ModelPart,
            vec![layer(SlicingMode::Regular, expolygons)],
        )],
    )]
}

fn assert_volume(
    volume: &PostClosingVolume,
    source: usize,
    ordinal: u32,
    volume_type: ProjectVolumeType,
) {
    assert_eq!(volume.source_volume_index(), source);
    assert_eq!(volume.ordinal(), ordinal);
    assert_eq!(volume.volume_type(), volume_type);
}

fn plan(source: usize, transform: usize, count: usize) -> PlannedPrintObject {
    PlannedPrintObject {
        source_object_index: source,
        transform_index: transform,
        layers: (0..count)
            .map(|id| PlannedLayer {
                id,
                height: 0.2,
                print_z: (id + 1) as f64 * 0.2,
                slice_z: (id as f64 + 0.5) * 0.2,
            })
            .collect(),
    }
}

fn layer(mode: SlicingMode, expolygons: Vec<ExPolygon>) -> PostClosingLayer {
    PostClosingLayer::new(mode, expolygons)
}

fn expolygon(points: &[(i64, i64)]) -> ExPolygon {
    ExPolygon::new(
        Polygon::new(points.iter().map(|&(x, y)| Point::new(x, y)).collect()),
        Vec::new(),
    )
}
