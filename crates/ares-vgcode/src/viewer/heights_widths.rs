use super::ViewerImpl;
use crate::{
    DEFAULT_TRAVELS_RADIUS_MM, DEFAULT_WIPES_RADIUS_MM, MAX_TRAVELS_RADIUS_MM, MAX_WIPES_RADIUS_MM,
    MIN_TRAVELS_RADIUS_MM, MIN_WIPES_RADIUS_MM, Vec3,
};

/// `m_travels_radius` / `m_wipes_radius` (ViewerImpl.hpp:234,238).
#[derive(Clone, Copy, Debug)]
pub(super) struct Radii {
    travels: f32,
    wipes: f32,
}

impl Default for Radii {
    fn default() -> Self {
        Self {
            travels: DEFAULT_TRAVELS_RADIUS_MM,
            wipes: DEFAULT_WIPES_RADIUS_MM,
        }
    }
}

impl ViewerImpl {
    /// `set_travels_radius` (ViewerImpl.cpp:1648-1652).
    pub fn set_travels_radius(&mut self, radius: f32) {
        self.radii.travels = radius.clamp(MIN_TRAVELS_RADIUS_MM, MAX_TRAVELS_RADIUS_MM);
    }

    /// `get_travels_radius` (ViewerImpl.hpp:169).
    pub fn get_travels_radius(&self) -> f32 {
        self.radii.travels
    }

    /// `set_wipes_radius` (ViewerImpl.cpp:1654-1658).
    pub fn set_wipes_radius(&mut self, radius: f32) {
        self.radii.wipes = radius.clamp(MIN_WIPES_RADIUS_MM, MAX_WIPES_RADIUS_MM);
    }

    /// `get_wipes_radius` (ViewerImpl.hpp:171).
    pub fn get_wipes_radius(&self) -> f32 {
        self.radii.wipes
    }

    /// The `[height, width]` of each vertex in `extract_pos_and_or_hwa`
    /// (ViewerImpl.cpp:954-968): the travels radius for travels, the wipes
    /// radius for wipes, the vertex height and width otherwise. The radii are
    /// clamped to at least 0.05, so the `<= 0` early return (:918-919) never
    /// applies.
    pub fn heights_widths(&self) -> Vec<[f32; 2]> {
        self.heights_widths_range(0..self.vertices.len())
    }

    pub fn heights_widths_range(&self, range: std::ops::Range<usize>) -> Vec<[f32; 2]> {
        let Radii { travels, wipes } = self.radii;
        self.vertices[range]
            .iter()
            .map(|v| {
                if v.is_travel() {
                    [travels; 2]
                } else if v.is_wipe() {
                    [wipes; 2]
                } else {
                    [v.height, v.width]
                }
            })
            .collect()
    }

    /// The positions branch of `extract_pos_and_or_hwa` (ViewerImpl.cpp:945-952):
    /// extrusion vertices are pushed down by half their height, the others keep
    /// the nozzle z.
    pub fn positions(&self) -> Vec<Vec3> {
        self.positions_range(0..self.vertices.len())
    }

    pub fn positions_range(&self, range: std::ops::Range<usize>) -> Vec<Vec3> {
        self.vertices[range]
            .iter()
            .map(|v| {
                let [x, y, z] = v.position;
                if v.is_extrusion() {
                    [x, y, z - 0.5 * v.height]
                } else {
                    v.position
                }
            })
            .collect()
    }

    /// The join angle of `extract_pos_and_or_hwa` (ViewerImpl.cpp:928-934,
    /// 979-980): `atan2` of the z cross and the dot of the previous and the next
    /// line, each zero when that line is not valid (`valid_lines`).
    pub fn angles(&self) -> Vec<f32> {
        self.angles_range(0..self.vertices.len())
    }

    pub fn angles_range(&self, range: std::ops::Range<usize>) -> Vec<f32> {
        let line = |i: usize| {
            let [ax, ay, az] = self.vertices[i].position;
            let [bx, by, bz] = self.vertices[i + 1].position;
            [bx - ax, by - ay, bz - az]
        };
        range
            .map(|i| {
                let prev = if i > 0 && self.valid_lines[i - 1] {
                    line(i - 1)
                } else {
                    [0.0; 3]
                };
                let this = if self.valid_lines[i] {
                    line(i)
                } else {
                    [0.0; 3]
                };
                let dot = prev[0] * this[0] + prev[1] * this[1] + prev[2] * this[2];
                let cross = prev[0] * this[1] - prev[1] * this[0];
                // f64 evaluation plus rounding matches the platform-independent
                // result of the upstream float `std::atan2` (ViewerImpl.cpp:980).
                f64::from(cross).atan2(f64::from(dot)) as f32
            })
            .collect()
    }
}
