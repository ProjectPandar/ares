//! `SupportCommon.cpp::generate_support_layers` and `SupportLayer.hpp::operator<`.

use super::{
    adhesion::RaftLayerType, extrusion_path::SupportExtrusionEntity,
    grid_generation::GeneratedSupportGeometry, intermediate_heights::SupportLayerType,
};
use crate::geometry::ExPolygon;

const EPSILON: f64 = 1e-4;

pub(in crate::project_slice) struct InstalledSupportLayer {
    pub(in crate::project_slice) id: usize,
    /// Upstream `SupportInnerType` fact: only the classic-tree layer
    /// factory (`add_tree_support_layer`, PrintObject.cpp:1073-1077)
    /// sets stInnerTree; every other factory (`add_support_layer`,
    /// PrintObject.cpp:1080-1083) leaves the Layer.hpp:288 default
    /// stInnerNormal.
    pub(in crate::project_slice) inner_tree: bool,
    pub(super) interface_id: usize,
    pub(in crate::project_slice) print_z: f64,
    pub(in crate::project_slice) height: f64,
    pub(in crate::project_slice) slices: Vec<ExPolygon>,
    pub(in crate::project_slice) support_islands: Vec<ExPolygon>,
    pub(in crate::project_slice) tree_base_areas: Option<Vec<ExPolygon>>,
    pub(in crate::project_slice) support_fills: Vec<SupportExtrusionEntity>,
}

impl InstalledSupportLayer {
    /// `Print::_make_wipe_tower`: synthetic support layer with source ID -1.
    pub(in crate::project_slice) fn for_wipe_tower(print_z: f64, height: f64) -> Self {
        Self {
            id: usize::MAX,
            inner_tree: false,
            interface_id: 0,
            print_z,
            height,
            slices: Vec::new(),
            support_islands: Vec::new(),
            tree_base_areas: None,
            support_fills: Vec::new(),
        }
    }
}

struct LayerView<'a> {
    print_z: &'a mut f64,
    height: f64,
    bottom_z: f64,
    bridging: bool,
    nonempty: bool,
    interface: bool,
    top_contact: bool,
}

pub(super) fn generate_support_layers(
    geometry: &mut GeneratedSupportGeometry,
) -> Vec<InstalledSupportLayer> {
    let mut sorted = Vec::with_capacity(
        geometry.rafts.len()
            + geometry.bottoms.len()
            + geometry.tops.len()
            + geometry.bases.len()
            + geometry.interfaces.len()
            + geometry.base_interfaces.len(),
    );
    for layer in &mut geometry.rafts {
        sorted.push(LayerView {
            print_z: &mut layer.print_z,
            height: layer.height,
            bottom_z: layer.bottom_z,
            bridging: false,
            nonempty: !layer.polygons.is_empty(),
            interface: matches!(layer.layer_type, RaftLayerType::Interface),
            top_contact: false,
        });
    }
    for layer in &mut geometry.bottoms {
        sorted.push(LayerView {
            print_z: &mut layer.print_z,
            height: layer.height,
            bottom_z: layer.bottom_z,
            bridging: layer.bridging,
            nonempty: !layer.polygons.is_empty(),
            interface: true,
            top_contact: false,
        });
    }
    for layer in &mut geometry.tops {
        sorted.push(LayerView {
            print_z: &mut layer.print_z,
            height: layer.height,
            bottom_z: layer.bottom_z,
            bridging: false,
            nonempty: !layer.polygons.is_empty(),
            interface: true,
            top_contact: true,
        });
    }
    for (is_base_interface, is_interface, layers) in [
        (false, false, &mut geometry.bases),
        (false, true, &mut geometry.interfaces),
        (true, true, &mut geometry.base_interfaces),
    ] {
        for (index, layer) in layers.iter_mut().enumerate() {
            // Native removes repeated pointer identities before Z grouping.
            if is_interface && geometry.ownership.interface(index, is_base_interface) != index {
                continue;
            }
            sorted.push(LayerView {
                print_z: &mut layer.print_z,
                height: layer.height,
                bottom_z: layer.bottom_z,
                bridging: false,
                nonempty: !layer.polygons.is_empty(),
                interface: is_base_interface
                    || matches!(
                        layer.layer_type,
                        SupportLayerType::TopInterface | SupportLayerType::BottomInterface
                    ),
                top_contact: false,
            });
        }
    }
    // Canonical source identities appear once, matching native pointer unique.
    sorted.sort_unstable_by(|left, right| {
        left.print_z
            .total_cmp(right.print_z)
            .then_with(|| right.height.total_cmp(&left.height))
            .then_with(|| right.bridging.cmp(&left.bridging))
    });
    install_groups(&mut sorted)
}

fn install_groups(sorted: &mut [LayerView<'_>]) -> Vec<InstalledSupportLayer> {
    let mut layers: Vec<InstalledSupportLayer> = Vec::new();
    let mut interface_id = 0;
    let mut start = 0;
    while start < sorted.len() {
        let zmax = *sorted[start].print_z + EPSILON;
        let mut end = start + 1;
        while end < sorted.len() && *sorted[end].print_z <= zmax {
            end += 1;
        }
        let print_z = 0.5 * (*sorted[start].print_z + *sorted[end - 1].print_z);
        let mut height = sorted[start].height;
        let mut nonempty = false;
        let mut interfaces = 0;
        let mut top_contacts = 0;
        let mut top_bottom_z = 0.0;
        for layer in &mut sorted[start..end] {
            nonempty |= layer.nonempty;
            interfaces += usize::from(layer.nonempty && layer.interface);
            if layer.nonempty && layer.top_contact {
                top_contacts += 1;
                top_bottom_z = layer.bottom_z;
            }
            *layer.print_z = print_z;
            height = height.min(layer.height);
        }
        if nonempty {
            let contacts_only = top_contacts > 0 && top_contacts == interfaces;
            let mut this_interface_id = interface_id;
            // Native's reverse scan has no break: the oldest match wins.
            if contacts_only
                && let Some(layer) = layers
                    .iter()
                    .find(|layer| (layer.print_z - top_bottom_z).abs() < EPSILON)
            {
                this_interface_id = layer.interface_id + 1;
            }
            layers.push(InstalledSupportLayer {
                id: layers.len(),
                inner_tree: false,
                interface_id: this_interface_id,
                print_z,
                height,
                slices: Vec::new(),
                support_islands: Vec::new(),
                tree_base_areas: None,
                support_fills: Vec::new(),
            });
            if interfaces > 0 && !contacts_only {
                interface_id += 1;
            }
        }
        start = end;
    }
    layers
}
