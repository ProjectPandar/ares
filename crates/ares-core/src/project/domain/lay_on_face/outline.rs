use super::{Vec3, fixed_gcc_sort_by};

// `Geometry/ConvexHull.cpp:50–105`: monotone chain with fixed-point XY tests,
// keeping the original (rescaled) 3D points for the plane-distance test.
pub(super) fn outline_hull(mut points: Vec<Vec3>) -> Vec<Vec3> {
    let size: [f64; 2] = std::array::from_fn(|axis| {
        let (lo, hi) = points
            .iter()
            .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), p| {
                (lo.min(p[axis]), hi.max(p[axis]))
            });
        hi - lo
    });
    let scale = f64::from((1.0 / size[0]).min(1.0 / size[1]) as f32);
    for point in &mut points {
        point[0] *= scale;
        point[1] *= scale;
    }
    fixed_gcc_sort_by(&mut points, |a, b| {
        a[0] < b[0] || (a[0] == b[0] && a[1] < b[1])
    });
    let fixed: Vec<_> = points
        .iter()
        .map(|p| [(p[0] * 1e6) as i64, (p[1] * 1e6) as i64])
        .collect();
    let ccw = |a: usize, b: usize, c: usize| {
        let [[ax, ay], [bx, by], [cx, cy]] = [fixed[a], fixed[b], fixed[c]];
        (i128::from(bx) - i128::from(ax)) * (i128::from(cy) - i128::from(ay))
            > (i128::from(by) - i128::from(ay)) * (i128::from(cx) - i128::from(ax))
    };
    let mut outline = Vec::with_capacity(points.len());
    let count = points.len();
    for reverse in [false, true] {
        let floor = if reverse { outline.len() + 1 } else { 2 };
        for i in 0..count - usize::from(reverse) {
            let index = if reverse { count - 2 - i } else { i };
            while outline.len() >= floor
                && !ccw(
                    outline[outline.len() - 2],
                    outline[outline.len() - 1],
                    index,
                )
            {
                outline.pop();
            }
            outline.push(index);
        }
    }
    outline.pop();
    // Eigen forms the entire inverse diagonal via cofactors and a
    // reciprocal determinant; even the nominally unscaled z rounds.
    let determinant = scale * scale;
    let inverse_scale = scale * (1.0 / determinant);
    let inverse_z = determinant * (1.0 / determinant);
    outline
        .into_iter()
        .map(|i| {
            [
                points[i][0] * inverse_scale,
                points[i][1] * inverse_scale,
                points[i][2] * inverse_z,
            ]
        })
        .collect()
}
