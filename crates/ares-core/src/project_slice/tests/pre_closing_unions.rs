use crate::{
    ProjectVolumeType, SliceError, Transform3d,
    geometry::{FillRule, Point, Polygon},
    mesh_slicer::SlicingMode,
};

use super::{
    super::{
        chained_intersections::chain_project_intersections,
        looped_intersections::loop_project_intersections,
        pre_closing_unions::{
            apply_project_pre_closing_unions, fill_rule_for_mode, union_layer_polygons,
        },
        slicing_mode_intersections::{SlicingModePrintObject, apply_project_slicing_modes},
    },
    raw_support::{intersections, planned_layers},
    support::{identity_resolved, object, project_volume},
};

#[test]
fn task22f_pre_closing_sorts_volume_ordinals_and_preserves_owned_empty_slots() {
    use ProjectVolumeType::{ModelPart, NegativeVolume, ParameterModifier};

    let mut input = synthetic_input();
    assert_eq!(ordinals(&input[0]), [2, 3, 5]);
    input[0].volumes_mut().rotate_right(1);
    assert_eq!(ordinals(&input[0]), [5, 2, 3]);

    let output = apply_project_pre_closing_unions(input).unwrap();
    assert_eq!(output.len(), 1);
    assert_eq!(output[0].plan().source_object_index, 0);
    assert_eq!(output[0].plan().transform_index, 0);
    assert_eq!(output[0].plan().layers.len(), 2);
    assert_eq!(
        output[0]
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
        [
            (2, 2, ModelPart),
            (5, 3, ParameterModifier),
            (7, 5, NegativeVolume),
        ]
    );
    for volume in output[0].volumes() {
        assert_eq!(volume.layers().len(), 2);
        for layer in volume.layers() {
            assert_eq!(layer.mode(), SlicingMode::Regular);
            assert!(layer.expolygons().is_empty());
        }
    }
}

#[test]
fn task22f_pre_closing_maps_every_mode_and_retains_all_largest_mode_expolygons() {
    assert_eq!(
        [
            SlicingMode::Regular,
            SlicingMode::EvenOdd,
            SlicingMode::Positive,
            SlicingMode::PositiveLargestContour,
        ]
        .map(fill_rule_for_mode),
        [
            FillRule::NonZero,
            FillRule::EvenOdd,
            FillRule::NonZero,
            FillRule::Positive,
        ]
    );

    let disjoint = [square(0, 0, 40), square(100, 0, 40)];
    let output = union_layer_polygons(SlicingMode::PositiveLargestContour, &disjoint).unwrap();
    assert_eq!(output.len(), 2);
    assert_eq!(
        output
            .iter()
            .map(|expolygon| expolygon.contour())
            .collect::<Vec<_>>(),
        [
            &polygon(&[(40, 40), (0, 40), (0, 0), (40, 0)]),
            &polygon(&[(140, 40), (100, 40), (100, 0), (140, 0)]),
        ]
    );

    let negative = polygon(&[(0, 0), (0, 40), (40, 40), (40, 0)]);
    assert_eq!(
        union_layer_polygons(SlicingMode::Regular, std::slice::from_ref(&negative))
            .unwrap()
            .len(),
        1
    );
    assert!(
        union_layer_polygons(
            SlicingMode::PositiveLargestContour,
            std::slice::from_ref(&negative),
        )
        .unwrap()
        .is_empty()
    );
}

#[test]
fn task22f_pre_closing_maps_external_coordinate_failure_once_without_option_text() {
    let outside = 0x4000_0000_0000_0000_i64;
    let input = polygon(&[(outside, 0), (0, 1), (0, 0)]);
    assert_eq!(
        union_layer_polygons(SlicingMode::Regular, &[input]),
        Err(SliceError::InvalidInput(
            "project pre-closing polygon coordinate is outside the supported Clipper range"
                .to_owned()
        ))
    );
}

#[test]
#[should_panic(expected = "duplicate pre-closing volume ordinal")]
fn task22f_pre_closing_duplicate_ordinal_is_an_internal_invariant_failure() {
    let mut input = synthetic_input();
    let duplicate = input[0].volumes()[0].ordinal();
    input[0].volumes_mut()[1].set_ordinal_for_test(duplicate);
    let _ = apply_project_pre_closing_unions(input);
}

fn synthetic_input() -> Vec<SlicingModePrintObject> {
    use ProjectVolumeType::{
        ModelPart, NegativeVolume, ParameterModifier, SupportBlocker, SupportEnforcer,
    };

    let volumes = vec![
        project_volume("pre-closing.model", 900, ModelPart, false, false),
        project_volume("pre-closing.model", 17, SupportBlocker, true, false),
        project_volume("pre-closing.model", 500, ModelPart, true, false),
        project_volume("pre-closing.model", 3, ParameterModifier, false, false),
        project_volume("pre-closing.model", 700, NegativeVolume, false, false),
        project_volume("pre-closing.model", 44, ParameterModifier, true, false),
        project_volume("pre-closing.model", 5, SupportEnforcer, true, false),
        project_volume("pre-closing.model", 100, NegativeVolume, true, false),
    ];
    let source = object("pre-closing.model", 10, volumes, &[Transform3d::IDENTITY]);
    let resolved = identity_resolved(0);
    let plan = planned_layers(0, 0, &[(0.2, 0.1), (0.4, 0.3)]);
    let raw = intersections(&[source], std::slice::from_ref(&resolved), vec![plan]).unwrap();
    let chained = chain_project_intersections(raw);
    let looped = loop_project_intersections(chained, 2_000_000);
    apply_project_slicing_modes(looped, &[resolved], false).unwrap()
}

fn ordinals(object: &SlicingModePrintObject) -> Vec<u32> {
    object
        .volumes()
        .iter()
        .map(|volume| volume.ordinal())
        .collect()
}

fn square(x: i64, y: i64, size: i64) -> Polygon {
    polygon(&[(x, y), (x + size, y), (x + size, y + size), (x, y + size)])
}

fn polygon(points: &[(i64, i64)]) -> Polygon {
    Polygon::new(points.iter().map(|&(x, y)| Point::new(x, y)).collect())
}
