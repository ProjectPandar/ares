use super::fixture::{holed_square, square, surface};
use crate::{
    geometry::ExPolygon,
    project_slice::{
        prepare_infill::horizontal_shell_propagation::{
            GeometryStep, geometry_events, rebuild, reset_hooks,
        },
        region_slices::{RegionSurface, RegionSurfaceKind},
    },
};

fn metadata_surface(kind: RegionSurfaceKind, expolygon: ExPolygon, extra: u16) -> RegionSurface {
    RegionSurface::internal_with_metadata(expolygon, 2.5, 3, 0.75, extra).clone_with_kind(kind)
}

#[test]
fn rebuild_unions_solids_drops_void_and_preserves_first_external_template() {
    let original = vec![
        surface(RegionSurfaceKind::Internal, square(0, 0, 5_000)),
        surface(RegionSurfaceKind::InternalSolid, square(8_000, 0, 1_000)),
        surface(RegionSurfaceKind::InternalVoid, square(10_000, 0, 1_000)),
        metadata_surface(RegionSurfaceKind::Top, square(12_000, 0, 1_000), 9),
        metadata_surface(RegionSurfaceKind::Top, square(14_000, 0, 1_000), 2),
    ];
    reset_hooks();
    let rebuilt = rebuild::neighbor(
        &original,
        vec![square(1_000, 1_000, 1_000).contour().clone()],
    )
    .unwrap();
    let kinds = rebuilt
        .iter()
        .map(|surface| surface.as_parts().0)
        .collect::<Vec<_>>();

    assert_eq!(kinds[0], RegionSurfaceKind::InternalSolid);
    assert!(rebuilt.iter().any(|surface| {
        let (kind, expolygon, _, _, _, _) = surface.as_parts();
        kind == RegionSurfaceKind::InternalSolid
            && expolygon
                .contour()
                .points()
                .iter()
                .any(|point| point.x() >= 8_000)
    }));
    assert!(kinds.contains(&RegionSurfaceKind::Internal));
    assert!(!kinds.contains(&RegionSurfaceKind::InternalVoid));
    for surface in rebuilt.iter().filter(|surface| {
        matches!(
            surface.as_parts().0,
            RegionSurfaceKind::Internal | RegionSurfaceKind::InternalSolid
        )
    }) {
        let (_, _, thickness, layers, angle, extra) = surface.as_parts();
        assert_eq!(
            (thickness.to_bits(), layers, angle.to_bits(), extra),
            ((-1.0_f64).to_bits(), 1, (-1.0_f64).to_bits(), 0)
        );
    }
    let top = rebuilt
        .iter()
        .filter(|surface| surface.as_parts().0 == RegionSurfaceKind::Top)
        .collect::<Vec<_>>();
    assert_eq!(top.len(), 2);
    assert!(top.iter().all(|surface| surface.as_parts().5 == 9));
    assert_eq!(
        geometry_events(),
        vec![
            GeometryStep::SolidUnion,
            GeometryStep::InternalSafetyDifference,
            GeometryStep::ExternalGroupDifference,
        ]
    );
}

#[test]
fn rebuild_preserves_flat_holed_solid_topology() {
    let holed = holed_square(0, 0, 1_000);
    let mut paths = vec![holed.contour().clone()];
    paths.extend(holed.holes().iter().cloned());
    reset_hooks();
    let rebuilt = rebuild::neighbor(&[], paths).unwrap();
    assert_eq!(rebuilt.len(), 1);
    let (kind, expolygon, _, _, _, _) = rebuilt[0].as_parts();
    assert_eq!(kind, RegionSurfaceKind::InternalSolid);
    assert_eq!(expolygon.holes().len(), 1);
}

#[test]
fn external_merge_key_includes_metadata_but_excludes_extra_perimeters() {
    let original = vec![
        metadata_surface(RegionSurfaceKind::Bottom, square(0, 0, 100), 1),
        RegionSurface::internal_with_metadata(square(200, 0, 100), 3.5, 3, 0.75, 1)
            .clone_with_kind(RegionSurfaceKind::Bottom),
        metadata_surface(RegionSurfaceKind::Bottom, square(400, 0, 100), 8),
    ];
    reset_hooks();
    let rebuilt = rebuild::neighbor(&original, Vec::new()).unwrap();
    assert_eq!(
        geometry_events()
            .iter()
            .filter(|step| **step == GeometryStep::ExternalGroupDifference)
            .count(),
        2
    );
    let extras = rebuilt
        .iter()
        .map(|surface| surface.as_parts().5)
        .collect::<Vec<_>>();
    assert_eq!(extras, vec![1, 1, 1]);
}
