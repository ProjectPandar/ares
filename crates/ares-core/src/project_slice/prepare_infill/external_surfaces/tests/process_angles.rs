use super::helpers::{expolygon, square, surface};
use crate::{
    geometry::CoordinateScale,
    project_slice::{
        prepare_infill::external_surfaces::{
            parameters::ProcessExternalSurfacesConfig, process::process_external_surfaces,
        },
        region_slices::{RegionSurface, RegionSurfaceKind},
    },
};

fn config(
    bridge_angle_degrees: f64,
    relative_bridge_angle: bool,
    model_rotation_radians: f64,
) -> ProcessExternalSurfacesConfig {
    ProcessExternalSurfacesConfig {
        wall_loops: 0,
        perimeter_spacing: 400_000,
        external_width: 420_000,
        external_spacing: 380_000,
        solid_infill_spacing: 10_000,
        bridge_angle_degrees,
        relative_bridge_angle,
        model_rotation_radians,
        sparse_infill_density_percent: 0.0,
        minimum_sparse_infill_area_mm2: 15.0,
        spiral_mode: false,
        scale: CoordinateScale::Normal,
    }
}

fn bridge_with_lower_edge_support() -> Vec<RegionSurface> {
    vec![
        surface(
            RegionSurfaceKind::InternalSolid,
            expolygon(
                &[
                    (400_000, -200_000),
                    (600_000, -200_000),
                    (600_000, 200_000),
                    (400_000, 200_000),
                ],
                Vec::new(),
            ),
            (0.2, 1, -1.0, 0),
        ),
        surface(
            RegionSurfaceKind::BottomBridge,
            square(0, 1_000_000),
            (0.2, 1, -1.0, 0),
        ),
    ]
}

fn output_bridge_angle_bits(surfaces: &[RegionSurface]) -> Vec<u64> {
    surfaces
        .iter()
        .filter(|surface| surface.as_parts().0 == RegionSurfaceKind::BottomBridge)
        .map(|surface| surface.as_parts().4.to_bits())
        .collect()
}

#[test]
fn task22o42_absolute_custom_bridge_angle_includes_model_rotation() {
    let mut surfaces = bridge_with_lower_edge_support();

    process_external_surfaces(&mut surfaces, config(90.0, false, 0.25)).unwrap();

    assert_eq!(
        output_bridge_angle_bits(&surfaces),
        vec![0x3ffd_21fb_5444_2d18]
    );
}

#[test]
fn task22o42_relative_custom_bridge_angle_adds_to_auto_but_not_model_rotation() {
    let mut surfaces = bridge_with_lower_edge_support();

    process_external_surfaces(&mut surfaces, config(30.0, true, 0.375)).unwrap();

    let angles = output_bridge_angle_bits(&surfaces);
    assert_eq!(angles, vec![0x4000_c152_382d_7365]);
    assert_ne!(angles, vec![0x4003_c152_382d_7365]);
}

#[test]
fn task22o42_zero_bridge_angle_uses_auto_orientation_without_model_rotation() {
    let mut surfaces = bridge_with_lower_edge_support();

    process_external_surfaces(&mut surfaces, config(0.0, false, 0.75)).unwrap();

    assert_eq!(
        output_bridge_angle_bits(&surfaces),
        vec![0x3ff9_21fb_5444_2d18]
    );
}
