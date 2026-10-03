//! `SupportMaterial.cpp::generate_base_layers`.

use super::{
    bottom_contacts::BottomContactLayer,
    contact_layer::TopContactLayer,
    intermediate_heights::{IntermediateLayer, SupportLayerType},
    layer_index::idx_lower_or_equal,
    trim_object::{ObjectTrimming, SupportLayerView, trim_support_layers_by_object},
};
use crate::geometry::{ClipperError, Polygon, difference_polygons_paths_with_safety_offset};
use crate::project_slice::layers::PlannedLayer;

const EPSILON: f64 = 1e-4;

pub(super) struct BaseLayersInput<'a> {
    pub(super) planned: &'a [PlannedLayer],
    pub(super) bottoms: &'a [BottomContactLayer],
    pub(super) tops: &'a [TopContactLayer],
    pub(super) support_areas: &'a [Vec<Polygon>],
    pub(super) trimming: &'a ObjectTrimming<'a>,
}

pub(super) fn generate_base_layers(
    input: &BaseLayersInput<'_>,
    layers: &mut [IntermediateLayer],
) -> Result<(), ClipperError> {
    if input.tops.is_empty() {
        return Ok(());
    }
    let mut top_above = -2;
    let mut bottom_overlapping = -2;
    let mut object_above = -2;
    for base in layers.iter_mut().rev() {
        let object_index = idx_lower_or_equal(input.planned, &mut object_above, |layer| {
            layer.print_z <= base.print_z + EPSILON
        });
        let top_index = idx_lower_or_equal(input.tops, &mut top_above, |top| {
            top.bottom_z <= base.print_z - EPSILON
        });
        let mut trimming = Vec::new();
        for index in (0..=top_index).rev() {
            let top = &input.tops[index as usize];
            if top.print_z < base.bottom_z + EPSILON {
                break;
            }
            if base.print_z <= top.print_z + EPSILON && base.bottom_z >= top.bottom_z - EPSILON {
                trimming.extend_from_slice(&top.polygons);
            }
        }
        let mut polygons = if object_index < 0 {
            input.support_areas[0].clone()
        } else {
            input.support_areas[object_index as usize].clone()
        };
        if object_index < 0 {
            let first_layer_z = input.planned[0].print_z;
            for top in input.tops[(top_index + 1) as usize..]
                .iter()
                .take_while(|top| top.print_z <= first_layer_z + EPSILON)
            {
                polygons.extend_from_slice(&top.polygons);
            }
        }
        let bottom_index = idx_lower_or_equal(input.bottoms, &mut bottom_overlapping, |bottom| {
            bottom.print_z - bottom.height <= base.print_z - EPSILON
        });
        for index in (0..=bottom_index).rev() {
            let bottom = &input.bottoms[index as usize];
            if bottom.print_z < base.print_z - base.height + EPSILON {
                break;
            }
            if base.print_z <= bottom.print_z + EPSILON
                && base.bottom_z >= bottom.print_z - bottom.height - EPSILON
            {
                trimming.extend_from_slice(&bottom.polygons);
            }
        }
        base.polygons = if trimming.is_empty() {
            polygons
        } else {
            difference_polygons_paths_with_safety_offset(&polygons, &trimming)?
        };
        base.layer_type = SupportLayerType::Base;
    }
    trim_support_layers_by_object(
        input.trimming,
        layers.iter_mut().map(|layer| SupportLayerView {
            print_z: layer.print_z,
            bottom_z: layer.bottom_z,
            height: layer.height,
            polygons: &mut layer.polygons,
        }),
    )
}
