mod topology;

use super::{
    GeometryStep,
    assign::{commit, stage_record},
    fail_geometry_at, geometry_events, range_error, reset_geometry_hooks,
};
use crate::{
    geometry::{ExPolygon, Point, Polygon},
    project_slice::{
        prepare_infill::{
            surface_type_detection::types::PreparedSurfaceTypeRecord,
            vertical_shell_filtering::types::VerticalShellTinyFilter,
        },
        region_slices::{RegionSurface, RegionSurfaceKind},
    },
};

fn rectangle(min_x: i64, min_y: i64, max_x: i64, max_y: i64) -> ExPolygon {
    ExPolygon::new(
        Polygon::new(vec![
            Point::new(min_x, min_y),
            Point::new(max_x, min_y),
            Point::new(max_x, max_y),
            Point::new(min_x, max_y),
        ]),
        Vec::new(),
    )
}

fn surface(kind: RegionSurfaceKind, x: i64) -> RegionSurface {
    RegionSurface::internal_with_metadata(rectangle(x, 0, x + 100, 100), 2.5, 3, 0.75, 4)
        .clone_with_kind(kind)
}

fn record(fill_surfaces: Vec<RegionSurface>) -> PreparedSurfaceTypeRecord {
    PreparedSurfaceTypeRecord {
        perimeters: Vec::new(),
        thin_fills: Vec::new(),
        slices: Vec::new(),
        fill_surfaces,
        fill_expolygons: Vec::new(),
        fill_no_overlap_expolygons: Vec::new(),
    }
}

#[test]
fn task22o24_empty_filter_is_an_allocation_exact_noop() {
    let mut record = record(vec![
        surface(RegionSurfaceKind::Top, 0),
        surface(RegionSurfaceKind::Internal, 200),
    ]);
    let pointer = record.fill_surfaces.as_ptr();
    let before = snapshot(&record);
    reset_geometry_hooks();
    let staged = stage_record(
        &record,
        &VerticalShellTinyFilter {
            filtered_shell: Vec::new(),
        },
    )
    .unwrap();
    assert!(geometry_events().is_empty());
    commit(&mut record, staged);
    assert_eq!(record.fill_surfaces.as_ptr(), pointer);
    assert_eq!(snapshot(&record), before);
}

#[test]
fn task22o24_assignment_keeps_externals_then_appends_exact_internal_groups() {
    let mut record = record(vec![
        surface(RegionSurfaceKind::Top, 600),
        surface(RegionSurfaceKind::Internal, 0),
        surface(RegionSurfaceKind::Bottom, 700),
        surface(RegionSurfaceKind::InternalVoid, 200),
        surface(RegionSurfaceKind::InternalSolid, 400),
        surface(RegionSurfaceKind::BottomBridge, 800),
    ]);
    let retained = [
        record.fill_surfaces[0]
            .as_parts()
            .1
            .contour()
            .points()
            .as_ptr(),
        record.fill_surfaces[2]
            .as_parts()
            .1
            .contour()
            .points()
            .as_ptr(),
        record.fill_surfaces[5]
            .as_parts()
            .1
            .contour()
            .points()
            .as_ptr(),
    ];
    let filter = VerticalShellTinyFilter {
        filtered_shell: vec![rectangle(-10, 0, 510, 50)],
    };
    let source_points = record
        .fill_surfaces
        .iter()
        .map(|surface| surface.as_parts().1.contour().points().as_ptr())
        .chain(
            filter
                .filtered_shell
                .iter()
                .map(|expolygon| expolygon.contour().points().as_ptr()),
        )
        .collect::<Vec<_>>();
    reset_geometry_hooks();
    let staged = stage_record(&record, &filter).unwrap();
    assert_eq!(
        geometry_events(),
        vec![
            GeometryStep::SolidIntersection,
            GeometryStep::InternalDifference,
            GeometryStep::InternalVoidDifference,
        ]
    );
    commit(&mut record, staged);

    let kinds = record
        .fill_surfaces
        .iter()
        .map(|surface| surface.as_parts().0)
        .collect::<Vec<_>>();
    assert_eq!(
        kinds,
        vec![
            RegionSurfaceKind::Top,
            RegionSurfaceKind::Bottom,
            RegionSurfaceKind::BottomBridge,
            RegionSurfaceKind::Internal,
            RegionSurfaceKind::InternalVoid,
            RegionSurfaceKind::InternalSolid,
            RegionSurfaceKind::InternalSolid,
            RegionSurfaceKind::InternalSolid,
        ]
    );
    for (surface, pointer) in record.fill_surfaces[..3].iter().zip(retained) {
        assert_eq!(surface.as_parts().1.contour().points().as_ptr(), pointer);
        assert_eq!(
            metadata(surface),
            (2.5_f64.to_bits(), 3, 0.75_f64.to_bits(), 4)
        );
    }
    for surface in &record.fill_surfaces[3..] {
        assert_eq!(
            metadata(surface),
            ((-1.0_f64).to_bits(), 1, (-1.0_f64).to_bits(), 0)
        );
    }
    for surface in &record.fill_surfaces[3..] {
        assert!(!source_points.contains(&surface.as_parts().1.contour().points().as_ptr()));
    }
    assert_eq!(
        record.fill_surfaces[3..]
            .iter()
            .map(bounds)
            .collect::<Vec<_>>(),
        vec![
            (0, 50, 100, 100),
            (200, 50, 300, 100),
            (400, 0, 500, 50),
            (200, 0, 300, 50),
            (0, 0, 100, 50),
        ]
    );
}

#[test]
fn task22o24_nonempty_filter_runs_empty_subjects_and_freezes_failures() {
    let record = record(Vec::new());
    let filter = VerticalShellTinyFilter {
        filtered_shell: vec![rectangle(0, 0, 100, 100)],
    };
    reset_geometry_hooks();
    stage_record(&record, &filter).unwrap();
    assert_eq!(
        geometry_events(),
        vec![
            GeometryStep::SolidIntersection,
            GeometryStep::InternalDifference,
            GeometryStep::InternalVoidDifference,
        ]
    );

    for (step, expected) in [
        (
            GeometryStep::SolidIntersection,
            vec![GeometryStep::SolidIntersection],
        ),
        (
            GeometryStep::InternalDifference,
            vec![
                GeometryStep::SolidIntersection,
                GeometryStep::InternalDifference,
            ],
        ),
        (
            GeometryStep::InternalVoidDifference,
            vec![
                GeometryStep::SolidIntersection,
                GeometryStep::InternalDifference,
                GeometryStep::InternalVoidDifference,
            ],
        ),
    ] {
        reset_geometry_hooks();
        fail_geometry_at(step);
        assert_eq!(stage_record(&record, &filter).unwrap_err(), range_error());
        assert_eq!(geometry_events(), expected);
    }
    reset_geometry_hooks();
}

fn bounds(surface: &RegionSurface) -> (i64, i64, i64, i64) {
    let points = surface.as_parts().1.contour().points();
    (
        points.iter().map(|point| point.x()).min().unwrap(),
        points.iter().map(|point| point.y()).min().unwrap(),
        points.iter().map(|point| point.x()).max().unwrap(),
        points.iter().map(|point| point.y()).max().unwrap(),
    )
}

fn metadata(surface: &RegionSurface) -> (u64, u16, u64, u16) {
    let (_, _, thickness, layers, angle, extra) = surface.as_parts();
    (thickness.to_bits(), layers, angle.to_bits(), extra)
}

fn snapshot(
    record: &PreparedSurfaceTypeRecord,
) -> Vec<(RegionSurfaceKind, i64, u64, u16, u64, u16)> {
    record
        .fill_surfaces
        .iter()
        .map(|surface| {
            let (kind, expolygon, thickness, layers, angle, extra) = surface.as_parts();
            (
                kind,
                expolygon.contour().points()[0].x(),
                thickness.to_bits(),
                layers,
                angle.to_bits(),
                extra,
            )
        })
        .collect()
}
