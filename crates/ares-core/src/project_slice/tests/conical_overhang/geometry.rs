use crate::geometry::{ClipperError, CoordinateScale, ExPolygon, JoinType, offset_expolygons};

use crate::project_slice::conical_overhang::geometry::{
    merged_layer_footprint, region_participates_in_merged_footprint,
};
use crate::project_slice::conical_overhang::{
    ConicalOverhangStage, classify_conical_overhang_stage, validate_conical_overhang_options,
};

use super::{
    expolygon, object_options, planned_layers, polygon, post_region, region_options, square,
};

#[test]
fn task22l_stage_derivation_matches_fixed_normal_and_large_float_bits() {
    let normal = geometry(55.0, 1.099_511_627_776, 0.2, CoordinateScale::Normal);
    assert_eq!(normal.epsilon_scaled.to_bits(), 0x42c8_0000);
    assert_eq!(normal.distance_scaled.to_bits(), 0xc88b_77b3);
    assert_eq!(normal.hole_area_scaled.to_bits(), 0x5380_0000);

    let large = geometry(55.0, 1.25, 0.2, CoordinateScale::LargeBed);
    assert_eq!(large.epsilon_scaled.to_bits(), 0x4120_0000);
    assert_eq!(large.distance_scaled.to_bits(), 0xc6df_25ec);
    assert_eq!(large.hole_area_scaled.to_bits(), 0x503a_43b7);
}

#[test]
fn task22l_stage_signed_zero_bits_follow_exact_f64_f32_chain() {
    let positive_angle = geometry(0.0, -0.0, 0.2, CoordinateScale::Normal);
    assert_eq!(positive_angle.distance_scaled.to_bits(), 0x8000_0000);
    assert_eq!(positive_angle.hole_area_scaled.to_bits(), 0x8000_0000);

    let negative_angle = geometry(-0.0, 0.0, 0.2, CoordinateScale::Normal);
    assert_eq!(negative_angle.distance_scaled.to_bits(), 0x0000_0000);
    assert_eq!(negative_angle.hole_area_scaled.to_bits(), 0x0000_0000);
}

#[test]
fn task22l_stage_huge_nominal_height_can_stay_finite_before_independent_overflows() {
    for scale in [CoordinateScale::Normal, CoordinateScale::LargeBed] {
        let finite = geometry(0.0, 0.0, 1.0e34, scale);
        assert_eq!(finite.distance_scaled.to_bits(), 0x8000_0000);

        for (angle, hole_size) in [(0.0, 1.0e30), (55.0, 0.0)] {
            let options = object_options(angle, hole_size, 1.0e34);
            let validated = validate_conical_overhang_options(&[&options]).unwrap()[0];
            assert_eq!(
                classify_conical_overhang_stage(validated, &planned_layers(&[0.2]), scale),
                Err(crate::SliceError::InvalidInput(
                    "project conical overhang geometry is nonfinite or outside the supported Clipper range"
                        .to_owned()
                ))
            );
        }
    }
}

#[test]
fn task22l_stage_uses_nominal_height_not_planned_layer_heights() {
    let options = object_options(45.0, 0.0, 0.2);
    let validated = validate_conical_overhang_options(&[&options]).unwrap()[0];
    let stage = classify_conical_overhang_stage(
        validated,
        &planned_layers(&[0.08, 0.32, 0.11]),
        CoordinateScale::Normal,
    )
    .unwrap();
    let ConicalOverhangStage::Geometry(geometry) = stage else {
        panic!("nonempty non-90 object must derive geometry");
    };
    assert_eq!(
        geometry.distance_scaled.to_bits(),
        (-200_000.0_f32).to_bits()
    );
}

#[test]
fn task22l_stage_each_merged_eligibility_field_is_independent() {
    for options in [
        region_options(false, 1, 0, 0.0, 0),
        region_options(false, 0, 1, 0.0, 0),
        region_options(false, 0, 0, 0.01, 0),
        region_options(false, 0, 0, 0.0, 1),
    ] {
        assert!(region_participates_in_merged_footprint(&options));
    }
    assert!(!region_participates_in_merged_footprint(&region_options(
        true, 0, 0, 0.0, 0,
    )));
    for options in [
        region_options(true, -1, 0, 0.0, 0),
        region_options(true, 0, -1, 0.0, 0),
        region_options(true, 0, 0, -0.01, 0),
        region_options(true, 0, 0, 0.0, -1),
    ] {
        assert!(!region_participates_in_merged_footprint(&options));
    }
}

#[test]
fn task22l_stage_merged_footprint_unions_arbitrary_nonrectangular_regions() {
    let regions = vec![
        post_region(
            30,
            region_options(true, 1, 0, 0.0, 0),
            vec![vec![expolygon(&[
                (0, 0),
                (100, 0),
                (100, 40),
                (40, 40),
                (40, 100),
                (0, 100),
            ])]],
        ),
        post_region(
            10,
            region_options(false, 0, 1, 0.0, 0),
            vec![vec![expolygon(&[(30, 30), (80, 30), (80, 80), (30, 80)])]],
        ),
        post_region(
            20,
            region_options(false, 0, 0, 0.0, 0),
            vec![vec![square(1_000, 1_100)]],
        ),
    ];

    let footprint = merged_layer_footprint(&regions, 0, 10.0).unwrap();
    assert_eq!(footprint.len(), 1);
    assert!(footprint[0].holes().is_empty());
    assert_eq!(footprint[0].contour().area(), 12_400.0);
    assert_eq!(
        footprint,
        vec![expolygon(&[
            (110, 50),
            (90, 50),
            (90, 90),
            (50, 90),
            (50, 110),
            (-10, 110),
            (-10, -10),
            (110, -10),
        ])]
    );
}

#[test]
fn task22l_stage_merged_footprint_freezes_holes_and_provider_order() {
    let donut = ExPolygon::new(
        polygon(&[(0, 0), (200, 0), (200, 200), (0, 200)]),
        vec![polygon(&[(50, 50), (50, 150), (150, 150), (150, 50)])],
    );
    let regions = vec![post_region(
        70,
        region_options(false, 1, 0, 0.0, 0),
        vec![vec![donut, square(300, 400)]],
    )];

    assert_eq!(
        merged_layer_footprint(&regions, 0, 10.0),
        Ok(vec![
            expolygon(&[(410, 410), (290, 410), (290, 290), (410, 290)]),
            ExPolygon::new(
                polygon(&[(210, 210), (-10, 210), (-10, -10), (210, -10)]),
                vec![polygon(&[(60, 60), (60, 140), (140, 140), (140, 60)])],
            ),
        ])
    );
}

#[test]
fn task22l_stage_merged_footprint_uses_miter_limit_three_at_acute_corners() {
    let regions = vec![post_region(
        10,
        region_options(false, 1, 0, 0.0, 0),
        vec![vec![expolygon(&[(0, 0), (100, 0), (100, 100)])]],
    )];

    assert_eq!(
        merged_layer_footprint(&regions, 0, 10.0),
        Ok(vec![expolygon(&[(110, 124), (-24, -10), (110, -10)])])
    );
}

#[test]
fn task22l_stage_merged_footprint_propagates_clipper_coordinate_errors() {
    const HI_RANGE: i64 = 0x3fff_ffff_ffff_ffff;
    let regions = vec![post_region(
        90,
        region_options(true, 1, 0, 0.0, 0),
        vec![vec![expolygon(&[
            (HI_RANGE - 16_384, 0),
            (HI_RANGE - 8_192, 0),
            (HI_RANGE - 8_192, 8_192),
            (HI_RANGE - 16_384, 8_192),
        ])]],
    )];
    assert_eq!(
        merged_layer_footprint(&regions, 0, 16_384.0),
        Err(ClipperError::CoordinateOutOfRange)
    );
}

#[test]
fn task22l_stage_derived_negative_offset_can_completely_erode_at_both_scales() {
    for scale in [CoordinateScale::Normal, CoordinateScale::LargeBed] {
        let derived = geometry(55.0, 0.0, 0.001, scale);
        assert!(derived.distance_scaled < -50.0);
        assert_eq!(
            offset_expolygons(
                &[square(0, 100)],
                derived.distance_scaled,
                JoinType::Miter,
                3.0,
            ),
            Ok(Vec::new())
        );
    }
}

fn geometry(
    angle: f64,
    hole_size: f64,
    layer_height: f64,
    scale: CoordinateScale,
) -> crate::project_slice::conical_overhang::geometry::ConicalOverhangGeometry {
    let options = object_options(angle, hole_size, layer_height);
    let validated = validate_conical_overhang_options(&[&options]).unwrap()[0];
    let stage = classify_conical_overhang_stage(validated, &planned_layers(&[0.2]), scale).unwrap();
    let ConicalOverhangStage::Geometry(geometry) = stage else {
        panic!("one-layer non-90 object must derive geometry");
    };
    geometry
}
