//! GCode's pointwise first-layer and effective-layer queries.

use super::EmitState;
use crate::{geometry::CoordinateScale, project_slice::belt::BeltFloor};

pub(in crate::project_slice::gcode_emit) fn belt_fan_layers(
    traversal: &super::super::PreparedPostClassicTraversal,
    tool: usize,
) -> usize {
    let values = &traversal
        .resolved
        .views
        .full
        .filament
        .print
        .close_fan_the_first_x_layers
        .0;
    values
        .get(tool)
        .or_else(|| values.first())
        .map_or(0, |value| value.0.max(0) as usize)
}

pub(super) const EPSILON: f64 = 1e-4;

#[derive(Clone, Copy)]
pub(in crate::project_slice::gcode_emit) struct BeltBands {
    pub(in crate::project_slice::gcode_emit) floor: BeltFloor,
    pub(in crate::project_slice::gcode_emit) first: f64,
    pub(in crate::project_slice::gcode_emit) print_z: f64,
}

impl BeltBands {
    pub(in crate::project_slice::gcode_emit) fn thickness(self) -> f64 {
        if self.first > 0. { self.first } else { 0.2 }
    }

    pub(super) fn index(self, height: f64) -> usize {
        (height / self.thickness()).floor() as usize
    }

    pub(super) fn height(self, (x, y): (i64, i64), scale: CoordinateScale, width: f32) -> f64 {
        self.floor.height(
            [scale.unscale(x), scale.unscale(y)],
            self.print_z,
            f64::from(width),
        )
    }

    pub(super) fn is_first(self, height: f64) -> bool {
        height <= self.first + EPSILON
    }
}

impl EmitState {
    pub(super) fn path_on_first_layer(&self) -> bool {
        self.belt_piece
            .map_or(self.on_first_layer, |(first, _)| first)
    }

    pub(super) fn mark_belt_line(
        &mut self,
        output: &mut Vec<u8>,
        width: f32,
        (a, b): ((i64, i64), (i64, i64)),
        scale: CoordinateScale,
    ) {
        self.mark_belt_segment(output, width, || {
            [
                scale.unscale((a.0 + b.0) / 2),
                scale.unscale((a.1 + b.1) / 2),
            ]
        });
    }

    pub(super) fn mark_belt_segment(
        &mut self,
        output: &mut Vec<u8>,
        width: f32,
        midpoint: impl FnOnce() -> [f64; 2],
    ) {
        if self.belt.is_none() {
            return;
        }
        let index = self.belt_bands.map_or(
            if self.on_first_layer {
                0
            } else {
                self.layer_index
            },
            |bands| {
                bands.index(
                    bands
                        .floor
                        .height(midpoint(), bands.print_z, f64::from(width)),
                )
            },
        );
        let in_band = index < self.belt_fan_layers;
        if self.belt_in_fan_band != in_band {
            super::super::cooling::mark_belt_extrusion(output, in_band);
            self.belt_in_fan_band = in_band;
        }
    }

    pub(super) fn pointwise_travel_feedrate(&self, [x, y, z]: [f64; 3]) -> f64 {
        let Some(bands) = self.belt_bands else {
            return self.travel_feedrate;
        };
        let local = [x - self.offset.0, y - self.offset.1];
        if bands.is_first(bands.floor.height(local, z, self.belt_travel_width)) {
            self.options.first_layer_travel_feedrate
        } else {
            self.options.travel_feedrate
        }
    }
}
