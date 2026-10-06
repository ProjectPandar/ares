//! `BuildVolume` print-volume gate (`BuildVolume.cpp:12-65,377-430`,
//! `BuildVolume.hpp:15-30,91-106`; `Model.cpp:692-702,2370-2379,3423-3469`).

use crate::{Point3d, Project, ProjectObject, SliceError, Transform3d};

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

/// `Model.hpp:1240-1243`: instance-level aggregation of volume states.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum InstancePvs {
    Inside,
    PartlyOutside,
    FullyOutside,
}

/// A fitted circle: centroid + radius (`BuildVolume.cpp:52-63`).
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

impl PrintVolume {
    /// `BuildVolume.cpp:28-89`: classifies the bed polygon. Rectangle
    /// requires 4 axis-aligned corners matching the bbox; Circle fits a
    /// center/radius to the corners (`:52-63`, vertex tolerance 0.005,
    /// midpoint undercut 3 mm); Convex uses the convex hull (non-convex
    /// shapes are `Unported`).
    pub(crate) fn new(project: &Project) -> Result<Self, SliceError> {
        let printer = &project.settings().printer.remaining;
        let area = &printer.printable_area.0;
        let max_z = printer.printable_height.0;
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
        let r = circle.radius + SCENE_EPSILON;
        let distance = |p: &Point3d| {
            ((p.x - circle.center[0]).powi(2) + (p.y - circle.center[1]).powi(2)).sqrt()
        };
        let mut inside = false;
        let mut outside = false;
        for vertex in vertices {
            if vertex.z <= -SCENE_EPSILON {
                continue; // below the bed; doesn't prevent Inside
            }
            let xy_inside = distance(vertex) <= r;
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

/// 4 corners on the bbox edges (axis-aligned rectangle).
fn is_axis_aligned_rectangle(area: &[crate::Point2d]) -> bool {
    let min = [
        area.iter().map(|p| p.x).fold(f64::INFINITY, f64::min),
        area.iter().map(|p| p.y).fold(f64::INFINITY, f64::min),
    ];
    let max = [
        area.iter().map(|p| p.x).fold(f64::NEG_INFINITY, f64::max),
        area.iter().map(|p| p.y).fold(f64::NEG_INFINITY, f64::max),
    ];
    area.iter().all(|p| {
        let on_x = (p.x - min[0]).abs() < SCENE_EPSILON || (p.x - max[0]).abs() < SCENE_EPSILON;
        let on_y = (p.y - min[1]).abs() < SCENE_EPSILON || (p.y - max[1]).abs() < SCENE_EPSILON;
        on_x && on_y
    })
}

/// `BuildVolume.cpp:52-63`: centroid + mean radius; vertices must lie
/// within 0.005 of the fitted radius and edge midpoints must not undercut
/// more than 3 mm (the discretization tolerance from BedShapePanel).
fn fit_circle(area: &[crate::Point2d]) -> Option<Circle> {
    if area.len() < 8 {
        return None;
    }
    let n = area.len() as f64;
    let center = [
        area.iter().map(|p| p.x).sum::<f64>() / n,
        area.iter().map(|p| p.y).sum::<f64>() / n,
    ];
    let mean_r = area
        .iter()
        .map(|p| ((p.x - center[0]).powi(2) + (p.y - center[1]).powi(2)).sqrt())
        .sum::<f64>()
        / n;
    if mean_r <= 0.0 {
        return None;
    }
    // Vertex tolerance 0.005 (`BuildVolume.cpp:59`).
    let vertex_ok = area.iter().all(|p| {
        (((p.x - center[0]).powi(2) + (p.y - center[1]).powi(2)).sqrt() - mean_r).abs() <= 0.005
    });
    if !vertex_ok {
        return None;
    }
    // Midpoint undercut 3 mm (`BuildVolume.cpp:60`).
    let midpoint_ok = (0..area.len()).all(|i| {
        let a = &area[i];
        let b = &area[(i + 1) % area.len()];
        let mid = [(a.x + b.x) * 0.5, (a.y + b.y) * 0.5];
        let d = ((mid[0] - center[0]).powi(2) + (mid[1] - center[1]).powi(2)).sqrt();
        mean_r - d <= 3.0
    });
    if !midpoint_ok {
        return None;
    }
    Some(Circle {
        center,
        radius: mean_r,
    })
}

/// Andrew's monotone chain convex hull.
fn convex_hull(points: &[crate::Point2d]) -> Vec<[f64; 2]> {
    let mut pts: Vec<[f64; 2]> = points.iter().map(|p| [p.x, p.y]).collect();
    pts.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    pts.dedup();
    if pts.len() < 3 {
        return pts;
    }
    let cross = |o: [f64; 2], a: [f64; 2], b: [f64; 2]| {
        (a[0] - o[0]) * (b[1] - o[1]) - (a[1] - o[1]) * (b[0] - o[0])
    };
    let mut lower = Vec::new();
    for &p in &pts {
        while lower.len() >= 2 && cross(lower[lower.len() - 2], lower[lower.len() - 1], p) <= 0.0 {
            lower.pop();
        }
        lower.push(p);
    }
    let mut upper = Vec::new();
    for &p in pts.iter().rev() {
        while upper.len() >= 2 && cross(upper[upper.len() - 2], upper[upper.len() - 1], p) <= 0.0 {
            upper.pop();
        }
        upper.push(p);
    }
    lower.pop();
    upper.pop();
    lower.extend(upper);
    lower
}

/// Point-in-convex-polygon (CCW hull).
fn point_in_convex(hull: &[[f64; 2]], p: [f64; 2]) -> bool {
    (0..hull.len()).all(|i| {
        let a = hull[i];
        let b = hull[(i + 1) % hull.len()];
        let cross = (b[0] - a[0]) * (p[1] - a[1]) - (b[1] - a[1]) * (p[0] - a[0]);
        cross >= 0.0
    })
}

/// `Model.cpp:2370-2379`: count instances with `InstancePvs::Inside`.
pub(crate) fn count_inside_instances(project: &Project) -> Result<usize, SliceError> {
    let volume = PrintVolume::new(project)?;
    let mut count = 0;
    for object in project.objects() {
        for instance in object.instances() {
            if calc_print_volume_state(object, instance.transform(), &volume) == InstancePvs::Inside
            {
                count += 1;
            }
        }
    }
    Ok(count)
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
            .map(|vertex| matrix.transform_point(*vertex))
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
