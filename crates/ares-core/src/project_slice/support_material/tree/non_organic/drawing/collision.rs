//! `TreeSupport.cpp:2069-2086` layer-local base/sharp-tail draw collisions.

use crate::{
    ObjectOptions,
    geometry::{
        ClipperError, CoordinateScale, ExPolygon, JoinType, offset_expolygons, union_expolygons,
    },
    project_slice::support_material::layer_input::LayerInput,
};

pub(super) struct DrawCollision<'a> {
    pub(super) object: &'a ObjectOptions,
    pub(super) outlines: &'a [Vec<ExPolygon>],
    pub(super) layers: &'a [LayerInput<'a>],
    pub(super) object_layer: usize,
    pub(super) top_z_distance: f64,
    pub(super) xy_distance: f64,
    pub(super) scale: CoordinateScale,
    pub(super) base: Vec<ExPolygon>,
    pub(super) sharp_tail: Vec<ExPolygon>,
}

impl DrawCollision<'_> {
    pub(super) fn get(&mut self, sharp_tail: bool) -> Result<&[ExPolygon], ClipperError> {
        let empty = if sharp_tail {
            self.sharp_tail.is_empty()
        } else {
            self.base.is_empty()
        };
        if empty {
            let collision = self.build(sharp_tail)?;
            if sharp_tail {
                self.sharp_tail = collision;
            } else {
                self.base = collision;
            }
        }
        Ok(if sharp_tail {
            &self.sharp_tail
        } else {
            &self.base
        })
    }

    fn build(&self, sharp_tail: bool) -> Result<Vec<ExPolygon>, ClipperError> {
        let distance = if sharp_tail {
            self.top_z_distance
        } else if self.object_layer == 0 {
            self.object.support_object_first_layer_gap.0
        } else {
            self.xy_distance
        };
        let mut collision = offset_expolygons(
            &self.outlines[self.object_layer],
            self.scale.scaled_delta(distance) as f32,
            JoinType::Miter,
            3.0,
        )?;
        if self.top_z_distance <= 0.0001 {
            return Ok(collision);
        }
        let mut accumulated = 0.0_f32;
        for layer in self.object_layer + 1..self.outlines.len() {
            accumulated = (f64::from(accumulated) + self.layers[layer].height) as f32;
            if accumulated == 0.0 || f64::from(accumulated) > self.top_z_distance {
                break;
            }
            let upper = offset_expolygons(
                &self.outlines[layer],
                self.scale.scaled_delta(self.top_z_distance) as f32,
                JoinType::Miter,
                3.0,
            )?;
            collision = union_expolygons(&collision, &upper)?;
        }
        Ok(collision)
    }
}
