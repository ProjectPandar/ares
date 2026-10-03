//! `SupportMaterial.cpp::trim_support_layers_by_object`.

use super::bridging_perimeters::collect_bridging_perimeter_areas;
use crate::geometry::{
    ClipperError, CoordinateScale, ExPolygon, JoinType, Polygon, difference_polygons_paths,
    intersection_ex, offset_expolygons_paths,
};
use crate::project_slice::{
    layers::PlannedLayer, perimeters::classic::entity_collections::ExtrusionEntityCollection,
};

const EPSILON: f64 = 1e-4;
// Native namespace constants are doubles initialized from 0.2f.
const SHARP_TAIL_XY_GAP: f64 = 0.2_f32 as f64;
const NO_OVERLAP_XY_GAP: f64 = 0.2_f32 as f64;

pub(super) struct TrimRegion<'a> {
    pub(super) bridging_height: f64,
    // All stBottomBridge fill surfaces, including undetermined angles.
    pub(super) bottom_bridges: &'a [ExPolygon],
    pub(super) detect_overhang_wall: bool,
    pub(super) perimeters: &'a [ExtrusionEntityCollection],
}

pub(super) struct TrimLayer<'a> {
    pub(super) slices: &'a [ExPolygon],
    pub(super) sharp_tails: &'a [ExPolygon],
    pub(super) regions: &'a [TrimRegion<'a>],
}

pub(super) struct ObjectTrimming<'a> {
    pub(super) planned: &'a [PlannedLayer],
    pub(super) layers: &'a [TrimLayer<'a>],
    pub(super) raft_contact_top_z: f64,
    pub(super) gap_extra_above: f64,
    pub(super) gap_extra_below: f64,
    pub(super) gap_xy: f64,
    pub(super) zero_gap_interface_top: bool,
    pub(super) thick_bridges: bool,
    pub(super) scale: CoordinateScale,
}

pub(super) struct SupportLayerView<'a> {
    pub(super) print_z: f64,
    pub(super) bottom_z: f64,
    pub(super) height: f64,
    pub(super) polygons: &'a mut Vec<Polygon>,
}

pub(super) fn trim_support_layers_by_object<'a>(
    input: &ObjectTrimming<'_>,
    support_layers: impl IntoIterator<Item = SupportLayerView<'a>>,
) -> Result<(), ClipperError> {
    let gap_xy_scaled = (input.gap_xy / input.scale.factor()) as f32;
    let mut overlapping_index = None;
    for support in support_layers {
        if support.polygons.is_empty() || support.print_z < input.raft_contact_top_z + EPSILON {
            continue;
        }
        let threshold = support.print_z - support.height - input.gap_extra_below + EPSILON;
        let mut index = overlapping_index.unwrap_or_else(|| {
            input
                .planned
                .partition_point(|layer| layer.print_z < threshold)
        });
        while index < input.planned.len() && input.planned[index].print_z < threshold {
            index += 1;
        }
        overlapping_index = Some(index);
        let mut trimming = Vec::new();
        while index < input.planned.len() {
            let layer = &input.planned[index];
            if layer.print_z - layer.height > support.print_z + input.gap_extra_above - EPSILON {
                break;
            }
            let overlaps = layers_overlap(&support, layer, 0.0);
            for polygon in input.layers[index].slices {
                let sharp_tail = !intersection_ex(
                    std::slice::from_ref(polygon),
                    input.layers[index].sharp_tails,
                )?
                .is_empty();
                let offset = match (sharp_tail, overlaps) {
                    (true, _) => SHARP_TAIL_XY_GAP / input.scale.factor(),
                    (false, true) => f64::from(gap_xy_scaled),
                    (false, false) => NO_OVERLAP_XY_GAP / input.scale.factor(),
                };
                trimming.extend(offset_expolygons_paths(
                    std::slice::from_ref(polygon),
                    offset as f32,
                    JoinType::Square,
                    0.0,
                )?);
            }
            index += 1;
        }
        if !input.zero_gap_interface_top && input.thick_bridges {
            append_bridge_trimming(input, &support, index, gap_xy_scaled, &mut trimming)?;
        }
        *support.polygons = difference_polygons_paths(support.polygons, &trimming)?;
    }
    Ok(())
}

fn append_bridge_trimming(
    input: &ObjectTrimming<'_>,
    support: &SupportLayerView<'_>,
    start: usize,
    gap_xy_scaled: f32,
    trimming: &mut Vec<Polygon>,
) -> Result<(), ClipperError> {
    for index in start..input.planned.len() {
        let layer = &input.planned[index];
        let mut some_region_overlaps = false;
        for region in input.layers[index].regions {
            if layer.print_z - region.bridging_height
                > support.print_z + input.gap_extra_above - EPSILON
            {
                break;
            }
            some_region_overlaps = true;
            let offset = if layers_overlap(support, layer, region.bridging_height) {
                f64::from(gap_xy_scaled)
            } else {
                NO_OVERLAP_XY_GAP / input.scale.factor()
            };
            trimming.extend(offset_expolygons_paths(
                region.bottom_bridges,
                offset as f32,
                JoinType::Square,
                0.0,
            )?);
            if region.detect_overhang_wall {
                collect_bridging_perimeter_areas(
                    region.perimeters,
                    gap_xy_scaled,
                    input.scale,
                    trimming,
                )?;
            }
        }
        if !some_region_overlaps {
            break;
        }
    }
    Ok(())
}

fn layers_overlap(
    support: &SupportLayerView<'_>,
    object: &PlannedLayer,
    bridging_height: f64,
) -> bool {
    if (support.print_z - object.print_z).abs() < EPSILON {
        return true;
    }
    let height = if bridging_height > EPSILON {
        bridging_height
    } else {
        object.height
    };
    (support.print_z < object.print_z && support.print_z > object.print_z - height)
        || (support.print_z > object.print_z && support.bottom_z < object.print_z - EPSILON)
}
