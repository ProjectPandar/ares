use super::prepare;
use crate::geometry::{ExPolygon, Point, Polygon};
use crate::project_slice::region_slices::{PostRegion, RegionLayer, RegionSurface};
use crate::{FloatOrPercent, OrcaBool, OrcaFloat, Percent, RegionOptions};

fn region(id: usize, density: f64, min_x: i64, max_x: i64) -> PostRegion {
    PostRegion {
        id,
        options: RegionOptions {
            sparse_infill_density: Percent(density),
            ..RegionOptions::from_base(&crate::ProcessRegionSourceOptions::default())
        },
        layers: vec![RegionLayer {
            surfaces: vec![RegionSurface::internal(ExPolygon::new(
                Polygon::new(vec![
                    Point::new(min_x, 0),
                    Point::new(max_x, 0),
                    Point::new(max_x, 1000),
                    Point::new(min_x, 1000),
                ]),
                Vec::new(),
            ))],
        }],
    }
}

#[test]
fn higher_density_owns_merged_walls_but_bridge_flow_does_not_split_them() {
    let first = region(7, 0.0, 0, 1000);
    let mut second = region(9, 50.0, 1000, 2000);
    second.options.bridge_flow = OrcaFloat(2.0);
    let groups = prepare(&[first, second], 1).unwrap();
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].owner.region_index, 1);
    let surfaces = groups[0].merged_surfaces.as_ref().unwrap();
    assert_eq!(surfaces.len(), 1);
    let polygon = surfaces[0].as_parts().1;
    let points = polygon.contour().points();
    assert_eq!(points.iter().map(|p| p.x()).min(), Some(-10));
    assert_eq!(points.iter().map(|p| p.x()).max(), Some(2010));
    assert_eq!(points.iter().map(|p| p.y()).min(), Some(-10));
    assert_eq!(points.iter().map(|p| p.y()).max(), Some(1010));
}

#[test]
fn equal_density_preserves_first_owner_and_wall_count_flags_split_groups() {
    let first = region(7, 50.0, 0, 1000);
    let second = region(9, 50.0, 1000, 2000);
    let groups = prepare(&[first, second], 1).unwrap();
    assert_eq!(groups[0].owner.region_index, 0);
    let first = region(7, 50.0, 0, 1000);
    let mut second = region(9, 50.0, 1000, 2000);
    second.options.only_one_wall_first_layer = OrcaBool(!first.options.only_one_wall_first_layer.0);
    let groups = prepare(&[first, second], 1).unwrap();
    assert_eq!(groups.len(), 2);
    assert!(groups.iter().all(|group| group.merged_surfaces.is_none()));
}

#[test]
fn width_serialization_merges_equal_six_digit_tokens() {
    let mut first = region(7, 0.0, 0, 1000);
    let mut second = region(9, 50.0, 1000, 2000);
    first.options.inner_wall_line_width = FloatOrPercent::Float(0.41234561);
    second.options.inner_wall_line_width = FloatOrPercent::Float(0.41234562);
    assert_eq!(prepare(&[first, second], 1).unwrap().len(), 1);
}

#[test]
fn width_serialization_distinguishes_negative_zero() {
    let mut first = region(7, 0.0, 0, 1000);
    let mut second = region(9, 50.0, 1000, 2000);
    first.options.outer_wall_line_width = FloatOrPercent::Float(-0.0);
    second.options.outer_wall_line_width = FloatOrPercent::Float(0.0);
    assert_eq!(prepare(&[first, second], 1).unwrap().len(), 2);
}

#[test]
fn scarf_start_height_uses_native_approximate_option_equality() {
    let mut first = region(7, 0.0, 0, 1000);
    let mut second = region(9, 50.0, 1000, 2000);
    first.options.seam_slope_start_height = FloatOrPercent::Float(0.0);
    second.options.seam_slope_start_height = FloatOrPercent::Float(0.00005);
    assert_eq!(prepare(&[first, second], 1).unwrap().len(), 1);
}
