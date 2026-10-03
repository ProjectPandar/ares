//! Native gracious/non-gracious landings and bottom contact retention.

use super::{Branch, BranchSlices, Drawing, clipping};
use crate::geometry::{ClipperError, Polygon};

impl Drawing<'_, '_> {
    pub(super) fn root_contacts(
        &mut self,
        branch: &Branch,
        slices: &mut BranchSlices,
    ) -> Result<(), ClipperError> {
        let state = self.element(branch.path[0]).state;
        let front = slices.polygons[0].clone();
        let floors = self.config().mesh.floor_layers > 0;
        let model = self.config().support_rests_on_model;
        let gap = self.config().z_distance_bottom_layers > 0;
        if state.bits.to_model_gracious {
            self.gracious_contact(&front, slices)?;
        } else if slices.begin > 0 {
            self.descend_root(
                &front,
                state.actual_radius(self.config()),
                state.bits.very_lost,
                slices,
            )?;
        }
        if model
            && state.layer_index > 0
            && floors
            && gap
            && slices.contacts.is_empty()
            && !front.is_empty()
        {
            slices.contacts.push(front);
        }
        Ok(())
    }

    fn gracious_contact(
        &mut self,
        front: &[Polygon],
        slices: &mut BranchSlices,
    ) -> Result<(), ClipperError> {
        if self.config().mesh.floor_layers <= 0 {
            return Ok(());
        }
        let model = self.config().support_rests_on_model;
        let gap = self.config().z_distance_bottom_layers > 0;
        let mut contacts = if model && gap && slices.begin > 0 {
            front.to_vec()
        } else {
            self.placeable_contact(front, slices.begin)?
        };
        clipping::remove_small(&mut contacts, self.extrusion.scale);
        if contacts.is_empty() && model && slices.begin > 0 && !front.is_empty() {
            contacts.extend_from_slice(front);
        }
        if !contacts.is_empty() {
            slices.contacts.push(contacts);
        }
        Ok(())
    }

    fn placeable_contact(
        &mut self,
        polygons: &[Polygon],
        layer: usize,
    ) -> Result<Vec<Polygon>, ClipperError> {
        let scale = self.extrusion.scale;
        clipping::intersection(polygons, self.volumes.placeable_areas(0, layer)?, scale)
    }

    fn descend_root(
        &mut self,
        front: &[Polygon],
        radius: i64,
        very_lost: bool,
        slices: &mut BranchSlices,
    ) -> Result<(), ClipperError> {
        let config = self.config();
        let maximum = 5 * radius / config.mesh.layer_height;
        let lowest = if very_lost {
            0
        } else {
            (slices.begin as i64 - maximum).max(0) as usize
        };
        let minimum_radius_area = std::f64::consts::PI * (config.branch_radius as f64).powi(2);
        let stop_area =
            (0.2 * std::f64::consts::PI * (radius as f64).powi(2)).max(0.5 * minimum_radius_area);
        let scale = self.extrusion.scale;
        let mut extra = Vec::new();
        let mut rest = Vec::new();
        for layer in (lowest..slices.begin).rev() {
            let collision_layer = if layer == slices.begin - 1 {
                slices.begin
            } else {
                layer
            };
            let collision = self.volumes.collision(0, collision_layer, false)?;
            rest = clipping::difference(
                if rest.is_empty() { front } else { &rest },
                collision,
                scale,
            )?;
            clipping::remove_small(&mut rest, scale);
            if rest.iter().map(Polygon::area).sum::<f64>() < stop_area {
                break;
            }
            extra.push(rest.clone());
        }
        if self.config().mesh.floor_layers > 0 {
            let model = self.config().support_rests_on_model;
            let mut contacts = self.descended_contact(front, &extra, slices.begin)?;
            clipping::remove_small(&mut contacts, scale);
            if !contacts.is_empty() {
                slices.contacts.push(contacts);
            }
            if slices.contacts.is_empty() && model && slices.begin > 0 && !front.is_empty() {
                slices.contacts.push(front.to_vec());
            }
        }
        slices.begin -= extra.len();
        extra.reverse();
        extra.append(&mut slices.polygons);
        slices.polygons = extra;
        Ok(())
    }

    fn descended_contact(
        &mut self,
        front: &[Polygon],
        extra: &[Vec<Polygon>],
        begin: usize,
    ) -> Result<Vec<Polygon>, ClipperError> {
        let model = self.config().support_rests_on_model;
        let gap = self.config().z_distance_bottom_layers > 0;
        if let Some(lowest) = extra.last() {
            if model && gap && begin > 0 {
                clipping::intersection(
                    lowest,
                    std::slice::from_ref(&self.volumes.bed_area),
                    self.extrusion.scale,
                )
            } else {
                self.placeable_contact(lowest, begin)
            }
        } else if model && gap && begin > 0 {
            Ok(front.to_vec())
        } else {
            self.placeable_contact(front, begin)
        }
    }
}
