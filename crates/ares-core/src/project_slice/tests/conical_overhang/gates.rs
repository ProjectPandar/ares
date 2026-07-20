use crate::{SliceError, geometry::CoordinateScale};

use crate::project_slice::conical_overhang::{
    ConicalOverhangStage, LayerPairClassification, classify_conical_overhang_stage,
    classify_layer_pair, validate_conical_overhang_options,
};

use super::super::support::resolved;
use super::{
    apply_resolved, layer_geometry, object_options, output_rectangle, planned_layers, post_region,
    print_object, rectangle, region_options, square,
};

#[test]
fn task22l_stage_rejects_every_invalid_raw_angle_with_exact_error() {
    let expected = Err(SliceError::InvalidInput(
        "invalid Orca option make_overhang_printable_angle".to_owned(),
    ));
    for angle in [f64::NAN, f64::NEG_INFINITY, f64::INFINITY, -0.1, 90.1] {
        let options = object_options(angle, 0.0, 0.2);
        assert_eq!(validate_conical_overhang_options(&[&options]), expected);
    }
}

#[test]
fn task22l_stage_rejects_raw_hole_and_locks_vector_and_field_precedence() {
    for hole_size in [f64::NAN, f64::NEG_INFINITY, f64::INFINITY, -0.1] {
        let options = object_options(55.0, hole_size, 0.2);
        assert_eq!(
            validate_conical_overhang_options(&[&options]),
            Err(hole_error())
        );
    }

    let invalid_hole_first = object_options(55.0, -0.1, 0.2);
    let invalid_angle_second = object_options(90.1, 0.0, 0.2);
    assert_eq!(
        validate_conical_overhang_options(&[&invalid_hole_first, &invalid_angle_second]),
        Err(hole_error())
    );

    let invalid_both = object_options(-0.1, -0.1, 0.2);
    assert_eq!(
        validate_conical_overhang_options(&[&invalid_both]),
        Err(SliceError::InvalidInput(
            "invalid Orca option make_overhang_printable_angle".to_owned()
        ))
    );
}

#[test]
fn task22l_stage_validation_records_raw_values_without_scaled_precomputation() {
    let positive_zero = object_options(0.0, -0.0, 0.37);
    let negative_zero = object_options(-0.0, 2.5, 0.19);
    let validated = validate_conical_overhang_options(&[&positive_zero, &negative_zero]).unwrap();

    assert_eq!(validated.len(), 2);
    assert_eq!(validated[0].angle_degrees.to_bits(), 0);
    assert_eq!(validated[0].hole_size_mm2.to_bits(), 1_u64 << 63);
    assert_eq!(validated[0].layer_height_mm, 0.37);
    assert_eq!(validated[1].angle_degrees.to_bits(), 1_u64 << 63);
    assert_eq!(validated[1].hole_size_mm2, 2.5);
    assert_eq!(validated[1].layer_height_mm, 0.19);
}

#[test]
fn task22l_stage_orders_empty_then_ninety_then_derivation() {
    let overflow = object_options(0.0, 1.0e30, 1.0e34);
    let validated = validate_conical_overhang_options(&[&overflow]).unwrap()[0];
    assert_eq!(
        classify_conical_overhang_stage(validated, &[], CoordinateScale::Normal),
        Ok(ConicalOverhangStage::Empty)
    );
    let empty_ninety = object_options(90.0, 1.0e30, 1.0e34);
    assert_eq!(
        classify_conical_overhang_stage(
            validate_conical_overhang_options(&[&empty_ninety]).unwrap()[0],
            &[],
            CoordinateScale::LargeBed,
        ),
        Ok(ConicalOverhangStage::Empty)
    );

    let ninety = object_options(90.0, 1.0e30, 1.0e34);
    let validated = validate_conical_overhang_options(&[&ninety]).unwrap()[0];
    assert_eq!(
        classify_conical_overhang_stage(
            validated,
            &planned_layers(&[0.07, 0.31]),
            CoordinateScale::LargeBed,
        ),
        Ok(ConicalOverhangStage::AngleNinety)
    );

    for scale in [CoordinateScale::Normal, CoordinateScale::LargeBed] {
        assert_eq!(
            classify_conical_overhang_stage(
                validate_conical_overhang_options(&[&overflow]).unwrap()[0],
                &planned_layers(&[0.2]),
                scale,
            ),
            Err(geometry_error())
        );
    }
}

#[test]
fn task22l_stage_derives_before_zero_region_and_pair_gates() {
    let options = object_options(55.0, 0.0, 0.2);
    let validated = validate_conical_overhang_options(&[&options]).unwrap()[0];
    let layers = planned_layers(&[0.08, 0.32]);
    assert!(matches!(
        classify_conical_overhang_stage(validated, &layers, CoordinateScale::Normal),
        Ok(ConicalOverhangStage::Geometry(_))
    ));
    assert_eq!(
        classify_layer_pair(&[], 0, 1),
        LayerPairClassification::UpperEmpty
    );

    let upper_empty = vec![post_region(
        10,
        region_options(true, 1, 0, 0.0, 0),
        vec![vec![square(0, 100)], vec![]],
    )];
    assert_eq!(
        classify_layer_pair(&upper_empty, 0, 1),
        LayerPairClassification::UpperEmpty
    );

    let disabled = vec![post_region(
        10,
        region_options(false, 1, 0, 0.0, 0),
        vec![vec![square(0, 100)], vec![square(20, 120)]],
    )];
    assert_eq!(
        classify_layer_pair(&disabled, 0, 1),
        LayerPairClassification::CurrentGated
    );
}

#[test]
fn task22l_stage_pair_gate_is_cardinality_based_and_cross_region() {
    let empty_shape =
        crate::geometry::ExPolygon::new(crate::geometry::Polygon::new(Vec::new()), Vec::new());
    let cardinality_nonempty = vec![post_region(
        10,
        region_options(true, 1, 0, 0.0, 0),
        vec![vec![empty_shape.clone()], vec![empty_shape]],
    )];
    assert_eq!(
        classify_layer_pair(&cardinality_nonempty, 0, 1),
        LayerPairClassification::Geometry
    );

    let cross_region = vec![
        post_region(
            10,
            region_options(true, 1, 0, 0.0, 0),
            vec![vec![square(0, 100)], vec![]],
        ),
        post_region(
            20,
            region_options(false, 1, 0, 0.0, 0),
            vec![vec![], vec![square(200, 300)]],
        ),
    ];
    assert_eq!(
        classify_layer_pair(&cross_region, 0, 1),
        LayerPairClassification::Geometry
    );

    let all_current_empty = vec![
        post_region(
            10,
            region_options(true, 1, 0, 0.0, 0),
            vec![vec![], vec![square(0, 100)]],
        ),
        post_region(20, region_options(true, 1, 0, 0.0, 0), vec![vec![], vec![]]),
    ];
    assert_eq!(
        classify_layer_pair(&all_current_empty, 0, 1),
        LayerPairClassification::CurrentGated
    );

    let upper_empty_and_current_gated = vec![post_region(
        30,
        region_options(false, 1, 0, 0.0, 0),
        vec![vec![square(0, 100)], vec![]],
    )];
    assert_eq!(
        classify_layer_pair(&upper_empty_and_current_gated, 0, 1),
        LayerPairClassification::UpperEmpty
    );

    let enabled_empty_and_disabled_nonempty = vec![
        post_region(
            30,
            region_options(true, 1, 0, 0.0, 0),
            vec![vec![], vec![square(0, 100)]],
        ),
        post_region(
            10,
            region_options(false, 1, 0, 0.0, 0),
            vec![vec![square(200, 300)], vec![]],
        ),
    ];
    assert_eq!(
        classify_layer_pair(&enabled_empty_and_disabled_nonempty, 0, 1),
        LayerPairClassification::CurrentGated
    );

    let geometry_without_merged_eligibility = vec![post_region(
        20,
        region_options(true, 0, 0, 0.0, 0),
        vec![vec![square(0, 100)], vec![square(20, 120)]],
    )];
    assert_eq!(
        classify_layer_pair(&geometry_without_merged_eligibility, 0, 1),
        LayerPairClassification::Geometry
    );
}

#[test]
fn task22l_stage_derived_overflow_precedes_each_later_pair_gate() {
    let options = object_options(0.0, 1.0e30, 1.0e34);
    let validated = validate_conical_overhang_options(&[&options]).unwrap()[0];
    let layers = planned_layers(&[0.2, 0.2]);
    let gated_regions = [
        Vec::new(),
        vec![post_region(
            30,
            region_options(true, 1, 0, 0.0, 0),
            vec![vec![square(0, 100)], vec![]],
        )],
        vec![post_region(
            10,
            region_options(false, 1, 0, 0.0, 0),
            vec![vec![square(0, 100)], vec![square(20, 120)]],
        )],
    ];
    for scale in [CoordinateScale::Normal, CoordinateScale::LargeBed] {
        assert_derived_error_before_pair_gate(
            validated,
            &layers,
            &gated_regions[0],
            scale,
            LayerPairClassification::UpperEmpty,
        );
        assert_derived_error_before_pair_gate(
            validated,
            &layers,
            &gated_regions[1],
            scale,
            LayerPairClassification::UpperEmpty,
        );
        assert_derived_error_before_pair_gate(
            validated,
            &layers,
            &gated_regions[2],
            scale,
            LayerPairClassification::CurrentGated,
        );
    }
}

#[test]
fn task22l_stage_flattens_resolved_options_across_print_instances() {
    let mut first = resolved(0, object_options(90.0, 0.0, 0.2), Vec::new());
    first.print_objects.push(first.print_objects[0].clone());
    let second = resolved(1, object_options(0.0, 0.0, 0.2), Vec::new());
    let mut objects = vec![
        projection_object(0, 0),
        projection_object(0, 1),
        projection_object(1, 0),
    ];
    let before_order = objects
        .iter()
        .map(|object| (object.plan.source_object_index, object.plan.transform_index))
        .collect::<Vec<_>>();

    apply_resolved(&mut objects, &[first, second], CoordinateScale::Normal).unwrap();

    assert_eq!(objects.len(), 3);
    assert_eq!(
        layer_geometry(&objects[0], 0, 0),
        vec![square(0, 1_000_000)]
    );
    assert_eq!(
        layer_geometry(&objects[1], 0, 0),
        vec![square(0, 1_000_000)]
    );
    assert_eq!(
        layer_geometry(&objects[2], 0, 0),
        vec![output_rectangle(0, 0, 1_800_000, 1_000_000)]
    );
    assert_eq!(
        objects
            .iter()
            .map(|object| (object.plan.source_object_index, object.plan.transform_index))
            .collect::<Vec<_>>(),
        before_order
    );
}

fn projection_object(
    source_object_index: usize,
    transform_index: usize,
) -> super::PostRegionPrintObject {
    print_object(
        source_object_index,
        transform_index,
        &[0.2, 0.2],
        vec![post_region(
            0,
            region_options(true, 1, 0, 0.0, 0),
            vec![
                vec![square(0, 1_000_000)],
                vec![rectangle(800_000, 0, 1_800_000, 1_000_000)],
            ],
        )],
    )
}

fn assert_derived_error_before_pair_gate(
    options: crate::project_slice::conical_overhang::ValidatedConicalOverhangOptions,
    layers: &[crate::project_slice::layers::PlannedLayer],
    regions: &[crate::project_slice::region_slices::PostRegion],
    scale: CoordinateScale,
    expected_gate: LayerPairClassification,
) {
    assert_eq!(
        classify_conical_overhang_stage(options, layers, scale),
        Err(geometry_error())
    );
    assert_eq!(classify_layer_pair(regions, 0, 1), expected_gate);
}

fn geometry_error() -> SliceError {
    SliceError::InvalidInput(
        "project conical overhang geometry is nonfinite or outside the supported Clipper range"
            .to_owned(),
    )
}

fn hole_error() -> SliceError {
    SliceError::InvalidInput("invalid Orca option make_overhang_printable_hole_size".to_owned())
}
