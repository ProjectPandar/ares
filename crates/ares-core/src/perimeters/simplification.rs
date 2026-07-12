use crate::{PerimeterOptions, Point2, WallGenerator};

pub(super) fn simplify_closed_loop(points: Vec<Point2>, options: PerimeterOptions) -> Vec<Point2> {
    if options.wall_generator() != WallGenerator::Arachne || points.len() <= 3 {
        return points;
    }

    let max_segment = options.wall_maximum_resolution_mm();
    let max_deviation = options.wall_maximum_deviation_mm();
    let mut simplified = points;
    let mut changed = true;
    while changed && simplified.len() > 3 {
        changed = false;
        let mut index = 0;
        while simplified.len() > 3 && index < simplified.len() {
            if removable_vertex(&simplified, index, max_segment, max_deviation) {
                simplified.remove(index);
                changed = true;
            } else {
                index += 1;
            }
        }
    }
    simplified
}

fn removable_vertex(points: &[Point2], index: usize, max_segment: f64, max_deviation: f64) -> bool {
    let previous = points[(index + points.len() - 1) % points.len()];
    let current = points[index];
    let next = points[(index + 1) % points.len()];

    distance(previous, current) <= max_segment
        && distance(current, next) <= max_segment
        && point_segment_distance(current, previous, next) <= max_deviation
}

fn point_segment_distance(point: Point2, start: Point2, end: Point2) -> f64 {
    let dx = end.x() - start.x();
    let dy = end.y() - start.y();
    let length_squared = dx * dx + dy * dy;
    if length_squared == 0.0 {
        return distance(point, start);
    }
    let t = (((point.x() - start.x()) * dx + (point.y() - start.y()) * dy) / length_squared)
        .clamp(0.0, 1.0);
    distance(point, Point2::new(start.x() + t * dx, start.y() + t * dy))
}

fn distance(start: Point2, end: Point2) -> f64 {
    let dx = end.x() - start.x();
    let dy = end.y() - start.y();
    (dx * dx + dy * dy).sqrt()
}
