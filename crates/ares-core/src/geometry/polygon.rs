use super::Point;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Polygon {
    points: Vec<Point>,
}

impl Polygon {
    pub(crate) fn new(points: Vec<Point>) -> Self {
        Self { points }
    }

    pub(crate) fn points(&self) -> &[Point] {
        &self.points
    }

    pub(crate) fn into_points(self) -> Vec<Point> {
        self.points
    }

    pub(crate) fn reverse(&mut self) {
        self.points.reverse();
    }

    pub(crate) fn split_at_first_point(&self) -> super::Polyline {
        let mut points = Vec::with_capacity(self.points.len() + 1);
        points.extend_from_slice(&self.points);
        if let Some(&first) = self.points.first() {
            points.push(first);
        }
        super::Polyline::new(points)
    }

    pub(crate) fn douglas_peucker(&mut self, tolerance: f64) {
        self.points = super::simplification::simplify_closed_points(
            std::mem::take(&mut self.points),
            tolerance,
        );
    }

    pub(crate) fn contains(&self, point: &Point) -> bool {
        super::clipper::point_in_polygon(*point, &self.points) != 0
    }

    pub(crate) fn area(&self) -> f64 {
        let Some(previous) = self.points.last().filter(|_| self.points.len() >= 3) else {
            return 0.0;
        };
        let mut previous_x = previous.x() as f64;
        let mut previous_y = previous.y() as f64;
        let mut area = 0.0;
        for point in &self.points {
            let current_x = point.x() as f64;
            let current_y = point.y() as f64;
            area += previous_x * current_y - previous_y * current_x;
            previous_x = current_x;
            previous_y = current_y;
        }
        0.5 * area
    }
}
