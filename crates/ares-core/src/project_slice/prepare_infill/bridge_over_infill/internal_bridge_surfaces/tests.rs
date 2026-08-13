use super::*;
use crate::{
    geometry::{Point, Polygon},
    project_slice::prepare_infill::bridge_over_infill::types::CandidateSource,
};

type Points = Vec<(i64, i64)>;
type SurfaceSnapshot = (RegionSurfaceKind, Points, Vec<Points>, u64, u16, u64, u16);
type CandidateSnapshot = (CandidateSource, Vec<Points>, u64);

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

fn candidate(region: usize, index: usize, angle: f64, polys: Vec<Polygon>) -> CandidateSurface {
    CandidateSurface {
        source: CandidateSource {
            layer_index: 3,
            region_index: region,
            surface_index: index,
        },
        new_polygons: polys,
        bridge_angle: angle,
    }
}

fn polygon_snapshot(polygon: &Polygon) -> Points {
    polygon
        .points()
        .iter()
        .map(|point| (point.x(), point.y()))
        .collect()
}

fn surface_snapshot(surfaces: &[RegionSurface]) -> Vec<SurfaceSnapshot> {
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

fn candidate_snapshot(candidates: &[CandidateSurface]) -> Vec<CandidateSnapshot> {
    candidates
        .iter()
        .map(|candidate| {
            (
                candidate.source,
                candidate
                    .new_polygons
                    .iter()
                    .map(polygon_snapshot)
                    .collect(),
                candidate.bridge_angle.to_bits(),
            )
        })
        .collect()
}

#[test]
fn task22o68_matches_source_indices_preserves_metadata_angle_and_order() {
    let surfaces = [
        RegionSurface::new(RegionSurfaceKind::Internal, ep(-20)),
        RegionSurface::internal_with_metadata(ep(0), 0.8, 4, 0.2, 7)
            .clone_with_kind(RegionSurfaceKind::InternalSolid),
        RegionSurface::internal_with_metadata(ep(20), 0.6, 3, 0.4, 5)
            .clone_with_kind(RegionSurfaceKind::InternalSolid),
    ];
    let candidates = [
        candidate(2, 1, 1.25, vec![rect(100)]),
        candidate(1, 2, 2.5, vec![rect(200)]),
        candidate(1, 1, 3.5, vec![rect(300)]),
    ];
    let before = (surface_snapshot(&surfaces), candidate_snapshot(&candidates));
    let mut calls = 0;
    let output = build_internal_bridge_surfaces_using(1, &surfaces, &candidates, |polys| {
        calls += 1;
        Ok(vec![ExPolygon::new(polys[0].clone(), Vec::new())])
    })
    .unwrap();

    assert_eq!(calls, 2);
    assert_eq!(output.len(), 2);
    let a = output[0].as_parts();
    let b = output[1].as_parts();
    assert_eq!(
        (a.0, a.2, a.3, a.4, a.5),
        (RegionSurfaceKind::InternalBridge, 0.6, 3, 2.5, 5)
    );
    assert_eq!(
        (b.0, b.2, b.3, b.4, b.5),
        (RegionSurfaceKind::InternalBridge, 0.8, 4, 3.5, 7)
    );
    assert_eq!(a.1.contour(), &rect(200));
    assert_eq!(b.1.contour(), &rect(300));
    assert_eq!(
        (surface_snapshot(&surfaces), candidate_snapshot(&candidates)),
        before
    );
}

#[test]
fn task22o68_unmatched_and_wrong_kind_candidates_emit_nothing() {
    let surfaces = [RegionSurface::new(RegionSurfaceKind::Internal, ep(0))];
    let candidates = [
        candidate(0, 0, 1.0, vec![rect(10)]),
        candidate(0, 99, 2.0, vec![rect(20)]),
        candidate(2, 0, 3.0, vec![rect(30)]),
    ];
    let output = build_internal_bridge_surfaces_using(0, &surfaces, &candidates, |_| {
        panic!("union forbidden")
    })
    .unwrap();
    assert!(output.is_empty());
}

#[test]
fn task22o68_matched_candidate_empty_union_emits_nothing_and_preserves_input() {
    let surfaces = [RegionSurface::new(RegionSurfaceKind::InternalSolid, ep(0))];
    let candidates = [candidate(0, 0, 1.0, vec![rect(10)])];
    let before = (surface_snapshot(&surfaces), candidate_snapshot(&candidates));
    let output =
        build_internal_bridge_surfaces_using(0, &surfaces, &candidates, |_| Ok(Vec::new()))
            .unwrap();
    assert!(output.is_empty());
    assert_eq!(
        (surface_snapshot(&surfaces), candidate_snapshot(&candidates)),
        before
    );
}

#[test]
fn task22o68_multiple_union_outputs_keep_engine_order() {
    let surfaces = [RegionSurface::new(RegionSurfaceKind::InternalSolid, ep(0))];
    let candidates = [candidate(0, 0, 1.0, vec![rect(10)])];
    let output = build_internal_bridge_surfaces_using(0, &surfaces, &candidates, |_| {
        Ok(vec![ep(200), ep(-100)])
    })
    .unwrap();
    assert_eq!(output[0].as_parts().1.contour(), &rect(200));
    assert_eq!(output[1].as_parts().1.contour(), &rect(-100));
}

#[test]
fn task22o68_real_union_preserves_topology_and_range_error() {
    let surfaces = [RegionSurface::new(RegionSurfaceKind::InternalSolid, ep(0))];
    let candidates = [candidate(0, 0, 0.75, vec![rect(0), rect(5)])];
    let out = build_internal_bridge_surfaces(0, &surfaces, &candidates).unwrap();
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].as_parts().0, RegionSurfaceKind::InternalBridge);

    let h = 0x3fff_ffff_ffff_ffff_i64;
    let bad = [candidate(
        0,
        0,
        0.0,
        vec![Polygon::new(vec![
            Point::new(h + 1, 0),
            Point::new(h, 1),
            Point::new(h - 1, 0),
        ])],
    )];
    assert!(matches!(
        build_internal_bridge_surfaces(0, &surfaces, &bad),
        Err(ClipperError::CoordinateOutOfRange)
    ));
}

#[test]
fn task22o68_first_union_error_stops_candidate_order_and_preserves_input() {
    let surfaces = [
        RegionSurface::new(RegionSurfaceKind::InternalSolid, ep(0)),
        RegionSurface::new(RegionSurfaceKind::InternalSolid, ep(20)),
    ];
    let candidates = [
        candidate(0, 0, 0.0, vec![rect(0)]),
        candidate(0, 1, 0.0, vec![rect(20)]),
    ];
    let before = (surface_snapshot(&surfaces), candidate_snapshot(&candidates));
    let mut calls = 0;
    let result = build_internal_bridge_surfaces_using(0, &surfaces, &candidates, |_| {
        calls += 1;
        Err(ClipperError::CoordinateOutOfRange)
    });
    assert!(matches!(result, Err(ClipperError::CoordinateOutOfRange)));
    assert_eq!(calls, 1);
    assert_eq!(
        (surface_snapshot(&surfaces), candidate_snapshot(&candidates)),
        before
    );
}
