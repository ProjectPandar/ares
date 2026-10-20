//! Reached planar `GCodeProcessor::process_G1` / `SeamsDetector` lifecycle.

#[derive(Default)]
pub(super) struct SeamDetector {
    detect_layer_based_on_tag: bool,
    active: bool,
    first_vertex: Option<[f32; 3]>,
    last_vertex: [f32; 3],
    external: bool,
    overhang: bool,
}

impl SeamDetector {
    pub(super) fn new(detect_layer_based_on_tag: bool) -> Self {
        Self {
            detect_layer_based_on_tag,
            ..Self::default()
        }
    }

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

    /// Returns the `Seam` vertex position `0.5f * (new_pos + first_vertex)`
    /// (`GCodeProcessor.cpp:4214-4216`) when the loop closes.
    pub(super) fn associate(&mut self, extruding: bool, end: [f64; 3]) -> Option<[f32; 3]> {
        // `GCodeProcessor.cpp:4199-4204`: refresh only an existing first vertex.
        if self.active
            && extruding
            && self.external
            && self.detect_layer_based_on_tag
            && self
                .first_vertex
                .is_some_and(|first| self.last_vertex[2] > first[2])
        {
            self.first_vertex = Some(self.last_vertex);
        }
        let mut seam_vertex = None;
        if self.active {
            if extruding && self.external {
                self.first_vertex.get_or_insert(self.last_vertex);
            } else if (!extruding || !self.overhang)
                && let Some(first) = self.first_vertex
            {
                // Upstream tests the previous stored vertex, not this move's
                // endpoint, after assigning the block's next-vertex index.
                let delta = std::array::from_fn::<_, 3, _>(|i| self.last_vertex[i] - first[i]);
                seam_vertex = (delta[0] * delta[0] + delta[1] * delta[1] + delta[2] * delta[2]
                    < 0.0625_f32)
                    .then(|| std::array::from_fn(|i| 0.5 * (self.last_vertex[i] + first[i])));
                self.active = false;
            }
        } else if extruding && self.external {
            self.active = true;
            self.first_vertex = Some(self.last_vertex);
        }
        self.last_vertex = end.map(|value| value as f32);
        seam_vertex
    }
}
