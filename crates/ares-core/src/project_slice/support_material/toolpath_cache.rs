//! Layer selection prefix of `SupportCommon.cpp::generate_support_toolpaths:1594-1630`.

mod base;
pub(super) mod contacts;
pub(super) mod interfaces;
mod ordering;

use super::{
    extruded_layer::{ExtrudedLayer, SourceLayer},
    grid_generation::GeneratedSupportGeometry,
    layer_index::{idx_higher_or_equal, idx_higher_or_equal_indexed},
};

pub(super) struct LayerCache<E> {
    pub(super) bottom: Option<ExtrudedLayer<E>>,
    pub(super) top: Option<ExtrudedLayer<E>>,
    pub(super) base: Option<ExtrudedLayer<E>>,
    pub(super) interface: Option<ExtrudedLayer<E>>,
    pub(super) base_interface: Option<ExtrudedLayer<E>>,
}

pub(super) struct LayerCacheIndices {
    bottom: i32,
    top: i32,
    base: i32,
    interface: i32,
    base_interface: i32,
}

impl Default for LayerCacheIndices {
    fn default() -> Self {
        Self {
            bottom: -1,
            top: -1,
            base: -1,
            interface: -1,
            base_interface: -1,
        }
    }
}

impl LayerCacheIndices {
    pub(super) fn select<E>(
        &mut self,
        geometry: &GeneratedSupportGeometry,
        print_z: f64,
    ) -> LayerCache<E> {
        LayerCache {
            bottom: select_layer(
                &geometry.bottoms,
                &mut self.bottom,
                print_z,
                |layer| layer.print_z,
                SourceLayer::Bottom,
            ),
            top: select_layer(
                &geometry.tops,
                &mut self.top,
                print_z,
                |layer| layer.print_z,
                SourceLayer::Top,
            ),
            base: select_layer(
                &geometry.bases,
                &mut self.base,
                print_z,
                |layer| layer.print_z,
                SourceLayer::Base,
            ),
            interface: select_interface(geometry, &mut self.interface, print_z, false),
            base_interface: select_interface(geometry, &mut self.base_interface, print_z, true),
        }
    }
}

fn select_interface<E>(
    geometry: &GeneratedSupportGeometry,
    cached: &mut i32,
    print_z: f64,
    base: bool,
) -> Option<ExtrudedLayer<E>> {
    let (len, source): (_, fn(usize) -> SourceLayer) = if base {
        (geometry.base_interfaces.len(), SourceLayer::BaseInterface)
    } else {
        (geometry.interfaces.len(), SourceLayer::Interface)
    };
    let index = idx_higher_or_equal_indexed(len, cached, |index| {
        source(index).print_z(geometry) >= print_z - 1e-4
    }) as usize;
    (index < len && source(index).print_z(geometry) < print_z + 1e-4)
        .then(|| ExtrudedLayer::new(source(index).canonical(geometry)))
}

fn select_layer<T, E>(
    layers: &[T],
    cached: &mut i32,
    print_z: f64,
    source_z: impl Fn(&T) -> f64,
    source: impl Fn(usize) -> SourceLayer,
) -> Option<ExtrudedLayer<E>> {
    let index =
        idx_higher_or_equal(layers, cached, |layer| source_z(layer) >= print_z - 1e-4) as usize;
    if index < layers.len() && source_z(&layers[index]) < print_z + 1e-4 {
        Some(ExtrudedLayer::new(source(index)))
    } else {
        None
    }
}
