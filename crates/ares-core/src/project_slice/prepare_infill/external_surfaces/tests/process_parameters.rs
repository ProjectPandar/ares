use crate::{
    geometry::CoordinateScale,
    project_slice::prepare_infill::external_surfaces::parameters::{
        ProcessExternalSurfacesConfig, derive,
    },
};

fn config(
    wall_loops: i32,
    flow: [i64; 4],
    scale: CoordinateScale,
) -> ProcessExternalSurfacesConfig {
    let [
        perimeter_spacing,
        external_width,
        external_spacing,
        solid_infill_spacing,
    ] = flow;
    ProcessExternalSurfacesConfig {
        wall_loops,
        perimeter_spacing,
        external_width,
        external_spacing,
        solid_infill_spacing,
        bridge_angle_degrees: 0.0,
        relative_bridge_angle: false,
        model_rotation_radians: 0.0,
        sparse_infill_density_percent: 15.0,
        minimum_sparse_infill_area_mm2: 15.0,
        spiral_mode: false,
        scale,
    }
}

#[test]
fn task22o42_derives_exact_ksr_first_layer_parameters() {
    let parameters = derive(config(
        2,
        [457_079, 500_000, 457_079, 457_079],
        CoordinateScale::Normal,
    ));

    assert_eq!(parameters.expansion_min, 457_079.0);
    assert_eq!(parameters.expansion_min.to_bits(), 0x48df_2ee0);
    assert_eq!(parameters.expansion_top, 1_646_368.0);
    assert_eq!(
        [
            parameters.expansion_top.to_bits(),
            parameters.expansion_bottom.to_bits(),
            parameters.expansion_bottom_bridge.to_bits(),
        ],
        [0x49c8_f900; 3]
    );
    assert_eq!(parameters.expansion_step, 100_000.0);
    assert_eq!(parameters.expansion_step.to_bits(), 0x47c3_5000);
    assert_eq!(parameters.closing_radius.to_bits(), 0x4827_8e01);
    assert_eq!(parameters.minimum_sparse_area, 15_000_000_000_000.0);
    assert_eq!(
        parameters.minimum_sparse_area.to_bits(),
        0x42ab_48eb_57e0_0000
    );
}

#[test]
fn task22o42_derives_exact_ksr_later_layer_parameters() {
    let parameters = derive(config(
        2,
        [407_079, 419_999, 377_079, 377_079],
        CoordinateScale::Normal,
    ));

    assert_eq!(parameters.expansion_min, 407_079.0);
    assert_eq!(parameters.expansion_min.to_bits(), 0x48c6_c4e0);
    assert_eq!(parameters.expansion_top, 1_405_951.0);
    assert_eq!(
        [
            parameters.expansion_top.to_bits(),
            parameters.expansion_bottom.to_bits(),
            parameters.expansion_bottom_bridge.to_bits(),
        ],
        [0x49ab_9ff8; 3]
    );
    assert_eq!(parameters.expansion_step, 100_000.0);
    assert_eq!(parameters.expansion_step.to_bits(), 0x47c3_5000);
    assert_eq!(parameters.closing_radius.to_bits(), 0x480a_3a81);
    assert_eq!(parameters.minimum_sparse_area, 15_000_000_000_000.0);
    assert_eq!(
        parameters.minimum_sparse_area.to_bits(),
        0x42ab_48eb_57e0_0000
    );
}

#[test]
fn task22o42_derives_exact_nonzero_wall_large_bed_parameters() {
    let parameters = derive(config(
        2,
        [45_708, 50_000, 45_708, 45_708],
        CoordinateScale::LargeBed,
    ));

    assert_eq!(parameters.expansion_min, 45_708.0);
    assert_eq!(parameters.expansion_min.to_bits(), 0x4732_8c00);
    assert_eq!(parameters.expansion_top, 164_637.1);
    assert_eq!(parameters.expansion_top.to_bits(), 0x4820_c746);
    assert_eq!(parameters.expansion_bottom, 164_637.1);
    assert_eq!(parameters.expansion_bottom.to_bits(), 0x4820_c746);
    assert_eq!(parameters.expansion_bottom_bridge, 164_637.1);
    assert_eq!(parameters.expansion_bottom_bridge.to_bits(), 0x4820_c746);
    assert_eq!(parameters.expansion_step, 10_000.0);
    assert_eq!(parameters.expansion_step.to_bits(), 0x461c_4000);
    assert_eq!(parameters.closing_radius, 17_157.639);
    assert_eq!(parameters.closing_radius.to_bits(), 0x4686_0b47);
    assert_eq!(parameters.minimum_sparse_area, 149_999_999_999.999_97);
    assert_eq!(
        parameters.minimum_sparse_area.to_bits(),
        0x4241_7659_2dff_ffff
    );
}

#[test]
fn task22o42_zero_walls_use_scale_specific_epsilon_expansion() {
    let normal = derive(config(
        0,
        [457_079, 500_000, 457_079, 457_079],
        CoordinateScale::Normal,
    ));
    let large_bed = derive(config(
        0,
        [45_708, 50_000, 45_708, 45_708],
        CoordinateScale::LargeBed,
    ));

    assert_eq!(normal.expansion_min, 100.0);
    assert_eq!(normal.expansion_min.to_bits(), 0x42c8_0000);
    assert_eq!(normal.expansion_top, 141.421_36);
    assert_eq!(
        [
            normal.expansion_top.to_bits(),
            normal.expansion_bottom.to_bits(),
            normal.expansion_bottom_bridge.to_bits(),
        ],
        [0x430d_6bde; 3]
    );

    assert_eq!(large_bed.expansion_min, 10.0);
    assert_eq!(large_bed.expansion_min.to_bits(), 0x4120_0000);
    assert_eq!(large_bed.expansion_top, 14.142_136);
    assert_eq!(
        [
            large_bed.expansion_top.to_bits(),
            large_bed.expansion_bottom.to_bits(),
            large_bed.expansion_bottom_bridge.to_bits(),
        ],
        [0x4162_4630; 3]
    );
}

#[test]
fn task22o42_scales_step_and_minimum_sparse_area_twice() {
    let normal = derive(config(0, [0; 4], CoordinateScale::Normal));
    let large_bed = derive(config(0, [0; 4], CoordinateScale::LargeBed));

    assert_eq!(normal.expansion_step, 100_000.0);
    assert_eq!(normal.expansion_step.to_bits(), 0x47c3_5000);
    assert_eq!(normal.minimum_sparse_area, 15_000_000_000_000.0);
    assert_eq!(normal.minimum_sparse_area.to_bits(), 0x42ab_48eb_57e0_0000);

    assert_eq!(large_bed.expansion_step, 10_000.0);
    assert_eq!(large_bed.expansion_step.to_bits(), 0x461c_4000);
    assert_eq!(large_bed.minimum_sparse_area, 149_999_999_999.999_97);
    assert_eq!(
        large_bed.minimum_sparse_area.to_bits(),
        0x4241_7659_2dff_ffff
    );
}
