mod auto_orient;
mod documents;
mod layer_height_profile;
mod mesh;
mod multipart;
mod placement;
mod shared_mesh;
mod sharing;
pub use mesh::ProjectMesh;
use shared_mesh::MeshPreparation;
pub(crate) use shared_mesh::materialize_meshes;
mod import_scope;
mod instance;
pub(crate) use instance::InstancePvs;
mod instance_copies;
mod model_snapshot;
pub(crate) use model_snapshot::ModelSnapshot;
pub(super) mod object_id;
mod plate_selection;
mod plater_edits;
use import_scope::ImportScope;
pub(crate) use plate_selection::{compute_colum_count, plate_dimensions, plate_origin_at};
mod preset_refresh;
mod profile_input;
mod volume_offsets;

pub use documents::PlateMetadata;
pub(crate) use documents::ProjectDocuments;
pub(crate) use volume_offsets::normalize_volume_offsets;

use super::{layer_config_ranges::LayerConfigRange, transform::Transform3d};
use crate::{
    ProjectSettings,
    options::{ObjectOptionOverrides, RegionOptionOverrides},
};

pub(crate) use profile_input::SelectedProfile;

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

#[derive(Debug)]
pub struct Project {
    models: Vec<ProjectModel>,
    objects: Vec<ProjectObject>,
    plates: Vec<PlateMetadata>,
    settings: ProjectSettings,
    documents: ProjectDocuments,
    import_scope: ImportScope,
    plate_view: bool,
    /// The selected plate's grid origin in the project-global coordinate
    /// space (`PartPlate.cpp:3914-3962`; zero until a plate is selected).
    /// Instances stay in global coordinates upstream
    /// (`OrcaSlicer.cpp:4837-4845`), so the volume gate tests them against
    /// the bed shape translated to this origin (`PartPlate.cpp:3179-3187`)
    /// and the G-code emitter subtracts it again
    /// (`Print.cpp:2636-2638` `set_gcode_offset`).
    plate_origin: (f64, f64),
    pub(in crate::project) plates_custom_gcodes:
        std::collections::BTreeMap<i32, super::plate_custom_gcodes::CustomGCodeInfo>,
    /// `Model::curr_plate_index` (`Model.hpp:1562`): the selected plate's position.
    pub(in crate::project) curr_plate_index: i32,
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
            import_scope: ImportScope::All,
            plate_view: false,
            plate_origin: (0.0, 0.0),
            plates_custom_gcodes: Default::default(),
            curr_plate_index: 0,
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

    pub(crate) fn initialize_plate(&mut self, instances: Vec<[u32; 3]>) {
        self.plates = vec![PlateMetadata::new(1, instances)];
    }

    /// The selected plate's grid origin (`PartPlateList::compute_origin`,
    /// `PartPlate.cpp:3914-3921`). Zero for unselected projects.
    pub(crate) fn plate_origin(&self) -> (f64, f64) {
        self.plate_origin
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
    layer_height_profile: Vec<f64>,
    volumes: Vec<ProjectVolume>,
    instances: Vec<ProjectInstance>,
    import_instances: Vec<ProjectInstance>,
    pub(in crate::project) brim_points: Vec<crate::project::brim_ear_points::BrimPoint>,
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
            layer_height_profile: Vec::new(),
            volumes,
            import_instances: instances.clone(),
            instances,
            brim_points: Vec::new(),
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

    /// PrintApply.cpp:998 and PrintObjectSlice.cpp:1208–1210.
    pub(crate) fn is_mm_painted(&self, logical_filament_count: usize) -> bool {
        logical_filament_count > 1
            && self
                .volumes
                .iter()
                .any(|v| !v.mesh().mmu_segmentation_facets.mapping.is_empty())
    }

    pub(crate) fn is_fuzzy_skin_painted(&self) -> bool {
        self.volumes
            .iter()
            .any(|volume| !volume.mesh().fuzzy_skin_facets.mapping.is_empty())
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
    mesh_preparation: MeshPreparation,
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
            mesh_preparation: MeshPreparation::new(&mesh, transform, id as i32),
            mesh,
            transform,
            name: metadata.0,
            volume_type: metadata.1,
            region_overrides: metadata.2,
            source_transform: metadata.3,
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

    pub(crate) fn set_mesh_preparation(
        &mut self,
        shift: Point3d,
        component: Transform3d,
        key: i32,
    ) {
        self.mesh_preparation.init_shift = shift;
        self.mesh_preparation.component = component;
        self.mesh_preparation.key = key;
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProjectInstance {
    object_id: u32,
    instance_id: u32,
    /// Loaded plate identity or the ObjectBase ID allocated by a fresh model import.
    label: instance::InstanceLabel,
    printable: bool,
    auto_drop: bool,
    transform: Transform3d,
    /// `ModelInstance::print_volume_state` (`Model.hpp:1430`), `Inside` until the CLI's
    /// `update_print_volume_state` stores it (`Model.hpp:1564`).
    print_volume_state: InstancePvs,
}
