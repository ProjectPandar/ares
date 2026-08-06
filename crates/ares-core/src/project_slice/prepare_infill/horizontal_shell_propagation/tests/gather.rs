use super::fixture::{holed_square, record, square, surface};
use crate::project_slice::{
    prepare_infill::horizontal_shell_propagation::{gather, types::SourceKind},
    region_slices::RegionSurfaceKind,
};

#[test]
fn source_gather_is_slices_then_fill_and_contour_then_holes() {
    let source = record(
        vec![
            surface(RegionSurfaceKind::Top, holed_square(0, 0, 100)),
            surface(RegionSurfaceKind::Bottom, square(1_000, 0, 100)),
        ],
        Vec::new(),
    );
    let working = vec![
        surface(RegionSurfaceKind::Top, holed_square(2_000, 0, 100)),
        surface(RegionSurfaceKind::Internal, square(3_000, 0, 100)),
    ];

    let paths = gather::source_paths(&source, &working, SourceKind::Top);
    assert_eq!(
        paths
            .iter()
            .map(|path| path.points()[0].x())
            .collect::<Vec<_>>(),
        vec![0, 25, 2_000, 2_025]
    );
}

#[test]
fn neighbor_and_repair_gathers_use_the_exact_reachable_kinds() {
    let fill = [
        surface(RegionSurfaceKind::Top, square(0, 0, 10)),
        surface(RegionSurfaceKind::Internal, square(20, 0, 10)),
        surface(RegionSurfaceKind::InternalSolid, square(40, 0, 10)),
        surface(RegionSurfaceKind::InternalVoid, square(60, 0, 10)),
        surface(RegionSurfaceKind::BottomBridge, square(80, 0, 10)),
    ];
    let starts = |paths: Vec<crate::geometry::Polygon>| {
        paths
            .iter()
            .map(|path| path.points()[0].x())
            .collect::<Vec<_>>()
    };

    assert_eq!(starts(gather::neighbor_internal_paths(&fill)), vec![20, 40]);
    assert_eq!(starts(gather::repair_clip_paths(&fill)), vec![20, 40, 60]);
}
