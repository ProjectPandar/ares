use super::*;
use crate::{
    geometry::Point,
    project_slice::region_slices::{RegionSurface, RegionSurfaceKind},
};

fn rect(x: i64) -> Polygon {
    Polygon::new(vec![
        Point::new(x, 0),
        Point::new(x + 10, 0),
        Point::new(x + 10, 10),
        Point::new(x, 10),
    ])
}

fn ep(x: i64) -> ExPolygon {
    ExPolygon::new(rect(x), Vec::new())
}

fn first_x(expolygon: &ExPolygon) -> i64 {
    expolygon.contour().points()[0].x()
}

fn surface_snapshot(
    surfaces: &[RegionSurface],
) -> Vec<(RegionSurfaceKind, i64, u64, u16, u64, u16)> {
    surfaces
        .iter()
        .map(|surface| {
            let (kind, expolygon, thickness, layers, angle, extra) = surface.as_parts();
            (
                kind,
                first_x(expolygon),
                thickness.to_bits(),
                layers,
                angle.to_bits(),
                extra,
            )
        })
        .collect()
}

#[test]
fn task22o69_exact_operands_order_flattening_metadata_and_nonmutation() {
    let surfaces = [
        RegionSurface::new(RegionSurfaceKind::Internal, ep(-20)),
        RegionSurface::internal_with_metadata(ep(70), 0.8, 4, 0.3, 7)
            .clone_with_kind(RegionSurfaceKind::InternalSolid),
        RegionSurface::new(RegionSurfaceKind::InternalSolid, ep(10)),
    ];
    let ensuring = [ep(50), ep(50), ep(0)];
    let cut = [rect(900)];
    let before = (surface_snapshot(&surfaces), ensuring.clone(), cut.clone());
    let events = std::cell::RefCell::new(Vec::new());
    let output = recompose_internal_solids_using(
        &surfaces,
        &ensuring,
        &cut,
        |subject, clip| {
            events.borrow_mut().push(format!(
                "difference:{:?}:{:?}",
                subject.iter().map(first_x).collect::<Vec<_>>(),
                clip.iter()
                    .map(|polygon| polygon.points()[0].x())
                    .collect::<Vec<_>>()
            ));
            Ok(vec![
                ExPolygon::new(rect(200), vec![rect(220), rect(210)]),
                ep(-100),
            ])
        },
        |polygons| {
            events.borrow_mut().push(format!(
                "union:{:?}",
                polygons
                    .iter()
                    .map(|expolygon| (
                        first_x(expolygon),
                        expolygon
                            .holes()
                            .iter()
                            .map(|hole| hole.points()[0].x())
                            .collect::<Vec<_>>()
                    ))
                    .collect::<Vec<_>>()
            ));
            Ok(vec![ep(300), ep(-300)])
        },
    )
    .unwrap();

    assert_eq!(
        events.into_inner(),
        [
            "difference:[70, 10, 50, 50, 0]:[900]",
            "union:[(200, [220, 210]), (-100, [])]"
        ]
    );
    assert_eq!(output.len(), 2);
    for (surface, expected_x) in output.iter().zip([300, -300]) {
        let (kind, expolygon, thickness, layers, angle, extra) = surface.as_parts();
        assert_eq!(kind, RegionSurfaceKind::InternalSolid);
        assert_eq!(first_x(expolygon), expected_x);
        assert_eq!((thickness, layers, angle, extra), (-1.0, 1, -1.0, 0));
    }
    assert_eq!(
        (surface_snapshot(&surfaces), ensuring.clone(), cut.clone()),
        before
    );
}

#[test]
fn task22o69_empty_inputs_still_call_difference_then_union_once() {
    let mut difference_calls = 0;
    let mut union_calls = 0;
    let output = recompose_internal_solids_using(
        &[],
        &[],
        &[],
        |subject, clip| {
            difference_calls += 1;
            assert!(subject.is_empty());
            assert!(clip.is_empty());
            Ok(Vec::new())
        },
        |polygons| {
            union_calls += 1;
            assert!(polygons.is_empty());
            Ok(Vec::new())
        },
    )
    .unwrap();
    assert!(output.is_empty());
    assert_eq!((difference_calls, union_calls), (1, 1));
}

#[test]
fn task22o69_difference_error_precedes_union_and_union_error_precedes_output() {
    let mut union_calls = 0;
    let difference_error = recompose_internal_solids_using(
        &[],
        &[],
        &[],
        |_, _| Err(ClipperError::CoordinateOutOfRange),
        |_| {
            union_calls += 1;
            Ok(Vec::new())
        },
    );
    assert!(matches!(
        difference_error,
        Err(ClipperError::CoordinateOutOfRange)
    ));
    assert_eq!(union_calls, 0);

    let union_error = recompose_internal_solids_using(
        &[],
        &[],
        &[],
        |_, _| Ok(vec![ep(0)]),
        |_| Err(ClipperError::CoordinateOutOfRange),
    );
    assert!(matches!(
        union_error,
        Err(ClipperError::CoordinateOutOfRange)
    ));
}

#[test]
fn task22o69_real_geometry_is_repeatable_and_preserves_inputs() {
    let surfaces = [
        RegionSurface::new(RegionSurfaceKind::InternalSolid, ep(0)),
        RegionSurface::new(RegionSurfaceKind::Internal, ep(50)),
    ];
    let ensuring = [ep(20)];
    let cut = [rect(5)];
    let before = (surface_snapshot(&surfaces), ensuring.clone(), cut.clone());
    let first = recompose_internal_solids(&surfaces, &ensuring, &cut).unwrap();
    let second = recompose_internal_solids(&surfaces, &ensuring, &cut).unwrap();
    assert_eq!(surface_snapshot(&first), surface_snapshot(&second));
    assert_eq!(
        (surface_snapshot(&surfaces), ensuring.clone(), cut.clone()),
        before
    );
}

#[test]
fn task22o69_natural_difference_range_error_is_atomic() {
    let h = 0x3fff_ffff_ffff_ffff_i64;
    let surfaces = [RegionSurface::new(RegionSurfaceKind::InternalSolid, ep(0))];
    let cut = [Polygon::new(vec![
        Point::new(h + 1, 0),
        Point::new(h, 1),
        Point::new(h - 1, 0),
    ])];
    let before = (surface_snapshot(&surfaces), cut.clone());
    assert!(matches!(
        recompose_internal_solids(&surfaces, &[], &cut),
        Err(ClipperError::CoordinateOutOfRange)
    ));
    assert_eq!((surface_snapshot(&surfaces), cut.clone()), before);
}

#[test]
fn task22o69_expolygon_safety_union_preserves_holes_and_disconnected_components() {
    let outer = Polygon::new(vec![
        Point::new(0, 0),
        Point::new(100, 0),
        Point::new(100, 100),
        Point::new(0, 100),
    ]);
    let mut hole = Polygon::new(vec![
        Point::new(30, 30),
        Point::new(30, 70),
        Point::new(70, 70),
        Point::new(70, 30),
    ]);
    if hole.area() > 0.0 {
        hole.reverse();
    }
    let second = Polygon::new(vec![
        Point::new(200, 0),
        Point::new(300, 0),
        Point::new(300, 100),
        Point::new(200, 100),
    ]);
    let output = union_safety_offset_expolygons(&[
        ExPolygon::new(outer, vec![hole]),
        ExPolygon::new(second, Vec::new()),
    ])
    .unwrap();
    let snapshot = output
        .iter()
        .map(|expolygon| {
            (
                expolygon
                    .contour()
                    .points()
                    .iter()
                    .map(|point| (point.x(), point.y()))
                    .collect::<Vec<_>>(),
                expolygon
                    .holes()
                    .iter()
                    .map(|hole| {
                        hole.points()
                            .iter()
                            .map(|point| (point.x(), point.y()))
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        snapshot,
        [
            (vec![(310, 110), (190, 110), (190, -10), (310, -10)], vec![],),
            (
                vec![(110, 110), (-10, 110), (-10, -10), (110, -10)],
                vec![vec![(40, 40), (40, 60), (60, 60), (60, 40)]],
            ),
        ]
    );
}
