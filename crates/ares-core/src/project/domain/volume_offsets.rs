//! Single-volume offset redistribution (`Format/bbs_3mf.cpp:2187-2207`).

use super::{Point3d, ProjectObject};

pub(crate) fn normalize_volume_offsets(objects: &mut [ProjectObject]) {
    let origin = Point3d::new(0.0, 0.0, 0.0);
    for object in objects {
        let [volume] = object.volumes.as_mut_slice() else {
            continue;
        };
        let volume_offset = volume.transform.transform_point(origin);
        let first = object.instances[0].transform;
        let world_volume = first.translated(volume_offset).transform_point(origin);
        let world_instance = first.transform_point(origin);
        let world_volume = [world_volume.x, world_volume.y, world_volume.z];
        let world_instance = [world_instance.x, world_instance.y, world_instance.z];
        let difference = (0..3)
            .map(|axis| (world_volume[axis] - world_instance[axis]).powi(2))
            .sum::<f64>();
        let volume_norm = world_volume
            .into_iter()
            .map(|value| value * value)
            .sum::<f64>();
        let instance_norm = world_instance
            .into_iter()
            .map(|value| value * value)
            .sum::<f64>();
        if difference <= 1e-24 * volume_norm.min(instance_norm) {
            continue;
        }
        for instance in &mut object.instances {
            instance.transform = instance.transform.translated(volume_offset);
        }
        volume.transform = volume.transform.without_translation();
    }
}
