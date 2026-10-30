// Copyright (c) Angus Johnson 2010-2025.
// BSL-1.0. Rust seed by MatterHackers.
// Derived from clipper2-rust 2f5e9d4249bc4d624773094ff6164335d9ff6ae7.
// See ../LICENSE-Boost-1.0 and ../SOURCE.md for pinned source authority.

use super::*;

impl PolyTreeD {
    pub fn is_hole(&self, node_idx: usize) -> bool {
        let lvl = self.level(node_idx);
        lvl > 0 && (lvl & 1) == 0
    }
}

impl Default for PolyTreeD {
    fn default() -> Self {
        Self::new()
    }
}

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
    /// Get error code
    pub fn error_code(&self) -> i32 {
        self.base.error_code
    }
}

impl Clipper64 {
    /// Set/get preserve collinear
    pub fn set_preserve_collinear(&mut self, val: bool) {
        self.base.preserve_collinear = val;
    }
}

impl Clipper64 {
    pub fn preserve_collinear(&self) -> bool {
        self.base.preserve_collinear
    }
}

impl Clipper64 {
    /// Set/get reverse solution
    pub fn set_reverse_solution(&mut self, val: bool) {
        self.base.reverse_solution = val;
    }
}

impl Clipper64 {
    pub fn reverse_solution(&self) -> bool {
        self.base.reverse_solution
    }
}

impl Clipper64 {
    /// Clear the clipper
    pub fn clear(&mut self) {
        self.base.clear();
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

            if self.base.outrec_list[i].is_open {
                let op = self.base.outrec_list[i].pts.unwrap();
                if let Some(path) = build_path64_from_outpt(
                    op,
                    self.base.reverse_solution,
                    true,
                    &self.base.outpt_arena,
                ) {
                    open_paths.push(path);
                }
            } else {
                self.base.clean_collinear(i);
                if self.base.outrec_list[i].pts.is_some() {
                    let op = self.base.outrec_list[i].pts.unwrap();
                    if let Some(path) = build_path64_from_outpt(
                        op,
                        self.base.reverse_solution,
                        false,
                        &self.base.outpt_arena,
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
                let op = self.base.outrec_list[i].pts.unwrap();
                if let Some(path) = build_path64_from_outpt(
                    op,
                    self.base.reverse_solution,
                    true,
                    &self.base.outpt_arena,
                ) {
                    open_paths.push(path);
                }
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

/// Double-precision polygon clipper that scales to int64 internally
/// Direct port from clipper.engine.h line 520
pub struct ClipperD {
    pub base: ClipperBase,
    pub(crate) scale: f64,
    pub(crate) inv_scale: f64,
}

impl ClipperD {
    pub fn new(precision: i32) -> Self {
        let mut prec = precision;
        let mut error_code = 0;
        check_precision_range(&mut prec, &mut error_code);

        // Set the scale to a power of double's radix (2)
        let scale = 2.0f64.powi(((10.0f64.powi(prec)).log2().floor() as i32) + 1);
        let inv_scale = 1.0 / scale;

        let mut base = ClipperBase::new();
        base.error_code = error_code;

        Self {
            base,
            scale,
            inv_scale,
        }
    }
}

impl ClipperD {
    pub fn scale(&self) -> f64 {
        self.scale
    }
}

impl ClipperD {
    pub fn inv_scale(&self) -> f64 {
        self.inv_scale
    }
}

impl ClipperD {
    /// Set the callback invoked at each intersection to assign the z value.
    /// The user callback sees de-scaled (double) coordinates; the stored proxy
    /// closure replaces C++'s ZCB member + std::bind indirection.
    /// Direct port from clipper.engine.h lines 540-556 (ClipperD::SetZCallback/ZCB)

    pub fn set_z_callback(&mut self, mut cb: impl ZCallbackD + 'static) {
        let inv_scale = self.inv_scale;
        self.base.z_callback = Some(Box::new(move |e1bot, e1top, e2bot, e2top, pt| {
            // de-scale (x & y)
            // temporarily convert integers to their initial float values
            // this will slow clipping marginally but will make it much easier
            // to understand the coordinates passed to the callback function
            let mut tmp = PointD::from(pt as &Point64).scale(inv_scale);
            let e1b = PointD::from(e1bot).scale(inv_scale);
            let e1t = PointD::from(e1top).scale(inv_scale);
            let e2b = PointD::from(e2bot).scale(inv_scale);
            let e2t = PointD::from(e2top).scale(inv_scale);
            cb(&e1b, &e1t, &e2b, &e2t, &mut tmp);
            pt.z = tmp.z; // only update 'z'
        }));
    }
}
