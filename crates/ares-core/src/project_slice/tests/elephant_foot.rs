use crate::geometry::{Coord, EdgeGrid, ExPolygon, Point, Polygon};

use super::super::elephant_foot::distance::{DistanceThresholds, ResampledPoint, resample_polygon};

mod distance;
mod kernel;
mod oracle;
mod profile;

const RESAMPLE_INTERVAL: f64 = 500_000.0;
const SCALED_EPSILON: f64 = f64::from_bits(0x4059_0000_0000_0001);

fn points(values: &[(Coord, Coord)]) -> Vec<Point> {
    values.iter().map(|&(x, y)| Point::new(x, y)).collect()
}

fn build_grid(outer: &[(Coord, Coord)], holes: &[&[(Coord, Coord)]], radius: f64) -> EdgeGrid {
    let outer = points(outer);
    let min_x = outer.iter().map(|point| point.x()).min().unwrap();
    let min_y = outer.iter().map(|point| point.y()).min().unwrap();
    let max_x = outer.iter().map(|point| point.x()).max().unwrap();
    let max_y = outer.iter().map(|point| point.y()).max().unwrap();
    let epsilon = SCALED_EPSILON as Coord;
    let expolygon = ExPolygon::new(
        Polygon::new(outer),
        holes
            .iter()
            .map(|hole| Polygon::new(points(hole)))
            .collect(),
    );
    EdgeGrid::new(
        &expolygon,
        Point::new(min_x - epsilon, min_y - epsilon),
        Point::new(max_x + epsilon, max_y + epsilon),
        (0.7 * radius) as Coord,
    )
    .unwrap()
}

fn resampled_contour(grid: &EdgeGrid, contour_index: usize) -> (Vec<Point>, Vec<ResampledPoint>) {
    resample_polygon(grid.contour(contour_index), RESAMPLE_INTERVAL).unwrap()
}

const fn thresholds(compensation: f64, radius: f64) -> DistanceThresholds {
    DistanceThresholds::new(compensation, radius, SCALED_EPSILON)
}
