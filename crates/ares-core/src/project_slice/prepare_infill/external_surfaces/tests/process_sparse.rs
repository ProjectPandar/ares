use super::helpers::{expolygon, square, surface};
use crate::{
    geometry::{ClipperError, CoordinateScale},
    project_slice::{
        prepare_infill::external_surfaces::{
            parameters::ProcessExternalSurfacesConfig, process::process_external_surfaces,
        },
        region_slices::{RegionSurface, RegionSurfaceKind},
    },
};

const OUTSIDE_CLIPPER_RANGE: i64 = 0x4000_0000_0000_0000;

fn config() -> ProcessExternalSurfacesConfig {
    ProcessExternalSurfacesConfig {
        wall_loops: 2,
        perimeter_spacing: 400_000,
        external_width: 420_000,
        external_spacing: 380_000,
        solid_infill_spacing: 380_000,
        bridge_angle_degrees: 0.0,
        relative_bridge_angle: false,
        model_rotation_radians: 0.0,
        sparse_infill_density_percent: 15.0,
        minimum_sparse_infill_area_mm2: 1.0,
        spiral_mode: false,
        scale: CoordinateScale::Normal,
    }
}

fn sparse_square(side: i64) -> Vec<RegionSurface> {
    vec![surface(
        RegionSurfaceKind::Internal,
        square(0, side),
        (0.2, 1, -1.0, 0),
    )]
}

fn process_kind(side: i64, config: ProcessExternalSurfacesConfig) -> RegionSurfaceKind {
    let mut surfaces = sparse_square(side);
    process_external_surfaces(&mut surfaces, config).unwrap();
    assert_eq!(surfaces.len(), 1);
    surfaces[0].as_parts().0
}

#[test]
fn task22o42_sparse_area_equal_to_threshold_promotes_to_solid() {
    assert_eq!(
        process_kind(1_000_000, config()),
        RegionSurfaceKind::InternalSolid
    );
}

#[test]
fn task22o42_sparse_area_above_threshold_remains_sparse() {
    assert_eq!(
        process_kind(1_000_001, config()),
        RegionSurfaceKind::Internal
    );
}

#[test]
fn task22o42_zero_sparse_density_disables_small_area_promotion() {
    let mut disabled = config();
    disabled.sparse_infill_density_percent = 0.0;

    assert_eq!(
        process_kind(1_000_000, disabled),
        RegionSurfaceKind::Internal
    );
}

#[test]
fn task22o42_spiral_mode_disables_small_area_promotion() {
    let mut spiral = config();
    spiral.spiral_mode = true;

    assert_eq!(process_kind(1_000_000, spiral), RegionSurfaceKind::Internal);
}

#[test]
fn task22o42_first_union_error_leaves_only_extracted_solid_geometry_moved() {
    let invalid = expolygon(
        &[
            (OUTSIDE_CLIPPER_RANGE, 0),
            (OUTSIDE_CLIPPER_RANGE, 10),
            (OUTSIDE_CLIPPER_RANGE - 1, 10),
        ],
        Vec::new(),
    );
    let untouched = square(0, 1_000_000);
    let untouched_pointer = untouched.contour().points().as_ptr();
    let mut surfaces = vec![
        surface(
            RegionSurfaceKind::InternalSolid,
            invalid,
            (0.25, 3, 0.75, 4),
        ),
        surface(RegionSurfaceKind::Internal, untouched, (0.3, 5, 1.25, 6)),
    ];

    assert_eq!(
        process_external_surfaces(&mut surfaces, config()),
        Err(ClipperError::CoordinateOutOfRange)
    );

    let (solid_kind, solid_geometry, solid_thickness, solid_layers, solid_angle, solid_extra) =
        surfaces[0].as_parts();
    assert_eq!(solid_kind, RegionSurfaceKind::InternalSolid);
    assert!(solid_geometry.contour().points().is_empty());
    assert!(solid_geometry.holes().is_empty());
    assert_eq!(
        (solid_thickness, solid_layers, solid_angle, solid_extra),
        (0.25, 3, 0.75, 4)
    );

    let (sparse_kind, sparse_geometry, sparse_thickness, sparse_layers, sparse_angle, sparse_extra) =
        surfaces[1].as_parts();
    assert_eq!(sparse_kind, RegionSurfaceKind::Internal);
    assert_eq!(
        sparse_geometry.contour().points().as_ptr(),
        untouched_pointer
    );
    assert_eq!(
        (sparse_thickness, sparse_layers, sparse_angle, sparse_extra),
        (0.3, 5, 1.25, 6)
    );
}
