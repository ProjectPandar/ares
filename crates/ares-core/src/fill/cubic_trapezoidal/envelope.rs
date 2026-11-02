// FillRectilinear.cpp:3057-3098, cubic_upper_level.
pub(super) fn upper_level(tau: f64, h: f64, period: f64, d1: f64) -> Vec<[f64; 2]> {
    let s3 = 3.0_f64.sqrt();
    let y_cut = (tau - 0.5 * d1).clamp(0.0, h - d1) + d1;
    let y_flat = h.min(h + y_cut - 2.0 * d1);
    let x2 = (h - tau) / s3 + d1;
    let x3 = (h + tau) / s3 - d1;
    let lines = [
        [s3, tau],
        [1.0 / s3, h - x2 / s3],
        [0.0, y_flat],
        [-1.0 / s3, h + x3 / s3],
        [-s3, tau + s3 * period],
    ];
    let y_at = |x: f64| {
        let mut y = f64::MAX;
        for line in lines {
            y = y.min(line[0] * x + line[1]);
        }
        y.max(y_cut)
    };
    let mut xs = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if line[0] != 0.0 {
            xs.push((y_cut - line[1]) / line[0]);
        }
        for next in &lines[i + 1..] {
            xs.push((next[1] - line[1]) / (line[0] - next[0]));
        }
    }
    xs.retain(|x| *x > 1.0 && *x < period - 1.0);
    xs.extend([0.0, period]);
    xs.sort_by(f64::total_cmp);
    xs.dedup_by(|b, a| *b - *a < 1.0);
    let mut points: Vec<[f64; 2]> = Vec::new();
    for x in xs {
        let point = [x, y_at(x)];
        if points.len() >= 2 {
            let a = points[points.len() - 2];
            let b = points[points.len() - 1];
            if ((b[1] - a[1]) / (b[0] - a[0]) - (point[1] - b[1]) / (point[0] - b[0])).abs()
                < 1.0e-4
            {
                points.pop();
            }
        }
        points.push(point);
    }
    points
}
