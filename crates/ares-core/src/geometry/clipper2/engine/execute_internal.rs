// Copyright (c) Angus Johnson 2010-2025.
// BSL-1.0. Rust seed by MatterHackers.
// Derived from clipper2-rust 2f5e9d4249bc4d624773094ff6164335d9ff6ae7.
// See ../LICENSE-Boost-1.0 and ../SOURCE.md for pinned source authority.

use super::*;

impl ClipperBase {
    /// Main execution loop of the sweep-line algorithm
    /// Direct port from clipper.engine.cpp ExecuteInternal (line 2129)
    pub fn execute_internal(
        &mut self,
        ct: ClipType,
        fillrule: FillRule,
        use_polytrees: bool,
    ) -> bool {
        self.cliptype = ct;
        self.fillrule = fillrule;
        self.using_polytree = use_polytrees;
        self.reset();

        if ct == ClipType::NoClip {
            return true;
        }

        let y = match self.pop_scanline() {
            Some(y) => y,
            None => return true,
        };

        let mut y = y;
        while self.succeeded {
            self.insert_local_minima_into_ael(y);

            while let Some(e) = self.pop_horz() {
                self.do_horizontal(e);
            }

            if !self.horz_seg_list.is_empty() {
                self.convert_horz_segs_to_joins();
                self.horz_seg_list.clear();
            }

            self.bot_y = y;

            match self.pop_scanline() {
                Some(new_y) => y = new_y,
                None => break,
            }

            self.do_intersections(y);
            self.do_top_of_scanbeam(y);

            while let Some(e) = self.pop_horz() {
                self.do_horizontal(e);
            }
        }

        if self.succeeded {
            self.process_horz_joins();
        }

        self.succeeded
    }
}
