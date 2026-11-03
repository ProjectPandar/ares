//! `BuildVolume` print-volume gate (`BuildVolume.cpp:12-65,377-430`,
//! `BuildVolume.hpp:15-30,91-106`; `Model.cpp:692-702,2370-2379,3423-3469`).

mod circle;
mod convex;
mod height;

use crate::{
    Point3d, Project, ProjectObject, SliceError, Transform3d, geometry::CoordinateScale,
    project::InstancePvs,
};
use circle::fit_circle;
use convex::{convex_hull, is_axis_aligned_rectangle, point_in_convex};

/// `BuildVolume.hpp:106`: the plater-level epsilon.
const SCENE_EPSILON: f64 = 1e-4;

/// `BuildVolume.hpp:91-99`: object-level containment classification.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ObjectState {
    Inside,
    Colliding,
    Outside,
    Below,
}

/// RANSAC-fitted circle in fixed coordinates (`BuildVolume.cpp:38-63`).
struct Circle {
    center: [f64; 2],
    radius: f64,
}

/// `BuildVolume.cpp:15-21`: the bed's classified shape. `Rectangle` uses
/// `volume_state_bbox`; `Circle` and `Convex` test per transformed mesh
/// vertex (`BuildVolume.cpp:384-410 object_state`); `Unported` (non-convex
/// Custom) bypasses the check as a known gap recorded in the ARD.
enum BedShape {
    Rectangle { min: [f64; 2], max: [f64; 2] },
    Circle(Circle),
    Convex { hull: Vec<[f64; 2]> },
    Unported,
}

/// A classified print volume ready for containment tests.
pub(crate) struct PrintVolume {
    shape: BedShape,
    max_z: f64,
}

/// `PartPlate::set_shape` (PartPlate.cpp:3179–3187): shared global bed frame.
pub(crate) fn global_bed_points(
    project: &Project,
    points: &[crate::Point2d],
) -> impl Iterator<Item = crate::Point2d> {
    let (origin_x, origin_y) = project.plate_origin();
    points.iter().map(move |point| crate::Point2d {
        x: point.x + origin_x,
        y: point.y + origin_y,
    })
}

impl PrintVolume {
    /// `BuildVolume.cpp:28-89`: classifies the bed polygon. Rectangle
    /// requires 4 axis-aligned corners matching the bbox; Circle fits a
    /// center/radius to the corners (`:52-63`, vertex tolerance 0.005,
    /// midpoint undercut 3 mm); Convex uses the convex hull (non-convex
    /// shapes are `Unported`).
    pub(crate) fn new(project: &Project) -> Result<Self, SliceError> {
        Self::at(project, project.plate_origin())
    }

    /// The print volume of the plate at `origin`.
    fn at(project: &Project, (origin_x, origin_y): (f64, f64)) -> Result<Self, SliceError> {
        let printer = &project.settings().printer.remaining;
        let area = &printer.printable_area.0;
        let max_z = height::scene_limit(printer.printable_height.0);
        // The bed shape is translated to the plate's grid position before
        // the gate (`PartPlate.cpp:3179-3187` `set_shape` adds the position
        // to every point) so the still-global instance coordinates are
        // tested against the equally-global shape.
        let translated = area
            .iter()
            .map(|point| crate::Point2d {
                x: point.x + origin_x,
                y: point.y + origin_y,
            })
            .collect::<Vec<_>>();
        let area = &translated;
        let shape = if area.len() == 4 && is_axis_aligned_rectangle(area) {
            BedShape::Rectangle {
                min: [
                    area.iter().map(|p| p.x).fold(f64::INFINITY, f64::min),
                    area.iter().map(|p| p.y).fold(f64::INFINITY, f64::min),
                ],
                max: [
                    area.iter().map(|p| p.x).fold(f64::NEG_INFINITY, f64::max),
                    area.iter().map(|p| p.y).fold(f64::NEG_INFINITY, f64::max),
                ],
            }
        } else if let Some(circle) = fit_circle(area) {
            BedShape::Circle(circle)
        } else {
            let hull = convex_hull(area);
            if hull.len() == area.len() {
                BedShape::Convex { hull }
            } else {
                BedShape::Unported
            }
        };
        Ok(Self { shape, max_z })
    }

    /// `BuildVolume::object_state` (`BuildVolume.cpp:377-410`): dispatches
    /// per shape type on the transformed mesh vertices. Rectangle uses
    /// `volume_state_bbox` on the bbox built FROM those vertices; Circle
    /// and Convex test each vertex directly.
    fn object_state(&self, vertices: &[Point3d]) -> ObjectState {
        match &self.shape {
            BedShape::Rectangle { min, max } => self.volume_state_bbox(vertices, *min, *max),
            BedShape::Circle(circle) => self.object_state_circle(vertices, circle),
            BedShape::Convex { hull } => self.object_state_convex(vertices, hull),
            BedShape::Unported => ObjectState::Inside,
        }
    }

    /// `BuildVolume::volume_state_bbox` (`BuildVolume.cpp:412-421`): bbox
    /// containment with `SceneEpsilon` inflation. The bbox is computed from
    /// the transformed vertices (not transforming the model bbox), so
    /// rotated instances are handled correctly.
    fn volume_state_bbox(&self, vertices: &[Point3d], min: [f64; 2], max: [f64; 2]) -> ObjectState {
        let inflated = [
            [min[0] - SCENE_EPSILON, min[1] - SCENE_EPSILON],
            [max[0] + SCENE_EPSILON, max[1] + SCENE_EPSILON],
        ];
        let z_max = self.max_z + SCENE_EPSILON;
        let bbox_min = vertices.iter().fold([f64::INFINITY; 3], |acc, v| {
            [acc[0].min(v.x), acc[1].min(v.y), acc[2].min(v.z)]
        });
        let bbox_max = vertices.iter().fold([f64::NEG_INFINITY; 3], |acc, v| {
            [acc[0].max(v.x), acc[1].max(v.y), acc[2].max(v.z)]
        });
        if bbox_max[2] <= -SCENE_EPSILON {
            return ObjectState::Below;
        }
        let contains = bbox_min[0] >= inflated[0][0]
            && bbox_min[1] >= inflated[0][1]
            && bbox_max[0] <= inflated[1][0]
            && bbox_max[1] <= inflated[1][1]
            && bbox_max[2] <= z_max;
        if contains {
            return ObjectState::Inside;
        }
        let intersects = bbox_min[0] <= inflated[1][0]
            && bbox_min[1] <= inflated[1][1]
            && bbox_max[0] >= inflated[0][0]
            && bbox_max[1] >= inflated[0][1]
            && bbox_min[2] <= z_max;
        if intersects {
            return ObjectState::Colliding;
        }
        ObjectState::Outside
    }

    /// `BuildVolume::object_state` Circle branch (`BuildVolume.cpp:393-396`):
    /// per-vertex containment against the circle (radius + SceneEpsilon) and
    /// z < max_print_height + SceneEpsilon. All inside → Inside; all
    /// outside → Outside; mixed → Colliding.
    fn object_state_circle(&self, vertices: &[Point3d], circle: &Circle) -> ObjectState {
        let z_max = self.max_z + SCENE_EPSILON;
        // Native adds SceneEpsilon in scaled units, then unscales to Circlef.
        let factor = CoordinateScale::Normal.factor() as f32;
        let center = circle.center.map(|c| c as f32 * factor);
        let r = (circle.radius + SCENE_EPSILON) as f32 * factor;
        let mut inside = false;
        let mut outside = false;
        for vertex in vertices {
            if vertex.z <= -SCENE_EPSILON {
                continue; // below the bed; doesn't prevent Inside
            }
            let dx = vertex.x as f32 - center[0];
            let dy = vertex.y as f32 - center[1];
            let xy_inside = dx * dx + dy * dy <= r * r;
            let z_inside = vertex.z < z_max;
            if xy_inside && z_inside {
                inside = true;
            } else {
                outside = true;
            }
        }
        classify(inside, outside)
    }

    /// Convex branch: per-vertex convex-hull containment.
    fn object_state_convex(&self, vertices: &[Point3d], hull: &[[f64; 2]]) -> ObjectState {
        let z_max = self.max_z + SCENE_EPSILON;
        let mut inside = false;
        let mut outside = false;
        for vertex in vertices {
            if vertex.z <= -SCENE_EPSILON {
                continue;
            }
            let xy_inside = point_in_convex(hull, [vertex.x, vertex.y]);
            let z_inside = vertex.z < z_max;
            if xy_inside && z_inside {
                inside = true;
            } else {
                outside = true;
            }
        }
        classify(inside, outside)
    }
}

/// Aggregates per-vertex inside/outside flags: all inside → Inside, all
/// outside → Outside, mixed → Colliding (`object_state_templ`).
fn classify(inside: bool, outside: bool) -> ObjectState {
    match (inside, outside) {
        (true, false) => ObjectState::Inside,
        (false, true) => ObjectState::Outside,
        (true, true) => ObjectState::Colliding,
        (false, false) => ObjectState::Below,
    }
}

/// `Model::update_print_volume_state` (`Model.cpp:839-852` at v2.5.0-beta `345efec64`) through
/// `ModelObject::update_instances_print_volume_state` (`:2548-2562`): every instance stores its
/// `calc_print_volume_state`, and the inside ones are counted.
pub(crate) fn update_print_volume_state(project: &mut Project) -> Result<usize, SliceError> {
    let volume = PrintVolume::new(project)?;
    let states: Vec<Vec<InstancePvs>> = project
        .objects()
        .iter()
        .map(|object| {
            object
                .instances()
                .iter()
                .map(|instance| calc_print_volume_state(object, instance.transform(), &volume))
                .collect()
        })
        .collect();
    let mut count = 0;
    for (object, states) in states.into_iter().enumerate() {
        for (instance, state) in states.into_iter().enumerate() {
            count += usize::from(state == InstancePvs::Inside);
            project.set_instance_print_volume_state(object, instance, state);
        }
    }
    Ok(count)
}

/// The plate's `triangle_count` for the CLI success report (`OrcaSlicer.cpp:6576-6640` at v2.5.0-beta
/// `345efec64`): the facets of every model-part volume of each instance whose print-volume state is
/// inside. The Ares CLI skips no object, so no instance is excluded as skipped.
pub(crate) fn inside_triangle_count(project: &Project) -> Result<usize, SliceError> {
    let volume = PrintVolume::new(project)?;
    let mut count = 0;
    for object in project.objects() {
        for instance in object.instances() {
            if calc_print_volume_state(object, instance.transform(), &volume) == InstancePvs::Inside
            {
                count += object
                    .volumes()
                    .iter()
                    .filter(|volume| volume.volume_type() == crate::ProjectVolumeType::ModelPart)
                    .map(|volume| volume.mesh().triangles().len())
                    .sum::<usize>();
            }
        }
    }
    Ok(count)
}

/// `PartPlate::check_outside`'s sinking test (`PartPlate.cpp:4565-4572` at v2.5.0-beta
/// `345efec64`): the instance partly outside the build volume of the plate at `origin`.
pub(crate) fn instance_partly_outside(
    project: &Project,
    origin: (f64, f64),
    (object, instance): (usize, usize),
) -> Result<bool, SliceError> {
    let volume = PrintVolume::at(project, origin)?;
    let object = &project.objects()[object];
    let transform = object.instances()[instance].transform();
    Ok(calc_print_volume_state(object, transform, &volume) == InstancePvs::PartlyOutside)
}

/// `ModelInstance::calc_print_volume_state` (`Model.cpp:3423-3469`):
/// aggregates per-volume ObjectState into an instance-level PVS.
/// Inside requires every model-part volume to be Inside (no OUTSIDE flag);
/// Below volumes do not prevent Inside.
fn calc_print_volume_state(
    object: &ProjectObject,
    transform: Transform3d,
    volume: &PrintVolume,
) -> InstancePvs {
    let mut has_inside = false;
    let mut has_outside = false;
    for vol in object.volumes() {
        if vol.volume_type() != crate::ProjectVolumeType::ModelPart {
            continue;
        }
        let matrix = transform.then(vol.transform());
        let vertices: Vec<Point3d> = vol
            .mesh()
            .vertices()
            .iter()
            .map(|vertex| {
                if matches!(&volume.shape, BedShape::Circle(_)) {
                    let [x, y, z] = matrix.transform_point_f32(*vertex);
                    Point3d {
                        x: f64::from(x),
                        y: f64::from(y),
                        z: f64::from(z),
                    }
                } else {
                    matrix.transform_point(*vertex)
                }
            })
            .collect();
        match volume.object_state(&vertices) {
            ObjectState::Inside => has_inside = true,
            ObjectState::Outside => has_outside = true,
            // Below does not prevent Inside (`Model.cpp:3458-3460`).
            ObjectState::Below => {}
            // Colliding sets both flags (`Model.cpp:3461-3463`).
            ObjectState::Colliding => {
                has_inside = true;
                has_outside = true;
            }
        }
    }
    match (has_inside, has_outside) {
        (true, false) => InstancePvs::Inside,
        (false, true) => InstancePvs::FullyOutside,
        (true, true) => InstancePvs::PartlyOutside,
        (false, false) => InstancePvs::FullyOutside,
    }
}
