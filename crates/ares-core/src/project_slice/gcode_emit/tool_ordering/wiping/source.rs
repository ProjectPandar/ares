//! Original collection/copy views for `ToolOrdering.cpp:1585–1755`.
use super::{Collection, CollectionId, Object, Region};
use crate::project_slice::{
    extrusion_islands::PreparedPostExtrusionIslands,
    perimeters::classic::traversal::PreparedPostClassicTraversal,
    support_material::{InstalledSupportLayer, SupportExtrusionEntity, SupportExtrusionPath},
};
use crate::{ExtrusionRole, RegionOptions, geometry::CoordinateScale};

pub(in crate::project_slice::gcode_emit) fn objects(
    traversal: &PreparedPostClassicTraversal,
    original: &PreparedPostExtrusionIslands,
    support: &[Vec<InstalledSupportLayer>],
    print_z: f64,
) -> Vec<Object> {
    traversal
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, prepared)| {
            let prelude = &prepared
                .predecessor
                .predecessor
                .predecessor
                .predecessor
                .object;
            let (source, transform) = prelude.identity();
            let resolved = traversal
                .resolved
                .objects
                .iter()
                .find(|object| object.source_object_index == source)
                .expect("print object retains resolved owner");
            let options = &resolved.object;
            if !options.flush_into_objects.0
                && !options.flush_into_infill.0
                && !options.flush_into_support.0
            {
                return None;
            }
            let regions = prelude.as_parts().0.as_parts().0.as_parts().2;
            let layers = prepared.physical_layers();
            let index = layers.partition_point(|layer| layer.print_z < print_z - 1e-4);
            let regions = layers
                .get(index)
                .filter(|layer| (layer.print_z - print_z).abs() < 1e-4)
                .map(|_| {
                    let collections = &original.objects[id][index].wiping_volumes;
                    let mut output = regions
                        .iter()
                        .map(|input| {
                            let config = &input.options;
                            Region {
                                infill_first: config.is_infill_first.0,
                                wall_tool: (config.outer_wall_filament_id.0.max(1) - 1) as usize,
                                sparse_tool: (config.sparse_infill_filament_id.0.max(1) - 1)
                                    as usize,
                                infills: Vec::new(),
                                perimeters: Vec::new(),
                            }
                        })
                        .collect::<Vec<_>>();
                    for entry in &collections.infills {
                        output[entry.region].infills.push(Collection {
                            id: CollectionId::Infill(entry.id),
                            tool: tool(
                                entry.role,
                                entry.infill,
                                entry.solid,
                                &regions[entry.region].options,
                            ),
                            role: entry.role,
                            volume: entry.volume,
                        });
                    }
                    for entry in &collections.perimeters {
                        output[entry.region].perimeters.push(Collection {
                            id: CollectionId::Perimeter(entry.id),
                            tool: tool(
                                entry.role,
                                entry.infill,
                                entry.solid,
                                &regions[entry.region].options,
                            ),
                            role: entry.role,
                            volume: entry.volume,
                        });
                    }
                    output
                });
            let support = &support[id];
            let index = support.partition_point(|layer| layer.print_z < print_z - 1e-4);
            let support = support
                .get(index)
                .filter(|_| options.flush_into_support.0)
                .filter(|layer| (layer.print_z - print_z).abs() < 1e-4)
                .map(|layer| {
                    layer
                        .support_fills
                        .iter()
                        .map(|entity| (entity.role(), support_volume(entity, traversal.scale)))
                        .collect()
                });
            Some(Object {
                id,
                copies: resolved.print_objects[transform].instance_indices.len(),
                flush_objects: options.flush_into_objects.0,
                flush_infill: options.flush_into_infill.0,
                flush_support: options.flush_into_support.0,
                support_tools: [
                    options.support_filament.0 as usize,
                    options.support_interface_filament.0 as usize,
                ],
                support_interface_not_for_body: options.support_interface_not_for_body.0,
                regions,
                support,
            })
        })
        .collect()
}

fn tool(role: ExtrusionRole, infill: bool, solid: bool, region: &RegionOptions) -> usize {
    let id = if infill {
        if !solid {
            region.sparse_infill_filament_id.0
        } else if matches!(role, ExtrusionRole::TopSolidInfill | ExtrusionRole::Ironing) {
            region.top_surface_filament_id.0
        } else if role == ExtrusionRole::BottomSurface {
            region.bottom_surface_filament_id.0
        } else {
            region.internal_solid_filament_id.0
        }
    } else if role == ExtrusionRole::Perimeter {
        region.inner_wall_filament_id.0
    } else {
        region.outer_wall_filament_id.0
    };
    (id.max(1) - 1) as usize
}

fn support_volume(entity: &SupportExtrusionEntity, scale: CoordinateScale) -> f64 {
    match entity {
        SupportExtrusionEntity::Path(path) | SupportExtrusionEntity::Loop(path) => {
            support_path_volume(path, scale)
        }
        SupportExtrusionEntity::MultiPath(paths) => paths
            .iter()
            .map(|path| support_path_volume(path, scale))
            .sum(),
        SupportExtrusionEntity::NoSort(entities) => entities
            .iter()
            .map(|entity| support_volume(entity, scale))
            .sum(),
    }
}

fn support_path_volume(path: &SupportExtrusionPath, scale: CoordinateScale) -> f64 {
    let length: f64 = path
        .polyline
        .points
        .windows(2)
        .map(|pair| {
            let dx = (pair[1].x - pair[0].x) as f64;
            let dy = (pair[1].y - pair[0].y) as f64;
            let dz = (pair[1].z - pair[0].z) as f64;
            (dx * dx + dy * dy + dz * dz).sqrt()
        })
        .sum();
    path.mm3_per_mm * (length * scale.factor())
}
