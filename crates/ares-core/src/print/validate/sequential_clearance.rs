//! Print.cpp:613-930: sequential hull and vertical clearance validation.

use crate::{
    ProcessPrintSequence, ProcessTimelapseType, Project, ProjectVolumeType, SliceError,
    geometry::{
        BoundingBox, CoordinateScale, JoinType, Point, Polygon, convex_hull,
        intersection_polygons_paths, offset_paths,
    },
    print::object_skirt::{FirstObject, object_skirt_offset},
    project::effective_config::types::BoundedResolvedProjectConfig,
};

struct Instance<'a> {
    name: &'a str,
    hull: Polygon,
    max_z: f64,
    height: f64,
}

pub(super) fn validate(
    project: &Project,
    resolved: &BoundedResolvedProjectConfig,
    plate_index: u32,
    is_bbl: bool,
) -> Result<(), SliceError> {
    let settings = &resolved.views.full;
    if settings.process.print.print_sequence != ProcessPrintSequence::ByObject {
        return Ok(());
    }
    let count = project
        .objects()
        .iter()
        .map(|o| o.instances().len())
        .sum::<usize>();
    if count < 2 {
        return Ok(());
    }
    if settings.process.print.timelapse_type == ProcessTimelapseType::Smooth {
        return Err(SliceError::Validate {
            message:
                "Smooth mode of timelapse is not supported when \"by object\" sequence is enabled."
                    .to_owned(),
            plate_index,
            layer_height: f64::from(settings.process.object.layer_height.0 as f32),
            wall_loops: settings.process.region.wall_loops.0,
            sparse_infill_density: f64::from(
                settings.process.region.sparse_infill_density.0 as f32,
            ),
        });
    }
    if settings.process.gcode.enable_wrapping_detection.0 {
        return Err(SliceError::Validate {
            message: "Clumping detection is not supported when \"by object\" sequence is enabled."
                .to_owned(),
            plate_index,
            layer_height: f64::from(settings.process.object.layer_height.0 as f32),
            wall_loops: settings.process.region.wall_loops.0,
            sparse_infill_density: f64::from(
                settings.process.region.sparse_infill_density.0 as f32,
            ),
        });
    }
    let scale = CoordinateScale::Normal;
    let instances = instances(project, scale)?;
    let short = instances
        .iter()
        .all(|i| i.height < settings.printer.remaining.nozzle_height.0);
    // `Print.cpp:917`: the print's `object_skirt_offset()`.
    let first = resolved.objects.first().map(|first| FirstObject {
        line_width: first.object.line_width,
        support_filament: first.object.support_filament.0,
    });
    let (skirt, _) = object_skirt_offset(settings, first, short, is_bbl)?;
    let radius = settings.printer.remaining.extruder_clearance_radius.0;
    let distance = if short {
        2.0_f64.max(f64::from(skirt)) - 0.1
    } else {
        0.5 * radius + f64::from(skirt) - 0.1
    };
    let (ox, oy) = project.plate_origin();
    let excluded = Polygon::new(
        settings
            .printer
            .remaining
            .bed_exclude_area
            .0
            .iter()
            .map(|p| point(scale, p.x + ox, p.y + oy))
            .collect::<Result<Vec<_>, _>>()?,
    );
    let mut inflated = Vec::with_capacity(instances.len());
    let mut messages = Vec::new();
    for instance in &instances {
        let hull = offset_paths(
            std::slice::from_ref(&instance.hull),
            scale.scaled_delta(distance) as f32,
            JoinType::Round,
            scale.scaled_delta(0.1),
        )
        .map_err(|_| SliceError::InvalidInput("sequential clearance offset failed".to_owned()))?
        .into_iter()
        .next()
        .unwrap_or_else(|| Polygon::new(Vec::new()));
        if intersects(&excluded, &instance.hull)? {
            messages.push(format!(
                "{} is too close to exclusion area, there may be collisions when printing.",
                instance.name
            ));
        }
        for other in &inflated {
            if intersects(other, &hull)? {
                messages.push(format!(
                    "{} is too close to others, and collisions may be caused.",
                    instance.name
                ));
                break;
            }
        }
        inflated.push(hull);
    }
    let shrink = scale.scaled_delta(0.5 * radius + f64::from(skirt)) as i64;
    for (index, instance) in instances.iter().enumerate() {
        let mut limit = if index + 1 == instances.len() {
            settings.printer.remaining.printable_height.0
        } else {
            settings
                .printer
                .remaining
                .extruder_clearance_height_to_lid
                .0
        };
        if let Some(bounds) = BoundingBox::from_polygon(&inflated[index]) {
            let min = bounds.min().y() + shrink;
            let max = bounds.max().y() - shrink;
            if inflated[index + 1..].iter().any(|later| {
                BoundingBox::from_polygon(later)
                    .is_some_and(|later| max.min(later.max().y()) > min.max(later.min().y()))
            }) {
                limit = settings
                    .printer
                    .remaining
                    .extruder_clearance_height_to_rod
                    .0;
            }
        }
        if limit < instance.max_z {
            messages.push(format!(
                "{} is too tall, and collisions will be caused.",
                instance.name
            ));
        }
    }
    if messages.is_empty() {
        return Ok(());
    }
    Err(SliceError::SequentialPrintCollision {
        message: messages.join("\n"),
        plate_index,
        layer_height: f64::from(settings.process.object.layer_height.0 as f32),
        wall_loops: settings.process.region.wall_loops.0,
        sparse_infill_density: f64::from(settings.process.region.sparse_infill_density.0 as f32),
    })
}

fn instances(project: &Project, scale: CoordinateScale) -> Result<Vec<Instance<'_>>, SliceError> {
    let mut output = Vec::new();
    for (object, instance) in project.objects().iter().flat_map(|object| {
        object
            .instances()
            .iter()
            .map(move |instance| (object, instance))
    }) {
        let mut points = Vec::new();
        let mut min_z = f64::INFINITY;
        let mut max_z = f64::NEG_INFINITY;
        for volume in object
            .volumes()
            .iter()
            .filter(|v| v.volume_type() == ProjectVolumeType::ModelPart)
        {
            let matrix = instance.transform().then(volume.transform());
            for vertex in volume.mesh().vertices() {
                let p = matrix.transform_point(*vertex);
                points.push(point(scale, p.x, p.y)?);
                min_z = min_z.min(p.z);
                max_z = max_z.max(p.z);
            }
        }
        output.push(Instance {
            name: object.name(),
            hull: Polygon::new(convex_hull(&points)),
            max_z,
            height: max_z - min_z,
        });
    }
    Ok(output)
}

fn point(scale: CoordinateScale, x: f64, y: f64) -> Result<Point, SliceError> {
    let x = scale.checked_scale(x).ok_or_else(|| {
        SliceError::InvalidInput("sequential clearance coordinate out of range".to_owned())
    })?;
    let y = scale.checked_scale(y).ok_or_else(|| {
        SliceError::InvalidInput("sequential clearance coordinate out of range".to_owned())
    })?;
    Ok(Point::new(x, y))
}

fn intersects(first: &Polygon, second: &Polygon) -> Result<bool, SliceError> {
    intersection_polygons_paths(std::slice::from_ref(first), std::slice::from_ref(second))
        .map(|paths| !paths.is_empty())
        .map_err(|_| {
            SliceError::InvalidInput("sequential clearance intersection failed".to_owned())
        })
}
