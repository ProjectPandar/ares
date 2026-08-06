mod lines;

use crate::geometry::clipper::fixed_round;
use crate::geometry::{Point, Polygon};

use super::{ClipperOffset, EndType, JoinType};

#[allow(clippy::approx_constant)]
const PI: f64 = 3.141_592_653_589_793;
const TWO_PI: f64 = PI * 2.0;
const DEFAULT_ARC_TOLERANCE: f64 = 0.25;
const NEAR_ZERO: f64 = 1.0e-20;

#[derive(Clone, Copy)]
struct UnitNormal {
    x: f64,
    y: f64,
}

struct PreparedOffset {
    delta: f64,
    sin: f64,
    cos: f64,
    miter_threshold: f64,
    steps: f64,
    steps_per_radian: f64,
}

#[derive(Clone, Copy)]
struct Corner {
    point: Point,
    previous: UnitNormal,
    current: UnitNormal,
    sin_a: f64,
    dot: f64,
}

impl ClipperOffset {
    pub(crate) fn generate_raw(&mut self, delta: f64) -> Vec<Polygon> {
        self.fix_orientations();
        if -NEAR_ZERO < delta && delta < NEAR_ZERO {
            return self
                .paths
                .iter()
                .filter(|path| path.end_type == EndType::ClosedPolygon)
                .map(|path| path.contour.clone())
                .collect();
        }

        let prepared = PreparedOffset::new(self, delta);
        let mut output = Vec::with_capacity(self.paths.len() * 2);
        for path in &self.paths {
            let source = path.contour.points();
            if source.is_empty()
                || (delta <= 0.0 && (source.len() < 3 || path.end_type != EndType::ClosedPolygon))
            {
                continue;
            }
            if source.len() == 1 {
                output.push(generate_one_point(source[0], path.join_type, &prepared));
                continue;
            }
            match path.end_type {
                EndType::ClosedPolygon => {
                    output.push(generate_closed(source, path.join_type, &prepared));
                }
                EndType::ClosedLine => {
                    output.extend(lines::generate_closed_line(
                        source,
                        path.join_type,
                        &prepared,
                    ));
                }
                EndType::OpenButt => output.push(lines::generate_open(
                    source,
                    path.join_type,
                    false,
                    &prepared,
                )),
                EndType::OpenRound => output.push(lines::generate_open(
                    source,
                    path.join_type,
                    true,
                    &prepared,
                )),
            }
        }
        output
    }
}

fn generate_one_point(point: Point, join_type: JoinType, prepared: &PreparedOffset) -> Polygon {
    if join_type != JoinType::Round {
        let delta = prepared.delta;
        return Polygon::new(vec![
            Point::new(
                fixed_round(point.x() as f64 - delta),
                fixed_round(point.y() as f64 - delta),
            ),
            Point::new(
                fixed_round(point.x() as f64 + delta),
                fixed_round(point.y() as f64 - delta),
            ),
            Point::new(
                fixed_round(point.x() as f64 + delta),
                fixed_round(point.y() as f64 + delta),
            ),
            Point::new(
                fixed_round(point.x() as f64 - delta),
                fixed_round(point.y() as f64 + delta),
            ),
        ]);
    }

    let mut output = Vec::new();
    let (mut x, mut y) = (1.0, 0.0);
    for _ in 0..prepared.steps.floor() as usize {
        output.push(Point::new(
            fixed_round(point.x() as f64 + x * prepared.delta),
            fixed_round(point.y() as f64 + y * prepared.delta),
        ));
        let previous_x = x;
        x = x * prepared.cos - prepared.sin * y;
        y = previous_x * prepared.sin + y * prepared.cos;
    }
    Polygon::new(output)
}

impl PreparedOffset {
    fn new(offset: &ClipperOffset, delta: f64) -> Self {
        let miter_threshold = if offset.miter_limit > 2.0 {
            2.0 / (offset.miter_limit * offset.miter_limit)
        } else {
            0.5
        };
        let absolute_delta = delta.abs();
        let arc = if offset.arc_tolerance <= 0.0 {
            DEFAULT_ARC_TOLERANCE
        } else {
            offset
                .arc_tolerance
                .min(absolute_delta * DEFAULT_ARC_TOLERANCE)
        };
        let mut steps = PI / (1.0 - arc / absolute_delta).acos();
        if steps > absolute_delta * PI {
            steps = absolute_delta * PI;
        }
        let mut sin = (TWO_PI / steps).sin();
        let cos = (TWO_PI / steps).cos();
        if delta < 0.0 {
            sin = -sin;
        }
        Self {
            delta,
            sin,
            cos,
            miter_threshold,
            steps,
            steps_per_radian: steps / TWO_PI,
        }
    }
}

impl Corner {
    fn new(point: Point, previous: UnitNormal, current: UnitNormal) -> Self {
        Self {
            point,
            previous,
            current,
            sin_a: previous.x * current.y - current.x * previous.y,
            dot: previous.x * current.x + current.y * previous.y,
        }
    }
}

fn generate_closed(source: &[Point], join_type: JoinType, prepared: &PreparedOffset) -> Polygon {
    let mut normals = Vec::with_capacity(source.len());
    for edge in source.windows(2) {
        normals.push(unit_normal(edge[0], edge[1]));
    }
    normals.push(unit_normal(source[source.len() - 1], source[0]));

    let mut output = Vec::new();
    let mut previous = source.len() - 1;
    for current in 0..source.len() {
        let corner = Corner::new(source[current], normals[previous], normals[current]);
        if generate_corner(corner, join_type, prepared, &mut output) {
            previous = current;
        }
    }
    Polygon::new(output)
}

fn unit_normal(first: Point, second: Point) -> UnitNormal {
    if first == second {
        return UnitNormal { x: 0.0, y: 0.0 };
    }
    let mut dx = (second.x() - first.x()) as f64;
    let mut dy = (second.y() - first.y()) as f64;
    let factor = 1.0 / (dx * dx + dy * dy).sqrt();
    dx *= factor;
    dy *= factor;
    UnitNormal { x: dy, y: -dx }
}

fn generate_corner(
    mut corner: Corner,
    join_type: JoinType,
    prepared: &PreparedOffset,
    output: &mut Vec<Point>,
) -> bool {
    if (corner.sin_a * prepared.delta).abs() < 1.0 {
        if corner.dot > 0.0 {
            output.push(offset_point_with_normal(
                corner.point,
                corner.previous,
                prepared.delta,
            ));
            return false;
        }
    } else {
        corner.sin_a = corner.sin_a.clamp(-1.0, 1.0);
    }

    if corner.sin_a * prepared.delta < 0.0 {
        output.push(offset_point_with_normal(
            corner.point,
            corner.previous,
            prepared.delta,
        ));
        output.push(corner.point);
        output.push(offset_point_with_normal(
            corner.point,
            corner.current,
            prepared.delta,
        ));
    } else {
        match join_type {
            JoinType::Miter => {
                let ratio = 1.0 + corner.dot;
                if ratio >= prepared.miter_threshold {
                    do_miter(corner, prepared.delta, ratio, output);
                } else {
                    do_square(corner, prepared.delta, output);
                }
            }
            JoinType::Square => do_square(corner, prepared.delta, output),
            JoinType::Round => do_round(corner, prepared, output),
        }
    }
    true
}

fn offset_point_with_normal(point: Point, normal: UnitNormal, delta: f64) -> Point {
    Point::new(
        fixed_round(point.x() as f64 + normal.x * delta),
        fixed_round(point.y() as f64 + normal.y * delta),
    )
}

fn do_miter(corner: Corner, delta: f64, ratio: f64, output: &mut Vec<Point>) {
    let quotient = delta / ratio;
    output.push(Point::new(
        fixed_round(corner.point.x() as f64 + (corner.previous.x + corner.current.x) * quotient),
        fixed_round(corner.point.y() as f64 + (corner.previous.y + corner.current.y) * quotient),
    ));
}

fn do_square(corner: Corner, delta: f64, output: &mut Vec<Point>) {
    let tangent = (corner.sin_a.atan2(corner.dot) / 4.0).tan();
    output.push(Point::new(
        fixed_round(
            corner.point.x() as f64 + delta * (corner.previous.x - corner.previous.y * tangent),
        ),
        fixed_round(
            corner.point.y() as f64 + delta * (corner.previous.y + corner.previous.x * tangent),
        ),
    ));
    output.push(Point::new(
        fixed_round(
            corner.point.x() as f64 + delta * (corner.current.x + corner.current.y * tangent),
        ),
        fixed_round(
            corner.point.y() as f64 + delta * (corner.current.y - corner.current.x * tangent),
        ),
    ));
}

fn do_round(corner: Corner, prepared: &PreparedOffset, output: &mut Vec<Point>) {
    let angle = corner.sin_a.atan2(corner.dot);
    let steps = fixed_round(prepared.steps_per_radian * angle.abs()).max(1) as usize;
    let (mut x, mut y) = (corner.previous.x, corner.previous.y);
    for _ in 0..steps {
        output.push(Point::new(
            fixed_round(corner.point.x() as f64 + x * prepared.delta),
            fixed_round(corner.point.y() as f64 + y * prepared.delta),
        ));
        let previous_x = x;
        x = x * prepared.cos - prepared.sin * y;
        y = previous_x * prepared.sin + y * prepared.cos;
    }
    output.push(offset_point_with_normal(
        corner.point,
        corner.current,
        prepared.delta,
    ));
}
