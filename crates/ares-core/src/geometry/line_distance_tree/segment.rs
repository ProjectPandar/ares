//! `Line.hpp:40-78::line_alg::distance_to_squared`, native endpoint precision.

use super::floating::NativeFloat;

pub(crate) fn distance_squared<F: NativeFloat>(
    a: [F; 2],
    b: [F; 2],
    origin: [F; 2],
) -> (F, [F; 2]) {
    // Upstream subtracts native endpoints, then casts the vectors to double.
    let v: [f64; 2] = [b[0] - a[0], b[1] - a[1]].map(Into::into);
    let va: [f64; 2] = [origin[0] - a[0], origin[1] - a[1]].map(Into::into);
    let length_squared = v[0] * v[0] + v[1] * v[1];
    let (distance, nearest) = if length_squared == 0.0 {
        (va[0] * va[0] + va[1] * va[1], a)
    } else {
        let t = (va[0] * v[0] + va[1] * v[1]) / length_squared;
        if t <= 0.0 {
            (va[0] * va[0] + va[1] * va[1], a)
        } else if t >= 1.0 {
            let vb: [f64; 2] = [origin[0] - b[0], origin[1] - b[1]].map(Into::into);
            (vb[0] * vb[0] + vb[1] * vb[1], b)
        } else {
            let nearest = [
                F::from_f64(a[0].into() + t * v[0]),
                F::from_f64(a[1].into() + t * v[1]),
            ];
            let delta = [t * v[0] - va[0], t * v[1] - va[1]];
            (delta[0] * delta[0] + delta[1] * delta[1], nearest)
        }
    };
    (F::from_f64(distance), nearest)
}
