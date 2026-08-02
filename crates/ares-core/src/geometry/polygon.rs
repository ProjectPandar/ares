use super::{Line, Point};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Polygon {
    points: Vec<Point>,
}

fn distance_squared(left: Point, right: Point) -> f64 {
    let dx = (left.x() - right.x()) as f64;
    let dy = (left.y() - right.y()) as f64;
    dx * dx + dy * dy
}

fn consider_projection(point: Point, candidate: Point, projection: &mut Point, minimum: &mut f64) {
    let distance = distance_squared(point, candidate).sqrt();
    if distance < *minimum {
        *projection = candidate;
        *minimum = distance;
    }
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

    pub(crate) fn lines(&self) -> Vec<Line> {
        if self.points.len() < 3 {
            return Vec::new();
        }
        let mut lines = self
            .points
            .windows(2)
            .map(|points| Line::new(points[0], points[1]))
            .collect::<Vec<_>>();
        if let (Some(&first), Some(&last)) = (self.points.first(), self.points.last()) {
            lines.push(Line::new(last, first));
        }
        lines
    }

    pub(crate) fn point_projection(&self, point: Point) -> Point {
        let Some(&first) = self.points.first() else {
            return point;
        };
        let mut projection = first;
        let mut minimum = f64::MAX;
        for index in 0..self.points.len() {
            let pt0 = self.points[index];
            let pt1 = self.points[(index + 1) % self.points.len()];
            consider_projection(point, pt0, &mut projection, &mut minimum);
            consider_projection(point, pt1, &mut projection, &mut minimum);
            let vx = (pt1.x() - pt0.x()) as f64;
            let vy = (pt1.y() - pt0.y()) as f64;
            let vax = (point.x() - pt0.x()) as f64;
            let vay = (point.y() - pt0.y()) as f64;
            let t = (vax * vx + vay * vy) / (vx * vx + vy * vy);
            if t > 0.0 && t < 1.0 {
                let candidate = Point::new(
                    (pt0.x() as f64 + t * vx + 0.5).floor() as i64,
                    (pt0.y() as f64 + t * vy + 0.5).floor() as i64,
                );
                consider_projection(point, candidate, &mut projection, &mut minimum);
            }
        }
        projection
    }

    pub(crate) fn on_boundary(&self, point: Point, epsilon: f64) -> bool {
        distance_squared(self.point_projection(point), point) < epsilon * epsilon
    }

    pub(crate) fn intersection(&self, line: Line) -> Option<Point> {
        let (&first, &last) = (self.points.first()?, self.points.last()?);
        Line::new(first, last).intersection(line).or_else(|| {
            self.points
                .windows(2)
                .find_map(|points| Line::new(points[0], points[1]).intersection(line))
        })
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
