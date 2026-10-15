// Ports AGPL-licensed OrcaSlicer `src/libvgcode/src/ExtrusionRoles.hpp` and `src/ExtrusionRoles.cpp`.

use crate::{GCodeExtrusionRole, TimeMode};

/// `ExtrusionRoles`: per-role estimated times of the extrusion moves. The
/// upstream `std::map` keyed by role is a fixed array in role order.
#[derive(Clone, Debug, Default)]
pub struct ExtrusionRoles {
    items: [Option<[f32; TimeMode::COUNT]>; GCodeExtrusionRole::COUNT],
}

impl ExtrusionRoles {
    /// ExtrusionRoles.cpp:9-18.
    pub fn add(&mut self, role: GCodeExtrusionRole, times: [f32; TimeMode::COUNT]) {
        add_times(self.items[role.index()].get_or_insert_default(), times);
    }

    pub fn get_roles_count(&self) -> usize {
        self.items.iter().flatten().count()
    }

    /// ExtrusionRoles.cpp:20-28: in role order.
    pub fn get_roles(&self) -> impl Iterator<Item = GCodeExtrusionRole> + '_ {
        GCodeExtrusionRole::ALL
            .into_iter()
            .filter(|role| self.items[role.index()].is_some())
    }

    /// ExtrusionRoles.cpp:30-37.
    pub fn get_time(&self, role: GCodeExtrusionRole, mode: TimeMode) -> f32 {
        self.items[role.index()].map_or(0.0, |times| times[mode.index()])
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

/// The per-time-mode f32 `+=` of libvgcode's time accumulators.
pub(crate) fn add_times(totals: &mut [f32; TimeMode::COUNT], times: [f32; TimeMode::COUNT]) {
    for (total, time) in totals.iter_mut().zip(times) {
        *total += time;
    }
}
