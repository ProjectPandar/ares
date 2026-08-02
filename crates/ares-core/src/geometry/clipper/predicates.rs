use super::super::Point;
use super::types::Edge;

pub(crate) const LO_RANGE: i64 = 0x3fff_ffff;
pub(crate) const HI_RANGE: i64 = 0x3fff_ffff_ffff_ffff;
pub(crate) const HORIZONTAL: f64 = -1.0e40;

pub(crate) fn fixed_round(value: f64) -> i64 {
    let rounded = if value == 0.499_999_999_999_999_94 {
        0.0
    } else {
        (value + 0.5).floor()
    };
    rounded as i64
}

pub(crate) fn slopes_equal(dx1: i64, dy1: i64, dx2: i64, dy2: i64, use_full_range: bool) -> bool {
    if use_full_range {
        i128::from(dy1) * i128::from(dx2) == i128::from(dx1) * i128::from(dy2)
    } else {
        dy1 * dx2 == dx1 * dy2
    }
}

pub(crate) fn get_dx(point1: Point, point2: Point) -> f64 {
    if point1.y() == point2.y() {
        HORIZONTAL
    } else {
        (point2.x() - point1.x()) as f64 / (point2.y() - point1.y()) as f64
    }
}

pub(super) fn top_x(edge: Edge, current_y: i64) -> i64 {
    if current_y == edge.top.y() {
        edge.top.x()
    } else {
        edge.bottom.x() + fixed_round(edge.dx * (current_y - edge.bottom.y()) as f64)
    }
}

pub(super) fn intersect_point(first: Edge, second: Edge) -> Point {
    if first.dx == second.dx {
        return Point::new(top_x(first, first.current.y()), first.current.y());
    }

    let (mut x, mut y) = if first.dx == 0.0 {
        let x = first.bottom.x();
        let y = if second.is_horizontal() {
            second.bottom.y()
        } else {
            fixed_round(
                x as f64 / second.dx + second.bottom.y() as f64
                    - second.bottom.x() as f64 / second.dx,
            )
        };
        (x, y)
    } else if second.dx == 0.0 {
        let x = second.bottom.x();
        let y = if first.is_horizontal() {
            first.bottom.y()
        } else {
            fixed_round(
                x as f64 / first.dx + first.bottom.y() as f64 - first.bottom.x() as f64 / first.dx,
            )
        };
        (x, y)
    } else {
        let first_intercept = first.bottom.x() as f64 - first.bottom.y() as f64 * first.dx;
        let second_intercept = second.bottom.x() as f64 - second.bottom.y() as f64 * second.dx;
        let quotient = (second_intercept - first_intercept) / (first.dx - second.dx);
        let y = fixed_round(quotient);
        let x = if first.dx.abs() < second.dx.abs() {
            fixed_round(first.dx * quotient + first_intercept)
        } else {
            fixed_round(second.dx * quotient + second_intercept)
        };
        (x, y)
    };

    if y < first.top.y() || y < second.top.y() {
        y = if first.top.y() > second.top.y() {
            first.top.y()
        } else {
            second.top.y()
        };
        x = top_x(
            if first.dx.abs() < second.dx.abs() {
                first
            } else {
                second
            },
            y,
        );
    }
    if y > first.current.y() {
        y = first.current.y();
        x = top_x(
            if first.dx.abs() > second.dx.abs() {
                second
            } else {
                first
            },
            y,
        );
    }

    Point::new(x, y)
}

pub(super) fn slopes_equal_three(first: Point, middle: Point, last: Point, full: bool) -> bool {
    slopes_equal(
        first.x() - middle.x(),
        first.y() - middle.y(),
        middle.x() - last.x(),
        middle.y() - last.y(),
        full,
    )
}

pub(super) fn slopes_equal_four(
    first_start: Point,
    first_end: Point,
    second_start: Point,
    second_end: Point,
    full: bool,
) -> bool {
    slopes_equal(
        first_start.x() - first_end.x(),
        first_start.y() - first_end.y(),
        second_start.x() - second_end.x(),
        second_start.y() - second_end.y(),
        full,
    )
}

pub(crate) fn area(path: &[Point]) -> f64 {
    if path.len() < 3 {
        return 0.0;
    }

    let mut accumulated = 0.0;
    let mut previous = path.len() - 1;
    for index in 0..path.len() {
        accumulated += (path[previous].x() as f64 + path[index].x() as f64)
            * (path[previous].y() as f64 - path[index].y() as f64);
        previous = index;
    }
    -accumulated * 0.5
}
