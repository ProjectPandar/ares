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

#[cfg(test)]
mod tests {
    use crate::project::{
        domain::{Point3d, ProjectDocuments, ProjectMesh, ProjectVolume},
        filament_sequence::FilamentSequences,
        model_settings::ModelSettings,
        raw_settings::ProjectSettingsRaw,
        slice_info::{SliceInfo, SliceInfoHeader},
    };
    use crate::{
        Project, ProjectInstance, ProjectModel, ProjectObject, ProjectSettings, ProjectVolumeType,
        Transform3d,
    };

    /// One 10 mm cube volume at [0,10]^2 x [z, z+10], one instance at `offset`.
    fn project_with_instance(offset: [f64; 3], z: f64) -> Project {
        let vertices = [
            [0.0, 0.0, z],
            [10.0, 0.0, z],
            [0.0, 10.0, z],
            [10.0, 10.0, z],
            [0.0, 0.0, z + 10.0],
            [10.0, 0.0, z + 10.0],
            [0.0, 10.0, z + 10.0],
            [10.0, 10.0, z + 10.0],
        ]
        .map(|vertex| Point3d::new(vertex[0], vertex[1], vertex[2]));
        let triangles = [
            [0, 1, 2],
            [1, 3, 2],
            [4, 6, 5],
            [5, 6, 7],
            [0, 4, 1],
            [1, 4, 5],
            [2, 3, 6],
            [3, 7, 6],
            [0, 2, 4],
            [2, 6, 4],
            [1, 5, 3],
            [3, 5, 7],
        ];
        let volume = ProjectVolume::new(
            "cube".to_owned(),
            0,
            ProjectMesh::new(vertices.to_vec(), triangles.to_vec()),
            Transform3d::IDENTITY,
            (
                "cube".to_owned(),
                ProjectVolumeType::ModelPart,
                Default::default(),
                Transform3d::IDENTITY,
            ),
        );
        let instance = ProjectInstance::new(
            [1, 0, 0],
            true,
            false,
            Transform3d::IDENTITY.pretranslated(Point3d::new(offset[0], offset[1], offset[2])),
        );
        Project::new(
            vec![ProjectModel::new("cube".to_owned(), vec![2])],
            vec![ProjectObject::new(
                "cube".to_owned(),
                2,
                (
                    "cube".to_owned(),
                    String::new(),
                    Default::default(),
                    Default::default(),
                ),
                vec![volume],
                vec![instance],
            )],
            vec![crate::PlateMetadata::new(1, vec![[2, 0, 1]])],
            serde_json::from_str::<ProjectSettings>("{}").unwrap(),
            ProjectDocuments {
                model_settings: ModelSettings {
                    objects: Vec::new(),
                    plates: Vec::new(),
                    assemble: None,
                },
                project_settings_raw: ProjectSettingsRaw::default(),
                slice_info: SliceInfo {
                    header: SliceInfoHeader { items: Vec::new() },
                },
                filament_sequences: FilamentSequences(Default::default()),
                plate_documents: Vec::new(),
                has_painted_layer_height_profile: false,
                bbl_family_3mf: false,
            },
        )
    }

    #[test]
    fn centering_is_noop_within_epsilon() {
        // Model.cpp:712: |shift| < EPSILON (1e-4) leaves instances untouched.
        let mut project = project_with_instance([113.0, 133.0, 5.0], 5.0);
        let before = project.objects()[0].instances()[0].transform();
        let moved = project.center_instances_around_point([118.0, 138.0]);
        assert!(!moved, "already centered: world bbox center is (118,138)");
        assert_eq!(project.objects()[0].instances()[0].transform(), before);
    }
}
