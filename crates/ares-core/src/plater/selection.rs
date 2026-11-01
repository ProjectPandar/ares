//! Instance-mode `Selection` edits and the canvas follow-up that writes
//! them to the model (`GLCanvas3D::do_move` / `do_rotate` / `do_scale`).

use super::eigen::{rotation_transform, scale_transform, translation_transform};
use super::min_sphere::bounding_sphere_center;
use super::plate_list::{BoundingBox, InstanceKey};
use super::transformation::{synchronized, without_scaling_factor};
use crate::{Point3d, Project, ProjectVolumeType, Transform3d};

/// `SINKING_Z_THRESHOLD` (`Model.hpp:1781`, a `float`).
const SINKING_Z_THRESHOLD: f64 = -0.001_f32 as f64;

fn instance_transform(project: &Project, (object, instance): InstanceKey) -> Transform3d {
    project.objects()[object].instances()[instance].transform()
}

/// `Selection::translate` for a full instance in world coordinates
/// (`Selection.cpp:1253-1268`, `transform_instance_relative` `:3347-3357`):
/// `T(c) · T(d) · T(-c) · M` with `c` the dragging centre.
pub(super) fn translate(project: &mut Project, key: InstanceKey, displacement: [f64; 3]) {
    let center = selection_bounding_box_center(project, key);
    let trafo = translation_transform(center)
        .then(translation_transform(displacement))
        .then(translation_transform(center.map(|value| -value)));
    let transform = trafo.then(instance_transform(project, key));
    project.set_instance_transform(key.0, key.1, transform);
}

/// `Selection::rotate` for a full instance in world coordinates
/// (`Selection.cpp:1331-1384`): `T(p) · R · T(-p) · M` with `p` the
/// bounding-sphere pivot and `R` = `Geometry::rotation_transform(rotation)`.
pub(super) fn rotate(project: &mut Project, key: InstanceKey, rotation: [f64; 3]) {
    let pivot = rotation_pivot(project, key);
    let trafo = translation_transform(pivot)
        .then(rotation_transform(rotation))
        .then(translation_transform(pivot.map(|value| -value)));
    let transform = trafo.then(instance_transform(project, key));
    project.set_instance_transform(key.0, key.1, transform);
}

/// `Selection::scale` for a full instance, absolute in world coordinates
/// (`Selection.cpp:1475-1478, 1627-1650`): the factor made relative through the full unscaled
/// instance box, applied about the dragging centre (`transform_instance_relative`, `:3364-3372`,
/// `setup_cache`'s centre `:2547`), and the object's other instances synchronized
/// (`synchronize_unselected_instances(GENERAL)`, `:1693-1697, 3045-3095`).
pub(super) fn scale(project: &mut Project, key: InstanceKey, scale: [f64; 3]) {
    let current_size = world_box_size(project, key);
    let original_size = unscaled_instance_box_size(project, key, false);
    let relative: [f64; 3] =
        std::array::from_fn(|axis| original_size[axis] * scale[axis] / current_size[axis]);
    let center = selection_bounding_box_center(project, key);
    let trafo = translation_transform(center)
        .then(scale_transform(relative))
        .then(translation_transform(center.map(|value| -value)));
    let old = instance_transform(project, key);
    let current = trafo.then(old);
    project.set_instance_transform(key.0, key.1, current);
    for sibling in 0..project.objects()[key.0].instances().len() {
        if sibling != key.1 {
            let old_sibling = instance_transform(project, (key.0, sibling));
            let transform = synchronized(old_sibling, old, current);
            project.set_instance_transform(key.0, sibling, transform);
        }
    }
}

/// The size of `Selection::get_bounding_box_in_reference_system(World)`
/// (`Selection.cpp:1126-1196`): every volume's hull vertices in world coordinates.
pub(super) fn world_box_size(project: &Project, key: InstanceKey) -> [f64; 3] {
    let transform = instance_transform(project, key);
    let mut bbox = ([f64::MAX; 3], [f64::MIN; 3]);
    for volume in project.objects()[key.0].volumes() {
        let world = transform.then(volume.transform());
        for vertex in volume.mesh().vertices() {
            merge(&mut bbox, world.transform_point(*vertex));
        }
    }
    size(bbox)
}

/// The size of `Selection::get_full_unscaled_instance_bounding_box` (`Selection.cpp:1050-1066`),
/// or with `model_parts_only` of `get_unscaled_instance_bounding_box` (`:1008-1027`): the
/// volumes' hulls through the instance matrix without its scaling factor.
pub(super) fn unscaled_instance_box_size(
    project: &Project,
    key: InstanceKey,
    model_parts_only: bool,
) -> [f64; 3] {
    let transform = without_scaling_factor(instance_transform(project, key));
    let mut bbox = ([f64::MAX; 3], [f64::MIN; 3]);
    let volumes = project.objects()[key.0]
        .volumes()
        .iter()
        .filter(|volume| !model_parts_only || volume.volume_type() == ProjectVolumeType::ModelPart);
    for volume in volumes {
        let matrix = transform.then(volume.transform());
        for vertex in volume.mesh().vertices() {
            merge(&mut bbox, matrix.transform_point(*vertex));
        }
    }
    size(bbox)
}

fn size((min, max): BoundingBox) -> [f64; 3] {
    std::array::from_fn(|axis| max[axis] - min[axis])
}

/// `GLCanvas3D::do_move` (`GLCanvas3D.cpp:4970-4995`): an instance whose
/// lowest point is not sinking drops onto the bed.
pub(super) fn drop_after_move(project: &mut Project, key: InstanceKey) -> bool {
    let shift_z = instance_min_z(project, key);
    drop_by(project, key, shift_z, shift_z > SINKING_Z_THRESHOLD)
}

/// `GLCanvas3D::do_rotate` (`GLCanvas3D.cpp:5094-5110`) and `do_scale` (`:5949-5962`): as
/// `do_move`, also when the instance was not sinking before the edit.
pub(super) fn drop_after_transform(
    project: &mut Project,
    key: InstanceKey,
    min_z_before: f64,
) -> bool {
    let shift_z = instance_min_z(project, key);
    drop_by(
        project,
        key,
        shift_z,
        min_z_before >= SINKING_Z_THRESHOLD || shift_z > SINKING_Z_THRESHOLD,
    )
}

/// `ModelObject::translate_instance(i, (0, 0, -shift_z))` (`Model.cpp:1758-1764`).
fn drop_by(project: &mut Project, key: InstanceKey, shift_z: f64, allowed: bool) -> bool {
    if !(allowed && auto_drop(project, key) && shift_z != 0.0) {
        return false;
    }
    let mut rows = *instance_transform(project, key).as_matrix();
    rows[2][3] += -shift_z;
    project.set_instance_transform(key.0, key.1, Transform3d::from_rows(rows));
    true
}

fn auto_drop(project: &Project, (object, instance): InstanceKey) -> bool {
    project.objects()[object].instances()[instance].auto_drop()
}

/// `ModelObject::get_instance_min_z` (`Model.cpp:2313-2343`): the model
/// parts' hull points through the offset-free instance matrix, plus the
/// offset's Z.
pub(super) fn instance_min_z(project: &Project, key: InstanceKey) -> f64 {
    let transform = instance_transform(project, key);
    let no_offset = transform.without_translation();
    let mut min_z = f64::MAX;
    for volume in model_parts(project, key.0) {
        let matrix = no_offset.then(volume.transform());
        for vertex in volume.mesh().vertices() {
            min_z = min_z.min(matrix.transform_point(*vertex).z);
        }
    }
    if min_z == f64::MAX {
        min_z = 0.0;
    }
    min_z + transform.as_matrix()[2][3]
}

/// `ModelObject::instance_bounding_box` (`Model.cpp:1607-1619`) and
/// `instance_convex_hull_bounding_box` (`:1628-1638`); the hull's extreme
/// points are mesh vertices, so both give this box.
pub(super) fn instance_bounding_box(project: &Project, key: InstanceKey) -> BoundingBox {
    let transform = instance_transform(project, key);
    let mut bbox = ([f64::MAX; 3], [f64::MIN; 3]);
    for volume in model_parts(project, key.0) {
        let matrix = transform.then(volume.transform());
        for vertex in volume.mesh().vertices() {
            merge(&mut bbox, matrix.transform_point(*vertex));
        }
    }
    bbox
}

fn model_parts(project: &Project, object: usize) -> impl Iterator<Item = &crate::ProjectVolume> {
    project.objects()[object]
        .volumes()
        .iter()
        .filter(|volume| volume.volume_type() == ProjectVolumeType::ModelPart)
}

/// `Selection::get_bounding_box().center()` (`Selection.cpp:2530`): every
/// selected volume's local box corners through its world matrix
/// (`GLVolume::transformed_bounding_box`).
fn selection_bounding_box_center(project: &Project, key: InstanceKey) -> [f64; 3] {
    let transform = instance_transform(project, key);
    let mut bbox = ([f64::MAX; 3], [f64::MIN; 3]);
    for volume in project.objects()[key.0].volumes() {
        let mut local = ([f64::MAX; 3], [f64::MIN; 3]);
        for vertex in volume.mesh().vertices() {
            let vertex = [vertex.x as f32, vertex.y as f32, vertex.z as f32].map(f64::from);
            merge(&mut local, Point3d::new(vertex[0], vertex[1], vertex[2]));
        }
        let world = transform.then(volume.transform());
        for corner in 0..8 {
            let point = Point3d::new(
                if corner & 1 == 0 {
                    local.0[0]
                } else {
                    local.1[0]
                },
                if corner & 2 == 0 {
                    local.0[1]
                } else {
                    local.1[1]
                },
                if corner & 4 == 0 {
                    local.0[2]
                } else {
                    local.1[2]
                },
            );
            merge(&mut bbox, world.transform_point(point));
        }
    }
    std::array::from_fn(|axis| 0.5 * (bbox.0[axis] + bbox.1[axis]))
}

/// The rotation pivot over every volume of the instance (`Selection.cpp:1188-1205`).
fn rotation_pivot(project: &Project, key: InstanceKey) -> [f64; 3] {
    let transform = instance_transform(project, key);
    let mut points = Vec::new();
    for volume in project.objects()[key.0].volumes() {
        let world = transform.then(volume.transform());
        points.extend(volume.mesh().vertices().iter().map(|vertex| {
            let point = world.transform_point(*vertex);
            [point.x as f32, point.y as f32, point.z as f32]
        }));
    }
    bounding_sphere_center(&points)
}

fn merge(bbox: &mut BoundingBox, point: Point3d) {
    for (axis, value) in [point.x, point.y, point.z].into_iter().enumerate() {
        bbox.0[axis] = bbox.0[axis].min(value);
        bbox.1[axis] = bbox.1[axis].max(value);
    }
}
