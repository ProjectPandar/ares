//! Print.cpp:932-979,1304-1310: by-layer wrapping-region hull admission.
use crate::{
    Point3d, ProcessPrintSequence, Project, ProjectInstance, ProjectObject, ProjectVolumeType,
    SliceError,
    geometry::{CoordinateScale, Point, Polygon, convex_hull, intersection_polygons_paths},
    project::effective_config::types::BoundedResolvedProjectConfig,
};

pub(super) fn validate(
    project: &Project,
    resolved: &BoundedResolvedProjectConfig,
    plate_index: u32,
) -> Result<(), SliceError> {
    let settings = &resolved.views.full;
    if settings.process.print.print_sequence != ProcessPrintSequence::ByLayer
        || !settings.process.gcode.enable_wrapping_detection.0
    {
        return Ok(());
    }
    let scale = CoordinateScale::from_printable_area(&settings.printer.remaining.printable_area);
    let origin = project.plate_origin();
    let area = Polygon::new(
        settings
            .printer
            .gcode
            .wrapping_exclude_area
            .0
            .iter()
            .map(|p| point(scale, p.x + origin.0, p.y + origin.1))
            .collect::<Result<Vec<_>, _>>()?,
    );
    for object in project.objects() {
        for instance in object
            .instances()
            .iter()
            .filter(|instance| instance.printable())
        {
            if intersects(object, instance, &area, scale)? {
                return Err(SliceError::LayeredPrintCollision {
                    message: format!(
                        "{} is too close to clumping detection area, there may be collisions when printing.\n",
                        object.name()
                    ),
                    plate_index,
                    layer_height: f64::from(settings.process.object.layer_height.0 as f32),
                    wall_loops: settings.process.region.wall_loops.0,
                    sparse_infill_density: f64::from(
                        settings.process.region.sparse_infill_density.0 as f32,
                    ),
                });
            }
        }
    }
    Ok(())
}

fn intersects(
    object: &ProjectObject,
    instance: &ProjectInstance,
    area: &Polygon,
    scale: CoordinateScale,
) -> Result<bool, SliceError> {
    let instance_offset = instance
        .transform()
        .transform_point(Point3d::new(0.0, 0.0, 0.0));
    let instance_shift = point(scale, instance_offset.x, instance_offset.y)?;
    for volume in object
        .volumes()
        .iter()
        .filter(|volume| volume.volume_type() == ProjectVolumeType::ModelPart)
    {
        let matrix = instance
            .transform()
            .without_translation()
            .then(volume.transform());
        let offset = matrix.transform_point(Point3d::new(0.0, 0.0, 0.0));
        let volume_shift = point(scale, offset.x, offset.y)?;
        let linear = matrix.without_translation();
        // Model.cpp:2625-2644 scales the linear projection of Vec3f vertices
        // before translating the hull; Print.cpp:960 then adds the instance shift.
        let points = volume
            .mesh()
            .vertices()
            .iter()
            .map(|vertex| {
                let transformed = linear.transform_point(Point3d::new(
                    f64::from(vertex.x as f32),
                    f64::from(vertex.y as f32),
                    f64::from(vertex.z as f32),
                ));
                point(scale, transformed.x, transformed.y)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let hull = Polygon::new(
            convex_hull(&points)
                .into_iter()
                .map(|p| {
                    Point::new(
                        p.x() + volume_shift.x() + instance_shift.x(),
                        p.y() + volume_shift.y() + instance_shift.y(),
                    )
                })
                .collect(),
        );
        let intersection =
            intersection_polygons_paths(std::slice::from_ref(area), std::slice::from_ref(&hull))
                .map_err(|_| {
                    SliceError::InvalidInput("wrapping region intersection failed".into())
                })?;
        if !intersection.is_empty() {
            return Ok(true);
        }
    }
    Ok(false)
}

fn point(scale: CoordinateScale, x: f64, y: f64) -> Result<Point, SliceError> {
    let coordinate = |value| {
        scale.checked_scale(value).ok_or_else(|| {
            SliceError::InvalidInput("wrapping region coordinate out of range".into())
        })
    };
    Ok(Point::new(coordinate(x)?, coordinate(y)?))
}
