//! Fresh STL volumes, `Model::convert_multipart_object` and the multi-volume
//! `ModelObject::split` branch (OrcaSlicer v2.5.0 Model.cpp:926–977,2220–2346).

use super::{
    Point3d, Project, ProjectInstance, ProjectMesh, ProjectModel, ProjectObject, ProjectVolume,
    ProjectVolumeType, Transform3d,
    documents::{PlateMetadata, ProjectDocuments},
};
use crate::{SliceError, StlFile, StlImportOptions};

pub(super) fn stl_volume(
    input: &[u8],
    name: &str,
    id: u32,
) -> Result<(ProjectVolume, Option<f32>), SliceError> {
    let (model, solid_volume) = crate::load_stl_model(input)?;
    let (vertices, triangles) = super::profile_input::index_stl_vertices(&model);
    let mut min = [f64::INFINITY; 3];
    let mut max = [f64::NEG_INFINITY; 3];
    for vertex in &vertices {
        for axis in 0..3 {
            min[axis] = min[axis].min(f64::from(vertex[axis]));
            max[axis] = max[axis].max(f64::from(vertex[axis]));
        }
    }
    let center: [f64; 3] = std::array::from_fn(|axis| (min[axis] + max[axis]) * 0.5);
    let vertices = vertices
        .into_iter()
        .map(|vertex| {
            let point: [f64; 3] =
                std::array::from_fn(|axis| f64::from(vertex[axis] - center[axis] as f32));
            Point3d::new(point[0], point[1], point[2])
        })
        .collect();
    let volume = ProjectVolume::new(
        name.to_owned(),
        id,
        ProjectMesh::new(vertices, triangles, Vec::new()),
        Transform3d::IDENTITY.pretranslated(Point3d::new(center[0], center[1], center[2])),
        (
            name.to_owned(),
            ProjectVolumeType::ModelPart,
            Default::default(),
            Transform3d::IDENTITY,
        ),
    );
    Ok((volume, solid_volume))
}

impl Project {
    pub(crate) fn from_cli_stl_files(files: &[StlFile<'_>]) -> Result<Self, SliceError> {
        let mut project = Self::from_stl_files(files)?;
        let mut labels = super::object_id::ObjectIdCounter::cli_stl_merge(files.len() as u32);
        let mut instances = Vec::with_capacity(files.len());
        for object in &mut project.objects {
            let identity = [object.id, 0, labels.clone_stl_instance()];
            object.instances.push(ProjectInstance::new(
                identity,
                true,
                true,
                Transform3d::IDENTITY,
            ));
            instances.push(identity);
        }
        project.initialize_plate(instances);
        project.drop_instances_to_ground();
        Ok(project)
    }

    pub(crate) fn from_stl_files(files: &[StlFile<'_>]) -> Result<Self, SliceError> {
        if files.len() < 2 {
            return Err(SliceError::InvalidInput(
                "select at least two STL files".to_owned(),
            ));
        }
        let mut objects = Vec::with_capacity(files.len());
        for (index, file) in files.iter().enumerate() {
            if !file.name.to_ascii_lowercase().ends_with(".stl") {
                return Err(SliceError::InvalidInput(
                    "multiple inputs must all be STL files".to_owned(),
                ));
            }
            let (volume, _) = stl_volume(file.bytes, file.name, index as u32)?;
            objects.push(ProjectObject::new(
                file.name.to_owned(),
                index as u32 + 1,
                (
                    file.name.to_owned(),
                    String::new(),
                    Default::default(),
                    Default::default(),
                ),
                vec![volume],
                Vec::new(),
            ));
        }
        Ok(Self::new(
            vec![ProjectModel::new(
                files[0].name.to_owned(),
                (1..=files.len() as u32).collect(),
            )],
            objects,
            vec![PlateMetadata::new(1, Vec::new())],
            Default::default(),
            ProjectDocuments {
                model_settings: crate::project::model_settings::ModelSettings {
                    objects: Vec::new(),
                    plates: Vec::new(),
                    assemble: None,
                },
                project_settings_raw: Default::default(),
                slice_info: crate::project::slice_info::SliceInfo {
                    header: crate::project::slice_info::SliceInfoHeader { items: Vec::new() },
                },
                filament_sequences: crate::project::filament_sequence::FilamentSequences(
                    Default::default(),
                ),
                plate_documents: Vec::new(),
                has_painted_layer_height_profile: false,
                bbl_family_3mf: false,
                preset_file_version: None,
                preset_selection: None,
            },
        ))
    }

    pub(crate) fn copy_stl_model(&self) -> Self {
        Self::new(
            self.models.clone(),
            self.objects.clone(),
            self.plates.clone(),
            self.settings.clone(),
            self.documents.clone_shallow(),
        )
    }

    pub(crate) fn place_stl_files(&mut self, options: StlImportOptions) {
        // Plater.cpp:10075–10125 combines before placement, including No/Off.
        if options.as_multipart || !options.auto_drop {
            let first = &self.objects[0];
            let name = first.name.rsplit_once('.').unwrap().0.to_owned();
            let path = first.source_model_path.clone();
            let volumes = std::mem::take(&mut self.objects)
                .into_iter()
                .flat_map(|o| o.volumes)
                .collect();
            self.objects.push(ProjectObject::new(
                path,
                1,
                (name, String::new(), Default::default(), Default::default()),
                volumes,
                Vec::new(),
            ));
        }
        let area = &self.settings.printer.remaining.printable_area.0;
        let min_x = area.iter().map(|p| p.x).fold(f64::INFINITY, f64::min);
        let min_y = area.iter().map(|p| p.y).fold(f64::INFINITY, f64::min);
        let max_x = area.iter().map(|p| p.x).fold(f64::NEG_INFINITY, f64::max);
        let max_y = area.iter().map(|p| p.y).fold(f64::NEG_INFINITY, f64::max);
        let bed = [(min_x + max_x) * 0.5, (min_y + max_y) * 0.5];
        for object in &mut self.objects {
            let mut min = [f64::INFINITY; 3];
            let mut max = [f64::NEG_INFINITY; 3];
            let vertices = object.volumes.iter().flat_map(|volume| {
                volume
                    .mesh
                    .vertices()
                    .iter()
                    .map(|vertex| volume.transform.transform_point(*vertex))
            });
            for point in vertices {
                let position = [point.x, point.y, point.z];
                min = std::array::from_fn(|axis| min[axis].min(position[axis]));
                max = std::array::from_fn(|axis| max[axis].max(position[axis]));
            }
            let center: [f64; 3] = std::array::from_fn(|axis| (min[axis] + max[axis]) * 0.5);
            for volume in &mut object.volumes {
                volume.transform = volume
                    .transform
                    .pretranslated(Point3d::new(-center[0], -center[1], -center[2]));
                let transform = volume.transform;
                volume.set_mesh_preparation(
                    Point3d::new(0.0, 0.0, 0.0),
                    transform,
                    volume.id as i32,
                );
            }
            let lift = if options.auto_drop {
                -min[2]
            } else {
                -min[2].min(0.0)
            };
            let transform =
                Transform3d::IDENTITY.pretranslated(Point3d::new(bed[0], bed[1], center[2] + lift));
            object.instances = vec![ProjectInstance::new(
                [object.id, 0, object.id],
                true,
                options.auto_drop,
                transform,
            )];
            object.import_instances = object.instances.clone();
        }
        if !options.as_multipart && !options.auto_drop {
            let mut combined = self.objects.pop().unwrap();
            for (index, mut volume) in combined.volumes.drain(..).enumerate() {
                let offset = volume
                    .transform
                    .transform_point(Point3d::new(0.0, 0.0, 0.0));
                let transform = combined.instances[0].transform.pretranslated(offset);
                volume.transform = volume.transform.without_translation();
                volume.set_mesh_preparation(
                    Point3d::new(0.0, 0.0, 0.0),
                    volume.transform,
                    volume.id as i32,
                );
                let id = index as u32 + 1;
                let instance = ProjectInstance::new([id, 0, id], true, false, transform);
                self.objects.push(ProjectObject::new(
                    volume.source_model_path.clone(),
                    id,
                    (
                        volume.name.clone(),
                        String::new(),
                        Default::default(),
                        Default::default(),
                    ),
                    vec![volume],
                    vec![instance],
                ));
            }
        }
        self.models[0].object_ids = self.objects.iter().map(|o| o.id).collect();
        self.plates = vec![PlateMetadata::new(
            1,
            self.objects.iter().map(|o| [o.id, 0, o.id]).collect(),
        )];
    }
}
