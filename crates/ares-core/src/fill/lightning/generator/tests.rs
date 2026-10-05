use super::*;

#[test]
fn fractional_heights_and_coordinate_scales_follow_double_radius_calculation() {
    for (scale, height, expected) in [
        (CoordinateScale::Normal, 0.2000009, 200_000),
        (CoordinateScale::LargeBed, 0.2, 19_999),
        (CoordinateScale::LargeBed, 0.2000009, 20_000),
    ] {
        let generator = Generator::new(
            Vec::new(),
            crate::fill::lightning::generator::InfillSettings {
                layer_height: height,
                infill_line_width: 0.45,
                line_width: 0.45,
                nozzle_diameter: 0.4,
                multiline: 1,
                density_percent: 15.0,
                overhang_angle_deg: 0.0,
                prune_angle_deg: 45.0,
                straightening_angle_deg: 45.0,
                scale,
            },
        )
        .unwrap();
        assert_eq!(generator.wall_supporting_radius, 0);
        assert_eq!(generator.prune_length, expected);
        assert_eq!(generator.straightening_max_distance, expected);
    }
}

#[test]
fn radii_keep_scaled_layer_height_fraction_until_final_cast() {
    let generator = Generator::new(
        Vec::new(),
        crate::fill::lightning::generator::InfillSettings {
            layer_height: 0.2,
            infill_line_width: 0.45,
            line_width: 0.45,
            nozzle_diameter: 0.4,
            multiline: 1,
            density_percent: 15.0,
            overhang_angle_deg: 45.0,
            prune_angle_deg: 45.0,
            straightening_angle_deg: 45.0,
            scale: CoordinateScale::Normal,
        },
    )
    .unwrap();
    assert_eq!(generator.supporting_radius, 3_000_000);
    assert_eq!(generator.wall_supporting_radius, 200_000);
    assert_eq!(generator.prune_length, 200_000);
    assert_eq!(generator.straightening_max_distance, 200_000);
}
