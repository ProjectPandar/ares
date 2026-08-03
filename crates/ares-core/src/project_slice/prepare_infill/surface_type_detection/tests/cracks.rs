use crate::{
    geometry::{ExPolygon, JoinType, Point, Polygon, offset_expolygon, offset_expolygons},
    project_slice::{
        prepare_infill::surface_type_detection::{
            GeometryStep,
            cracks::{belongs_to_large_bottom, resolve},
            geometry_events, reset_geometry_hooks,
        },
        region_slices::{RegionSurface, RegionSurfaceKind},
    },
};

fn polygon(x0: i64, y0: i64, x1: i64, y1: i64) -> Polygon {
    Polygon::new(vec![
        Point::new(x0, y0),
        Point::new(x1, y0),
        Point::new(x1, y1),
        Point::new(x0, y1),
    ])
}

fn rectangle(x0: i64, y0: i64, x1: i64, y1: i64) -> ExPolygon {
    ExPolygon::new(polygon(x0, y0, x1, y1), Vec::new())
}

fn surface(kind: RegionSurfaceKind, expolygon: ExPolygon) -> RegionSurface {
    RegionSurface::new(kind, expolygon)
}

#[test]
fn tiny_crack_without_a_large_bottom_is_removed_from_bottom() {
    let crack = rectangle(0, 0, 100, 100);
    let mut top = vec![surface(RegionSurfaceKind::Top, crack.clone())];
    let mut bottom = vec![surface(RegionSurfaceKind::BottomBridge, crack)];
    resolve(&mut top, &mut bottom, 100, true).unwrap();
    assert_eq!(top.len(), 1);
    assert!(bottom.is_empty());
}

#[test]
fn tiny_crack_inside_a_large_bottom_remains_bottom() {
    let crack = rectangle(400, 400, 500, 500);
    let mut top = vec![surface(RegionSurfaceKind::Top, crack)];
    let mut bottom = vec![surface(
        RegionSurfaceKind::BottomBridge,
        rectangle(0, 0, 1_000, 1_000),
    )];
    resolve(&mut top, &mut bottom, 100, true).unwrap();
    assert!(top.is_empty());
    assert_eq!(bottom.len(), 1);
}

#[test]
fn exactly_twice_the_crack_area_does_not_pass_the_strict_test() {
    let crack = rectangle(0, 0, 100, 100);
    let bottom = vec![surface(
        RegionSurfaceKind::BottomBridge,
        rectangle(0, 0, 200, 100),
    )];
    assert!(!belongs_to_large_bottom(&crack, &bottom, -150.0).unwrap());
}

#[test]
fn sub_ten_unit_gap_is_not_hidden_by_the_dropped_safety_argument() {
    let crack = rectangle(-5, 0, 95, 100);
    let bottom = vec![surface(
        RegionSurfaceKind::BottomBridge,
        rectangle(0, 0, 400, 100),
    )];
    assert!(!belongs_to_large_bottom(&crack, &bottom, -10.0).unwrap());
}

#[test]
fn holed_case_reaches_singleton_and_collection_offset_sites() {
    let mut hole = polygon(300, 300, 700, 700);
    hole.reverse();
    let bottom = ExPolygon::new(polygon(0, 0, 1_000, 1_000), vec![hole]);
    let crack = rectangle(100, 100, 150, 150);
    let mut top = vec![surface(RegionSurfaceKind::Top, crack)];
    let mut bottom = vec![surface(RegionSurfaceKind::BottomBridge, bottom)];
    reset_geometry_hooks();
    resolve(&mut top, &mut bottom, 20, true).unwrap();
    let events = geometry_events();
    assert!(events.contains(&GeometryStep::SingletonCrackErosion));
    assert!(events.contains(&GeometryStep::CollectionResidualErosion));
    reset_geometry_hooks();
}

#[test]
fn overlapping_holed_expolygons_distinguish_singleton_and_collection_outputs() {
    let mut first_hole = polygon(150, 150, 250, 250);
    first_hole.reverse();
    let mut second_hole = polygon(750, 150, 850, 250);
    second_hole.reverse();
    let first = ExPolygon::new(polygon(0, 0, 700, 700), vec![first_hole]);
    let second = ExPolygon::new(polygon(300, 0, 1_000, 700), vec![second_hole]);
    let mut singleton = offset_expolygon(&first, -10.0, JoinType::Miter, 3.0).unwrap();
    singleton.extend(offset_expolygon(&second, -10.0, JoinType::Miter, 3.0).unwrap());
    let collection = offset_expolygons(&[first, second], -10.0, JoinType::Miter, 3.0).unwrap();
    assert_ne!(singleton, collection);
}
