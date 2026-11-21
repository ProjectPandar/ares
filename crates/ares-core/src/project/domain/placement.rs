//! `--center` placement: `Model.cpp:704-722` (`center_instances_around_point`)
//! and `OrcaSlicer.cpp:4378-4392` (Z ground drop after centering).

use super::{Point3d, Project};

/// `libslic3r.h:52`: the no-op threshold for centering shifts.
const EPSILON: f64 = 1e-4;

impl Project {
    /// `Model::center_instances_around_point`: shift every instance offset by
    /// `point - world_bbox_center.xy`; `false` when the shift is below
    /// `EPSILON`. Z never moves here.
    pub(crate) fn center_instances_around_point(&mut self, point: [f64; 2]) -> bool {
        let mut min = [f64::INFINITY; 3];
        let mut max = [f64::NEG_INFINITY; 3];
        let instances = self.objects.iter().flat_map(|object| {
            object
                .instances
                .iter()
                .map(move |instance| (object, instance.transform))
        });
        for (object, transform) in instances {
            for point in world_vertices(object, transform) {
                merge_bbox_point(&mut min, &mut max, point);
            }
        }
        let shift = [
            point[0] - (min[0] + max[0]) * 0.5,
            point[1] - (min[1] + max[1]) * 0.5,
        ];
        if shift.iter().all(|value| value.abs() < EPSILON) {
            return false;
        }
        for object in &mut self.objects {
            for instance in object
                .instances
                .iter_mut()
                .chain(&mut object.import_instances)
            {
                instance.transform = instance
                    .transform
                    .pretranslated(Point3d::new(shift[0], shift[1], 0.0));
            }
        }
        true
    }

    /// `OrcaSlicer.cpp:4013` `plate_obj_size_infos` object bbox: the world
    /// bounding box over every model-part instance (`Model.cpp:1603-1618`).
    pub(crate) fn instances_world_bbox(&self) -> ([f64; 3], [f64; 3]) {
        let mut min = [f64::INFINITY; 3];
        let mut max = [f64::NEG_INFINITY; 3];
        let instances = self.objects.iter().flat_map(|object| {
            object
                .instances
                .iter()
                .map(move |instance| (object, instance.transform))
        });
        for (object, transform) in instances {
            for point in world_vertices(object, transform) {
                merge_bbox_point(&mut min, &mut max, point);
            }
        }
        (min, max)
    }

    /// `PartPlate::get_objects_bounding_box` (PartPlate.cpp:4741–4759) uses only
    /// the instances on that plate, from objects retained by the BBS importer.
    pub(crate) fn imported_plate_sizes(&self, selection: Option<u32>) -> Vec<[f64; 3]> {
        let selected = selection.and_then(|id| self.plates.iter().find(|plate| plate.id() == id));
        self.plates
            .iter()
            .filter_map(|plate| {
                let mut min = [f64::INFINITY; 3];
                let mut max = [f64::NEG_INFINITY; 3];
                for object in &self.objects {
                    if selected.is_some_and(|plate| {
                        !plate.instances().iter().any(|id| id[0] == object.id())
                    }) {
                        continue;
                    }
                    for instance in &object.instances {
                        if plate.instances().iter().any(|id| {
                            id[0] == instance.object_id()
                                && id[1] == instance.instance_id()
                                && Some(id[2]) == instance.saved_identify_id()
                        }) {
                            for point in world_vertices(object, instance.transform) {
                                merge_bbox_point(&mut min, &mut max, point);
                            }
                        }
                    }
                }
                min[0]
                    .is_finite()
                    .then(|| std::array::from_fn(|i| max[i] - min[i]))
            })
            .collect()
    }

    /// `PartPlate::translate_all_instance` (`OrcaSlicer.cpp:4037`): shift
    /// every instance offset by the shrink_to_new_bed plate re-centering
    /// delta.
    pub(crate) fn translate_all_instances(&mut self, shift: [f64; 2]) {
        if shift.iter().all(|value| value.abs() < EPSILON) {
            return;
        }
        for object in &mut self.objects {
            for instance in object
                .instances
                .iter_mut()
                .chain(&mut object.import_instances)
            {
                instance.transform = instance
                    .transform
                    .pretranslated(Point3d::new(shift[0], shift[1], 0.0));
            }
        }
    }

    /// `OrcaSlicer.cpp:4382-4392`: per object, instance 0's world bounding box
    /// min Z is subtracted from every instance offset.
    pub(crate) fn drop_instances_to_ground(&mut self) {
        for object in &mut self.objects {
            let Some(first) = object.instances.first() else {
                continue;
            };
            let mut min_z = f64::INFINITY;
            for point in world_vertices(object, first.transform) {
                min_z = min_z.min(point.z);
            }
            if !min_z.is_finite() {
                continue;
            }
            for instance in object
                .instances
                .iter_mut()
                .chain(&mut object.import_instances)
            {
                instance.transform = instance
                    .transform
                    .pretranslated(Point3d::new(0.0, 0.0, -min_z));
            }
        }
    }
}

/// `Model.cpp:1603-1618 instance_bounding_box`: model-part volume vertices
/// transformed by the instance × volume matrix.
fn world_vertices(
    object: &super::ProjectObject,
    instance: crate::Transform3d,
) -> impl Iterator<Item = Point3d> + '_ {
    object
        .volumes
        .iter()
        .filter(|volume| volume.volume_type() == super::ProjectVolumeType::ModelPart)
        .flat_map(move |volume| {
            let matrix = instance.then(volume.transform());
            volume
                .mesh()
                .vertices()
                .iter()
                .map(move |vertex| matrix.transform_point(*vertex))
        })
}

fn merge_bbox_point(min: &mut [f64; 3], max: &mut [f64; 3], point: Point3d) {
    for (axis, coordinate) in [point.x, point.y, point.z].into_iter().enumerate() {
        min[axis] = min[axis].min(coordinate);
        max[axis] = max[axis].max(coordinate);
    }
}
