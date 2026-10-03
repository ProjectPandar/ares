//! Organic generator layer storage into the shared `SupportCommon` consumers.

use super::{
    interface_placer::{GeneratorLayer, LayerType, RoofLayers},
    interfaces::indices,
};
use crate::{
    geometry::{ClipperError, Polygon, intersection_polygons_paths},
    project_slice::support_material::{
        bottom_contacts::BottomContactLayer,
        contact_layer::TopContactLayer,
        grid_generation::{GeneratedSupportGeometry, LayerOwnership},
        intermediate_heights::{IntermediateLayer, SupportLayerType},
    },
};

pub(super) fn geometry(roofs: &mut RoofLayers) -> GeneratedSupportGeometry {
    let (interfaces, interface_owners) = intermediate(&mut roofs.storage, &roofs.top_interfaces);
    let (base_interfaces, base_interface_owners) =
        intermediate(&mut roofs.storage, &roofs.top_base_interfaces);
    let (bases, _) = intermediate(&mut roofs.storage, &roofs.intermediate_layers);
    GeneratedSupportGeometry {
        ownership: LayerOwnership::Organic {
            interfaces: interface_owners,
            base_interfaces: base_interface_owners,
        },
        support_layers: Vec::new(),
        rafts: Vec::new(),
        sharp_tails: Vec::new(),
        tops: indices(&roofs.top_contacts)
            .into_iter()
            .map(|index| {
                let layer = &mut roofs.storage[index];
                TopContactLayer {
                    print_z: layer.print_z,
                    bottom_z: layer.bottom_z,
                    height: layer.height,
                    polygons: std::mem::take(&mut layer.polygons),
                    ..TopContactLayer::default()
                }
            })
            .collect(),
        bottoms: indices(&roofs.bottom_contacts)
            .into_iter()
            .map(|index| {
                let layer = &mut roofs.storage[index];
                BottomContactLayer {
                    print_z: layer.print_z,
                    bottom_z: layer.bottom_z,
                    height: layer.height,
                    polygons: std::mem::take(&mut layer.polygons),
                    bridging: false,
                }
            })
            .collect(),
        bases,
        interfaces,
        base_interfaces,
    }
}

fn intermediate(
    storage: &mut [GeneratorLayer],
    slots: &[Option<usize>],
) -> (Vec<IntermediateLayer>, Vec<usize>) {
    let mut identities = std::collections::HashMap::new();
    let mut owners = Vec::new();
    let layers = indices(slots)
        .into_iter()
        .enumerate()
        .map(|(slot, index)| {
            let owner = *identities.entry(index).or_insert(slot);
            owners.push(owner);
            let GeneratorLayer {
                kind,
                print_z,
                bottom_z,
                height,
                polygons,
            } = &mut storage[index];
            IntermediateLayer {
                print_z: *print_z,
                bottom_z: *bottom_z,
                height: *height,
                polygons: if owner == slot {
                    std::mem::take(polygons)
                } else {
                    Vec::new()
                },
                layer_type: match kind {
                    LayerType::Base => SupportLayerType::Base,
                    LayerType::TopInterface => SupportLayerType::TopInterface,
                    LayerType::BottomInterface => SupportLayerType::BottomInterface,
                    LayerType::TopContact | LayerType::BottomContact => {
                        unreachable!("contact slots have separate source types")
                    }
                },
            }
        })
        .collect();
    (layers, owners)
}

pub(super) fn clip_to_bed(
    geometry: &mut GeneratedSupportGeometry,
    bed: &[Polygon],
) -> Result<(), ClipperError> {
    for layer in &mut geometry.rafts {
        layer.polygons = intersection_polygons_paths(&layer.polygons, bed)?;
    }
    for layer in &mut geometry.tops {
        layer.polygons = intersection_polygons_paths(&layer.polygons, bed)?;
    }
    for layer in &mut geometry.bottoms {
        layer.polygons = intersection_polygons_paths(&layer.polygons, bed)?;
    }
    for layers in [
        &mut geometry.bases,
        &mut geometry.interfaces,
        &mut geometry.base_interfaces,
    ] {
        for layer in layers {
            layer.polygons = intersection_polygons_paths(&layer.polygons, bed)?;
        }
    }
    Ok(())
}
