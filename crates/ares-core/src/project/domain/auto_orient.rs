//! `ModelObject::rotate`, `ModelVolume::rotate`, `center_around_origin`,
//! `ensure_on_bed` (`Model.cpp:1883-1921,1967-1980,3098-3101`).
use super::{Point3d, Project, ProjectVolumeType};
use crate::plater::transformation::{rotation, with_rotation};

impl Project {
    pub(crate) fn apply_object_orientation(&mut self, index: usize, delta: [f64; 3]) {
        let object = &mut self.objects[index];
        for volume in &mut object.volumes {
            let angles = rotation(volume.transform);
            volume.transform = with_rotation(
                volume.transform,
                std::array::from_fn(|i| angles[i] + delta[i]),
            );
        }
        // The default center_around_origin includes modifiers, unlike the scoring mesh.
        let mut min = [f64::INFINITY; 3];
        let mut max = [f64::NEG_INFINITY; 3];
        let points = object.volumes.iter().flat_map(|volume| {
            volume
                .mesh
                .vertices()
                .iter()
                .map(move |vertex| volume.transform.transform_point(*vertex))
        });
        for point in points {
            for (axis, value) in [point.x, point.y, point.z].into_iter().enumerate() {
                min[axis] = min[axis].min(value);
                max[axis] = max[axis].max(value);
            }
        }
        let shift = Point3d::new(
            -(min[0] + max[0]) * 0.5,
            -(min[1] + max[1]) * 0.5,
            -(min[2] + max[2]) * 0.5,
        );
        for volume in &mut object.volumes {
            volume.transform = volume.transform.pretranslated(shift);
        }
        // The legacy build-item carrier is object-wide: any auto_drop="0" wins.
        if !object.instances.iter().all(|instance| instance.auto_drop) {
            return;
        }
        let mut min_z = f64::INFINITY;
        for instance in &object.instances {
            for volume in object
                .volumes
                .iter()
                .filter(|volume| volume.volume_type == ProjectVolumeType::ModelPart)
            {
                let transform = instance.transform.then(volume.transform);
                min_z = volume
                    .mesh
                    .vertices()
                    .iter()
                    .map(|vertex| transform.transform_point(*vertex).z)
                    .fold(min_z, f64::min);
            }
        }
        for instance in object
            .instances
            .iter_mut()
            .chain(&mut object.import_instances)
        {
            instance.transform = instance
                .transform
                .pretranslated(Point3d::new(0., 0., -min_z));
        }
    }
}
