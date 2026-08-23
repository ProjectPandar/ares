use super::{
    PromotionEvent, commits, events,
    promote::{StagedDecision, commit, stage_decision},
    reset_hooks,
};
use crate::{
    SliceError,
    geometry::{ExPolygon, Point, Polygon},
    project_slice::{
        prepare_infill::surface_type_detection::types::PreparedSurfaceTypeRecord,
        region_slices::{RegionSurface, RegionSurfaceKind},
    },
};

fn surface(kind: RegionSurfaceKind, x: i64) -> RegionSurface {
    RegionSurface::internal_with_metadata(
        ExPolygon::new(
            Polygon::new(vec![
                Point::new(x, 0),
                Point::new(x + 10, 0),
                Point::new(x + 10, 10),
                Point::new(x, 10),
            ]),
            vec![Polygon::new(vec![
                Point::new(x + 3, 3),
                Point::new(x + 3, 7),
                Point::new(x + 7, 7),
                Point::new(x + 7, 3),
            ])],
        ),
        2.5,
        3,
        0.75,
        4,
    )
    .clone_with_kind(kind)
}

fn record(fill_surfaces: Vec<RegionSurface>) -> PreparedSurfaceTypeRecord {
    PreparedSurfaceTypeRecord {
        perimeters: Vec::new(),
        thin_fills: Vec::new(),
        slices: vec![surface(RegionSurfaceKind::Internal, 1_000)],
        fill_surfaces,
        fill_expolygons: Vec::new(),
        fill_no_overlap_expolygons: Vec::new(),
    }
}

#[test]
fn raw_empty_is_the_only_parser_and_matcher_short_circuit() {
    reset_hooks();
    assert_eq!(stage_decision("", 0).unwrap(), StagedDecision::Noop);
    assert_eq!(events(), vec![PromotionEvent::RawScheduleVisit]);

    reset_hooks();
    assert_eq!(stage_decision("  ''  ", 0).unwrap(), StagedDecision::Noop);
    assert_eq!(
        events(),
        vec![
            PromotionEvent::RawScheduleVisit,
            PromotionEvent::NonemptySchedule,
            PromotionEvent::ParserInvocation,
            PromotionEvent::MatcherInvocation,
        ]
    );
}

#[test]
fn invalid_nonempty_schedule_returns_the_stable_error_before_matching() {
    reset_hooks();
    assert_eq!(
        stage_decision("2147483648", 0).unwrap_err(),
        SliceError::InvalidInput("invalid extra_solid_infills pattern".to_owned())
    );
    assert_eq!(
        events(),
        vec![
            PromotionEvent::RawScheduleVisit,
            PromotionEvent::NonemptySchedule,
            PromotionEvent::ParserInvocation,
        ]
    );
}

#[test]
fn matching_uses_one_based_planned_array_index() {
    reset_hooks();
    assert_eq!(
        stage_decision("2", 1).unwrap(),
        StagedDecision::PromoteInternal
    );
    assert_eq!(
        stage_decision("2", 41).unwrap(),
        StagedDecision::PromoteInternal
    );
    assert_eq!(stage_decision("2", 2).unwrap(), StagedDecision::Noop);
}

#[test]
fn matching_retags_every_and_only_internal_surface_in_place() {
    let mut record = record(vec![
        surface(RegionSurfaceKind::Top, 0),
        surface(RegionSurfaceKind::Internal, 20),
        surface(RegionSurfaceKind::Bottom, 40),
        surface(RegionSurfaceKind::InternalVoid, 60),
        surface(RegionSurfaceKind::InternalSolid, 80),
        surface(RegionSurfaceKind::BottomBridge, 100),
        surface(RegionSurfaceKind::Internal, 120),
    ]);
    let vector_pointer = record.fill_surfaces.as_ptr();
    let capacity = record.fill_surfaces.capacity();
    let geometry_pointers = path_pointers(&record.fill_surfaces);
    let geometry_before = geometry_snapshot(&record.fill_surfaces);
    let slices_before = snapshot(&record.slices);

    reset_hooks();
    commit(&mut record, StagedDecision::PromoteInternal);

    assert_eq!(record.fill_surfaces.as_ptr(), vector_pointer);
    assert_eq!(record.fill_surfaces.capacity(), capacity);
    assert_eq!(path_pointers(&record.fill_surfaces), geometry_pointers);
    assert_eq!(geometry_snapshot(&record.fill_surfaces), geometry_before);
    assert_eq!(snapshot(&record.slices), slices_before);
    assert_eq!(
        record
            .fill_surfaces
            .iter()
            .map(|surface| surface.as_parts().0)
            .collect::<Vec<_>>(),
        vec![
            RegionSurfaceKind::Top,
            RegionSurfaceKind::InternalSolid,
            RegionSurfaceKind::Bottom,
            RegionSurfaceKind::InternalVoid,
            RegionSurfaceKind::InternalSolid,
            RegionSurfaceKind::BottomBridge,
            RegionSurfaceKind::InternalSolid,
        ]
    );
    assert_eq!(commits(), 1);
    assert_eq!(
        events(),
        vec![
            PromotionEvent::PromotedSurface,
            PromotionEvent::PromotedSurface
        ]
    );
    for surface in &record.fill_surfaces {
        assert_eq!(
            metadata(surface),
            (2.5_f64.to_bits(), 3, 0.75_f64.to_bits(), 4)
        );
    }
}

#[test]
fn noop_is_allocation_exact_and_promotion_is_idempotent() {
    let mut record = record(vec![surface(RegionSurfaceKind::Internal, 0)]);
    let pointer = record.fill_surfaces.as_ptr();
    let before = snapshot(&record.fill_surfaces);
    reset_hooks();
    commit(&mut record, StagedDecision::Noop);
    assert_eq!(record.fill_surfaces.as_ptr(), pointer);
    assert_eq!(snapshot(&record.fill_surfaces), before);
    assert_eq!(commits(), 0);

    commit(&mut record, StagedDecision::PromoteInternal);
    let once = snapshot(&record.fill_surfaces);
    commit(&mut record, StagedDecision::PromoteInternal);
    assert_eq!(snapshot(&record.fill_surfaces), once);
}

#[test]
fn promotion_has_no_sparse_density_gate() {
    for _density_percent in [0.0, 15.0, 100.0] {
        let mut record = record(vec![surface(RegionSurfaceKind::Internal, 0)]);
        let decision = stage_decision("1#", 0).unwrap();
        commit(&mut record, decision);
        assert_eq!(
            record.fill_surfaces[0].as_parts().0,
            RegionSurfaceKind::InternalSolid
        );
    }
}

fn path_pointers(surfaces: &[RegionSurface]) -> Vec<Vec<*const Point>> {
    surfaces
        .iter()
        .map(|surface| {
            let expolygon = surface.as_parts().1;
            std::iter::once(expolygon.contour())
                .chain(expolygon.holes())
                .map(|path| path.points().as_ptr())
                .collect()
        })
        .collect()
}

fn geometry_snapshot(surfaces: &[RegionSurface]) -> Vec<Vec<Vec<(i64, i64)>>> {
    surfaces
        .iter()
        .map(|surface| {
            let expolygon = surface.as_parts().1;
            std::iter::once(expolygon.contour())
                .chain(expolygon.holes())
                .map(|path| {
                    path.points()
                        .iter()
                        .map(|point| (point.x(), point.y()))
                        .collect()
                })
                .collect()
        })
        .collect()
}

fn metadata(surface: &RegionSurface) -> (u64, u16, u64, u16) {
    let (_, _, thickness, layers, angle, extra) = surface.as_parts();
    (thickness.to_bits(), layers, angle.to_bits(), extra)
}

fn snapshot(surfaces: &[RegionSurface]) -> Vec<(RegionSurfaceKind, i64, u64, u16, u64, u16)> {
    surfaces
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
