// Copyright (c) Angus Johnson 2010-2025.
// BSL-1.0. Rust seed by MatterHackers.
// Derived from clipper2-rust 2f5e9d4249bc4d624773094ff6164335d9ff6ae7.
// See ../LICENSE-Boost-1.0 and ../SOURCE.md for pinned source authority.

use super::*;

impl ClipperOffset {
    /// Process a single group of paths.
    /// Direct port from ClipperOffset::DoGroupOffset (clipper.offset.cpp line 455-539).
    pub(crate) fn do_group_offset(&mut self, group_idx: usize) {
        let group_end_type = self.groups[group_idx].end_type;
        let group_join_type = self.groups[group_idx].join_type;
        let group_is_reversed = self.groups[group_idx].is_reversed;
        let group_lowest = self.groups[group_idx].lowest_path_idx;

        if group_end_type == EndType::Polygon {
            // A straight path (2 points) can now also be 'polygon' offset
            // where the ends will be treated as (180 deg.) joins
            if group_lowest.is_none() {
                self.delta = self.delta.abs();
            }
            self.group_delta = if group_is_reversed {
                -self.delta
            } else {
                self.delta
            };
        } else {
            self.group_delta = self.delta.abs();
        }

        let abs_delta = self.group_delta.abs();
        self.join_type = group_join_type;
        self.end_type = group_end_type;

        if group_join_type == JoinType::Round || group_end_type == EndType::Round {
            // Calculate the number of steps required to approximate a circle
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

        // clipper.offset.cpp:514–567: borrow owned group paths without copying them.
        let paths = std::mem::take(&mut self.groups[group_idx].paths_in);
        for path in &paths {
            let path_len = path.len();
            self.path_out.clear();

            if path_len == 1 {
                // Single point
                if let Some(callback) = &self.delta_callback {
                    self.group_delta = callback(path, &self.norms, 0, 0);
                    if group_is_reversed {
                        self.group_delta = -self.group_delta;
                    }
                }

                if self.group_delta < 1.0 {
                    continue;
                }
                let pt = path[0];
                let abs_delta_local = self.group_delta.abs();

                // Single vertex: build a circle or square
                if group_join_type == JoinType::Round {
                    let radius = abs_delta_local;
                    let steps = if self.steps_per_rad > 0.0 {
                        (self.steps_per_rad * 2.0 * constants::PI).ceil() as usize
                    } else {
                        0
                    };
                    self.path_out = ellipse_point64(pt, radius, radius, steps);

                    for p in self.path_out.iter_mut() {
                        p.z = pt.z;
                    }
                } else {
                    let d = i64::from(abs_delta_local.ceil() as i32);
                    let r = Rect64::new(pt.x - d, pt.y - d, pt.x + d, pt.y + d);
                    self.path_out = r.as_path();

                    for p in self.path_out.iter_mut() {
                        p.z = pt.z;
                    }
                }

                let path_out = std::mem::take(&mut self.path_out);
                self.solution.push(path_out);
                continue;
            } // end of offsetting a single point

            if path_len == 2 && group_end_type == EndType::Joined {
                self.end_type = if group_join_type == JoinType::Round {
                    EndType::Round
                } else {
                    EndType::Square
                };
            }

            self.build_normals(path);
            if self.end_type == EndType::Polygon {
                self.offset_polygon(group_idx, path);
            } else if self.end_type == EndType::Joined {
                self.offset_open_joined(group_idx, path);
            } else {
                self.offset_open_path(group_idx, path);
            }
        }
        self.groups[group_idx].paths_in = paths;
    }
}
