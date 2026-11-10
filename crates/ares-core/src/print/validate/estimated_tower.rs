//! `Print.cpp:1650-1695,1728-1733,5021-5034,5183-5226` at v2.5.0-beta
//! `345efec64`: the estimated tower's pre-generation printable-region rejection.

use crate::{
    Point2d, Point3d, ProcessPrintSequence, Project, ProjectObject, ProjectVolumeType, SliceError,
    Transform3d,
    gcode_processor::CustomGCodeType,
    geometry::{
        CoordinateScale, JoinType, Point, Polygon, difference_polygons_paths,
        intersection_polygons_paths, offset_paths,
    },
    project::effective_config::types::BoundedResolvedProjectConfig,
    project_slice::gcode_emit::wipe_tower_estimate::{
        config::{DynamicPrintConfig, get_at},
        estimate_footprint, resolve_wipe_tower_type,
    },
};

pub(super) fn validate(
    project: &Project,
    resolved: &BoundedResolvedProjectConfig,
    plate_number: u32,
) -> Result<(), SliceError> {
    let settings = &resolved.views.full;
    let print_objects = resolved
        .objects
        .iter()
        .flat_map(|object| &object.print_objects);
    let mut print_objects = print_objects.peekable();
    if settings.process.print.print_sequence == ProcessPrintSequence::ByObject
        && (resolved.print_object_count > 1
            || print_objects
                .peek()
                .is_some_and(|object| object.instance_indices.len() > 1))
    {
        return Ok(());
    }
    let config = DynamicPrintConfig::new(settings)?;
    let has_tower = has_tower(&config);
    let scale = CoordinateScale::from_printable_area(&settings.printer.remaining.printable_area);
    let mut checked = Vec::new();
    if has_tower {
        let slots = extruders(project, resolved, &config);
        let (mut max_height, mut layer_height) = (0.0_f64, f64::MAX);
        for object in &resolved.objects {
            let source = &project.objects()[object.source_object_index];
            max_height = max_height.max(height(source, source.instances()[0].transform(), scale));
            layer_height = layer_height.min(object.object.layer_height.0);
        }
        // `wipe_tower_data`: cache stays zero with no filaments or no object height.
        let (width, depth, brim) = if slots.is_empty() || max_height < 1e-4 {
            (0.0, 0.0, 0.0)
        } else {
            let footprint = estimate_footprint(
                &config,
                resolve_wipe_tower_type(&config),
                &slots,
                layer_height,
                max_height,
            );
            (
                footprint.width as f32,
                footprint.depth as f32,
                footprint.brim_width as f32,
            )
        };
        let index = (plate_number - 1) as usize;
        let (ox, oy) = project.plate_origin();
        let x = (get_at(config.option("wipe_tower_x").floats(), index) + ox) as f32;
        let y = (get_at(config.option("wipe_tower_y").floats(), index) + oy) as f32;
        let anchor = rounded_point(f64::from(x), f64::from(y), scale);
        let angle = config.option("wipe_tower_rotation_angle").float() as f32;
        let angle = f64::from(std::f32::consts::PI * angle / 180.0_f32);
        let (sin, cos) = angle.sin_cos();
        let points = [
            (x, y),
            (x + width, y),
            (x + width, y + depth),
            (x, y + depth),
        ]
        .map(|(x, y)| {
            let point = rounded_point(f64::from(x), f64::from(y), scale);
            let dx = (point.x() - anchor.x()) as f64;
            let dy = (point.y() - anchor.y()) as f64;
            Point::new(
                (anchor.x() as f64 + cos * dx - sin * dy).round() as i64,
                (anchor.y() as f64 + cos * dy + sin * dx).round() as i64,
            )
        });
        checked.push(Polygon::new(points.to_vec()));
        if config.option("prime_tower_brim_width").float() >= 0.0 {
            checked = offset_paths(
                &checked,
                (f64::from(brim) / scale.factor()) as f32,
                JoinType::Miter,
                3.0,
            )
            .map_err(|error| SliceError::InvalidInput(format!("{error:?}")))?;
        }
    }
    let mut printable = printable(resolved, scale)?;
    let (ox, oy) = project.plate_origin();
    let shift = rounded_point(ox, oy, scale);
    for polygon in &mut printable {
        *polygon = Polygon::new(
            polygon
                .points()
                .iter()
                .map(|point| Point::new(point.x() + shift.x(), point.y() + shift.y()))
                .collect(),
        );
    }
    if difference_polygons_paths(&checked, &printable)
        .map_err(|error| SliceError::InvalidInput(format!("{error:?}")))?
        .is_empty()
    {
        return Ok(());
    }
    Err(SliceError::LayeredPrintCollision {
        message: "Prime tower is partially outside the printable area, and it cannot be printed."
            .to_owned(),
        plate_index: plate_number,
        layer_height: f64::from(settings.process.object.layer_height.0 as f32),
        wall_loops: settings.process.region.wall_loops.0,
        sparse_infill_density: f64::from(settings.process.region.sparse_infill_density.0 as f32),
    })
}

fn has_tower(config: &DynamicPrintConfig) -> bool {
    !config.option("belt_printer").bool()
        && config.option("enable_prime_tower").bool()
        && ((config.option("enable_wrapping_detection").bool()
            && config.option("wrapping_exclude_area").points().len() > 2)
            || config.option("timelapse_type").key() == "1"
            || (!config.option("spiral_mode").bool()
                && config.option("filament_diameter").floats().len() > 1))
}

/// `Print.cpp:686-803`, `PrintRegion.cpp:79-101`: ordered aggregation before final deduplication.
fn extruders(
    project: &Project,
    resolved: &BoundedResolvedProjectConfig,
    config: &DynamicPrintConfig,
) -> Vec<u32> {
    let count = config.option("filament_diameter").floats().len() as u32;
    let bounded = |index| if index >= count { 0 } else { index };
    let has_brim = resolved.objects.iter().any(|object| {
        let object = &object.object;
        object.raft_layers.0 <= 0
            && (object.brim_type == crate::ProcessBrimType::AutoBrim
                || (object.brim_type != crate::ProcessBrimType::NoBrim
                    && object.brim_width.0 > 0.0))
    });
    let mut slots = Vec::new();
    for region in resolved
        .objects
        .iter()
        .flat_map(|object| &object.all_regions)
    {
        for (included, selector) in [
            (
                region.wall_loops.0 > 0 || has_brim,
                region.outer_wall_filament_id.0,
            ),
            (region.wall_loops.0 > 1, region.inner_wall_filament_id.0),
            (
                region.sparse_infill_density.0 > 0.0,
                region.sparse_infill_filament_id.0,
            ),
            (
                region.sparse_infill_density.0 > 0.0
                    || region.top_shell_layers.0 > 0
                    || region.bottom_shell_layers.0 > 0,
                region.internal_solid_filament_id.0,
            ),
            (
                region.top_shell_layers.0 > 0,
                region.top_surface_filament_id.0,
            ),
            (
                region.bottom_shell_layers.0 > 0,
                region.bottom_surface_filament_id.0,
            ),
        ] {
            if included {
                let index = (selector - 1).max(0) as u32;
                slots.push(bounded(index));
            }
        }
    }
    for object in &resolved.objects {
        let source = &project.objects()[object.source_object_index];
        for volume in source.volumes().iter().filter(|volume| {
            matches!(
                volume.volume_type(),
                ProjectVolumeType::ModelPart | ProjectVolumeType::ParameterModifier
            )
        }) {
            let selector = volume
                .region_overrides()
                .extruder
                .filter(|value| value.0 != 0)
                .or(source.region_overrides().extruder)
                .map_or(1, |value| value.0);
            if selector > 0 {
                slots.push((selector - 1) as u32);
            }
        }
        for range in source.layer_config_ranges() {
            if let Some(selector) = range
                .region_overrides()
                .extruder
                .filter(|value| value.0 > 0)
            {
                slots.push((selector.0 - 1) as u32);
            }
        }
    }
    slots.sort_unstable();
    slots.dedup();
    let mut support = Vec::new();
    let mut uses_current = false;
    for object in resolved
        .objects
        .iter()
        .map(|object| &object.object)
        .filter(|object| {
            object.enable_support.0
                || object.enforce_support_layers.0 > 0
                || object.raft_layers.0 > 0
        })
    {
        for selector in [
            object.support_filament.0,
            object.support_interface_filament.0,
        ] {
            if selector == 0 {
                uses_current = true;
            } else {
                let index = (selector - 1) as u32;
                support.push(bounded(index));
            }
        }
    }
    if uses_current {
        support.extend_from_slice(&slots);
    }
    support.sort_unstable();
    support.dedup();
    slots.extend(support);
    let colour_count = config.option("filament_colour").strings().len() as u32;
    for item in project.curr_plate_custom_gcodes().gcodes {
        if item.kind == CustomGCodeType::ToolChange && item.extruder <= colour_count {
            slots.push(item.extruder.wrapping_sub(1));
        }
    }
    let selected = config.option("wipe_tower_filament").int();
    if selected != 0 && slots.len() > 1 {
        slots.push((selected - 1) as u32);
    }
    slots.sort_unstable();
    slots.dedup();
    slots
}

/// `PrintObject.cpp:161`: the snug transformed model-part extent, scaled by
/// multiplication and truncated into `m_size`, then unscaled by the estimate producer.
fn height(object: &ProjectObject, transform: Transform3d, scale: CoordinateScale) -> f64 {
    let transform = transform.without_translation();
    let (min, max) = object
        .volumes()
        .iter()
        .filter(|volume| volume.volume_type() == ProjectVolumeType::ModelPart)
        .flat_map(|volume| {
            let matrix = transform.then(volume.transform());
            volume.mesh().vertices().iter().map(move |vertex| {
                matrix
                    .transform_point(Point3d::new(
                        f64::from(vertex.x as f32),
                        f64::from(vertex.y as f32),
                        f64::from(vertex.z as f32),
                    ))
                    .z
            })
        })
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(min, max), z| {
            (min.min(z), max.max(z))
        });
    let size = ((max - min) * (1.0 / scale.factor())) as i64;
    size as f64 * scale.factor()
}

fn rounded_point(x: f64, y: f64, scale: CoordinateScale) -> Point {
    Point::new(
        (x / scale.factor()).round() as i64,
        (y / scale.factor()).round() as i64,
    )
}

fn printable(
    resolved: &BoundedResolvedProjectConfig,
    scale: CoordinateScale,
) -> Result<Vec<Polygon>, SliceError> {
    let settings = &resolved.views.full;
    let areas = &settings.printer.remaining.extruder_printable_area.0;
    // `Polygon::new_scale` truncates, unlike `Point(double, double)` above.
    let polygon = |area: &[Point2d]| {
        Polygon::new(
            area.iter()
                .map(|point| {
                    Point::new(
                        (point.x / scale.factor()) as i64,
                        (point.y / scale.factor()) as i64,
                    )
                })
                .collect(),
        )
    };
    if settings.project.print.nozzle_diameter.0.len() < 2
        || areas.is_empty()
        || areas.iter().any(|area| area.len() < 3)
    {
        return Ok(vec![polygon(&settings.printer.remaining.printable_area.0)]);
    }
    let mut shared = vec![polygon(&areas[0])];
    for area in &areas[1..] {
        shared = intersection_polygons_paths(&shared, &[polygon(area)])
            .map_err(|error| SliceError::InvalidInput(format!("{error:?}")))?;
    }
    Ok(shared)
}
