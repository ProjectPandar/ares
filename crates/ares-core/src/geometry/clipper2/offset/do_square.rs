// Copyright (c) Angus Johnson 2010-2025.
// BSL-1.0. Rust seed by MatterHackers.
// Derived from clipper2-rust 2f5e9d4249bc4d624773094ff6164335d9ff6ae7.
// See ../LICENSE-Boost-1.0 and ../SOURCE.md for pinned source authority.

use super::*;

impl ClipperOffset {
    /// Square join implementation.
    /// Direct port from ClipperOffset::DoSquare (clipper.offset.cpp line 218-256).
    pub(crate) fn do_square(&mut self, path: &Path64, j: usize, k: usize) {
        let vec: PointD = if j == k {
            PointD::new(self.norms[j].y, -self.norms[j].x)
        } else {
            get_avg_unit_vector(
                &PointD::new(-self.norms[k].y, self.norms[k].x),
                &PointD::new(self.norms[j].y, -self.norms[j].x),
            )
        };

        let abs_delta = self.group_delta.abs();

        // Now offset the original vertex delta units along unit vector.
        // pt_q carries path[j].z like C++'s PointD(path[j]) conversion; the
        // z survives translation but is zeroed by a computed intersection.
        let pt_q = PointD {
            x: path[j].x as f64,
            y: path[j].y as f64,

            z: path[j].z,
        };
        let pt_q = translate_point(&pt_q, abs_delta * vec.x, abs_delta * vec.y);
        // Get perpendicular vertices
        let pt1 = translate_point(&pt_q, self.group_delta * vec.y, self.group_delta * -vec.x);
        let pt2 = translate_point(&pt_q, self.group_delta * -vec.y, self.group_delta * vec.x);
        // Get 2 vertices along one edge offset
        let pt3 = get_perpendic_d(&path[k], &self.norms[k], self.group_delta);

        if j == k {
            let pt4 = PointD::new(
                pt3.x + vec.x * self.group_delta,
                pt3.y + vec.y * self.group_delta,
            );
            let mut pt = pt_q;
            get_line_intersect_pt_d(pt1, pt2, pt3, pt4, &mut pt);
            // Get the second intersect point through reflection
            let reflected = reflect_point(&pt, &pt_q);
            self.path_out.push(Point64::from(&reflected));
            self.path_out.push(Point64::from(&pt));
        } else {
            let pt4 = get_perpendic_d(&path[j], &self.norms[k], self.group_delta);
            let mut pt = pt_q;
            get_line_intersect_pt_d(pt1, pt2, pt3, pt4, &mut pt);
            self.path_out.push(Point64::from(&pt));
            // Get the second intersect point through reflection
            let reflected = reflect_point(&pt, &pt_q);
            self.path_out.push(Point64::from(&reflected));
        }
    }
}

impl ClipperOffset {
    /// Miter join implementation.
    /// Direct port from ClipperOffset::DoMiter (clipper.offset.cpp line 258-271).
    pub(crate) fn do_miter(&mut self, path: &Path64, j: usize, k: usize, cos_a: f64) {
        let q = self.group_delta / (cos_a + 1.0);
        self.path_out.push(Point64 {
            x: (path[j].x as f64 + (self.norms[k].x + self.norms[j].x) * q).round() as i64,
            y: (path[j].y as f64 + (self.norms[k].y + self.norms[j].y) * q).round() as i64,

            z: path[j].z,
        });
    }
}

impl ClipperOffset {
    /// Round join implementation.
    /// Direct port from ClipperOffset::DoRound (clipper.offset.cpp line 273-309).
    pub(crate) fn do_round(&mut self, path: &Path64, j: usize, k: usize, angle: f64) {
        if self.delta_callback.is_some() {
            // When delta_callback is assigned, group_delta won't be constant,
            // so we need to do these calculations for *every* vertex.
            let abs_delta = self.group_delta.abs();
            let arc_tol = if self.arc_tolerance > FLOATING_POINT_TOLERANCE {
                abs_delta.min(self.arc_tolerance)
            } else {
                abs_delta * ARC_CONST
            };
            let steps_per_360 =
                (constants::PI / (1.0 - arc_tol / abs_delta).acos()).min(abs_delta * constants::PI);
            self.step_sin = (2.0 * constants::PI / steps_per_360).sin();
            self.step_cos = (2.0 * constants::PI / steps_per_360).cos();
            if self.group_delta < 0.0 {
                self.step_sin = -self.step_sin;
            }
            self.steps_per_rad = steps_per_360 / (2.0 * constants::PI);
        }

        let pt = path[j];
        let mut offset_vec = PointD::new(
            self.norms[k].x * self.group_delta,
            self.norms[k].y * self.group_delta,
        );

        if j == k {
            offset_vec = offset_vec.negate();
        }
        self.path_out.push(Point64 {
            x: (pt.x as f64 + offset_vec.x).round() as i64,
            y: (pt.y as f64 + offset_vec.y).round() as i64,

            z: pt.z,
        });

        // clipper.offset.cpp:301–303 (Orca): rounded, not ceiling, arc steps.
        let steps = ((self.steps_per_rad * angle.abs()).round() as i32).max(1);
        for _ in 1..steps {
            // ie 1 less than steps
            offset_vec = PointD::new(
                offset_vec.x * self.step_cos - self.step_sin * offset_vec.y,
                offset_vec.x * self.step_sin + offset_vec.y * self.step_cos,
            );
            self.path_out.push(Point64 {
                x: (pt.x as f64 + offset_vec.x).round() as i64,
                y: (pt.y as f64 + offset_vec.y).round() as i64,

                z: pt.z,
            });
        }
        self.path_out
            .push(get_perpendic(&path[j], &self.norms[j], self.group_delta));
    }
}

impl ClipperOffset {
    /// Offset a single vertex.
    /// Direct port from ClipperOffset::OffsetPoint (clipper.offset.cpp line 311-370).
    pub(crate) fn offset_point(&mut self, group_idx: usize, path: &Path64, j: usize, k: usize) {
        // Let A = change in angle where edges join
        // A == 0: ie no change in angle (flat join)
        // A == PI: edges 'spike'
        // sin(A) < 0: right turning
        // cos(A) < 0: change in angle is more than 90 degree

        if path[j] == path[k] {
            return;
        }

        let sin_a = cross_product_two_vectors(self.norms[j], self.norms[k]);
        let cos_a = dot_product_two_vectors(self.norms[j], self.norms[k]);
        let sin_a = sin_a.clamp(-1.0, 1.0);

        if let Some(ref cb) = self.delta_callback {
            self.group_delta = cb(path, &self.norms, j, k);
            if self.groups[group_idx].is_reversed {
                self.group_delta = -self.group_delta;
            }
        }
        if self.group_delta.abs() <= FLOATING_POINT_TOLERANCE {
            self.path_out.push(path[j]);
            return;
        }

        if cos_a > -0.999 && (sin_a * self.group_delta < 0.0) {
            // test for concavity first (#593)
            // Is concave.
            // By far the simplest way to construct concave joins, especially those
            // joining very short segments, is to insert 3 points that produce negative
            // regions. These regions will be removed later by the finishing union
            // operation. This is also the best way to ensure that path reversals
            // (ie over-shrunk paths) are removed.
            self.path_out
                .push(get_perpendic(&path[j], &self.norms[k], self.group_delta));
            self.path_out.push(path[j]); // (#405, #873, #916)
            self.path_out
                .push(get_perpendic(&path[j], &self.norms[j], self.group_delta));
        } else if cos_a > 0.999 && self.join_type != JoinType::Round {
            // Almost straight - less than 2.5 degree (#424, #482, #526 & #724)
            self.do_miter(path, j, k, cos_a);
        } else if self.join_type == JoinType::Miter {
            // Miter unless the angle is sufficiently acute to exceed ML
            if cos_a > self.temp_lim - 1.0 {
                self.do_miter(path, j, k, cos_a);
            } else {
                self.do_square(path, j, k);
            }
        } else if self.join_type == JoinType::Round {
            self.do_round(path, j, k, sin_a.atan2(cos_a));
        } else if self.join_type == JoinType::Bevel {
            self.do_bevel(path, j, k);
        } else {
            self.do_square(path, j, k);
        }
    }
}

impl ClipperOffset {
    /// `clipper.offset.cpp:378–395`: bypass three-point concave loops when safe.
    fn offset_concave_crossing(&mut self, path: &Path64, j: usize, k: usize, next: usize) -> bool {
        let sin_a = cross_product_two_vectors(self.norms[j], self.norms[k]);
        let cos_a = dot_product_two_vectors(self.norms[j], self.norms[k]);
        if cos_a <= -0.999 || sin_a * self.group_delta >= 0.0 {
            return false;
        }
        let x = (self.group_delta * sin_a).abs() / (1.0 + cos_a);
        if 4.0 * x * x > distance_sqr(path[k], path[j])
            || 4.0 * x * x > distance_sqr(path[j], path[next])
        {
            return false;
        }
        let q = self.group_delta / (1.0 + cos_a);
        self.path_out.push(Point64::new_z(
            (path[j].x as f64 + (self.norms[k].x + self.norms[j].x) * q).round() as i64,
            (path[j].y as f64 + (self.norms[k].y + self.norms[j].y) * q).round() as i64,
            path[j].z,
        ));
        true
    }

    /// `clipper.offset.cpp:397–404`: preserve callback, duplicate and concavity guards.
    pub(crate) fn offset_polygon(&mut self, group_idx: usize, path: &Path64) {
        self.path_out.clear();
        let len = path.len();
        let mut k = len - 1;
        for j in 0..len {
            if self.delta_callback.is_some()
                || path[j] == path[k]
                || !self.offset_concave_crossing(path, j, k, if j + 1 == len { 0 } else { j + 1 })
            {
                self.offset_point(group_idx, path, j, k);
            }
            k = j;
        }
        let path_out = std::mem::take(&mut self.path_out);
        self.solution.push(path_out);
    }
}

impl ClipperOffset {
    /// Offset an open path with joined ends.
    /// Direct port from ClipperOffset::OffsetOpenJoined (clipper.offset.cpp line 380-393).
    pub(crate) fn offset_open_joined(&mut self, group_idx: usize, path: &Path64) {
        self.offset_polygon(group_idx, path);
        let mut reverse_path = path.clone();
        reverse_path.reverse();

        // Rebuild normals
        self.norms.reverse();
        self.norms.push(self.norms[0]);
        self.norms.remove(0);
        negate_path(&mut self.norms);

        self.offset_polygon(group_idx, &reverse_path);
    }
}

impl ClipperOffset {
    /// Offset an open path.
    /// Direct port from ClipperOffset::OffsetOpenPath (clipper.offset.cpp line 395-453).
    pub(crate) fn offset_open_path(&mut self, group_idx: usize, path: &Path64) {
        self.path_out.clear();

        // Do the line start cap
        if let Some(ref cb) = self.delta_callback {
            self.group_delta = cb(path, &self.norms, 0, 0);
        }

        if self.group_delta.abs() <= FLOATING_POINT_TOLERANCE {
            self.path_out.push(path[0]);
        } else {
            match self.end_type {
                EndType::Butt => self.do_bevel(path, 0, 0),
                EndType::Round => self.do_round(path, 0, 0, constants::PI),
                _ => self.do_square(path, 0, 0),
            }
        }

        let high_i = path.len() - 1;
        // Offset the left side going forward
        let mut k = 0;
        for j in 1..high_i {
            self.offset_point(group_idx, path, j, k);
            k = j;
        }

        // Reverse normals
        for i in (1..=high_i).rev() {
            self.norms[i] = PointD::new(-self.norms[i - 1].x, -self.norms[i - 1].y);
        }
        self.norms[0] = self.norms[high_i];

        // Do the line end cap
        if let Some(ref cb) = self.delta_callback {
            self.group_delta = cb(path, &self.norms, high_i, high_i);
        }

        if self.group_delta.abs() <= FLOATING_POINT_TOLERANCE {
            self.path_out.push(path[high_i]);
        } else {
            match self.end_type {
                EndType::Butt => self.do_bevel(path, high_i, high_i),
                EndType::Round => self.do_round(path, high_i, high_i, constants::PI),
                _ => self.do_square(path, high_i, high_i),
            }
        }

        // Offset the right side going backward
        let mut k = high_i;
        for j in (1..high_i).rev() {
            self.offset_point(group_idx, path, j, k);
            k = j;
        }
        let path_out = std::mem::take(&mut self.path_out);
        self.solution.push(path_out);
    }
}
