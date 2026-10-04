//! `GCode/PchipInterpolatorHelper.cpp`'s sorted Hermite interpolation.

pub(super) struct Pchip {
    points: Vec<(f64, f64)>,
    intervals: Vec<f64>,
    derivatives: Vec<f64>,
}

impl Pchip {
    pub(super) fn new(mut points: Vec<(f64, f64)>) -> Self {
        points.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.total_cmp(&b.1)));
        let intervals: Vec<_> = points.windows(2).map(|p| p[1].0 - p[0].0).collect();
        let slopes: Vec<_> = points
            .windows(2)
            .zip(&intervals)
            .map(|(p, h)| (p[1].1 - p[0].1) / h)
            .collect();
        let last = points.len() - 1;
        let mut derivatives = vec![0.0; points.len()];
        derivatives[0] = slopes[0];
        derivatives[last] = slopes[last - 1];
        for i in 1..last {
            if slopes[i - 1] * slopes[i] > 0.0 {
                let w1 = 2.0 * intervals[i] + intervals[i - 1];
                let w2 = intervals[i] + 2.0 * intervals[i - 1];
                derivatives[i] = (w1 + w2) / (w1 / slopes[i - 1] + w2 / slopes[i]);
            }
        }
        Self {
            points,
            intervals,
            derivatives,
        }
    }

    pub(super) fn interpolate(&self, x: f64) -> f64 {
        let last = self.points.len() - 1;
        if x <= self.points[0].0 {
            return self.points[0].1;
        }
        if x >= self.points[last].0 {
            return self.points[last].1;
        }
        let i = self.points.partition_point(|p| p.0 < x) - 1;
        let h = self.intervals[i];
        let t = (x - self.points[i].0) / h;
        let t2 = t * t;
        let t3 = t2 * t;
        let h00 = 2.0 * t3 - 3.0 * t2 + 1.0;
        let h10 = t3 - 2.0 * t2 + t;
        let h01 = -2.0 * t3 + 3.0 * t2;
        let h11 = t3 - t2;
        h00 * self.points[i].1
            + h10 * h * self.derivatives[i]
            + h01 * self.points[i + 1].1
            + h11 * h * self.derivatives[i + 1]
    }
}
