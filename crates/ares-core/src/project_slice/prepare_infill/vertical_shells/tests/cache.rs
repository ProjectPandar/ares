use crate::{
    ProcessEnsureVerticalShellThickness,
    geometry::{ExPolygon, Point, Polygon},
    project_slice::{
        prepare_infill::vertical_shells::cache::{build, expansion},
        region_slices::{RegionSurface, RegionSurfaceKind},
    },
};

fn polygon(x: i64, y: i64, size: i64) -> Polygon {
    Polygon::new(vec![
        Point::new(x, y),
        Point::new(x + size, y),
        Point::new(x + size, y + size),
        Point::new(x, y + size),
    ])
}

fn expolygon(x: i64, y: i64) -> ExPolygon {
    ExPolygon::new(polygon(x, y, 1_000), Vec::new())
}

#[test]
fn active_cache_filters_kinds_and_flattens_fill_boundaries() {
    let slices = vec![
        RegionSurface::new(RegionSurfaceKind::BottomBridge, expolygon(0, 0)),
        RegionSurface::new(RegionSurfaceKind::Top, expolygon(3_000, 0)),
        RegionSurface::internal(expolygon(6_000, 0)),
        RegionSurface::new(RegionSurfaceKind::Bottom, expolygon(9_000, 0)),
    ];
    let fill = vec![ExPolygon::new(
        polygon(0, 0, 10_000),
        vec![polygon(2_000, 2_000, 1_000), polygon(5_000, 5_000, 1_000)],
    )];
    let cache = build(
        &slices,
        &fill,
        ProcessEnsureVerticalShellThickness::EnsureAll,
        1_000,
    )
    .unwrap();
    assert_eq!(cache.top_surfaces.len(), 1);
    assert_eq!(cache.bottom_surfaces.len(), 2);
    assert_eq!(
        cache.holes,
        vec![
            fill[0].contour().clone(),
            fill[0].holes()[0].clone(),
            fill[0].holes()[1].clone()
        ]
    );
}

#[test]
fn f32_expansion_cast_order_is_pinned_through_production_cache() {
    assert_eq!(expansion(16_777_217).to_bits(), 0x494c_cccd);
    let source = polygon(0, 0, 2_000_000);
    let slices = [RegionSurface::new(
        RegionSurfaceKind::Bottom,
        ExPolygon::new(source.clone(), Vec::new()),
    )];
    let cache = build(
        &slices,
        &[],
        ProcessEnsureVerticalShellThickness::EnsureAll,
        16_777_217,
    )
    .unwrap();
    let output = &cache.bottom_surfaces[0];
    let span = |polygon: &Polygon| {
        let (minimum, maximum) = polygon
            .points()
            .iter()
            .map(|point| point.x())
            .fold((i64::MAX, i64::MIN), |(minimum, maximum), x| {
                (minimum.min(x), maximum.max(x))
            });
        maximum - minimum
    };
    assert_eq!(span(output) - span(&source), 1_677_722);
}

#[test]
fn empty_active_input_is_empty() {
    let cache = build(
        &[],
        &[],
        ProcessEnsureVerticalShellThickness::EnsureAll,
        1_000,
    )
    .unwrap();
    assert!(cache.top_surfaces.is_empty());
    assert!(cache.bottom_surfaces.is_empty());
    assert!(cache.holes.is_empty());
}
