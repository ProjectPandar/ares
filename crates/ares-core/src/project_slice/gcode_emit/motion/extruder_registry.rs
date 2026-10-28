//! `GCodeWriter::set_extruders` / `Extruder` private and shared counters.

use super::extruder_state::ExtruderState;
use crate::{OrcaFloats, options::GCodeOptions};

#[derive(Clone, Copy, Default)]
pub(in crate::project_slice::gcode_emit) struct PrivateState {
    pub(in crate::project_slice::gcode_emit) position: f64,
    pub(in crate::project_slice::gcode_emit) retracted: f64,
    pub(in crate::project_slice::gcode_emit) restart_extra: f64,
    absolute_e: f64,
    retracted_flag: bool,
}

#[derive(Clone, Copy, Default)]
struct SharedAxis {
    position: f64,
    retracted: f64,
    retracted_flag: bool,
}

pub(in crate::project_slice::gcode_emit) struct ExtruderRegistry {
    active_id: usize,
    ids: Vec<usize>,
    private: Vec<PrivateState>,
    shared: Vec<SharedAxis>,
    physical: Vec<usize>,
    /// Last filament loaded into each physical extruder — upstream
    /// `m_writer.filament(extruder_id)->id()` (`GCode.cpp:7829`), read by
    /// `set_extruder` before `toolchange` activates the new filament.
    loaded: Vec<Option<usize>>,
    properties: Vec<(f64, f64)>,
}

impl Default for ExtruderRegistry {
    fn default() -> Self {
        Self {
            active_id: 0,
            ids: vec![0],
            private: vec![PrivateState::default()],
            shared: vec![SharedAxis::default()],
            physical: vec![0],
            loaded: vec![None],
            properties: vec![(1.75 * 1.75 * 0.25 * std::f64::consts::PI, 1.24)],
        }
    }
}

impl ExtruderRegistry {
    pub(in crate::project_slice::gcode_emit) fn new(
        mut ids: Vec<usize>,
        config: &GCodeOptions,
        physical_count: usize,
    ) -> Self {
        // `GCodeWriter::set_extruders` sorts its own copy, not the priming order.
        ids.sort_unstable();
        let count = ids.last().map_or(1, |id| id + 1);
        let physical = (0..count)
            .map(|id| {
                config
                    .filament_map
                    .0
                    .get(id)
                    .map_or(0, |value| (value.0 - 1) as usize)
            })
            .collect();
        let properties = (0..count)
            .map(|id| {
                let diameter = float_at(&config.filament_diameter, id, 1.75);
                (
                    diameter * diameter * 0.25 * std::f64::consts::PI,
                    float_at(&config.filament_density, id, 1.24),
                )
            })
            .collect();
        Self {
            active_id: 0,
            ids,
            private: vec![PrivateState::default(); count],
            shared: vec![SharedAxis::default(); physical_count],
            physical,
            loaded: vec![None; physical_count],
            properties,
        }
    }

    pub(in crate::project_slice::gcode_emit) fn ids(&self) -> &[usize] {
        &self.ids
    }

    pub(in crate::project_slice::gcode_emit) fn multiple(&self) -> bool {
        self.ids.last().is_some_and(|&id| id > 0)
    }

    pub(in crate::project_slice::gcode_emit) fn active_id(&self) -> usize {
        self.active_id
    }

    /// `m_writer.filament()->extruder_id()` (`Extruder.cpp:21-27`): the
    /// active filament's `filament_map` group minus one; a filament id
    /// beyond the map falls back to 0.
    pub(in crate::project_slice::gcode_emit) fn active_physical(&self) -> usize {
        self.physical[self.active_id]
    }

    pub(in crate::project_slice::gcode_emit) fn vector_size(&self) -> usize {
        self.ids.last().map_or(0, |id| id + 1)
    }

    pub(in crate::project_slice::gcode_emit) fn read(
        &self,
        id: usize,
        active: &ExtruderState,
        sharing: bool,
    ) -> PrivateState {
        if id == self.active_id {
            PrivateState {
                position: if sharing {
                    active.private_e_position
                } else {
                    active.e_position
                },
                retracted: if sharing {
                    active.private_retracted
                } else {
                    active.retracted_amount
                },
                restart_extra: active.restart_extra,
                absolute_e: active.filament_used,
                retracted_flag: active.retracted,
            }
        } else {
            self.private[id]
        }
    }

    pub(in crate::project_slice::gcode_emit) fn write_position(
        &mut self,
        id: usize,
        position: f64,
        active: &mut ExtruderState,
        sharing: bool,
    ) {
        if id == self.active_id {
            if sharing {
                active.private_e_position = position;
            } else {
                active.e_position = position;
            }
        } else {
            self.private[id].position = position;
        }
    }

    pub(in crate::project_slice::gcode_emit) fn write_retracted(
        &mut self,
        id: usize,
        (retracted, extra): (f64, f64),
        active: &mut ExtruderState,
        sharing: bool,
    ) {
        let (retracted, extra) = if retracted > 1e-4 {
            (retracted, if extra < 1e-4 { 0.0 } else { extra })
        } else {
            (0.0, 0.0)
        };
        if id == self.active_id {
            if sharing {
                active.private_retracted = retracted;
            } else {
                active.retracted_amount = retracted;
                active.retracted = retracted > 0.0;
            }
            active.restart_extra = extra;
        } else {
            self.private[id].retracted = retracted;
            self.private[id].retracted_flag = retracted > 0.0;
            self.private[id].restart_extra = extra;
        }
    }

    pub(in crate::project_slice::gcode_emit) fn activate(
        &mut self,
        id: usize,
        active: &mut ExtruderState,
        sharing: bool,
    ) {
        self.loaded[self.physical[id]] = Some(id);
        if id == self.active_id {
            return;
        }
        self.private[self.active_id] = self.read(self.active_id, active, sharing);
        if sharing {
            self.shared[self.physical[self.active_id]] = SharedAxis {
                position: active.e_position,
                retracted: active.retracted_amount,
                retracted_flag: active.retracted,
            };
        }
        let next = self.private[id];
        let axis = if sharing {
            self.shared[self.physical[id]]
        } else {
            SharedAxis {
                position: next.position,
                retracted: next.retracted,
                retracted_flag: next.retracted_flag,
            }
        };
        *active = ExtruderState {
            e_position: axis.position,
            retracted_amount: axis.retracted,
            retracted: axis.retracted_flag,
            private_e_position: next.position,
            private_retracted: next.retracted,
            restart_extra: next.restart_extra,
            filament_used: next.absolute_e,
        };
        self.active_id = id;
    }

    /// `GCodeWriter::filament(extruder_id)->id()` — filament last loaded
    /// into the physical extruder, `None` when it was never selected
    /// (`GCode.cpp:7829-7830`).
    pub(in crate::project_slice::gcode_emit) fn loaded_filament(
        &self,
        extruder: usize,
    ) -> Option<usize> {
        self.loaded.get(extruder).copied().flatten()
    }

    pub(in crate::project_slice::gcode_emit) fn used_filament(
        &self,
        id: usize,
        active: &ExtruderState,
        sharing: bool,
    ) -> f64 {
        let state = self.read(id, active, sharing);
        if sharing {
            state.absolute_e
        } else {
            state.absolute_e + state.retracted
        }
    }

    pub(in crate::project_slice::gcode_emit) fn statistics(
        &self,
        id: usize,
        active: &ExtruderState,
        (active_area, active_density): (f64, f64),
        sharing: bool,
    ) -> (f64, f64) {
        let used = self.used_filament(id, active, sharing);
        let (area, density) = if id == self.active_id {
            (active_area, active_density)
        } else {
            self.properties[id]
        };
        let volume = used * area;
        (volume, volume * density * 0.001)
    }
}

fn float_at(values: &OrcaFloats, id: usize, default: f64) -> f64 {
    values
        .0
        .get(id)
        .or_else(|| values.0.first())
        .map_or(default, |value| value.0)
}

#[cfg(test)]
mod tests;
