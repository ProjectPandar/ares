use crate::{
    geometry::{ExPolygon, Point, Polygon},
    project_slice::{
        prepare_infill::surface_type_detection::stage::clipped_fill,
        region_slices::{RegionSurface, RegionSurfaceKind},
    },
};

fn rectangle(x0: i64, y0: i64, x1: i64, y1: i64) -> ExPolygon {
    ExPolygon::new(
        Polygon::new(vec![
            Point::new(x0, y0),
            Point::new(x1, y0),
            Point::new(x1, y1),
            Point::new(x0, y1),
        ]),
        Vec::new(),
    )
}

#[test]
fn clipped_fill_visits_numeric_kinds_and_intersects_boundaries() {
    let slices = vec![
        RegionSurface::new(RegionSurfaceKind::Internal, rectangle(0, 0, 1_000, 1_000)),
        RegionSurface::new(
            RegionSurfaceKind::BottomBridge,
            rectangle(2_000, 0, 3_000, 1_000),
        ),
        RegionSurface::new(RegionSurfaceKind::Top, rectangle(4_000, 0, 5_000, 1_000)),
        RegionSurface::new(RegionSurfaceKind::Bottom, rectangle(6_000, 0, 7_000, 1_000)),
    ];
    let boundaries = vec![rectangle(-100, -100, 6_500, 1_100)];
    let output = clipped_fill(&slices, &boundaries).unwrap();
    let kinds = output
        .iter()
        .map(|surface| surface.as_parts().0)
        .collect::<Vec<_>>();
    assert_eq!(
        kinds,
        [
            RegionSurfaceKind::Top,
            RegionSurfaceKind::Bottom,
            RegionSurfaceKind::BottomBridge,
            RegionSurfaceKind::Internal,
        ]
    );
    let bottom = output
        .iter()
        .find(|surface| surface.as_parts().0 == RegionSurfaceKind::Bottom)
        .unwrap();
    assert_eq!(
        bottom
            .as_parts()
            .1
            .contour()
            .points()
            .iter()
            .map(|point| point.x())
            .max(),
        Some(6_500)
    );
    for surface in &output {
        let (_, _, thickness, layers, angle, extra) = surface.as_parts();
        assert_eq!((thickness, layers, angle, extra), (-1.0, 1, -1.0, 0));
    }
}

#[test]
fn empty_boundaries_clear_all_fill_groups() {
    let slices = vec![RegionSurface::new(
        RegionSurfaceKind::Internal,
        rectangle(0, 0, 1_000, 1_000),
    )];
    assert!(clipped_fill(&slices, &[]).unwrap().is_empty());
    assert!(
        clipped_fill(&[], &[rectangle(0, 0, 1_000, 1_000)])
            .unwrap()
            .is_empty()
    );
}

#[test]
fn rebuilt_fill_does_not_alias_the_boundary_geometry() {
    let boundary = rectangle(0, 0, 1_000, 1_000);
    let pointer = boundary.contour().points().as_ptr();
    let output = clipped_fill(
        &[RegionSurface::new(RegionSurfaceKind::Top, boundary.clone())],
        std::slice::from_ref(&boundary),
    )
    .unwrap();
    assert_eq!(output.len(), 1);
    assert_ne!(output[0].as_parts().1.contour().points().as_ptr(), pointer);
}

#[test]
fn repeated_holed_kind_has_stable_clipper_output_order() {
    let mut first_hole = rectangle(20, 20, 40, 40).into_parts().0;
    first_hole.reverse();
    let mut second_hole = rectangle(220, 20, 240, 40).into_parts().0;
    second_hole.reverse();
    let first = ExPolygon::new(rectangle(0, 0, 100, 100).into_parts().0, vec![first_hole]);
    let second = ExPolygon::new(
        rectangle(200, 0, 300, 100).into_parts().0,
        vec![second_hole],
    );
    let output = clipped_fill(
        &[
            RegionSurface::new(RegionSurfaceKind::Top, first),
            RegionSurface::new(RegionSurfaceKind::Top, second),
        ],
        &[rectangle(-10, -10, 310, 110)],
    )
    .unwrap();
    assert_eq!(output.len(), 2);
    assert!(
        output
            .iter()
            .all(|surface| surface.as_parts().1.holes().len() == 1)
    );
    assert_eq!(
        output
            .iter()
            .map(|surface| {
                surface
                    .as_parts()
                    .1
                    .contour()
                    .points()
                    .iter()
                    .map(|point| point.x())
                    .min()
                    .unwrap()
            })
            .collect::<Vec<_>>(),
        [0, 200]
    );
}
