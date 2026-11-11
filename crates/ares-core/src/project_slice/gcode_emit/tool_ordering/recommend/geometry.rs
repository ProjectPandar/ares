use crate::SliceError;
use crate::geometry::{Point, Polygon, difference_ex, difference_polygons_ex, intersection_ex};
use crate::project_slice::gcode_emit::wipe_tower::estimate::config::DynamicPrintConfig;
use crate::project_slice::island_print_order::OrderedExtrusionLayer;
use crate::project_slice::prepare_infill::surface_type_detection::PreparedSurfaceTypeObject;
use std::collections::BTreeSet;

pub(super) fn geometric_unprintables(
    retained: &[PreparedSurfaceTypeObject],
    traversal: &crate::project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal,
    ordered: &[Vec<OrderedExtrusionLayer>],
    config: &DynamicPrintConfig,
) -> Result<Vec<BTreeSet<usize>>, SliceError> {
    let settings = &traversal.resolved.views.full;
    let count = settings.project.print.nozzle_diameter.0.len();
    let mut result = vec![BTreeSet::new(); count];
    if count == 1 {
        return Ok(result);
    }
    let origin = traversal.project.plate_origin();
    let plate = Point::new(
        (origin.0 / 1e-6).round() as i64,
        (origin.1 / 1e-6).round() as i64,
    );
    let mut exclusions = crate::bed_exclusion::get_bed_excluded_regions_by_extruder(config);
    exclusions.resize_with(count, Vec::new);
    for region in exclusions.iter_mut().flatten() {
        region.polygon = translated(&region.polygon, plate);
    }
    let printer = &settings.printer.remaining;
    let heights = config.option("extruder_printable_height");
    let polygon = |points: &[crate::Point2d]| {
        Polygon::new(
            points
                .iter()
                .map(|p| Point::new((p.x / 1e-6).round() as i64, (p.y / 1e-6).round() as i64))
                .collect(),
        )
    };
    let bed = polygon(&printer.printable_area.0);
    let areas = printer
        .extruder_printable_area
        .0
        .iter()
        .map(|points| difference_polygons_ex(std::slice::from_ref(&bed), &[polygon(points)]))
        .collect::<Result<Vec<_>, _>>()
        .map_err(geometry_error)?;
    let tools = crate::project_slice::extruders::collect_project_object_extruders(
        traversal.project.objects(),
        &traversal.resolved.objects,
        traversal.resolved.logical_filament_count,
    );
    for (index, prepared) in traversal.objects.iter().enumerate() {
        let input = &prepared
            .predecessor
            .predecessor
            .predecessor
            .predecessor
            .object;
        let (source_index, _) = input.identity();
        let source = &traversal.project.objects()[source_index];
        let (resolved, print_object) = traversal.print_object_config(index);
        let resolved_index = traversal
            .resolved
            .objects
            .iter()
            .position(|candidate| candidate.source_object_index == source_index)
            .unwrap();
        let mut printing = tools[resolved_index].clone();
        let object = &resolved.object;
        if object.enable_support.0
            || object.enforce_support_layers.0 > 0
            || object.raft_layers.0 > 0
        {
            printing.extend(
                [
                    object.support_filament.0,
                    object.support_interface_filament.0,
                ]
                .into_iter()
                .filter(|id| *id > 0)
                .map(|id| (id as usize - 1).min(traversal.resolved.logical_filament_count - 1)),
            );
        }
        if printing.is_empty() {
            printing.push(0);
        }
        for (head, regions) in exclusions.iter().enumerate() {
            let collision = exclusion_collision(source, &print_object.instance_indices, regions);
            if collision {
                result[head].extend(printing.iter().copied());
            }
        }
        let center =
            crate::project_slice::centering::center(source, print_object, traversal.scale)?;
        let shift = crate::project_slice::gcode_emit::footprint::instance_shift(
            &source.instances()[print_object.instance_indices[0]],
            center,
            traversal.scale,
        );
        // PrintInstance::shift_without_plate_offset is the scaled instance shift minus plate origin.
        let delta = Point::new(plate.x() - shift.x(), plate.y() - shift.y());
        let local_areas = areas
            .iter()
            .map(|area| {
                area.iter()
                    .map(|p| {
                        crate::geometry::ExPolygon::new(
                            translated(p.contour(), delta),
                            p.holes().iter().map(|h| translated(h, delta)).collect(),
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let mut area_limits = vec![BTreeSet::new(); count];
        let layers = prepared.physical_layers();
        let (_, _, regions) = input.as_parts().0.as_parts().0.as_parts();
        for (slot, record) in retained[index].records.iter().enumerate() {
            let region_index = slot / layers.len();
            let layer_index = slot % layers.len();
            let options = &regions[region_index].options;
            let fill_ids = positive_ids([
                options.internal_solid_filament_id.0,
                options.top_surface_filament_id.0,
                options.bottom_surface_filament_id.0,
                options.sparse_infill_filament_id.0,
            ]);
            let wall_ids = positive_ids([
                options.outer_wall_filament_id.0,
                options.inner_wall_filament_id.0,
            ]);
            let current = &ordered[index][layer_index];
            let fills = current
                .islands
                .iter()
                .any(|island| island.infill_regions.contains(&region_index));
            let walls = current
                .islands
                .iter()
                .any(|island| island.perimeter_regions.contains(&region_index));
            for (_, limits) in result
                .iter_mut()
                .enumerate()
                .filter(|(head, _)| layers[layer_index].print_z > heights.floats()[*head])
            {
                limits.extend(
                    fill_ids
                        .iter()
                        .filter(|_| fills)
                        .chain(wall_ids.iter().filter(|_| walls))
                        .copied(),
                );
            }
            let Some(record) = record else {
                continue;
            };
            area_restrictions(
                (
                    &record.fill_expolygons,
                    input.raw_slices(region_index, layer_index),
                ),
                (&fill_ids, &wall_ids),
                (fills, walls),
                (&local_areas, &mut area_limits),
            )?;
        }
        for (limits, area) in result.iter_mut().zip(area_limits) {
            limits.extend(area);
        }
    }
    Ok(result)
}

fn exclusion_collision(
    source: &crate::ProjectObject,
    indices: &[usize],
    regions: &[crate::bed_exclusion::BedExcludeRegion],
) -> bool {
    indices.iter().any(|&instance| {
        source
            .volumes()
            .iter()
            .filter(|volume| volume.volume_type() == crate::ProjectVolumeType::ModelPart)
            .any(|volume| {
                let mesh = volume.mesh();
                let matrix = source.instances()[instance]
                    .transform()
                    .then(volume.transform());
                regions.iter().any(|region| {
                    crate::bed_exclusion::model_part_intersects_region(
                        mesh.vertices(),
                        mesh.triangles(),
                        matrix,
                        region,
                    )
                })
            })
    })
}

fn area_restrictions(
    geometry: (&[crate::geometry::ExPolygon], &[crate::geometry::ExPolygon]),
    families: (&[usize], &[usize]),
    presence: (bool, bool),
    data: (&[Vec<crate::geometry::ExPolygon>], &mut [BTreeSet<usize>]),
) -> Result<(), SliceError> {
    let (fill_area, raw) = geometry;
    let (fill_ids, wall_ids) = families;
    let (fills, walls) = presence;
    let (local_areas, area_limits) = data;
    let mut wall_area = None;
    for (head, area) in local_areas.iter().enumerate() {
        let limits = &mut area_limits[head];
        let mut infill_unprintable = fills && fill_ids.iter().any(|id| limits.contains(id));
        if fills
            && fill_ids.iter().any(|id| !limits.contains(id))
            && !intersection_ex(fill_area, area)
                .map_err(geometry_error)?
                .is_empty()
        {
            limits.extend(fill_ids.iter().copied());
            infill_unprintable = true;
        }
        if walls && wall_ids.iter().any(|id| !limits.contains(id)) {
            if infill_unprintable {
                limits.extend(wall_ids.iter().copied());
                continue;
            }
            if wall_area.is_none() {
                wall_area = Some(difference_ex(raw, fill_area).map_err(geometry_error)?);
            }
            if !intersection_ex(wall_area.as_ref().unwrap(), area)
                .map_err(geometry_error)?
                .is_empty()
            {
                limits.extend(wall_ids.iter().copied());
            }
        }
    }
    Ok(())
}

fn translated(polygon: &Polygon, delta: Point) -> Polygon {
    Polygon::new(
        polygon
            .points()
            .iter()
            .map(|p| Point::new(p.x() + delta.x(), p.y() + delta.y()))
            .collect(),
    )
}

fn positive_ids<const N: usize>(ids: [i32; N]) -> Vec<usize> {
    ids.into_iter()
        .filter(|id| *id > 0)
        .map(|id| id as usize - 1)
        .collect()
}

fn geometry_error(error: crate::geometry::ClipperError) -> SliceError {
    SliceError::InvalidInput(format!("geometric filament restrictions: {error:?}"))
}
