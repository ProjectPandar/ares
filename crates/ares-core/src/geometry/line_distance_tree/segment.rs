//! `Line.hpp::line_alg::distance_to_squared` for native `Linef` endpoints.

pub(super) fn distance_squared(a: [f64; 2], b: [f64; 2], origin: [f64; 2]) -> (f64, [f64; 2]) {
    let v = [b[0] - a[0], b[1] - a[1]];
    let va = [origin[0] - a[0], origin[1] - a[1]];
    let length_squared = v[0] * v[0] + v[1] * v[1];
    if length_squared == 0.0 {
        return (va[0] * va[0] + va[1] * va[1], a);
    }
    let t = (va[0] * v[0] + va[1] * v[1]) / length_squared;
    if t <= 0.0 {
        (va[0] * va[0] + va[1] * va[1], a)
    } else if t >= 1.0 {
        let vb = [origin[0] - b[0], origin[1] - b[1]];
        (vb[0] * vb[0] + vb[1] * vb[1], b)
    } else {
        let nearest = [a[0] + t * v[0], a[1] + t * v[1]];
        let delta = [t * v[0] - va[0], t * v[1] - va[1]];
        (delta[0] * delta[0] + delta[1] * delta[1], nearest)
    }
}
