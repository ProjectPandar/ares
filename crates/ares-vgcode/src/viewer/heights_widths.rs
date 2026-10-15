use super::ViewerImpl;
use crate::{
    DEFAULT_TRAVELS_RADIUS_MM, DEFAULT_WIPES_RADIUS_MM, MAX_TRAVELS_RADIUS_MM, MAX_WIPES_RADIUS_MM,
    MIN_TRAVELS_RADIUS_MM, MIN_WIPES_RADIUS_MM,
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
        let Radii { travels, wipes } = self.radii;
        self.vertices
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
}
