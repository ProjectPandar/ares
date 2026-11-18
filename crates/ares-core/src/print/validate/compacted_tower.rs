//! Fresh-estimate clearance for a compacted tower (`Print.cpp:1222–1478`).

use crate::{
    Point3d, ProcessWipeTowerWallType, Project, ProjectObject, ProjectVolumeType, SliceError,
    Transform3d,
    geometry::{
        ClipperError, CoordinateScale, JoinType, Point, Polygon, convex_hull,
        intersection_polygons_paths, offset_paths,
    },
    project::effective_config::types::BoundedResolvedProjectConfig,
};

const EPSILON: f64 = 1e-4;
const NOZZLE_DIAMETER: f64 = 4.0; // Print.hpp: MAX_OUTER_NOZZLE_DIAMETER
const MESSAGE: &str = "The relative position of the model and the prime tower does not meet the requirements of the \"No sparse layers\" feature. Please adjust their relative positions, lower the model height, or turn off \"No sparse layers\".";

pub(super) fn validate(
    project: &Project,
    resolved: &BoundedResolvedProjectConfig,
    plate_index: u32,
    scale: CoordinateScale,
    (depth, brim): (f32, f32),
) -> Result<(), SliceError> {
    if f64::from(depth) < EPSILON {
        return Ok(());
    }
    let settings = &resolved.views.full;
    let tower = &settings.process.print;
    let plate = &settings.project.print;
    let index = project.current_plate_index();
    let (ox, oy) = project.plate_origin();
    let x = plate
        .wipe_tower_x
        .0
        .get(index)
        .unwrap_or(&plate.wipe_tower_x.0[0])
        .0
        + ox;
    let y = plate
        .wipe_tower_y
        .0
        .get(index)
        .unwrap_or(&plate.wipe_tower_y.0[0])
        .0
        + oy;
    let width = if tower.wipe_tower_wall_type == ProcessWipeTowerWallType::Rib {
        f64::from(depth)
    } else {
        tower.prime_tower_width.0
    };
    let padding = f64::from(brim).max(0.0) + 2.0 * plate.nozzle_diameter.0[0].0;
    let (sin, cos) = tower.wipe_tower_rotation_angle.0.to_radians().sin_cos();
    let mut hull = Polygon::new(
        [
            (-padding, -padding),
            (width + padding, -padding),
            (width + padding, f64::from(depth) + padding),
            (-padding, f64::from(depth) + padding),
        ]
        .into_iter()
        .map(|(u, v)| {
            Ok(Point::new(
                scaled(scale, cos * u - sin * v + x, true)?,
                scaled(scale, sin * u + cos * v + y, true)?,
            ))
        })
        .collect::<Result<Vec<_>, SliceError>>()?,
    );

    let spiral = settings
        .project
        .gcode
        .z_hop
        .0
        .iter()
        .zip(&settings.printer.gcode.travel_slope.0)
        .filter_map(|(lift, slope)| {
            let lift = lift.0.min(5.0);
            (lift >= EPSILON && slope.0 >= EPSILON)
                .then(|| 2.0 * lift / (2.0 * std::f64::consts::PI * slope.0.atan()))
        })
        .fold(0.0, f64::max);
    if spiral > EPSILON {
        let grown = round_offset(&hull, spiral, scale)?;
        if !grown.is_empty() {
            let points: Vec<Point> = grown
                .iter()
                .flat_map(|polygon| polygon.points().iter().copied())
                .collect();
            hull = Polygon::new(convex_hull(&points));
        }
    }

    let printer = &settings.printer.remaining;
    let rod_margin = scaled(scale, printer.extruder_clearance_dist_to_rod.0 * 0.5, true)?;
    let tower_y = y_band(&hull, rod_margin);
    let nozzle_half = half_clearance(NOZZLE_DIAMETER);
    let body_half = half_clearance(printer.extruder_clearance_radius.0);
    let grown_nozzle = round_offset(&hull, nozzle_half, scale)?;
    let grown_body = round_offset(&hull, body_half, scale)?;
    let nozzle_height = printer.nozzle_height.0;

    for object in &resolved.objects {
        let source = &project.objects()[object.source_object_index];
        // PrintObject::max_z is measured from the bed using the model object's first instance.
        let top = object_top(source, scale)?;
        for index in object
            .print_objects
            .iter()
            .flat_map(|object| &object.instance_indices)
        {
            let instance = instance_hull(source, source.instances()[*index].transform(), scale)?;
            if instance.points().is_empty() {
                continue;
            }
            let (min_y, max_y) = y_band(&instance, rod_margin);
            let mut allowed = if max_y.min(tower_y.1) > min_y.max(tower_y.0) {
                printer.extruder_clearance_height_to_rod.0
            } else {
                printer.extruder_clearance_height_to_lid.0
            };
            if overlaps(&grown_nozzle, &round_offset(&instance, nozzle_half, scale)?)? {
                allowed = 0.0;
            } else if top > nozzle_height + EPSILON
                && overlaps(&grown_body, &round_offset(&instance, body_half, scale)?)?
            {
                allowed = allowed.min(nozzle_height);
            }
            if top > allowed + EPSILON {
                return Err(SliceError::LayeredPrintCollision {
                    message: MESSAGE.to_owned(),
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

fn object_top(object: &ProjectObject, scale: CoordinateScale) -> Result<f64, SliceError> {
    let transform = object.instances()[0].transform();
    let top = model_part_vertices(object, transform)
        .map(|(matrix, vertex)| matrix.transform_point(vertex).z)
        .reduce(f64::max)
        .unwrap_or(0.0);
    Ok(scale.unscale(scaled(scale, top, false)?))
}

fn instance_hull(
    object: &ProjectObject,
    transform: Transform3d,
    scale: CoordinateScale,
) -> Result<Polygon, SliceError> {
    let origin = Point3d::new(0.0, 0.0, 0.0);
    let shift = transform.transform_point(origin);
    let sx = scaled(scale, shift.x, false)?;
    let sy = scaled(scale, shift.y, false)?;
    // ModelVolume scales the linear projection before adding the volume offset.
    // PrintInstance then adds its XY shift without the centering offset.
    let points = model_part_vertices(object, transform.without_translation())
        .map(|(matrix, vertex)| {
            let shift = matrix.transform_point(origin);
            let point = matrix.without_translation().transform_point(vertex);
            let x = scaled(scale, point.x, false)? + scaled(scale, shift.x, false)? + sx;
            let y = scaled(scale, point.y, false)? + scaled(scale, shift.y, false)? + sy;
            Ok(Point::new(x, y))
        })
        .collect::<Result<Vec<_>, SliceError>>()?;
    Ok(Polygon::new(convex_hull(&points)))
}

fn model_part_vertices(
    object: &ProjectObject,
    transform: Transform3d,
) -> impl Iterator<Item = (Transform3d, Point3d)> + '_ {
    object
        .volumes()
        .iter()
        .filter(|volume| volume.volume_type() == ProjectVolumeType::ModelPart)
        .flat_map(move |volume| {
            let matrix = transform.then(volume.transform());
            volume.mesh().vertices().iter().map(move |vertex| {
                (
                    matrix,
                    Point3d::new(
                        f64::from(vertex.x as f32),
                        f64::from(vertex.y as f32),
                        f64::from(vertex.z as f32),
                    ),
                )
            })
        })
}

fn scaled(scale: CoordinateScale, mm: f64, round: bool) -> Result<i64, SliceError> {
    if round {
        scale.checked_scale_rounded(mm)
    } else {
        scale.checked_scale(mm)
    }
    .ok_or_else(|| {
        SliceError::InvalidInput("compacted tower clearance coordinate out of range".into())
    })
}

fn y_band(hull: &Polygon, margin: i64) -> (i64, i64) {
    hull.points()
        .iter()
        .fold((i64::MAX, i64::MIN), |(lo, hi), point| {
            (lo.min(point.y() - margin), hi.max(point.y() + margin))
        })
}

fn half_clearance(distance: f64) -> f64 {
    0.5 * (distance - 0.2)
}

fn round_offset(
    hull: &Polygon,
    mm: f64,
    scale: CoordinateScale,
) -> Result<Vec<Polygon>, SliceError> {
    offset_paths(
        std::slice::from_ref(hull),
        scale.scaled_delta(mm) as f32,
        JoinType::Round,
        scale.scaled_delta(0.1),
    )
    .map_err(clip_error)
}

fn overlaps(a: &[Polygon], b: &[Polygon]) -> Result<bool, SliceError> {
    intersection_polygons_paths(a, b)
        .map(|polygons| !polygons.is_empty())
        .map_err(clip_error)
}

fn clip_error(error: ClipperError) -> SliceError {
    SliceError::InvalidInput(format!("compacted tower clearance: {error:?}"))
}
