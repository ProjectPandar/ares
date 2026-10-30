// Copyright (c) Angus Johnson 2010-2025.
// BSL-1.0. Rust seed by MatterHackers.
// Derived from clipper2-rust 2f5e9d4249bc4d624773094ff6164335d9ff6ae7.
// See ../LICENSE-Boost-1.0 and ../SOURCE.md for pinned source authority.

use super::*;

impl ClipperD {
    /// Add subject paths (double precision)
    pub fn add_subject(&mut self, subjects: &PathsD) {
        let scaled: Paths64 =
            scale_paths(subjects, self.scale, self.scale, &mut self.base.error_code);
        self.base.add_paths(&scaled, PathType::Subject, false);
    }
}

impl ClipperD {
    /// Add open subject paths (double precision)
    pub fn add_open_subject(&mut self, open_subjects: &PathsD) {
        let scaled: Paths64 = scale_paths(
            open_subjects,
            self.scale,
            self.scale,
            &mut self.base.error_code,
        );
        self.base.add_paths(&scaled, PathType::Subject, true);
    }
}

impl ClipperD {
    /// Add clip paths (double precision)
    pub fn add_clip(&mut self, clips: &PathsD) {
        let scaled: Paths64 = scale_paths(clips, self.scale, self.scale, &mut self.base.error_code);
        self.base.add_paths(&scaled, PathType::Clip, false);
    }
}

impl ClipperD {
    pub fn error_code(&self) -> i32 {
        self.base.error_code
    }
}

impl ClipperD {
    pub fn set_preserve_collinear(&mut self, val: bool) {
        self.base.preserve_collinear = val;
    }
}

impl ClipperD {
    pub fn preserve_collinear(&self) -> bool {
        self.base.preserve_collinear
    }
}

impl ClipperD {
    pub fn set_reverse_solution(&mut self, val: bool) {
        self.base.reverse_solution = val;
    }
}

impl ClipperD {
    pub fn reverse_solution(&self) -> bool {
        self.base.reverse_solution
    }
}

impl ClipperD {
    pub fn clear(&mut self) {
        self.base.clear();
    }
}

impl ClipperD {
    /// Execute a clipping operation with double-precision paths
    /// Direct port from clipper.engine.h ClipperD::Execute
    pub fn execute(
        &mut self,
        clip_type: ClipType,
        fill_rule: FillRule,
        solution_closed: &mut PathsD,
        mut solution_open: Option<&mut PathsD>,
    ) -> bool {
        solution_closed.clear();
        if let Some(ref mut open) = solution_open {
            open.clear();
        }

        if self.base.execute_internal(clip_type, fill_rule, false) {
            self.build_paths_d(solution_closed, solution_open);
        }
        self.base.clean_up();
        self.base.succeeded
    }
}

impl ClipperD {
    /// Execute returning polytree with double-precision
    pub fn execute_tree(
        &mut self,
        clip_type: ClipType,
        fill_rule: FillRule,
        polytree: &mut PolyTreeD,
        open_paths: &mut PathsD,
    ) -> bool {
        polytree.clear();
        open_paths.clear();

        if self.base.execute_internal(clip_type, fill_rule, true) {
            self.build_tree_d(polytree, open_paths);
        }
        self.base.clean_up();
        self.base.succeeded
    }
}

impl ClipperD {
    /// Build output paths for double-precision
    /// Direct port from clipper.engine.cpp ClipperD::BuildPathsD (line 3101)
    pub(crate) fn build_paths_d(
        &mut self,
        solution_closed: &mut PathsD,
        solution_open: Option<&mut PathsD>,
    ) {
        solution_closed.clear();
        solution_closed.reserve(self.base.outrec_list.len());

        let mut open_paths = Vec::new();

        let mut i = 0;
        while i < self.base.outrec_list.len() {
            if self.base.outrec_list[i].pts.is_none() {
                i += 1;
                continue;
            }

            if self.base.outrec_list[i].is_open {
                let op = self.base.outrec_list[i].pts.unwrap();
                if let Some(path) = build_path_d_from_outpt(
                    op,
                    self.base.reverse_solution,
                    true,
                    &self.base.outpt_arena,
                    self.inv_scale,
                ) {
                    open_paths.push(path);
                }
            } else {
                self.base.clean_collinear(i);
                if self.base.outrec_list[i].pts.is_some() {
                    let op = self.base.outrec_list[i].pts.unwrap();
                    if let Some(path) = build_path_d_from_outpt(
                        op,
                        self.base.reverse_solution,
                        false,
                        &self.base.outpt_arena,
                        self.inv_scale,
                    ) {
                        solution_closed.push(path);
                    }
                }
            }
            i += 1;
        }

        if let Some(open) = solution_open {
            *open = open_paths;
        }
    }
}

impl ClipperD {
    /// Build polytree for double-precision
    /// Direct port from clipper.engine.cpp ClipperD::BuildTreeD (line 3135)
    pub(crate) fn build_tree_d(&mut self, polytree: &mut PolyTreeD, open_paths: &mut PathsD) {
        polytree.clear();
        polytree.set_scale(self.inv_scale);
        open_paths.clear();
        if self.base.has_open_paths {
            open_paths.reserve(self.base.outrec_list.len());
        }

        // Build a PolyTree64 internally, then convert to PolyTreeD
        // This matches the C++ approach where RecursiveCheckOwners works with Path64
        let mut polytree64 = PolyTree64::new();

        let mut i = 0;
        while i < self.base.outrec_list.len() {
            if self.base.outrec_list[i].pts.is_none() {
                i += 1;
                continue;
            }

            if self.base.outrec_list[i].is_open {
                let op = self.base.outrec_list[i].pts.unwrap();
                if let Some(path) = build_path_d_from_outpt(
                    op,
                    self.base.reverse_solution,
                    true,
                    &self.base.outpt_arena,
                    self.inv_scale,
                ) {
                    open_paths.push(path);
                }
                i += 1;
                continue;
            }

            if self.base.check_bounds(i) {
                self.base.recursive_check_owners(i, &mut polytree64);
            }
            i += 1;
        }

        // Convert PolyTree64 to PolyTreeD by scaling all paths
        self.convert_polytree64_to_d(&polytree64, polytree);
    }
}

impl ClipperD {
    /// Convert a PolyTree64 to PolyTreeD by scaling all paths
    pub(crate) fn convert_polytree64_to_d(&self, src: &PolyTree64, dst: &mut PolyTreeD) {
        dst.clear();
        dst.set_scale(self.inv_scale);
        // Recursively convert nodes, skipping the root (index 0)
        let root_children = src.nodes[0].children.clone();
        for &child_idx in &root_children {
            Self::convert_polypath64_node(src, child_idx, 0, dst);
        }
    }
}

impl ClipperD {
    pub(crate) fn convert_polypath64_node(
        src: &PolyTree64,
        src_idx: usize,
        dst_parent: usize,
        dst: &mut PolyTreeD,
    ) {
        let src_node = &src.nodes[src_idx];
        let dst_idx = dst.add_child_from_path64(dst_parent, &src_node.polygon);
        let children = src_node.children.clone();
        for &child_idx in &children {
            Self::convert_polypath64_node(src, child_idx, dst_idx, dst);
        }
    }
}
