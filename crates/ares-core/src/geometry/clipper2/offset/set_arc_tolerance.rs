// Copyright (c) Angus Johnson 2010-2025.
// BSL-1.0. Rust seed by MatterHackers.
// Derived from clipper2-rust 2f5e9d4249bc4d624773094ff6164335d9ff6ae7.
// See ../LICENSE-Boost-1.0 and ../SOURCE.md for pinned source authority.

use super::*;

impl ClipperOffset {
    /// Set the arc tolerance.
    /// Needed for rounded offsets. See offset_trigonometry2.svg.
    pub fn set_arc_tolerance(&mut self, arc_tolerance: f64) {
        self.arc_tolerance = arc_tolerance;
    }
}

impl ClipperOffset {
    /// Get the preserve_collinear flag.
    pub fn preserve_collinear(&self) -> bool {
        self.preserve_collinear
    }
}

impl ClipperOffset {
    /// Set the preserve_collinear flag.
    pub fn set_preserve_collinear(&mut self, preserve_collinear: bool) {
        self.preserve_collinear = preserve_collinear;
    }
}

impl ClipperOffset {
    /// Get the reverse_solution flag.
    pub fn reverse_solution(&self) -> bool {
        self.reverse_solution
    }
}

impl ClipperOffset {
    /// Set the reverse_solution flag.
    pub fn set_reverse_solution(&mut self, reverse_solution: bool) {
        self.reverse_solution = reverse_solution;
    }
}

impl ClipperOffset {
    /// Set the delta callback for variable offset.
    /// Direct port from ClipperOffset::SetDeltaCallback.
    pub fn set_delta_callback(&mut self, cb: Option<DeltaCallback64>) {
        self.delta_callback = cb;
    }
}

impl ClipperOffset {
    /// Set the fallback callback used to assign z at intersections created
    /// by the finishing union when z inheritance finds no agreeing vertex
    /// Direct port from clipper.offset.h line 118 (ClipperOffset::SetZCallback)
    pub fn set_z_callback(&mut self, cb: impl ZCallback64 + 'static) {
        self.z_callback = Some(Rc::new(RefCell::new(Box::new(cb))));
    }
}

impl ClipperOffset {
    /// Add a single path with the given join type and end type.
    /// Direct port from ClipperOffset::AddPath (clipper.offset.cpp line 168-171).
    pub fn add_path(&mut self, path: &Path64, jt: JoinType, et: EndType) {
        self.groups.push(Group::new(&vec![path.clone()], jt, et));
    }
}

impl ClipperOffset {
    /// Add multiple paths with the given join type and end type.
    /// Direct port from ClipperOffset::AddPaths (clipper.offset.cpp line 173-177).
    pub fn add_paths(&mut self, paths: &Paths64, jt: JoinType, et: EndType) {
        if paths.is_empty() {
            return;
        }
        self.groups.push(Group::new(paths, jt, et));
    }
}

impl ClipperOffset {
    /// Clear all groups and normals.
    /// Direct port from ClipperOffset::Clear (clipper.offset.h line 98).
    pub fn clear(&mut self) {
        self.groups.clear();
        self.norms.clear();
    }
}

impl ClipperOffset {
    /// Execute the offset operation, storing results in `paths`.
    /// Direct port from ClipperOffset::Execute(double, Paths64&)
    /// (clipper.offset.cpp line 636-642).
    pub fn execute(&mut self, delta: f64, paths: &mut Paths64) {
        paths.clear();
        self.solution.clear();
        self.execute_internal(delta, None);
        std::mem::swap(&mut self.solution, paths);
    }
}

impl ClipperOffset {
    /// Execute the offset operation, storing results in a `PolyTree64`.
    /// Direct port from ClipperOffset::Execute(double, PolyTree64&)
    /// (clipper.offset.cpp line 645-653).
    pub fn execute_tree(&mut self, delta: f64, polytree: &mut PolyTree64) {
        polytree.clear();
        self.solution.clear();
        self.execute_internal(delta, Some(polytree));
        self.solution.clear();
    }
}

impl ClipperOffset {
    /// Execute using a delta callback for variable offset.
    /// Direct port from ClipperOffset::Execute(DeltaCallback64, Paths64&)
    /// (clipper.offset.cpp line 655-659).
    pub fn execute_with_callback(&mut self, delta_cb: DeltaCallback64, paths: &mut Paths64) {
        self.delta_callback = Some(delta_cb);
        self.execute(1.0, paths);
    }
}

impl ClipperOffset {
    /// Calculate the expected solution capacity.
    /// Direct port from ClipperOffset::CalcSolutionCapacity (clipper.offset.cpp line 553-559).
    pub(crate) fn calc_solution_capacity(&self) -> usize {
        let mut result = 0;
        for g in &self.groups {
            result += if g.end_type == EndType::Joined {
                g.paths_in.len() * 2
            } else {
                g.paths_in.len()
            };
        }
        result
    }
}

impl ClipperOffset {
    /// Check if the groups have reversed orientation.
    /// Direct port from ClipperOffset::CheckReverseOrientation (clipper.offset.cpp line 561-572).
    pub(crate) fn check_reverse_orientation(&self) -> bool {
        // nb: this assumes there's consistency in orientation between groups
        for g in &self.groups {
            if g.end_type == EndType::Polygon {
                return g.is_reversed;
            }
        }
        false
    }
}

impl ClipperOffset {
    /// Core internal execution logic.
    /// Direct port from ClipperOffset::ExecuteInternal (clipper.offset.cpp line 574-634).
    pub(crate) fn execute_internal(&mut self, delta: f64, polytree: Option<&mut PolyTree64>) {
        self.error_code = 0;
        if self.groups.is_empty() {
            return;
        }
        self.solution.reserve(self.calc_solution_capacity());

        if delta.abs() < 0.5 {
            // offset is insignificant - just copy paths
            let mut sol_size = 0;
            for group in &self.groups {
                sol_size += group.paths_in.len();
            }
            self.solution.reserve(sol_size);
            for group in &self.groups {
                self.solution.extend(group.paths_in.iter().cloned());
            }
        } else {
            self.temp_lim = if self.miter_limit <= 1.0 {
                2.0
            } else {
                2.0 / (self.miter_limit * self.miter_limit)
            };

            self.delta = delta;
            // Process each group - we need indices because do_group_offset
            // borrows self mutably
            for i in 0..self.groups.len() {
                self.do_group_offset(i);
                if self.error_code != 0 {
                    self.solution.clear();
                }
            }
        }

        if self.solution.is_empty() {
            return;
        }

        let paths_reversed = self.check_reverse_orientation();
        // Clean up self-intersections using Clipper64 union
        let mut c = Clipper64::new();
        c.set_preserve_collinear(self.preserve_collinear);
        // The solution should retain the orientation of the input
        c.set_reverse_solution(self.reverse_solution != paths_reversed);
        // C++ always installs the ZCB proxy under USINGZ (even with no user
        // callback) so offset z values survive the finishing union
        // Direct port from clipper.offset.cpp line 543 (ClipperOffset::ZCB)

        {
            let user_cb = self.z_callback.clone();
            c.set_z_callback(move |bot1, top1, bot2, top2, ip| {
                if bot1.z != 0 && (bot1.z == bot2.z || bot1.z == top2.z) {
                    ip.z = bot1.z;
                } else if bot2.z != 0 && bot2.z == top1.z {
                    ip.z = bot2.z;
                } else if top1.z != 0 && top1.z == top2.z {
                    ip.z = top1.z;
                } else if let Some(ref cb) = user_cb {
                    (*cb.borrow_mut())(bot1, top1, bot2, top2, ip);
                }
            });
        }
        c.add_subject(&self.solution);

        let fill_rule = if paths_reversed {
            FillRule::Negative
        } else {
            FillRule::Positive
        };

        if let Some(tree) = polytree {
            let mut open_paths = Paths64::new();
            c.execute_tree(ClipType::Union, fill_rule, tree, &mut open_paths);
        } else {
            c.execute(ClipType::Union, fill_rule, &mut self.solution, None);
        }
    }
}

impl ClipperOffset {
    /// Build unit normals for each edge of a path.
    /// Direct port from ClipperOffset::BuildNormals (clipper.offset.cpp line 179-188).
    pub(crate) fn build_normals(&mut self, path: &Path64) {
        self.norms.clear();
        if path.is_empty() {
            return;
        }
        self.norms.reserve(path.len());
        for i in 0..path.len() - 1 {
            self.norms.push(get_unit_normal(&path[i], &path[i + 1]));
        }
        self.norms
            .push(get_unit_normal(path.last().unwrap(), &path[0]));
    }
}

impl ClipperOffset {
    /// Bevel join implementation.
    /// Direct port from ClipperOffset::DoBevel (clipper.offset.cpp line 190-216).
    pub(crate) fn do_bevel(&mut self, path: &Path64, j: usize, k: usize) {
        let pt1: PointD;
        let pt2: PointD;
        if j == k {
            let abs_delta = self.group_delta.abs();
            pt1 = PointD {
                x: path[j].x as f64 - abs_delta * self.norms[j].x,
                y: path[j].y as f64 - abs_delta * self.norms[j].y,

                z: path[j].z,
            };
            pt2 = PointD {
                x: path[j].x as f64 + abs_delta * self.norms[j].x,
                y: path[j].y as f64 + abs_delta * self.norms[j].y,

                z: path[j].z,
            };
        } else {
            pt1 = PointD {
                x: path[j].x as f64 + self.group_delta * self.norms[k].x,
                y: path[j].y as f64 + self.group_delta * self.norms[k].y,

                z: path[j].z,
            };
            pt2 = PointD {
                x: path[j].x as f64 + self.group_delta * self.norms[j].x,
                y: path[j].y as f64 + self.group_delta * self.norms[j].y,

                z: path[j].z,
            };
        }
        self.path_out.push(Point64::from(&pt1));
        self.path_out.push(Point64::from(&pt2));
    }
}
