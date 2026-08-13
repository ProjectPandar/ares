use super::*;
use crate::geometry::ClipperError;

#[test]
fn task22o57_closed_range_error_is_first_and_inputs_are_unchanged() {
    let deep = vec![outside_range()];
    let surfaces = vec![surface(
        RegionSurfaceKind::Internal,
        rectangle(0, 0, 1_000, 1_000),
    )];
    let fill = vec![expolygon(rectangle(0, 0, 1_000, 1_000), Vec::new())];
    let lines = vec![line(&[(0, 500), (1_000, 500)])];
    let regions = [region(&surfaces, &fill, ProcessInfillPattern::Lightning)];
    let before = snapshot_inputs(&deep, &regions, &lines);

    assert!(matches!(
        prepare_current_layer_bridge_context(&deep, &regions, &lines, 100, CoordinateScale::Normal,),
        Err(ClipperError::CoordinateOutOfRange)
    ));
    assert_eq!(snapshot_inputs(&deep, &regions, &lines), before);
}

#[test]
fn task22o57_open_range_error_is_atomic_after_closed_geometry_succeeds() {
    let deep = vec![rectangle(-1_000, -1_000, 2_000, 2_000)];
    let surfaces = vec![surface(
        RegionSurfaceKind::Internal,
        rectangle(0, 0, 1_000, 1_000),
    )];
    let lines = vec![line(&[(HI_RANGE, 500), (HI_RANGE + 10, 500)])];
    let regions = [region(&surfaces, &[], ProcessInfillPattern::Rectilinear)];
    let before = snapshot_inputs(&deep, &regions, &lines);

    assert!(matches!(
        prepare_current_layer_bridge_context(&deep, &regions, &lines, 100, CoordinateScale::Normal,),
        Err(ClipperError::CoordinateOutOfRange)
    ));
    assert_eq!(snapshot_inputs(&deep, &regions, &lines), before);
}

#[test]
fn task22o57_repeat_calls_are_identical_and_preserve_all_borrowed_allocations() {
    let deep = vec![rectangle(-1_000, -1_000, 3_000, 2_000)];
    let surfaces = vec![surface(
        RegionSurfaceKind::Internal,
        rectangle(0, 0, 2_000, 1_000),
    )];
    let fill = vec![expolygon(
        rectangle(-500, -500, 2_500, 1_500),
        vec![rectangle(500, 250, 1_000, 750)],
    )];
    let lines = vec![line(&[(-500, 500), (2_500, 500)])];
    let regions = [region(&surfaces, &fill, ProcessInfillPattern::Lightning)];
    let before = snapshot_inputs(&deep, &regions, &lines);

    let first = prepare(&deep, &regions, &lines, 100, CoordinateScale::Normal);
    let second = prepare(&deep, &regions, &lines, 100, CoordinateScale::Normal);
    assert_eq!(snapshot_context(&first), snapshot_context(&second));
    assert_eq!(snapshot_inputs(&deep, &regions, &lines), before);
}

type PointsSnapshot = Vec<(i64, i64)>;
type PolygonAllocationSnapshot = (usize, PointsSnapshot);
type SurfaceSnapshot = (RegionSurfaceKind, usize, PointsSnapshot);
type FillSnapshot = (usize, PointsSnapshot, Vec<PolygonAllocationSnapshot>);

#[derive(Debug, Eq, PartialEq)]
struct InputSnapshot {
    deep_outer: (usize, usize, usize),
    deep: Vec<PolygonAllocationSnapshot>,
    regions: Vec<(usize, usize, ProcessInfillPattern)>,
    surfaces: Vec<SurfaceSnapshot>,
    fill: Vec<FillSnapshot>,
    lines_outer: (usize, usize, usize),
    lines: Vec<PolygonAllocationSnapshot>,
}

fn snapshot_inputs(
    deep: &Vec<Polygon>,
    regions: &[CurrentLayerBridgeRegion<'_>],
    lines: &Vec<Polyline>,
) -> InputSnapshot {
    InputSnapshot {
        deep_outer: (deep.as_ptr() as usize, deep.len(), deep.capacity()),
        deep: deep
            .iter()
            .map(|polygon| (polygon.points().as_ptr() as usize, points(polygon.points())))
            .collect(),
        regions: regions
            .iter()
            .map(|region| {
                (
                    region.fill_surfaces.as_ptr() as usize,
                    region.fill_expolygons.as_ptr() as usize,
                    region.sparse_infill_pattern,
                )
            })
            .collect(),
        surfaces: regions
            .iter()
            .flat_map(|region| region.fill_surfaces)
            .map(|surface| {
                let (kind, expolygon, ..) = surface.as_parts();
                (
                    kind,
                    expolygon.contour().points().as_ptr() as usize,
                    points(expolygon.contour().points()),
                )
            })
            .collect(),
        fill: regions
            .iter()
            .flat_map(|region| region.fill_expolygons)
            .map(|expolygon| {
                (
                    expolygon.contour().points().as_ptr() as usize,
                    points(expolygon.contour().points()),
                    expolygon
                        .holes()
                        .iter()
                        .map(|hole| (hole.points().as_ptr() as usize, points(hole.points())))
                        .collect(),
                )
            })
            .collect(),
        lines_outer: (lines.as_ptr() as usize, lines.len(), lines.capacity()),
        lines: lines
            .iter()
            .map(|line| (line.points().as_ptr() as usize, points(line.points())))
            .collect(),
    }
}

fn points(points: &[Point]) -> Vec<(i64, i64)> {
    points.iter().map(|point| (point.x(), point.y())).collect()
}

fn snapshot_context(output: &CurrentLayerBridgeContext) -> Vec<Vec<Vec<(i64, i64)>>> {
    vec![
        snapshot_polygons(&output.deep_infill_area),
        snapshot_polygons(&output.lightning_area),
        snapshot_polygons(&output.expansion_area),
        snapshot_polygons(&output.total_fill_area),
        snapshot_polygons(&output.total_top_area),
        snapshot_polylines(&output.anchors),
        snapshot_polygons(&output.internal_unsupported_area),
    ]
}
