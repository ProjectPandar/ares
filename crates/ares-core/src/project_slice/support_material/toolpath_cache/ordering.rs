//! `SupportCommon.cpp:1532-1547,1818-1858` cache order, islands and overlaps.

use super::{LayerCache, LayerCacheIndices};
use crate::geometry::{ClipperError, CoordinateScale, ExPolygon, FillRule, union_ex};
use crate::project_slice::support_material::{
    extruded_layer::{ExtrudedLayer, SourceLayer},
    extrusion_path::SupportExtrusionEntity,
    grid_generation::GeneratedSupportGeometry,
    overlap_modulation::modulate_extrusion_by_overlapping_layers,
};

pub(in crate::project_slice::support_material) struct LayerCacheItem<E> {
    pub(in crate::project_slice::support_material) layer: ExtrudedLayer<E>,
    pub(in crate::project_slice::support_material) overlapping: Vec<SourceLayer>,
}

pub(in crate::project_slice::support_material) struct OrderedLayerCache<E> {
    pub(in crate::project_slice::support_material) nonempty: Vec<LayerCacheItem<E>>,
    pub(in crate::project_slice::support_material) islands: Vec<ExPolygon>,
}

impl OrderedLayerCache<SupportExtrusionEntity> {
    pub(in crate::project_slice::support_material) fn modulate_and_append(
        &mut self,
        output: &mut Vec<SupportExtrusionEntity>,
        geometry: &GeneratedSupportGeometry,
        scale: CoordinateScale,
    ) -> Result<(), crate::SliceError> {
        for item in &mut self.nonempty {
            modulate_extrusion_by_overlapping_layers(
                &mut item.layer.extrusions,
                item.layer.source,
                &item.overlapping,
                geometry,
                scale,
            )?;
            output.append(&mut item.layer.extrusions);
        }
        Ok(())
    }
}

impl<E> LayerCache<E> {
    pub(in crate::project_slice::support_material) fn collect_ordered(
        self,
        geometry: &GeneratedSupportGeometry,
        indices: &LayerCacheIndices,
    ) -> Result<OrderedLayerCache<E>, ClipperError> {
        let mut nonempty = Vec::with_capacity(5);
        for layer in [
            self.bottom,
            self.top,
            self.interface,
            self.base_interface,
            self.base,
        ]
        .into_iter()
        .flatten()
        {
            if !layer.empty(geometry) {
                nonempty.push(LayerCacheItem {
                    layer,
                    overlapping: Vec::with_capacity(20),
                });
            }
        }
        nonempty.sort_by(|left, right| {
            right
                .layer
                .source
                .height_and_bridging(geometry)
                .0
                .total_cmp(&left.layer.source.height_and_bridging(geometry).0)
        });
        let mut polygons = Vec::new();
        for item in &mut nonempty {
            item.layer.polygons_append(geometry, &mut polygons);
            let source = item.layer.source;
            let bottom_z = source.print_z(geometry) - source.height_and_bridging(geometry).0 + 1e-4;
            add_overlapping(
                &mut item.overlapping,
                geometry,
                indices.top,
                SourceLayer::Top,
                bottom_z,
            );
            if matches!(source, SourceLayer::Bottom(_)) {
                add_overlapping(
                    &mut item.overlapping,
                    geometry,
                    indices.base,
                    SourceLayer::Base,
                    bottom_z,
                );
                add_overlapping(
                    &mut item.overlapping,
                    geometry,
                    indices.interface,
                    SourceLayer::Interface,
                    bottom_z,
                );
                add_overlapping(
                    &mut item.overlapping,
                    geometry,
                    indices.base_interface,
                    SourceLayer::BaseInterface,
                    bottom_z,
                );
            }
            item.overlapping.sort_by(|left, right| {
                let (left_height, left_bridge) = left.height_and_bridging(geometry);
                let (right_height, right_bridge) = right.height_and_bridging(geometry);
                left.print_z(geometry)
                    .total_cmp(&right.print_z(geometry))
                    .then_with(|| right_height.total_cmp(&left_height))
                    .then_with(|| right_bridge.cmp(&left_bridge))
            });
        }
        let islands = if polygons.is_empty() {
            Vec::new()
        } else {
            union_ex(&polygons, FillRule::NonZero)?
        };
        Ok(OrderedLayerCache { nonempty, islands })
    }
}

fn add_overlapping(
    overlapping: &mut Vec<SourceLayer>,
    geometry: &GeneratedSupportGeometry,
    top: i32,
    source: fn(usize) -> SourceLayer,
    bottom_z: f64,
) {
    let mut index = top - 1;
    while index >= 0 {
        let layer = source(index as usize);
        if layer.print_z(geometry) <= bottom_z {
            break;
        }
        overlapping.push(layer);
        index -= 1;
    }
}
