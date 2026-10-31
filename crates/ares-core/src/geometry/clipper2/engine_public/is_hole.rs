// Copyright (c) Angus Johnson 2010-2025.
// BSL-1.0. Rust seed by MatterHackers.
// Derived from clipper2-rust 2f5e9d4249bc4d624773094ff6164335d9ff6ae7.
// See ../LICENSE-Boost-1.0 and ../SOURCE.md for pinned source authority.

use super::*;

/// Main integer-coordinate polygon clipper
/// Direct port from clipper.engine.h line 459
pub struct Clipper64 {
    pub base: ClipperBase,
}

impl Clipper64 {
    pub fn new() -> Self {
        Self {
            base: ClipperBase::new(),
        }
    }
}

impl Clipper64 {
    /// Set the callback invoked at each intersection to assign the z value
    /// Direct port from clipper.engine.h line 466 (Clipper64::SetZCallback)
    pub fn set_z_callback(&mut self, cb: impl ZCallback64 + 'static) {
        self.base.z_callback = Some(Box::new(cb));
    }
}

impl Clipper64 {
    /// Add subject paths
    pub fn add_subject(&mut self, subjects: &Paths64) {
        self.base.add_paths(subjects, PathType::Subject, false);
    }
}

impl Clipper64 {
    /// Add open subject paths
    pub fn add_open_subject(&mut self, open_subjects: &Paths64) {
        self.base.add_paths(open_subjects, PathType::Subject, true);
    }
}

impl Clipper64 {
    /// Add clip paths
    pub fn add_clip(&mut self, clips: &Paths64) {
        self.base.add_paths(clips, PathType::Clip, false);
    }
}

impl Clipper64 {
    /// Set/get preserve collinear
    pub fn set_preserve_collinear(&mut self, val: bool) {
        self.base.preserve_collinear = val;
    }
}

impl Clipper64 {
    /// Set/get reverse solution
    pub fn set_reverse_solution(&mut self, val: bool) {
        self.base.reverse_solution = val;
    }
}

impl Clipper64 {
    /// Execute a clipping operation, returning closed and open paths
    /// Direct port from clipper.engine.h Clipper64::Execute
    pub fn execute(
        &mut self,
        clip_type: ClipType,
        fill_rule: FillRule,
        solution_closed: &mut Paths64,
        mut solution_open: Option<&mut Paths64>,
    ) -> bool {
        solution_closed.clear();
        if let Some(ref mut open) = solution_open {
            open.clear();
        }

        if self.base.execute_internal(clip_type, fill_rule, false) {
            self.build_paths64(solution_closed, solution_open);
        }
        self.base.clean_up();
        self.base.succeeded
    }
}

impl Clipper64 {
    /// Execute a clipping operation, returning a polytree and open paths
    /// Direct port from clipper.engine.h Clipper64::Execute (polytree version)
    pub fn execute_tree(
        &mut self,
        clip_type: ClipType,
        fill_rule: FillRule,
        polytree: &mut PolyTree64,
        open_paths: &mut Paths64,
    ) -> bool {
        polytree.clear();
        open_paths.clear();

        if self.base.execute_internal(clip_type, fill_rule, true) {
            self.build_tree64(polytree, open_paths);
        }
        self.base.clean_up();
        self.base.succeeded
    }
}

impl Clipper64 {
    // clipper.engine.cpp:BuildPaths64/BuildTree64, BuildPath64 admission only.
    fn append_built_path(&self, record: usize, is_open: bool, output: &mut Paths64) {
        let Some(point) = self.base.outrec_list[record].pts else {
            return;
        };
        if let Some(path) = build_path64_from_outpt(
            point,
            self.base.reverse_solution,
            is_open,
            &self.base.outpt_arena,
        ) {
            output.push(path);
        }
    }
}

impl Clipper64 {
    /// Build output paths from outrec list
    /// Direct port from clipper.engine.cpp Clipper64::BuildPaths64 (line 2992)
    pub(crate) fn build_paths64(
        &mut self,
        solution_closed: &mut Paths64,
        solution_open: Option<&mut Paths64>,
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

            if solution_open.is_some() && self.base.outrec_list[i].is_open {
                self.append_built_path(i, true, &mut open_paths);
            } else {
                self.base.clean_collinear(i);
                self.append_built_path(i, false, solution_closed);
            }
            i += 1;
        }

        if let Some(open) = solution_open {
            *open = open_paths;
        }
    }
}

impl Clipper64 {
    /// Build polytree output
    /// Direct port from clipper.engine.cpp Clipper64::BuildTree64 (line 3027)
    pub(crate) fn build_tree64(&mut self, polytree: &mut PolyTree64, open_paths: &mut Paths64) {
        polytree.clear();
        open_paths.clear();
        if self.base.has_open_paths {
            open_paths.reserve(self.base.outrec_list.len());
        }

        let mut i = 0;
        while i < self.base.outrec_list.len() {
            if self.base.outrec_list[i].pts.is_none() {
                i += 1;
                continue;
            }

            if self.base.outrec_list[i].is_open {
                self.append_built_path(i, true, open_paths);
                i += 1;
                continue;
            }

            if self.base.check_bounds(i) {
                self.base.recursive_check_owners(i, polytree);
            }
            i += 1;
        }
    }
}

impl Default for Clipper64 {
    fn default() -> Self {
        Self::new()
    }
}
