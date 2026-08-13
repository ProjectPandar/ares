use super::*;
use crate::{
    geometry::{ExPolygon, Point, Polygon},
    project_slice::region_slices::{RegionSurface, RegionSurfaceKind},
};

type PolygonSnapshot = Vec<(i64, i64)>;
type Snapshot = (
    RegionSurfaceKind,
    PolygonSnapshot,
    Vec<PolygonSnapshot>,
    u64,
    u16,
    u64,
    u16,
);

fn polygon_snapshot(polygon: &Polygon) -> PolygonSnapshot {
    polygon
        .points()
        .iter()
        .map(|point| (point.x(), point.y()))
        .collect()
}

fn polygon(x: i64) -> Polygon {
    Polygon::new(vec![
        Point::new(x, 0),
        Point::new(x + 10, 0),
        Point::new(x + 10, 10),
        Point::new(x, 10),
    ])
}

fn surface(kind: RegionSurfaceKind, x: i64) -> RegionSurface {
    RegionSurface::new(kind, ExPolygon::new(polygon(x), Vec::new()))
}

fn metadata_surface(kind: RegionSurfaceKind, x: i64, tag: u16) -> RegionSurface {
    RegionSurface::internal_with_metadata(
        ExPolygon::new(polygon(x), vec![polygon(x + 1)]),
        f64::from(tag) + 0.25,
        tag,
        f64::from(tag) + 0.5,
        tag + 10,
    )
    .clone_with_kind(kind)
}

fn snapshot(surfaces: &[RegionSurface]) -> Vec<Snapshot> {
    surfaces
        .iter()
        .map(|surface| {
            let (kind, expolygon, thickness, layers, angle, extra) = surface.as_parts();
            (
                kind,
                polygon_snapshot(expolygon.contour()),
                expolygon.holes().iter().map(polygon_snapshot).collect(),
                thickness.to_bits(),
                layers,
                angle.to_bits(),
                extra,
            )
        })
        .collect()
}

#[test]
fn task22o70_stably_removes_old_internal_kinds_and_copy_appends_rebuilt_order() {
    let original = vec![
        metadata_surface(RegionSurfaceKind::Top, 70, 1),
        surface(RegionSurfaceKind::Internal, 20),
        metadata_surface(RegionSurfaceKind::Bottom, 10, 2),
        surface(RegionSurfaceKind::InternalSolid, 60),
        metadata_surface(RegionSurfaceKind::BottomBridge, 50, 3),
        metadata_surface(RegionSurfaceKind::InternalBridge, 0, 4),
        metadata_surface(RegionSurfaceKind::InternalVoid, 40, 5),
    ];
    let original_before = snapshot(&original);
    let rebuilt = [
        metadata_surface(RegionSurfaceKind::Internal, 30, 6),
        metadata_surface(RegionSurfaceKind::InternalBridge, -20, 7),
        metadata_surface(RegionSurfaceKind::InternalSolid, 30, 8),
    ];
    let rebuilt_before = snapshot(&rebuilt);
    let rebuilt_contour_ptr = rebuilt[0].as_parts().1.contour().points().as_ptr();
    let output = commit_region_bridge_surfaces(original, &rebuilt);

    assert_eq!(
        output
            .iter()
            .map(|surface| {
                let (kind, expolygon, ..) = surface.as_parts();
                (kind, expolygon.contour().points()[0].x())
            })
            .collect::<Vec<_>>(),
        [
            (RegionSurfaceKind::Top, 70),
            (RegionSurfaceKind::Bottom, 10),
            (RegionSurfaceKind::BottomBridge, 50),
            (RegionSurfaceKind::InternalBridge, 0),
            (RegionSurfaceKind::InternalVoid, 40),
            (RegionSurfaceKind::Internal, 30),
            (RegionSurfaceKind::InternalBridge, -20),
            (RegionSurfaceKind::InternalSolid, 30),
        ]
    );
    assert_eq!(&snapshot(&output)[5..], rebuilt_before);
    assert_eq!(
        snapshot(&output[..5]),
        [
            original_before[0].clone(),
            original_before[2].clone(),
            original_before[4].clone(),
            original_before[5].clone(),
            original_before[6].clone(),
        ]
    );
    assert_eq!(snapshot(&rebuilt), rebuilt_before);
    assert_ne!(
        output[5].as_parts().1.contour().points().as_ptr(),
        rebuilt_contour_ptr
    );
}

#[test]
fn task22o70_empty_original_new_only_original_only_and_duplicates_are_exact() {
    let duplicate = metadata_surface(RegionSurfaceKind::Top, 5, 2);
    let new_only = [duplicate.clone(), duplicate.clone()];
    let output = commit_region_bridge_surfaces(Vec::new(), &new_only);
    assert_eq!(snapshot(&output), snapshot(&new_only));

    let original_only = vec![
        metadata_surface(RegionSurfaceKind::Top, 20, 3),
        surface(RegionSurfaceKind::Internal, 30),
        metadata_surface(RegionSurfaceKind::Top, 20, 3),
    ];
    let output = commit_region_bridge_surfaces(original_only, &[]);
    assert_eq!(output.len(), 2);
    assert_eq!(snapshot(&output)[0], snapshot(&output)[1]);

    assert!(commit_region_bridge_surfaces(Vec::new(), &[]).is_empty());
}

#[test]
fn task22o70_repeatability_matches_complete_surface_snapshot() {
    let make_original = || {
        vec![
            metadata_surface(RegionSurfaceKind::Top, 40, 1),
            surface(RegionSurfaceKind::InternalSolid, 0),
            metadata_surface(RegionSurfaceKind::Bottom, -40, 2),
        ]
    };
    let rebuilt = [metadata_surface(RegionSurfaceKind::Internal, 10, 3)];
    assert_eq!(
        snapshot(&commit_region_bridge_surfaces(make_original(), &rebuilt)),
        snapshot(&commit_region_bridge_surfaces(make_original(), &rebuilt))
    );
}
