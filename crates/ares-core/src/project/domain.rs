use super::{
    filament_sequence::FilamentSequences, model_settings::ModelSettings, plate::PlateJson,
    slice_info::SliceInfo, transform::Transform3d,
};
use crate::ProjectSettings;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point3d {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Point3d {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProjectMesh {
    vertices: Vec<Point3d>,
    triangles: Vec<[u32; 3]>,
}

impl ProjectMesh {
    pub(crate) fn new(vertices: Vec<Point3d>, triangles: Vec<[u32; 3]>) -> Self {
        Self {
            vertices,
            triangles,
        }
    }

    pub fn vertices(&self) -> &[Point3d] {
        &self.vertices
    }

    pub fn triangles(&self) -> &[[u32; 3]] {
        &self.triangles
    }
}

#[derive(Debug)]
pub struct Project {
    models: Vec<ProjectModel>,
    objects: Vec<ProjectObject>,
    plates: Vec<PlateMetadata>,
    settings: ProjectSettings,
    documents: ProjectDocuments,
}

impl Project {
    pub(crate) fn new(
        models: Vec<ProjectModel>,
        objects: Vec<ProjectObject>,
        plates: Vec<PlateMetadata>,
        settings: ProjectSettings,
        documents: ProjectDocuments,
    ) -> Self {
        Self {
            models,
            objects,
            plates,
            settings,
            documents,
        }
    }

    pub fn models(&self) -> &[ProjectModel] {
        &self.models
    }

    pub fn objects(&self) -> &[ProjectObject] {
        &self.objects
    }

    pub fn plates(&self) -> &[PlateMetadata] {
        &self.plates
    }

    pub fn settings(&self) -> &ProjectSettings {
        &self.settings
    }

    pub(crate) fn documents(&self) -> &ProjectDocuments {
        &self.documents
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProjectModel {
    path: String,
    object_ids: Vec<u32>,
}

impl ProjectModel {
    pub(crate) fn new(path: String, object_ids: Vec<u32>) -> Self {
        Self { path, object_ids }
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn object_ids(&self) -> &[u32] {
        &self.object_ids
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProjectObject {
    source_model_path: String,
    id: u32,
    volumes: Vec<ProjectVolume>,
    instances: Vec<ProjectInstance>,
}

impl ProjectObject {
    pub(crate) fn new(
        source_model_path: String,
        id: u32,
        volumes: Vec<ProjectVolume>,
        instances: Vec<ProjectInstance>,
    ) -> Self {
        Self {
            source_model_path,
            id,
            volumes,
            instances,
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn source_model_path(&self) -> &str {
        &self.source_model_path
    }

    pub fn volumes(&self) -> &[ProjectVolume] {
        &self.volumes
    }

    pub fn instances(&self) -> &[ProjectInstance] {
        &self.instances
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProjectVolume {
    source_model_path: String,
    id: u32,
    mesh: ProjectMesh,
    transform: Transform3d,
    source_transform: Transform3d,
}

impl ProjectVolume {
    pub(crate) fn new(
        source_model_path: String,
        id: u32,
        mesh: ProjectMesh,
        transform: Transform3d,
        source_transform: Transform3d,
    ) -> Self {
        Self {
            source_model_path,
            id,
            mesh,
            transform,
            source_transform,
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn source_model_path(&self) -> &str {
        &self.source_model_path
    }

    pub fn mesh(&self) -> &ProjectMesh {
        &self.mesh
    }

    pub fn transform(&self) -> Transform3d {
        self.transform
    }

    pub fn source_transform(&self) -> Transform3d {
        self.source_transform
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProjectInstance {
    object_id: u32,
    instance_id: u32,
    loaded_label_id: u32,
    printable: bool,
    auto_drop: bool,
    transform: Transform3d,
}

impl ProjectInstance {
    pub(crate) fn new(
        identity: [u32; 3],
        printable: bool,
        auto_drop: bool,
        transform: Transform3d,
    ) -> Self {
        Self {
            object_id: identity[0],
            instance_id: identity[1],
            loaded_label_id: identity[2],
            printable,
            auto_drop,
            transform,
        }
    }

    pub fn object_id(&self) -> u32 {
        self.object_id
    }

    pub fn instance_id(&self) -> u32 {
        self.instance_id
    }

    pub fn loaded_label_id(&self) -> u32 {
        self.loaded_label_id
    }

    pub fn printable(&self) -> bool {
        self.printable
    }

    pub fn auto_drop(&self) -> bool {
        self.auto_drop
    }

    pub fn transform(&self) -> Transform3d {
        self.transform
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PlateMetadata {
    id: u32,
    instances: Vec<[u32; 3]>,
}

impl PlateMetadata {
    pub(crate) fn new(id: u32, instances: Vec<[u32; 3]>) -> Self {
        Self { id, instances }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn instances(&self) -> &[[u32; 3]] {
        &self.instances
    }
}

#[derive(Debug)]
pub(crate) struct ProjectDocuments {
    pub model_settings: ModelSettings,
    pub slice_info: SliceInfo,
    pub filament_sequences: FilamentSequences,
    pub plate_documents: Vec<PlateJson>,
}
