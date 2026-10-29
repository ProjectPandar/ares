//! ContourZ.cpp:33-159: eligible paths, source0.1mm samples and height bounds.
use super::{mesh::IndexedMesh, ray};
use crate::{
    ExtrusionRole, RegionOptions, geometry::CoordinateScale,
    project_slice::perimeters::classic::materialize::Point3,
};
pub(super) struct Context<'a> {
    pub(super) region: &'a RegionOptions,
    pub(super) slice_z: f64,
    pub(super) print_z: f64,
    pub(super) layer_height: f64,
    pub(super) scale: CoordinateScale,
}
pub(super) fn contour(
    points: &[Point3],
    role: ExtrusionRole,
    width: f32,
    mesh: &IndexedMesh,
    context: &Context<'_>,
) -> Option<Vec<Point3>> {
    if !matches!(
        role,
        ExtrusionRole::TopSolidInfill
            | ExtrusionRole::Ironing
            | ExtrusionRole::ExternalPerimeter
            | ExtrusionRole::Perimeter
    ) || points.len() < 2
    {
        return None;
    }
    let mut output = Vec::new();
    let mut was_contoured = false;
    for pair in points.windows(2) {
        let a = [
            context.scale.unscale(pair[0].x),
            context.scale.unscale(pair[0].y),
        ];
        let b = [
            context.scale.unscale(pair[1].x),
            context.scale.unscale(pair[1].y),
        ];
        let delta = [b[0] - a[0], b[1] - a[1]];
        let segments = ((delta[0] * delta[0] + delta[1] * delta[1]).sqrt() / 0.1).ceil() as usize;
        if segments == 0 {
            continue;
        }
        for i in 0..=segments {
            let p = std::array::from_fn::<_, 2, _>(|axis| {
                a[axis] + delta[axis] * i as f64 / segments as f64
            });
            let hit = mesh.query([p[0], p[1], context.slice_z + mesh.ground_level]);
            let d = height_difference(&hit, role, width, context);
            was_contoured |= d.abs() > 1e-4;
            let next = [p[0], p[1], d];
            if output.len() >= 2
                && i != 0
                && distance_squared(next, output[output.len() - 2], output[output.len() - 1]) < 1e-8
            {
                *output.last_mut().unwrap() = next;
            } else {
                output.push(next);
            }
        }
    }
    was_contoured.then(|| {
        // ContourZ.cpp:155 / Point.hpp:280 selects Point3(double): round, then append dedup.
        let mut points: Vec<Point3> = output
            .into_iter()
            .map(|p| Point3 {
                x: (p[0] / context.scale.factor()).round() as i64,
                y: (p[1] / context.scale.factor()).round() as i64,
                z: (p[2] / context.scale.factor()).round() as i64,
            })
            .collect();
        points.dedup();
        points
    })
}
fn height_difference(
    hit: &super::mesh::Hit,
    role: ExtrusionRole,
    width: f32,
    context: &Context<'_>,
) -> f64 {
    let height = context.layer_height;
    let min_z = context.region.zaa_min_z.0;
    let (max_up, min_down) = if role == ExtrusionRole::Ironing {
        (height, -(height + 0.1))
    } else {
        (min_z, -(height - min_z))
    };
    let mut d = hit.distance - (context.print_z - context.slice_z);
    let perimeter = matches!(
        role,
        ExtrusionRole::ExternalPerimeter | ExtrusionRole::Perimeter
    );
    if perimeter && hit.face.is_some() {
        let normal = hit.normal;
        let normal_length = ray::dot(normal, normal).sqrt();
        let angle = (normal[2] / normal_length).abs().acos();
        let minimize = context.region.zaa_minimize_perimeter_height.0;
        if d > min_down && minimize > 0. && minimize < angle.to_degrees() {
            d += -(f64::from(width) / 2.) * angle.sin();
            if d < min_down {
                d = min_down;
            }
        }
    }
    if d < -height || d > max_up + 0.03 {
        d = 0.;
    }
    if d < min_down {
        d = min_down;
    } else if d > max_up {
        d = max_up;
    }
    if perimeter && d > 0. {
        d = 0.;
    }
    d
}
fn distance_squared(p: [f64; 3], a: [f64; 3], b: [f64; 3]) -> f64 {
    let ab = ray::subtract(b, a);
    let ap = ray::subtract(p, a);
    let norm = ray::dot(ab, ab);
    // Line.hpp:282-295 preserves source zero-length and closest-point evaluation order.
    if norm == 0. {
        return ray::dot(ap, ap);
    }
    let projection = ray::dot(ap, ab) / norm;
    let closest = std::array::from_fn::<_, 3, _>(|i| a[i] + projection * ab[i]);
    let distance = ray::subtract(p, closest);
    ray::dot(distance, distance)
}
