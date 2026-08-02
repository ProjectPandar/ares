use super::super::Point;

pub(crate) fn point_in_polygon(point: Point, path: &[Point]) -> i32 {
    let count = path.len();
    if count < 3 {
        return 0;
    }

    let mut result = 0;
    let mut current = path[0];
    for index in 1..=count {
        let next = if index == count { path[0] } else { path[index] };
        if next.y() == point.y()
            && (next.x() == point.x()
                || (current.y() == point.y()
                    && ((next.x() > point.x()) == (current.x() < point.x()))))
        {
            return -1;
        }
        if crosses_boundary(current, next, point, &mut result) {
            return -1;
        }
        current = next;
    }
    result
}

fn crosses_boundary(current: Point, next: Point, point: Point, result: &mut i32) -> bool {
    if (current.y() < point.y()) == (next.y() < point.y()) {
        return false;
    }
    if current.x() >= point.x() {
        if next.x() > point.x() {
            *result = 1 - *result;
            return false;
        }
        return crossing(current, next, point, result);
    }
    next.x() > point.x() && crossing(current, next, point, result)
}

fn crossing(current: Point, next: Point, point: Point, result: &mut i32) -> bool {
    let determinant = (current.x() - point.x()) as f64 * (next.y() - point.y()) as f64
        - (next.x() - point.x()) as f64 * (current.y() - point.y()) as f64;
    if determinant == 0.0 {
        true
    } else {
        if (determinant > 0.0) == (next.y() > current.y()) {
            *result = 1 - *result;
        }
        false
    }
}
