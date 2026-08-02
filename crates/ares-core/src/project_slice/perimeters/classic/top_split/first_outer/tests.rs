use crate::{
    geometry::{BoundingBox, CoordinateScale, ExPolygon, Point, Polygon},
    project_slice::perimeters::{classic::types::ClassicPreludeRecord, types::Flow},
};

use super::apply;

fn rectangle(width: i64, height: i64) -> ExPolygon {
    ExPolygon::new(
        Polygon::new(vec![
            Point::new(0, 0),
            Point::new(width, 0),
            Point::new(width, height),
            Point::new(0, height),
        ]),
        Vec::new(),
    )
}

fn record(smaller_width_mm: f32) -> ClassicPreludeRecord {
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
            width: smaller_width_mm,
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

#[test]
fn task22o2_first_outer_selects_wide_short_narrow_and_long_narrow_paths() {
    let result = apply(
        &[
            rectangle(20_000_000, 20_000_000),
            rectangle(600_000, 5_000_000),
            rectangle(600_000, 20_000_000),
        ],
        &record(0.3),
        CoordinateScale::Normal,
    )
    .unwrap();

    assert_eq!(result.normal.len(), 2);
    assert_eq!(result.smaller.len(), 1);
    let smaller = BoundingBox::from_expolygon(&result.smaller[0]).unwrap();
    assert_eq!(smaller.min(), Point::new(150_000, 150_000));
    assert_eq!(smaller.max(), Point::new(450_000, 4_850_000));
}

#[test]
fn task22o2_first_outer_preserves_complete_collapse() {
    let result = apply(
        &[rectangle(200_000, 2_000_000)],
        &record(0.3),
        CoordinateScale::Normal,
    )
    .unwrap();

    assert!(result.normal.is_empty());
    assert!(result.smaller.is_empty());
}
