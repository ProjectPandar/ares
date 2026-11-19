//! Fixed-dimensional kernels from Qhull 8.0.2 `geom_r.c` and `geom2_r.c`.
//!
//! Copyright (c) 1993-2020 The Geometry Center.
//! See `QHULL-LICENSE.txt` for distribution and modification notices.

pub(super) type Point = [f64; 3];

pub(super) struct Precision {
    pub round: f64,
    pub visible: f64,
    pub outside: f64,
    pub near_inside: f64,
    zero: Point,
    max_abs: f64,
}

impl Precision {
    // qh_detmaxoutside with default premerge and postmerge centrums.
    pub fn outer_limit(&self) -> f64 {
        (3.0 * self.visible + self.round + self.round).max(self.outside)
    }

    // qh_maxmin, qh_detroundoff, qh_maxsimplex
    pub fn initial(points: &[Point]) -> Option<(Self, [usize; 4])> {
        let mut extremes = [0; 6];
        let mut width: f64 = 0.0;
        let mut max_abs: f64 = 0.0;
        let mut sum_abs = 0.0;
        let mut zero = [0.0; 3];
        for axis in 0..3 {
            let (low, high) = axis_extremes(points, axis);
            let (lo, hi) = (points[low][axis], points[high][axis]);
            extremes[2 * axis] = low;
            extremes[2 * axis + 1] = high;
            let abs = hi.max(-lo);
            width = width.max(hi - lo);
            max_abs = max_abs.max(abs);
            sum_abs += abs;
            zero[axis] = 80.0 * sum_abs * f64::EPSILON;
        }
        let [low, high, ..] = extremes;
        if low == high {
            return None;
        }
        let max_dist_sum = (3.0_f64.sqrt() * max_abs).min(sum_abs);
        let round = f64::EPSILON * (3.0 * max_dist_sum * 1.01 + max_abs);
        let visible = 2.0 * round;
        let precision = Self {
            round,
            visible,
            outside: 2.0 * visible,
            near_inside: (3.0 * visible + round) * 5.0,
            zero,
            max_abs,
        };
        let mut seed = [low, high, 0, 0];
        let mut best_det = points[high][0] - points[low][0];
        for dim in 2..=3 {
            let target_det = best_det * width;
            let near_zero = 10.0 * precision.zero[dim - 1];
            let (id, det) = seed_vertex(points, &seed[..dim], extremes, (near_zero, target_det))?;
            seed[dim] = id;
            best_det = det;
        }

        Some((precision, seed))
    }
}

fn axis_extremes(points: &[Point], axis: usize) -> (usize, usize) {
    let (mut low, mut high) = (0, 0);
    for (id, point) in points.iter().enumerate().skip(1) {
        if points[high][axis] < point[axis] {
            high = id;
        } else if points[low][axis] > point[axis] {
            low = id;
        }
    }
    (low, high)
}

fn seed_vertex(
    points: &[Point],
    seed: &[usize],
    extremes: [usize; 6],
    (near_zero, target_det): (f64, f64),
) -> Option<(usize, f64)> {
    let mut best = None;
    let mut best_det = -1.0;
    for id in extremes.into_iter().filter(|id| !seed.contains(id)) {
        let det = simplex_det(points, seed, id).abs();
        if det > best_det {
            best_det = det;
            best = Some(id);
        }
    }
    let narrow = best_det > 0.0 && best_det / target_det < 1.0e-3;
    if best.is_some() && best_det >= near_zero && !narrow {
        return best.map(|id| (id, best_det));
    }
    let min_det = 10.0 * 1.0e-3 * target_det;
    for id in (0..points.len()).filter(|id| !extremes.contains(id) && !seed.contains(id)) {
        let det = simplex_det(points, seed, id).abs();
        if det > best_det {
            best_det = det;
            best = Some(id);
            if det >= near_zero && det > min_det {
                break;
            }
        }
    }
    best.map(|id| (id, best_det))
}

#[derive(Clone, Copy)]
pub(super) struct Plane {
    pub normal: Point,
    pub offset: f64,
}

impl Plane {
    // qh_setfacetplane, qh_sethyperplane_det
    pub fn new([a, b, c]: [Point; 3], top: bool, inside: Point, p: &Precision) -> Self {
        let u = sub(b, a);
        let v = sub(c, a);
        let normal = normalize(
            [
                det2(v[1], v[2], u[1], u[2]),
                det2(u[0], u[2], v[0], v[2]),
                det2(v[0], v[1], u[0], u[1]),
            ],
            top,
            p,
        );
        let offset = -dot(a, normal);
        if [c, b]
            .into_iter()
            .any(|q| (offset + dot(q, normal)).abs() > p.round)
        {
            return gauss_plane(a, [u, v], top, inside, p);
        }
        Self { normal, offset }
    }

    // qh_distplane: the offset is the first summand, not the last.
    pub fn distance(self, [x, y, z]: Point) -> f64 {
        let [nx, ny, nz] = self.normal;
        self.offset + x * nx + y * ny + z * nz
    }

    pub fn orient_outside(&mut self, inside: Point) {
        if self.distance(inside) > 0.0 {
            self.normal = self.normal.map(|c| -c);
            self.offset = -self.offset;
        }
    }
}

fn simplex_det(points: &[Point], seed: &[usize], id: usize) -> f64 {
    let [a, b, c] = std::array::from_fn(|i| {
        seed.get(i)
            .map_or([0.0; 3], |&s| sub(points[s], points[id]))
    });
    if seed.len() == 2 {
        det2(a[0], a[1], b[0], b[1])
    } else {
        a[0] * det2(b[1], b[2], c[1], c[2]) - b[0] * det2(a[1], a[2], c[1], c[2])
            + c[0] * det2(a[1], a[2], b[1], b[2])
    }
}

fn det2(a: f64, b: f64, c: f64, d: f64) -> f64 {
    a * d - b * c
}

pub(super) fn dot(a: Point, b: Point) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

pub(super) fn sub(a: Point, b: Point) -> Point {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn normalize(mut normal: Point, top: bool, p: &Precision) -> Point {
    let length = dot(normal, normal).sqrt();
    let denom = if top { length } else { -length };
    if length > f64::MIN_POSITIVE * p.max_abs {
        return normal.map(|x| x / denom);
    }
    if length == 0.0 {
        return [(1.0_f64 / 3.0).sqrt(); 3];
    }
    for k in 0..3 {
        let Some(value) = div_nonzero(normal[k], denom, f64::MIN_POSITIVE) else {
            return dominant_axis(normal, denom);
        };
        normal[k] = value;
    }

    fn dominant_axis(normal: Point, denom: f64) -> Point {
        let mut max = 0;
        for i in 1..3 {
            if normal[i].abs() > normal[max].abs() {
                max = i;
            }
        }
        let mut axis = [0.0; 3];
        axis[max] = if normal[max] * denom >= 0.0 {
            1.0
        } else {
            -1.0
        };
        axis
    }
    normal
}

// qh_gausselim and qh_backnormal specialized to two rows and three columns.
fn gauss_plane(
    a: Point,
    mut rows: [Point; 2],
    mut sign: bool,
    inside: Point,
    p: &Precision,
) -> Plane {
    if rows[1][0].abs() > rows[0][0].abs() {
        rows.swap(0, 1);
        sign = !sign;
    }
    if rows[0][0] != 0.0 {
        let n = rows[1][0] / rows[0][0];
        rows[1][1] -= n * rows[0][1];
        rows[1][2] -= n * rows[0][2];
    }
    let mut near_zero = rows[0][0].abs() <= p.zero[0] || rows[1][1].abs() <= p.zero[1];
    sign ^= (rows[0][0] < 0.0) ^ (rows[1][1] < 0.0);
    let sign = if sign { -1.0 } else { 1.0 };
    let small = (f64::MIN_POSITIVE * 3.0).sqrt();
    let mut normal = [0.0, 0.0, sign];
    for i in (0..2).rev() {
        let mut numer = 0.0;
        for (coefficient, x) in rows[i][i + 1..].iter().zip(&normal[i + 1..]) {
            numer -= coefficient * x;
        }
        let diag = rows[i][i];
        if diag.abs() > small * p.max_abs {
            normal[i] = numer / diag;
        } else if let Some(value) = div_nonzero(numer, diag, small) {
            normal[i] = value;
        } else {
            near_zero = true;
            normal[i] = sign;
            normal[i + 1..].fill(0.0);
        }
    }
    normal = normalize(normal, true, p);
    let offset = -(a[0] * normal[0]) - a[1] * normal[1] - a[2] * normal[2];
    let mut plane = Plane { normal, offset };
    if near_zero {
        plane.orient_outside(inside);
    }
    plane
}

fn div_nonzero(numer: f64, denom: f64, minimum: f64) -> Option<f64> {
    let usable = if numer < minimum && numer > -minimum {
        numer.abs() < denom.abs()
    } else {
        (denom / numer).abs() > minimum
    };
    usable.then(|| numer / denom)
}
