//! `TreeSupport.cpp:1050-1125` final overhang geometry and type assignment.

use super::{OverhangLayer, OverhangType, SmartOverhangs};
use crate::{
    ProcessSupportType,
    geometry::{
        BoundingBox, ClipperError, CoordinateScale, ExPolygon, JoinType, Polygon,
        clip_clipper_expolygons_with_subject_bbox, difference_ex, difference_ex_polygons,
        intersection_ex_polygons, offset_expolygons, offset_paths_tree, union_polygons_paths,
    },
    project_slice::support_material::tree::remove_bridges::{
        BridgeContactParameters, RegionBridges, remove_bridges_from_contacts,
    },
};

pub(in super::super) struct FinalLayerInputs<'a> {
    pub(in super::super) lower_slices: Option<&'a [ExPolygon]>,
    pub(in super::super) regions: &'a [RegionBridges<'a>],
    pub(in super::super) blockers: Option<&'a [Polygon]>,
    pub(in super::super) enforcers: Option<&'a [Polygon]>,
}

impl SmartOverhangs<'_> {
    pub(in super::super) fn finalize_layer(
        &mut self,
        layer_index: usize,
        inputs: FinalLayerInputs<'_>,
    ) -> Result<(), ClipperError> {
        let (lower, current) = self.layers.split_at_mut(layer_index);
        let layer = &mut current[0];
        let lower = lower.last();
        let mut sharp_tail_overhangs = sharp_tail_overhangs(layer, lower, self.scale)?;
        if lower.is_some() && !sharp_tail_overhangs.is_empty() {
            self.has_sharp_tails = true;
        }
        if let Some(blockers) = inputs.blockers {
            let blockers = union_polygons_paths(blockers)?;
            let blockers = offset_paths_tree(
                &blockers,
                self.scale.scaled_delta(f64::from(0.2_f32)) as f32,
                JoinType::Miter,
                3.0,
            )?
            .into_expolygons(self.scale);
            layer.overhangs = difference_ex(&layer.overhangs, &blockers)?;
            layer.cantilevers = difference_ex(&layer.cantilevers, &blockers)?;
            sharp_tail_overhangs = difference_ex(&sharp_tail_overhangs, &blockers)?;
        }
        if self.object.support_critical_regions_only.0
            && matches!(self.object.support_type, ProcessSupportType::TreeAuto)
        {
            layer.overhangs.clone_from(&layer.cantilevers);
        }
        let max_bridge_length = self.scale.scaled_delta(self.object.max_bridge_length.0);
        if max_bridge_length > 0.0
            && !layer.overhangs.is_empty()
            && let Some(lower_slices) = inputs.lower_slices
        {
            remove_bridges_from_contacts(
                inputs.regions,
                lower_slices,
                &mut layer.overhangs,
                BridgeContactParameters {
                    extrusion_width_scaled: self.extrusion_width_scaled as f32,
                    max_bridge_length_scaled: max_bridge_length as f32,
                    scale: self.scale,
                },
            )?;
        }
        let detected_count = layer.overhangs.len();
        if let Some(enforcers) = inputs.enforcers
            && let Some(lower) = lower
        {
            let difference = difference_ex(&layer.extrudable, &lower.extrudable)?;
            let enforced = intersection_ex_polygons(&difference, enforcers)?;
            if !enforced.is_empty() {
                let expanded = offset_expolygons(
                    &enforced,
                    self.scale.scaled_delta(0.8) as f32,
                    JoinType::Miter,
                    3.0,
                )?;
                layer
                    .overhangs
                    .extend(difference_ex(&expanded, &lower.extrudable)?);
            }
        }
        let enforced_count = layer.overhangs.len();
        layer.overhangs.extend(sharp_tail_overhangs);
        layer
            .overhang_types
            .extend((0..layer.overhangs.len()).map(|index| {
                if index < detected_count {
                    OverhangType::Detected
                } else if index < enforced_count {
                    OverhangType::Enforced
                } else {
                    OverhangType::SharpTail
                }
            }));
        if !layer.overhangs.is_empty() {
            self.highest_overhang_layer = self.highest_overhang_layer.max(layer_index);
        }
        if !layer.cantilevers.is_empty() {
            self.has_cantilever = true;
        }
        Ok(())
    }
}

fn sharp_tail_overhangs(
    layer: &OverhangLayer,
    lower: Option<&OverhangLayer>,
    scale: CoordinateScale,
) -> Result<Vec<ExPolygon>, ClipperError> {
    let Some(lower) = lower else {
        return Ok(layer.sharp_tails.clone());
    };
    let expanded = offset_expolygons(
        &lower.extrudable,
        scale.scaled_delta(0.0125) as f32,
        JoinType::Miter,
        3.0,
    )?;
    let mut overhangs = Vec::new();
    for (index, &height) in layer.sharp_tail_heights.iter().enumerate() {
        let tail = &layer.sharp_tails[index];
        let bounds = BoundingBox::from_expolygon(tail)
            .unwrap()
            .inflated(scale.scaled_delta(1e-4) as i64);
        let clip = clip_clipper_expolygons_with_subject_bbox(&expanded, bounds);
        let areas = difference_ex_polygons(std::slice::from_ref(tail), &clip)?;
        if !areas.is_empty() && (height * 10.0) as i32 % 5 == 0 {
            overhangs.extend(areas);
        }
    }
    Ok(overhangs)
}
