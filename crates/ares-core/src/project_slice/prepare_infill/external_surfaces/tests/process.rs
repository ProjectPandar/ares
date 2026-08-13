use super::helpers::{square, surface};
use crate::{
    geometry::CoordinateScale,
    project_slice::{
        prepare_infill::external_surfaces::{
            parameters::ProcessExternalSurfacesConfig, process::process_external_surfaces,
        },
        region_slices::RegionSurfaceKind,
    },
};

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
        sparse_infill_density_percent: 0.0,
        minimum_sparse_infill_area_mm2: 15.0,
        spiral_mode: false,
        scale: CoordinateScale::Normal,
    }
}

#[test]
fn task22o42_rebuilds_external_surfaces_in_upstream_order() {
    let mut surfaces = vec![
        surface(
            RegionSurfaceKind::Top,
            square(40_000_000, 40_100_000),
            (0.6, 7, 1.0, 9),
        ),
        surface(
            RegionSurfaceKind::Bottom,
            square(30_000_000, 30_100_000),
            (0.5, 8, 1.5, 10),
        ),
        surface(
            RegionSurfaceKind::Internal,
            square(10_000_000, 10_100_000),
            (0.4, 6, 2.0, 8),
        ),
        surface(
            RegionSurfaceKind::BottomBridge,
            square(20_000_000, 20_100_000),
            (0.3, 4, 2.5, 6),
        ),
        surface(
            RegionSurfaceKind::InternalSolid,
            square(0, 100_000),
            (0.2, 5, 3.0, 7),
        ),
    ];

    process_external_surfaces(&mut surfaces, config()).unwrap();

    assert_eq!(
        surfaces
            .iter()
            .map(|surface| surface.as_parts().0)
            .collect::<Vec<_>>(),
        vec![
            RegionSurfaceKind::InternalSolid,
            RegionSurfaceKind::Internal,
            RegionSurfaceKind::BottomBridge,
            RegionSurfaceKind::Bottom,
            RegionSurfaceKind::Top,
        ]
    );
    assert_eq!(
        surfaces
            .iter()
            .map(|surface| surface.as_parts().2)
            .collect::<Vec<_>>(),
        vec![0.6, 0.6, -1.0, -1.0, -1.0]
    );
    assert!(surfaces.iter().all(|surface| {
        let (_, _, _, thickness_layers, _, extra_perimeters) = surface.as_parts();
        thickness_layers == 1 && extra_perimeters == 0
    }));
    assert!(surfaces.iter().all(|surface| {
        let (kind, _, _, _, bridge_angle, _) = surface.as_parts();
        kind == RegionSurfaceKind::BottomBridge || bridge_angle == -1.0
    }));
}

#[test]
fn task22o42_record_without_zone_sources_discards_unhandled_surfaces() {
    let mut surfaces = vec![surface(
        RegionSurfaceKind::InternalVoid,
        square(0, 100_000),
        (0.2, 3, 1.0, 4),
    )];

    process_external_surfaces(&mut surfaces, config()).unwrap();

    assert!(surfaces.is_empty());
}
