//! `TreeSupportCommon.hpp::InterfacePlacer` and its layer allocator prefix.

use super::TreeSettings;
use crate::geometry::Polygon;
use crate::project_slice::{
    parameters::SlicingParameters,
    support_material::{
        interface_counts::InterfaceCounts, toolpath_parameters::ToolpathParameters,
    },
};

pub(super) enum LayerType {
    TopContact,
    TopInterface,
    BottomInterface,
    BottomContact,
    Base,
}

pub(super) struct GeneratorLayer {
    pub(super) kind: LayerType,
    pub(super) print_z: f64,
    pub(super) bottom_z: f64,
    pub(super) height: f64,
    pub(super) polygons: Vec<Polygon>,
}

pub(super) struct RoofLayers {
    pub(super) storage: Vec<GeneratorLayer>,
    pub(super) top_contacts: Vec<Option<usize>>,
    pub(super) top_interfaces: Vec<Option<usize>>,
    pub(super) top_base_interfaces: Vec<Option<usize>>,
    pub(super) bottom_contacts: Vec<Option<usize>>,
    pub(super) intermediate_layers: Vec<Option<usize>>,
}

impl RoofLayers {
    pub(super) fn new(layer_count: usize) -> Self {
        Self {
            storage: Vec::new(),
            top_contacts: vec![None; layer_count],
            top_interfaces: vec![None; layer_count],
            top_base_interfaces: vec![None; layer_count],
            bottom_contacts: vec![None; layer_count],
            intermediate_layers: vec![None; layer_count],
        }
    }
}

pub(super) struct InterfacePlacer<'a> {
    pub(super) slicing: &'a SlicingParameters,
    pub(super) parameters: &'a ToolpathParameters,
    pub(super) config: &'a TreeSettings,
    pub(super) counts: &'a InterfaceCounts,
    pub(super) roofs: &'a mut RoofLayers,
}

impl InterfacePlacer<'_> {
    pub(super) fn add_roofs(&mut self, roofs: Vec<Vec<Polygon>>, insert_layer: usize) {
        for (depth, roof) in roofs.into_iter().enumerate() {
            if !roof.is_empty() {
                self.add_roof(roof, insert_layer - depth, depth);
            }
        }
    }

    pub(super) fn add_roof_build_plate(&mut self, polygons: Vec<Polygon>, depth: usize) {
        self.add_roof(polygons, 0, depth.min(self.counts.top));
    }

    pub(super) fn add_roof(&mut self, polygons: Vec<Polygon>, layer: usize, depth: usize) {
        let mut interface_threshold = self.counts.top - self.counts.top_base;
        if interface_threshold > 0 && self.counts.top_base > 0 {
            interface_threshold -= 1;
        }
        let slots = if depth == 0 {
            &mut self.roofs.top_contacts
        } else if depth <= interface_threshold {
            &mut self.roofs.top_interfaces
        } else {
            &mut self.roofs.top_base_interfaces
        };
        let index = *slots[layer].get_or_insert_with(|| {
            let print_z = layer_z(self.slicing, self.config, layer);
            let bottom_z = if layer > 0 {
                layer_z(self.slicing, self.config, layer - 1)
            } else {
                0.0
            };
            let index = self.roofs.storage.len();
            self.roofs.storage.push(GeneratorLayer {
                kind: if depth == 0 {
                    LayerType::TopContact
                } else {
                    LayerType::TopInterface
                },
                print_z,
                bottom_z,
                height: print_z - bottom_z,
                polygons: Vec::new(),
            });
            index
        });
        self.roofs.storage[index].polygons.extend(polygons);
    }
}

pub(super) fn layer_z(slicing: &SlicingParameters, config: &TreeSettings, layer: usize) -> f64 {
    if layer >= config.raft_layers.len() {
        slicing.object_print_z_min
            + slicing.first_object_layer_height
            + (layer - config.raft_layers.len()) as f64 * slicing.layer_height
    } else {
        config.raft_layers[layer]
    }
}
