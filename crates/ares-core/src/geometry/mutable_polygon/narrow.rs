//! `MutablePolygon.cpp::clip_narrow_corner`.

use super::{
    cross, delta, dot, dot_f64, norm2,
    ring::{Ring, Unprocessed},
    translated,
};
use crate::geometry::Point;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Status {
    Free,
    Blocked,
    Far,
}

pub(super) fn clip(
    ring: &mut Ring,
    range: &mut Unprocessed,
    (origin, mut before, mut after): (Point, usize, usize),
    mut distance2: i64,
    (length, epsilon): (i64, i64),
) -> bool {
    let length2 = length * length;
    let mut forward = Status::Free;
    let mut backward = Status::Free;
    let mut p0 = ring.point(before);
    let mut p2 = ring.point(after);
    let mut p02 = p0;
    let mut p22 = p2;
    let mut next_distance2 = 0;
    while ring.len >= 3 {
        if forward == Status::Far && backward == Status::Far {
            p02 = ring.point(ring.previous(before));
            p22 = ring.point(ring.next(after));
            let vector = delta(p22, p02);
            let candidate = dot(vector, vector);
            if candidate > length2 {
                next_distance2 = candidate;
                break;
            }
            let next = range.remove_back(ring, before);
            before = ring.previous(next);
            after = range.remove_front(ring, after);
            if ring.len <= 2 {
                return true;
            }
            forward = Status::Free;
            backward = Status::Free;
            distance2 = candidate;
            p0 = p02;
            p2 = p22;
        } else if forward != Status::Free && backward != Status::Free {
            break;
        }
        let v2 = delta(p2, origin);
        let v0 = delta(p0, origin);
        if forward == Status::Free && (backward != Status::Free || dot(v2, v2) < dot(v0, v0)) {
            p22 = ring.point(ring.next(after));
            if cross(delta(p2, p0), delta(p22, p0)) > 0 {
                forward = Status::Blocked;
                continue;
            }
            let vector = delta(p22, p0);
            let candidate = dot(vector, vector);
            if candidate > length2 {
                forward = Status::Far;
                next_distance2 = candidate;
            } else {
                forward = Status::Free;
                after = range.remove_front(ring, after);
                p2 = p22;
                distance2 = candidate;
            }
        } else {
            p02 = ring.point(ring.previous(before));
            if cross(delta(p02, p2), delta(p0, p2)) > 0 {
                backward = Status::Blocked;
                continue;
            }
            let vector = delta(p2, p02);
            let candidate = dot(vector, vector);
            if candidate > length2 {
                backward = Status::Far;
                next_distance2 = candidate;
            } else {
                backward = Status::Free;
                let next = range.remove_back(ring, before);
                before = ring.previous(next);
                p0 = p02;
                distance2 = candidate;
            }
        }
    }
    if ring.len <= 3 {
        if ring.len < 3 || (forward == Status::Far && backward == Status::Far) {
            ring.clear();
        }
        return true;
    }
    let almost2 = (length - epsilon) * (length - epsilon);
    if (forward == Status::Blocked && backward == Status::Blocked) || distance2 > almost2 {
        return false;
    }
    if next_distance2 < almost2 {
        if forward == Status::Far {
            range.remove_back(ring, before);
        }
        if backward == Status::Far {
            range.remove_front(ring, after);
        }
        return ring.len <= 2;
    }
    if forward == Status::Blocked || backward == Status::Blocked {
        if forward == Status::Far {
            std::mem::swap(&mut p0, &mut p2);
            std::mem::swap(&mut p02, &mut p22);
        }
        let vector = delta(p02, p0).map(|value| value as f64);
        let d = delta(p0, p2).map(|value| value as f64);
        let a = norm2(vector);
        let b = 2.0 * dot_f64(d, vector);
        let discriminant = b * b - 4.0 * a * (norm2(d) - length2 as f64);
        let t = (-b + discriminant.sqrt()) / (2.0 * a);
        let endpoint = if backward == Status::Far {
            after
        } else {
            before
        };
        ring.set_point(endpoint, translated(ring.point(endpoint), vector, t));
    } else {
        let current = (distance2 as f64).sqrt();
        let t = (length as f64 - current) / ((next_distance2 as f64).sqrt() - current);
        ring.set_point(
            before,
            translated(p0, delta(p02, p0).map(|value| value as f64), t),
        );
        ring.set_point(
            after,
            translated(p2, delta(p22, p2).map(|value| value as f64), t),
        );
    }
    false
}
