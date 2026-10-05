//! STL Model import (`Format/STL.cpp:17-40`, `Model.cpp:485-500,1254-1264,2600-2615`).

use std::collections::HashMap;

use super::{
    Point3d, Project, ProjectInstance, ProjectMesh, ProjectModel, ProjectObject, ProjectVolume,
    ProjectVolumeType, Transform3d,
    documents::{PlateMetadata, ProjectDocuments},
};
use crate::{
    ProjectSettings, SliceError,
    project::{
        filament_sequence::FilamentSequences,
        model_settings::ModelSettings,
        raw_settings::ProjectSettingsRaw,
        slice_info::{SliceInfo, SliceInfoHeader},
    },
};

impl Project {
    pub(crate) fn initialize_cli_loaded_project_config(&mut self) {
        // OrcaSlicer.cpp:1769: the CLI's 3MF import reads this dynamic option
        // with create=true. STL --load-settings does not perform that read.
        let raw = &mut self.documents.project_settings_raw;
        if !raw.contains_key("upward_compatible_machine") {
            raw.insert_strings(
                "upward_compatible_machine",
                &self.settings.printer.remaining.upward_compatible_machine.0,
            );
        }
    }

    pub(crate) fn from_stl_with_profiles(
        input: &[u8],
        filename: &str,
        settings: ProjectSettings,
        raw: ProjectSettingsRaw,
    ) -> Result<Self, SliceError> {
        let model = crate::load_model(input)?;
        let (vertices, triangles) = index_stl_vertices(&model);
        let mut min = [f64::INFINITY; 3];
        let mut max = [f64::NEG_INFINITY; 3];
        for vertex in &vertices {
            for axis in 0..3 {
                let coordinate = f64::from(vertex[axis]);
                min[axis] = min[axis].min(coordinate);
                max[axis] = max[axis].max(coordinate);
            }
        }
        let center: [f64; 3] = std::array::from_fn(|axis| (min[axis] + max[axis]) * 0.5);
        let vertices = vertices
            .into_iter()
            .map(|vertex| {
                let shifted: [f64; 3] =
                    std::array::from_fn(|axis| f64::from(vertex[axis] - center[axis] as f32));
                Point3d::new(shifted[0], shifted[1], shifted[2])
            })
            .collect();
        let transform =
            Transform3d::IDENTITY.pretranslated(Point3d::new(center[0], center[1], center[2]));
        let volume = ProjectVolume::new(
            filename.to_owned(),
            0,
            ProjectMesh::new(vertices, triangles),
            transform,
            (
                filename.to_owned(),
                ProjectVolumeType::ModelPart,
                Default::default(),
                Transform3d::IDENTITY,
            ),
        );
        let instance = ProjectInstance::new([1, 0, 0], true, false, Transform3d::IDENTITY);
        let object = ProjectObject::new(
            filename.to_owned(),
            1,
            (
                filename.to_owned(),
                String::new(),
                Default::default(),
                Default::default(),
            ),
            vec![volume],
            vec![instance],
        );
        Ok(Self::new(
            vec![ProjectModel::new(filename.to_owned(), vec![1])],
            vec![object],
            vec![PlateMetadata::new(1, vec![[1, 0, 0]])],
            settings,
            ProjectDocuments {
                model_settings: ModelSettings {
                    objects: Vec::new(),
                    plates: Vec::new(),
                    assemble: None,
                },
                project_settings_raw: raw,
                slice_info: SliceInfo {
                    header: SliceInfoHeader { items: Vec::new() },
                },
                filament_sequences: FilamentSequences(Default::default()),
                plate_documents: Vec::new(),
                has_painted_layer_height_profile: false,
            },
        ))
    }
}

fn index_stl_vertices(model: &crate::Model) -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
    let mut vertices = Vec::new();
    let mut index = HashMap::new();
    let mut triangles = Vec::with_capacity(model.triangles().len());
    for triangle in model.triangles() {
        let ids = triangle.vertices().map(|vertex| {
            let coordinates = [vertex.x, vertex.y, vertex.z];
            *index.entry(vertex_key(coordinates)).or_insert_with(|| {
                let id = vertices.len() as u32;
                vertices.push(coordinates);
                id
            })
        });
        triangles.push(ids);
    }
    (vertices, triangles)
}

fn vertex_key(coordinates: [f32; 3]) -> [u32; 3] {
    coordinates.map(|value| if value == 0.0 { 0 } else { value.to_bits() })
}
