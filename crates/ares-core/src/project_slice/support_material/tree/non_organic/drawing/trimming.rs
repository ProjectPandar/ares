//! Base clearance subset of `TreeSupport.cpp:1904-1975::get_trim_support_regions`.

use crate::geometry::{
    ClipperError, CoordinateScale, JoinType, Polygon, intersection_ex, offset_expolygons_paths,
};
use crate::project_slice::{layers::PlannedLayer, support_material::trim_object::TrimLayer};

pub(super) struct TreeTrimming<'a> {
    pub(super) planned: &'a [PlannedLayer],
    pub(super) layers: &'a [TrimLayer<'a>],
    pub(super) gap_below: f64,
    pub(super) zero_gap_interface_top: bool,
    pub(super) thick_bridges: bool,
    pub(super) scale: CoordinateScale,
}

impl TreeTrimming<'_> {
    pub(super) fn regions(&self, print_z: f64, height: f64) -> Result<Vec<Polygon>, ClipperError> {
        let bottom_z = print_z - height;
        let threshold = bottom_z - self.gap_below + 0.0001;
        let mut index = self
            .planned
            .partition_point(|layer| layer.print_z < threshold);
        let mut trimming = Vec::new();
        while index < self.planned.len() {
            let layer = &self.planned[index];
            if layer.print_z - layer.height > print_z - 0.0001 {
                break;
            }
            let overlap = layers_overlap(print_z, bottom_z, layer, 0.0);
            for polygon in self.layers[index].slices {
                let sharp_tail = !intersection_ex(
                    std::slice::from_ref(polygon),
                    self.layers[index].sharp_tails,
                )?
                .is_empty();
                let gap = trimming_gap(overlap, sharp_tail);
                trimming.extend(offset_expolygons_paths(
                    std::slice::from_ref(polygon),
                    self.scale.scaled_delta(gap) as f32,
                    JoinType::Square,
                    0.0,
                )?);
            }
            index += 1;
        }
        if !self.zero_gap_interface_top && self.thick_bridges {
            self.append_bridges(print_z, bottom_z, index, &mut trimming)?;
        }
        Ok(trimming)
    }

    fn append_bridges(
        &self,
        print_z: f64,
        bottom_z: f64,
        start: usize,
        trimming: &mut Vec<Polygon>,
    ) -> Result<(), ClipperError> {
        for index in start..self.planned.len() {
            if !self.append_layer_bridges(print_z, bottom_z, index, trimming)? {
                break;
            }
        }
        Ok(())
    }

    fn append_layer_bridges(
        &self,
        print_z: f64,
        bottom_z: f64,
        index: usize,
        trimming: &mut Vec<Polygon>,
    ) -> Result<bool, ClipperError> {
        let layer = &self.planned[index];
        let mut any_overlap = false;
        for region in self.layers[index].regions {
            if layer.print_z - region.bridging_height > print_z - 0.0001 {
                break;
            }
            any_overlap = true;
            let gap = trimming_gap(
                layers_overlap(print_z, bottom_z, layer, region.bridging_height),
                false,
            );
            trimming.extend(offset_expolygons_paths(
                region.bottom_bridges,
                self.scale.scaled_delta(gap) as f32,
                JoinType::Square,
                0.0,
            )?);
        }
        Ok(any_overlap)
    }
}

fn trimming_gap(overlap: bool, sharp_tail: bool) -> f64 {
    if sharp_tail || !overlap {
        f64::from(0.2_f32)
    } else {
        0.0
    }
}

fn layers_overlap(print_z: f64, bottom_z: f64, layer: &PlannedLayer, bridging_height: f64) -> bool {
    if (print_z - layer.print_z).abs() < 0.0001 {
        return true;
    }
    let height = if bridging_height > 0.0001 {
        bridging_height
    } else {
        layer.height
    };
    (print_z < layer.print_z && print_z > layer.print_z - height)
        || (print_z > layer.print_z && bottom_z < layer.print_z - 0.0001)
}
