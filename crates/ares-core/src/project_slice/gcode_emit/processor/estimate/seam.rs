//! Reached planar `GCodeProcessor::process_G1` / `SeamsDetector` lifecycle.

#[derive(Default)]
pub(super) struct SeamDetector {
    active: bool,
    first_vertex: Option<[f32; 3]>,
    last_vertex: [f32; 3],
    external: bool,
    overhang: bool,
}

impl SeamDetector {
    pub(super) fn role(&mut self, line: &str) {
        let Some(role) = line
            .strip_prefix(";TYPE:")
            .or_else(|| line.strip_prefix("; FEATURE: "))
        else {
            return;
        };
        self.external = role == "Outer wall";
        self.overhang = role == "Overhang wall";
        if self.external && !self.active {
            self.active = true;
            self.first_vertex = None;
        }
    }

    pub(super) fn associate(&mut self, extruding: bool, end: [f64; 3]) -> bool {
        let mut seam_vertex = false;
        if self.active {
            if extruding && self.external {
                self.first_vertex.get_or_insert(self.last_vertex);
            } else if (!extruding || !self.overhang)
                && let Some(first) = self.first_vertex
            {
                // Upstream tests the previous stored vertex, not this move's
                // endpoint, after assigning the block's next-vertex index.
                let delta = std::array::from_fn::<_, 3, _>(|i| self.last_vertex[i] - first[i]);
                seam_vertex =
                    delta[0] * delta[0] + delta[1] * delta[1] + delta[2] * delta[2] < 0.0625_f32;
                self.active = false;
            }
        } else if extruding && self.external {
            self.active = true;
            self.first_vertex = Some(self.last_vertex);
        }
        self.last_vertex = end.map(|value| value as f32);
        seam_vertex
    }

    pub(super) fn advance(&mut self, end: [f64; 3]) {
        self.last_vertex = end.map(|value| value as f32);
    }

    /// `store_move_vertex` seam-vertex insertion for an arc's first internal
    /// segment (`GCodeProcessor.cpp:4215-4225`): upstream checks
    /// `new_pos - first_vertex < 0.25` at the segment's start (the prior
    /// move's end). When an external-wall loop closed just before the arc,
    /// the arc start returns to `first_vertex`, so a `Seam` vertex is
    /// inserted and the arc's first block associates with it — excluded
    /// from `g1_times_cache` (:484-488), so `ExportLines::update` counts
    /// `remaining_internal_g1_lines == 0` for the arc.
    pub(super) fn associate_arc_start(&mut self, extruding: bool, start: [f64; 3]) -> bool {
        if self.active && self.first_vertex.is_some() {
            let first = self.first_vertex.unwrap();
            let delta = std::array::from_fn::<_, 3, _>(|i| start[i] as f32 - first[i]);
            if delta[0] * delta[0] + delta[1] * delta[1] + delta[2] * delta[2] < 0.0625_f32 {
                self.active = false;
                self.last_vertex = start.map(|value| value as f32);
                return true;
            }
        }
        // Mirror the ordinary path: an external extruding arc seeds
        // `first_vertex` when none is recorded yet.
        if extruding && self.external {
            if !self.active {
                self.active = true;
                self.first_vertex = Some(self.last_vertex);
            } else {
                self.first_vertex.get_or_insert(self.last_vertex);
            }
        }
        false
    }
}
