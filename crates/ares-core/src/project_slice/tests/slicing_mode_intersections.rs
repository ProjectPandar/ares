use crate::{
    OrcaFloat, OrcaInt, Point3d, ProcessSlicingMode, ProjectVolume, ProjectVolumeType,
    RegionOptions, SliceError, Transform3d, mesh_slicer::SlicingMode,
    project::effective_config::types::ResolvedModelPartCandidate,
};

use super::{
    super::{
        chained_intersections::chain_project_intersections,
        looped_intersections::loop_project_intersections,
        slicing_mode_intersections::{
            SlicingModePrintObject, apply_project_slicing_modes, map_process_slicing_mode,
            spiral_bottom_threshold,
        },
    },
    raw_support::{intersections, mesh_volume, ordinal_gap_object, planned_layers},
    support::{identity_resolved, object, region, resolved_object},
};

#[test]
fn task22e_external_modes_map_exhaustively_to_internal_modes() {
    let mappings = [
        (ProcessSlicingMode::Regular, SlicingMode::Regular),
        (ProcessSlicingMode::EvenOdd, SlicingMode::EvenOdd),
        (ProcessSlicingMode::CloseHoles, SlicingMode::Positive),
    ];
    assert_eq!(
        mappings
            .map(|(external, _)| external)
            .map(map_process_slicing_mode),
        mappings.map(|(_, internal)| internal)
    );
    for (external, internal) in mappings {
        let objects = slicing_mode_fixture(external, false, bottom_region(-1, f64::NAN)).unwrap();
        assert!(
            objects[0]
                .volumes()
                .iter()
                .flat_map(|volume| volume.layers())
                .all(|layer| layer.mode() == internal)
        );
    }
}

#[test]
fn task22e_spiral_threshold_matches_layer_count_thickness_zero_and_no_clamp_vectors() {
    let plan = planned_layers(0, 0, &[(9.0, 0.10), (8.0, 0.30), (7.0, 0.50), (6.0, 0.70)]);
    for (bottom, thickness, expected) in [(1, 0.0, 1), (1, 0.61, 3), (5, 0.0, 5), (0, 0.0, 0)] {
        assert_eq!(
            spiral_bottom_threshold(&plan.layers, &bottom_region(bottom, thickness)).unwrap(),
            expected
        );
    }
}

#[test]
fn task22e_spiral_threshold_uses_slice_z_f32_strict_equality_and_rounding() {
    let equality = planned_layers(0, 0, &[(0.0, 0.4999), (-10.0, 0.5), (-20.0, 0.5001)]);
    assert_eq!(
        spiral_bottom_threshold(&equality.layers, &bottom_region(0, 0.5001)).unwrap(),
        1
    );

    let rounded = planned_layers(0, 0, &[(-100.0, 0.5 - 1e-9)]);
    assert_eq!(
        spiral_bottom_threshold(&rounded.layers, &bottom_region(0, 0.5001)).unwrap(),
        0
    );
}

#[test]
fn task22e_spiral_threshold_rejects_invalid_consumed_bottom_options() {
    let plan = planned_layers(0, 0, &[(0.2, 0.1)]);
    assert_eq!(
        spiral_bottom_threshold(&plan.layers, &bottom_region(-1, 0.0)),
        Err(invalid_option("bottom_shell_layers"))
    );
    for thickness in [-0.1, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(
            spiral_bottom_threshold(&plan.layers, &bottom_region(0, thickness)),
            Err(invalid_option("bottom_shell_thickness"))
        );
    }
    assert!(
        slicing_mode_fixture(
            ProcessSlicingMode::EvenOdd,
            false,
            bottom_region(-1, f64::NAN),
        )
        .is_ok()
    );
    assert_eq!(
        slicing_mode_fixture(
            ProcessSlicingMode::EvenOdd,
            true,
            bottom_region(-1, f64::NAN),
        )
        .err(),
        Some(invalid_option("bottom_shell_layers"))
    );
}

#[test]
fn task22e_spiral_is_model_part_only_and_uses_each_source_index_region() {
    use ProjectVolumeType::{ModelPart, NegativeVolume, ParameterModifier};

    let off =
        slicing_mode_fixture(ProcessSlicingMode::EvenOdd, false, bottom_region(1, 0.0)).unwrap();
    assert!(
        off[0]
            .volumes()
            .iter()
            .flat_map(|volume| volume.layers())
            .all(|layer| layer.mode() == SlicingMode::EvenOdd)
    );

    let on =
        slicing_mode_fixture(ProcessSlicingMode::EvenOdd, true, bottom_region(1, 0.0)).unwrap();
    assert_eq!(on[0].plan().source_object_index, 0);
    let volumes = on[0].volumes();
    assert_eq!(volumes.len(), 4);
    assert_eq!(
        volumes
            .iter()
            .map(|volume| volume.volume_type())
            .collect::<Vec<_>>(),
        [ModelPart, NegativeVolume, ModelPart, ParameterModifier]
    );
    assert_eq!(
        volumes
            .iter()
            .map(|volume| (volume.source_volume_index(), volume.ordinal()))
            .collect::<Vec<_>>(),
        [(1, 2), (2, 3), (3, 4), (4, 5)]
    );
    assert_eq!(
        modes(&volumes[0]),
        [
            SlicingMode::EvenOdd,
            SlicingMode::PositiveLargestContour,
            SlicingMode::PositiveLargestContour,
            SlicingMode::PositiveLargestContour,
        ]
    );
    assert_eq!(modes(&volumes[1]), [SlicingMode::EvenOdd; 4]);
    assert_eq!(
        modes(&volumes[2]),
        [
            SlicingMode::EvenOdd,
            SlicingMode::EvenOdd,
            SlicingMode::EvenOdd,
            SlicingMode::PositiveLargestContour,
        ]
    );
    assert_eq!(modes(&volumes[3]), [SlicingMode::EvenOdd; 4]);

    let positive =
        slicing_mode_fixture(ProcessSlicingMode::CloseHoles, true, bottom_region(1, 0.0)).unwrap();
    assert_eq!(
        positive[0].volumes()[0].layers()[0].mode(),
        SlicingMode::Positive
    );
}

#[test]
fn task22e_project_largest_mode_adapts_to_positive_without_discarding_contours() {
    let objects =
        slicing_mode_fixture(ProcessSlicingMode::EvenOdd, true, bottom_region(1, 0.0)).unwrap();
    let layers = objects[0].volumes()[0].layers();
    assert_eq!(layers[0].looped_layer().polygons().len(), 2);
    assert!(
        layers[0]
            .looped_layer()
            .polygons()
            .iter()
            .all(|polygon| signed_area(polygon.points()) < 0.0)
    );
    for layer in &layers[1..] {
        assert_eq!(layer.mode(), SlicingMode::PositiveLargestContour);
        assert_eq!(layer.looped_layer().polygons().len(), 2);
        assert!(
            layer
                .looped_layer()
                .polygons()
                .iter()
                .all(|polygon| signed_area(polygon.points()) > 0.0)
        );
    }
}

#[test]
fn task22e_source_volume_index_survives_raw_chained_and_looped_ownership() {
    use ProjectVolumeType::{ModelPart, NegativeVolume, ParameterModifier};

    let source_objects = vec![ordinal_gap_object()];
    let resolved = vec![identity_resolved(0)];
    let plans = vec![planned_layers(0, 0, &[(0.2, 0.5)])];
    let expected = vec![
        (2, 2, ModelPart),
        (3, 3, ParameterModifier),
        (5, 5, NegativeVolume),
    ];

    let raw = intersections(&source_objects, &resolved, plans).unwrap();
    assert_eq!(
        raw[0]
            .volumes()
            .iter()
            .map(|volume| {
                (
                    volume.source_volume_index(),
                    volume.ordinal(),
                    volume.volume_type(),
                )
            })
            .collect::<Vec<_>>(),
        expected
    );

    let chained = chain_project_intersections(raw);
    assert_eq!(
        chained[0]
            .volumes()
            .iter()
            .map(|volume| {
                (
                    volume.source_volume_index(),
                    volume.ordinal(),
                    volume.volume_type(),
                )
            })
            .collect::<Vec<_>>(),
        expected
    );

    let looped = loop_project_intersections(chained, 2_000_000);
    assert_eq!(
        looped[0]
            .volumes()
            .iter()
            .map(|volume| {
                (
                    volume.source_volume_index(),
                    volume.ordinal(),
                    volume.volume_type(),
                )
            })
            .collect::<Vec<_>>(),
        expected
    );
}

fn slicing_mode_fixture(
    base_mode: ProcessSlicingMode,
    spiral_mode: bool,
    first_region: RegionOptions,
) -> Result<Vec<SlicingModePrintObject>, SliceError> {
    use ProjectVolumeType::{ModelPart, NegativeVolume, ParameterModifier, SupportBlocker};

    let source_objects = vec![object(
        "slicing-mode.model",
        10,
        vec![
            tetra_volume(9, SupportBlocker, &[20.0], true),
            tetra_volume(10, ModelPart, &[0.0, 4.0], true),
            tetra_volume(11, NegativeVolume, &[0.0], true),
            tetra_volume(12, ModelPart, &[8.0], false),
            tetra_volume(13, ParameterModifier, &[0.0], true),
        ],
        &[Transform3d::IDENTITY],
    )];
    let plan = planned_layers(0, 0, &[(0.2, 0.10), (0.4, 0.30), (0.6, 0.50), (0.8, 0.70)]);
    let mut resolved = resolved_object(0, &[Transform3d::IDENTITY]);
    resolved.object.slicing_mode = base_mode;
    resolved.layer_candidates[0].model_parts = vec![
        ResolvedModelPartCandidate {
            volume_index: 3,
            region: bottom_region(0, 0.61),
        },
        ResolvedModelPartCandidate {
            volume_index: 2,
            region: bottom_region(-1, f64::NAN),
        },
        ResolvedModelPartCandidate {
            volume_index: 1,
            region: first_region,
        },
        ResolvedModelPartCandidate {
            volume_index: 4,
            region: bottom_region(-1, f64::NAN),
        },
    ];
    let raw = intersections(&source_objects, std::slice::from_ref(&resolved), vec![plan]).unwrap();
    let chained = chain_project_intersections(raw);
    let looped = loop_project_intersections(chained, 2_000_000);
    apply_project_slicing_modes(looped, &[resolved], spiral_mode)
}

fn tetra_volume(
    id: u32,
    volume_type: ProjectVolumeType,
    x_offsets: &[f64],
    clockwise: bool,
) -> ProjectVolume {
    let mut vertices = Vec::new();
    let mut triangles = Vec::new();
    for &x in x_offsets {
        let base = u32::try_from(vertices.len()).unwrap();
        vertices.extend([
            Point3d::new(x, 0.0, 0.0),
            Point3d::new(x + 2.0, 0.0, 0.0),
            Point3d::new(x, 2.0, 0.0),
            Point3d::new(x, 0.0, 2.0),
        ]);
        let faces = if clockwise {
            [[0, 1, 2], [0, 3, 1], [0, 2, 3], [1, 3, 2]]
        } else {
            [[0, 2, 1], [0, 1, 3], [0, 3, 2], [1, 2, 3]]
        };
        triangles.extend(faces.map(|face| face.map(|vertex| base + vertex)));
    }
    mesh_volume(id, volume_type, vertices, triangles, Transform3d::IDENTITY)
}

fn bottom_region(bottom_shell_layers: i32, bottom_shell_thickness: f64) -> RegionOptions {
    let mut region = region();
    region.bottom_shell_layers = OrcaInt(bottom_shell_layers);
    region.bottom_shell_thickness = OrcaFloat(bottom_shell_thickness);
    region
}

fn modes(
    volume: &super::super::slicing_mode_intersections::SlicingModeVolumeIntersections,
) -> Vec<SlicingMode> {
    volume.layers().iter().map(|layer| layer.mode()).collect()
}

fn invalid_option(key: &str) -> SliceError {
    SliceError::InvalidInput(format!("invalid Orca option {key}"))
}

fn signed_area(points: &[crate::geometry::Point]) -> f64 {
    let mut area = 0.0;
    for index in 0..points.len() {
        let previous = if index == 0 {
            points.len() - 1
        } else {
            index - 1
        };
        area += points[previous].x() as f64 * points[index].y() as f64
            - points[previous].y() as f64 * points[index].x() as f64;
    }
    0.5 * area
}
