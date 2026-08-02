use crate::{
    geometry::{BoundingBox, CoordinateScale, ExPolygon, Point, Polygon},
    project_slice::perimeters::{classic::types::ClassicPreludeRecord, types::Flow},
};

use super::{super::config::ValidatedTopSplitConfig, SplitContext, apply};

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

fn record() -> ClassicPreludeRecord {
    ClassicPreludeRecord {
        perimeter_width: 500_000,
        perimeter_spacing: 400_000,
        external_width: 500_000,
        external_spacing: 450_000,
        external_to_internal_spacing: 450_000,
        solid_infill_spacing: 400_000,
        minimum_spacing: 300_000,
        external_minimum_spacing: 300_000,
        smaller_external_minimum_spacing: 300_000,
        has_gap_fill: true,
        smaller_external_flow: Flow {
            width: 0.3,
            height: 0.2,
            spacing: 0.25,
            nozzle_diameter: 0.4,
            bridge: false,
            mm3_per_mm: 0.05,
        },
        lower_slices_polygons: Vec::new(),
        lower_polygons_series: Vec::new(),
        external_lower_polygons_series: Vec::new(),
        smaller_external_lower_polygons_series: Vec::new(),
        surface_simplify_resolution: 1.0,
        surfaces: Vec::new(),
    }
}

fn config(has_gap_fill: bool) -> ValidatedTopSplitConfig {
    ValidatedTopSplitConfig {
        wall_loops: 2,
        only_one_wall_top: true,
        interface_shells: false,
        min_width_top_surface: 200_000.25,
        sparse_infill_width: 450_000.5,
        outer_nozzle_diameter: 0.4,
        has_gap_fill,
    }
}

fn context<'a>(
    upper_slices: &'a [ExPolygon],
    lower_slices: Option<&'a [ExPolygon]>,
    record: &'a ClassicPreludeRecord,
    has_gap_fill: bool,
) -> SplitContext<'a> {
    SplitContext {
        upper_slices,
        lower_slices,
        record,
        config: config(has_gap_fill),
        scale: CoordinateScale::Normal,
    }
}

#[test]
fn task22o2_split_distinguishes_covered_exposed_and_unsupported_topology() {
    let original = vec![rectangle(0, 0, 10_000_000, 10_000_000)];
    let record = record();
    let covered = apply(
        &original,
        context(&original, Some(&original), &record, false),
    )
    .unwrap();
    assert!(covered.top_fills.is_empty());
    assert!(!covered.non_top_polygons.is_empty());

    let exposed = apply(&original, context(&[], Some(&original), &record, false)).unwrap();
    assert!(!exposed.top_fills.is_empty());
    assert!(exposed.non_top_polygons.is_empty());

    let unsupported = apply(&original, context(&[], Some(&[]), &record, false)).unwrap();
    assert!(unsupported.top_fills.is_empty());
    assert!(!unsupported.non_top_polygons.is_empty());
}

#[test]
fn task22o2_split_preserves_fractional_scaled_sparse_width() {
    let original = vec![rectangle(0, 0, 10_000_000, 10_000_000)];
    let record = record();
    let output = apply(
        &original,
        context(&original, Some(&original), &record, false),
    )
    .unwrap();
    let bounds = BoundingBox::from_expolygons(&output.fill_clip).unwrap();

    assert_eq!(bounds.min(), Point::new(0, 0));
    assert_eq!(bounds.max(), Point::new(10_000_000, 10_000_000));
}
