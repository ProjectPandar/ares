//! Final `organic_draw_branches` outward smoothing, subtraction and allocation.

use super::super::interface_placer::{GeneratorLayer, LayerType, RoofLayers, layer_z};
use super::Slice;
use crate::geometry::{
    ClipperError, CoordinateScale, difference_polygons_paths, mutable_polygon::smooth_outward,
    simplification::polygons_simplify, union_polygons_paths,
};

pub(in super::super) fn allocate_layers(
    slices: Vec<Slice>,
    graph_layers: usize,
    roofs: &mut RoofLayers,
    layer_context: &super::OrganicLayerContext<'_>,
    scale: CoordinateScale,
) -> Result<(), ClipperError> {
    let config = layer_context.config;
    for (layer, slice) in slices.into_iter().take(graph_layers).enumerate() {
        debug_assert!(roofs.intermediate_layers[layer].is_none());
        let mut base = if slice.polygons.is_empty() {
            Vec::new()
        } else if slice.num_branches > 1 {
            union_polygons_paths(&slice.polygons)?
        } else {
            slice.polygons
        };
        let mut bottom = if slice.bottom_contacts.is_empty() {
            Vec::new()
        } else if slice.num_branches > 1 {
            union_polygons_paths(&slice.bottom_contacts)?
        } else {
            slice.bottom_contacts
        };
        if !base.is_empty() {
            base = smooth_outward(
                union_polygons_paths(&base)?,
                config.mesh.support_line_width,
                scale,
            );
            base = polygons_simplify(
                base,
                scale.scaled_delta(0.03).min(config.mesh.resolution as f64),
            )?;
        }
        if let Some(top) = roofs.top_contacts[layer]
            && !roofs.storage[top].polygons.is_empty()
            && !base.is_empty()
        {
            base = difference_polygons_paths(&base, &roofs.storage[top].polygons)?;
            if !bottom.is_empty() {
                bottom = difference_polygons_paths(&bottom, &roofs.storage[top].polygons)?;
            }
        }
        if !bottom.is_empty() {
            base = difference_polygons_paths(&base, &bottom)?;
            let index = allocate(
                roofs,
                LayerType::BottomContact,
                bottom,
                layer,
                layer_context,
            );
            roofs.bottom_contacts[layer] = Some(index);
        }
        if !base.is_empty() {
            let base = union_polygons_paths(&base)?;
            let index = allocate(roofs, LayerType::Base, base, layer, layer_context);
            roofs.intermediate_layers[layer] = Some(index);
        }
    }
    Ok(())
}

fn allocate(
    roofs: &mut RoofLayers,
    kind: LayerType,
    polygons: Vec<crate::geometry::Polygon>,
    layer: usize,
    layer_context: &super::OrganicLayerContext<'_>,
) -> usize {
    let config = layer_context.config;
    let slicing = layer_context.slicing;
    let print_z = layer_z(slicing, config, layer);
    let bottom_z = if layer > 0 {
        layer_z(slicing, config, layer - 1)
    } else {
        0.0
    };
    let index = roofs.storage.len();
    roofs.storage.push(GeneratorLayer {
        kind,
        print_z,
        bottom_z,
        height: print_z - bottom_z,
        polygons,
    });
    index
}
