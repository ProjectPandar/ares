use super::super::assign::{StagedAssignment, stage_record};
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

type Bounds = (i64, i64, i64, i64);
type Signature = (Bounds, Vec<Bounds>);

#[test]
fn task22o24_differences_preserve_multiple_holed_nested_subjects() {
    let staged = stage_record(
        &record(),
        &VerticalShellTinyFilter {
            filtered_shell: vec![expolygon(1_000, 0, 1_100, 100)],
        },
    )
    .unwrap();
    let StagedAssignment::Replace {
        new_internal,
        new_internal_void,
        new_internal_solid,
    } = staged
    else {
        panic!("nonempty filter must stage replacement geometry")
    };
    assert!(new_internal_solid.is_empty());
    assert_eq!(
        signatures(&new_internal),
        vec![
            ((0, 0, 300, 300), vec![(100, 100, 200, 200)]),
            ((130, 130, 170, 170), Vec::new()),
        ]
    );
    assert_eq!(
        signatures(&new_internal_void),
        vec![
            ((400, 0, 500, 100), Vec::new()),
            ((600, 0, 700, 100), Vec::new()),
        ]
    );
}

#[test]
fn task22o24_full_cover_moves_all_internal_topology_to_solid() {
    let staged = stage_record(
        &record(),
        &VerticalShellTinyFilter {
            filtered_shell: vec![expolygon(-10, -10, 710, 310)],
        },
    )
    .unwrap();
    let StagedAssignment::Replace {
        new_internal,
        new_internal_void,
        new_internal_solid,
    } = staged
    else {
        panic!("nonempty filter must stage replacement geometry")
    };
    assert!(new_internal.is_empty());
    assert!(new_internal_void.is_empty());
    assert_eq!(
        signatures(&new_internal_solid),
        vec![
            ((0, 0, 300, 300), vec![(100, 100, 200, 200)]),
            ((130, 130, 170, 170), Vec::new()),
            ((400, 0, 500, 100), Vec::new()),
            ((600, 0, 700, 100), Vec::new()),
        ]
    );
}

fn record() -> PreparedSurfaceTypeRecord {
    let mut record = super::record(Vec::new());
    record.fill_surfaces = vec![
        RegionSurface::new(
            RegionSurfaceKind::Internal,
            ExPolygon::new(
                rectangle(0, 0, 300, 300),
                vec![clockwise_rectangle(100, 100, 200, 200)],
            ),
        ),
        RegionSurface::new(RegionSurfaceKind::Internal, expolygon(130, 130, 170, 170)),
        RegionSurface::new(RegionSurfaceKind::InternalVoid, expolygon(400, 0, 500, 100)),
        RegionSurface::new(RegionSurfaceKind::InternalVoid, expolygon(600, 0, 700, 100)),
    ];
    record
}

fn signatures(expolygons: &[ExPolygon]) -> Vec<Signature> {
    let mut signatures = expolygons
        .iter()
        .map(|expolygon| {
            let mut holes = expolygon.holes().iter().map(bounds).collect::<Vec<_>>();
            holes.sort_unstable();
            (bounds(expolygon.contour()), holes)
        })
        .collect::<Vec<_>>();
    signatures.sort_unstable();
    signatures
}

fn bounds(polygon: &Polygon) -> Bounds {
    let mut points = polygon.points().iter();
    let first = points.next().unwrap();
    points.fold(
        (first.x(), first.y(), first.x(), first.y()),
        |(min_x, min_y, max_x, max_y), point| {
            (
                min_x.min(point.x()),
                min_y.min(point.y()),
                max_x.max(point.x()),
                max_y.max(point.y()),
            )
        },
    )
}

fn expolygon(min_x: i64, min_y: i64, max_x: i64, max_y: i64) -> ExPolygon {
    ExPolygon::new(rectangle(min_x, min_y, max_x, max_y), Vec::new())
}

fn rectangle(min_x: i64, min_y: i64, max_x: i64, max_y: i64) -> Polygon {
    Polygon::new(vec![
        Point::new(min_x, min_y),
        Point::new(max_x, min_y),
        Point::new(max_x, max_y),
        Point::new(min_x, max_y),
    ])
}

fn clockwise_rectangle(min_x: i64, min_y: i64, max_x: i64, max_y: i64) -> Polygon {
    Polygon::new(vec![
        Point::new(min_x, min_y),
        Point::new(min_x, max_y),
        Point::new(max_x, max_y),
        Point::new(max_x, min_y),
    ])
}
