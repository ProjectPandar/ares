mod documents;
mod volume_offsets;

pub use documents::PlateMetadata;
pub(crate) use documents::ProjectDocuments;
pub(crate) use volume_offsets::normalize_volume_offsets;

use super::{layer_config_ranges::LayerConfigRange, transform::Transform3d};
use crate::{
    ProjectSettings, SliceError,
    options::{ObjectOptionOverrides, RegionOptionOverrides},
};

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
    pub(crate) supported_facets: super::facets_annotation::FacetsAnnotation,
}

impl ProjectMesh {
    pub(crate) fn new(vertices: Vec<Point3d>, triangles: Vec<[u32; 3]>) -> Self {
        Self {
            vertices,
            triangles,
            supported_facets: super::facets_annotation::FacetsAnnotation::default(),
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

    /// Projects slice one plate at a time (OrcaSlicer exports `plate_<n>.gcode`
    /// per plate). Returns a view containing only the instances of `plate_id`.
    pub(crate) fn select_plate(&self, plate_id: u32) -> Result<Self, SliceError> {
        let index = self
            .plates
            .iter()
            .position(|plate| plate.id() == plate_id)
            .ok_or_else(|| SliceError::InvalidInput(format!("project has no plate {plate_id}")))?;
        let identity = self.plates[index]
            .instances()
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>();
        let objects = self
            .objects
            .iter()
            .filter_map(|object| {
                let mut instances = object.instances().to_vec();
                instances.retain(|instance| {
                    identity.contains(&[
                        instance.object_id(),
                        instance.instance_id(),
                        instance.loaded_label_id(),
                    ])
                });
                (!instances.is_empty()).then(|| {
                    let mut object = object.clone();
                    object.instances = instances;
                    object
                })
            })
            .collect::<Vec<_>>();
        if objects.is_empty() {
            return Err(SliceError::InvalidInput(format!(
                "plate {plate_id} has no printable objects"
            )));
        }
        let mut documents = self.documents.clone_shallow();
        documents.plate_documents = vec![self.documents.plate_documents.get(index).cloned()]
            .into_iter()
            .flatten()
            .collect();
        // The plate's own config entries (stored as `model_settings.config`
        // plate metadata) are applied over the project settings
        // (`OrcaSlicer.cpp:5979` `new_print_config.apply(*part_plate->config())`;
        // keys listed in `BBS_3MF_Importer`, `bbs_3mf.cpp:4418-4565`).
        let mut settings = self.settings.clone();
        if let Some(plate_settings) = self.documents.model_settings.plates.get(index) {
            super::plate_config::apply_plate_config_overrides(
                &mut settings,
                &plate_settings.metadata,
            );
        }
        Ok(Self {
            models: self.models.clone(),
            objects,
            plates: vec![self.plates[index].clone()],
            settings,
            documents,
        })
    }

    pub fn settings(&self) -> &ProjectSettings {
        &self.settings
    }

    pub(crate) fn documents(&self) -> &ProjectDocuments {
        &self.documents
    }

    pub(crate) fn has_painted_layer_height_profile(&self) -> bool {
        self.documents.has_painted_layer_height_profile
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
    name: String,
    module: String,
    object_overrides: ObjectOptionOverrides,
    region_overrides: RegionOptionOverrides,
    layer_config_ranges: Vec<LayerConfigRange>,
    volumes: Vec<ProjectVolume>,
    instances: Vec<ProjectInstance>,
}

impl ProjectObject {
    pub(crate) fn new(
        source_model_path: String,
        id: u32,
        metadata: (String, String, ObjectOptionOverrides, RegionOptionOverrides),
        volumes: Vec<ProjectVolume>,
        instances: Vec<ProjectInstance>,
    ) -> Self {
        Self {
            source_model_path,
            id,
            name: metadata.0,
            module: metadata.1,
            object_overrides: metadata.2,
            region_overrides: metadata.3,
            layer_config_ranges: Vec::new(),
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

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn module(&self) -> &str {
        &self.module
    }

    pub(crate) fn object_overrides(&self) -> &ObjectOptionOverrides {
        &self.object_overrides
    }

    pub(crate) fn region_overrides(&self) -> &RegionOptionOverrides {
        &self.region_overrides
    }

    pub fn layer_config_ranges(&self) -> &[LayerConfigRange] {
        &self.layer_config_ranges
    }

    pub(crate) fn set_layer_config_ranges(&mut self, ranges: Vec<LayerConfigRange>) {
        self.layer_config_ranges = ranges;
    }

    pub fn volumes(&self) -> &[ProjectVolume] {
        &self.volumes
    }

    pub fn instances(&self) -> &[ProjectInstance] {
        &self.instances
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectVolumeType {
    ModelPart,
    NegativeVolume,
    ParameterModifier,
    SupportEnforcer,
    SupportBlocker,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProjectVolume {
    source_model_path: String,
    id: u32,
    mesh: ProjectMesh,
    transform: Transform3d,
    name: String,
    volume_type: ProjectVolumeType,
    region_overrides: RegionOptionOverrides,
    source_transform: Transform3d,
    mesh_shared: bool,
}

impl ProjectVolume {
    pub(crate) fn new(
        source_model_path: String,
        id: u32,
        mesh: ProjectMesh,
        transform: Transform3d,
        metadata: (
            String,
            ProjectVolumeType,
            RegionOptionOverrides,
            Transform3d,
        ),
    ) -> Self {
        Self {
            source_model_path,
            id,
            mesh,
            transform,
            name: metadata.0,
            volume_type: metadata.1,
            region_overrides: metadata.2,
            source_transform: metadata.3,
            mesh_shared: false,
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

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn volume_type(&self) -> ProjectVolumeType {
        self.volume_type
    }

    pub(crate) fn region_overrides(&self) -> &RegionOptionOverrides {
        &self.region_overrides
    }

    pub fn source_transform(&self) -> Transform3d {
        self.source_transform
    }

    pub(crate) fn has_mesh_shared(&self) -> bool {
        self.mesh_shared
    }

    pub(crate) fn set_mesh_shared(&mut self, mesh_shared: bool) {
        self.mesh_shared = mesh_shared;
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
