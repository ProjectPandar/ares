//! Model volume copying and fixed plate construction (`OrcaSlicer.cpp:732-760,823-1097,4911-4936`).
mod list;

use std::collections::{BTreeMap, HashMap};

use super::{
    PlateMetadata, Point3d, Project, ProjectDocuments, ProjectInstance, ProjectModel,
    ProjectObject, ProjectVolume, Transform3d, multipart::stl_volume, plate_selection,
};
use crate::{OrcaInt, SliceError};

pub use list::AssemblyList;

#[derive(Debug)]
pub enum AssemblyError {
    Invalid(String),
    Missing(String),
    Config(String),
    Data(String),
}

impl AssemblyError {
    pub fn return_code(&self) -> i32 {
        match self {
            Self::Invalid(_) => -2,
            Self::Missing(_) => -3,
            Self::Config(_) => -5,
            Self::Data(_) => -6,
        }
    }

    pub fn error_string(&self) -> &'static str {
        match self {
            Self::Invalid(_) => "Invalid parameters to the slicer.",
            Self::Missing(_) => "The input files to the slicer are not found.",
            Self::Config(_) => "The input preset file is invalid and can not be parsed.",
            Self::Data(_) => "The input model file to the slicer can not be parsed.",
        }
    }
}

impl std::fmt::Display for AssemblyError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::Invalid(message)
            | Self::Missing(message)
            | Self::Config(message)
            | Self::Data(message) => message,
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for AssemblyError {}

#[derive(Clone, Copy, Debug)]
pub struct AssemblyFile<'a> {
    pub path: &'a str,
    pub bytes: &'a [u8],
}

/// A fixed-placement assembly, ready for profile application and plate slicing.
#[derive(Debug)]
pub struct Assembly<'a>(pub(crate) AssemblySource<'a>);

#[derive(Debug)]
pub(crate) enum AssemblySource<'a> {
    Model(Box<Project>),
    Archive(AssemblyFile<'a>),
}

impl Assembly<'_> {
    pub fn archive(&self) -> Option<AssemblyFile<'_>> {
        match self.0 {
            AssemblySource::Archive(file) => Some(file),
            AssemblySource::Model(_) => None,
        }
    }
}

fn has_suffix(path: &str, suffix: &str) -> bool {
    path.rsplit_once('.')
        .is_some_and(|(_, ext)| ext.eq_ignore_ascii_case(suffix))
}

impl<'a> Assembly<'a> {
    pub fn from_models(
        files: &[AssemblyFile<'a>],
        filament_ids: &[i32],
    ) -> Result<Self, AssemblyError> {
        if files.is_empty() {
            return Err(AssemblyError::Invalid(
                "Invalid params: --assemble needs at least one input model.".into(),
            ));
        }
        if !filament_ids.is_empty() && filament_ids.len() != files.len() {
            return Err(AssemblyError::Invalid(format!(
                "loaded_filament_ids size {} should be the same with input files size {}",
                filament_ids.len(),
                files.len()
            )));
        }
        if let [file] = files
            && has_suffix(file.path, "3mf")
        {
            if !filament_ids.is_empty() {
                return Err(AssemblyError::Invalid(
                    "can not load 3mf when set loaded_filament_ids or clone_objects".into(),
                ));
            }
            return Ok(Self(AssemblySource::Archive(*file)));
        }
        let volumes = files
            .iter()
            .enumerate()
            .map(|(index, file)| {
                if !has_suffix(file.path, "stl") {
                    return Err(AssemblyError::Invalid(
                        "positional assembly requires STL inputs or one 3MF".into(),
                    ));
                }
                let mut volume = read_stl(*file, basename(file.path), index as u32)?;
                set_volume_extruder(
                    &mut volume,
                    filament_ids.get(index).copied().unwrap_or(0).max(0),
                );
                Ok(volume)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let object = object(1, "Assembly".to_owned(), volumes, None);
        Ok(Self(AssemblySource::Model(Box::new(model(
            vec![object],
            vec![vec![1]],
            Vec::new(),
        )))))
    }

    pub fn from_list(
        list: &AssemblyList,
        files: &[AssemblyFile<'_>],
    ) -> Result<Self, AssemblyError> {
        let files = files
            .iter()
            .map(|file| (file.path, *file))
            .collect::<HashMap<_, _>>();
        let mut objects: Vec<ProjectObject> = Vec::new();
        let mut membership = Vec::new();
        let mut names = Vec::new();
        let mut volume_id = 0;
        for (plate_index, plate) in list.plates.iter().enumerate() {
            let start = objects.len();
            append_plate_objects(&mut objects, &mut volume_id, &files, plate, plate_index)?;
            membership.push(objects[start..].iter().map(ProjectObject::id).collect());
            names.push(plate.plate_name.clone());
        }
        Ok(Self(AssemblySource::Model(Box::new(model(
            objects, membership, names,
        )))))
    }
}

fn append_plate_objects(
    objects: &mut Vec<ProjectObject>,
    volume_id: &mut u32,
    files: &HashMap<&str, AssemblyFile<'_>>,
    plate: &list::Plate,
    plate_index: usize,
) -> Result<(), AssemblyError> {
    let mut groups = BTreeMap::new();
    for (index, entry) in plate.objects.iter().enumerate() {
        let file = *files.get(entry.path.as_str()).ok_or_else(|| {
            AssemblyError::Missing(format!(
                "construct_assemble_list: directory {} not exist in plate {}",
                entry.path,
                plate_index + 1
            ))
        })?;
        if !has_suffix(file.path, "stl") {
            return Err(AssemblyError::Invalid(format!(
                "construct_assemble_list: unsupported file {}, plate index {}, object index {}",
                entry.path,
                plate_index + 1,
                index + 1
            )));
        }
        let stem = basename(file.path).rsplit_once('.').unwrap().0;
        let source = read_stl(file, stem, *volume_id)?;
        for copy in 0..entry.count as usize {
            let mut volume = source.clone();
            volume.id = *volume_id;
            let [x, y, z] = entry.translation(copy);
            volume.transform = volume.transform.pretranslated(Point3d::new(x, y, z));
            volume.set_mesh_preparation(
                Point3d::new(0.0, 0.0, 0.0),
                volume.transform,
                *volume_id as i32,
            );
            *volume_id += 1;
            volume.name = format!("{stem}_{}", copy + 1);
            let group = repeat(&entry.assemble_index, copy).unwrap_or(0);
            let filament = repeat(&entry.filaments, copy).unwrap();
            if group > 0 {
                set_volume_extruder(&mut volume, filament);
            }
            if group > 0
                && let Some(&target) = groups.get(&group)
            {
                let target: &mut ProjectObject = &mut objects[target];
                target.volumes.push(volume);
                continue;
            }
            let name = if group > 0 {
                groups.insert(group, objects.len());
                format!("assemble_{group}")
            } else {
                volume.name.clone()
            };
            objects.push(object(
                objects.len() as u32 + 1,
                name,
                vec![volume],
                Some(filament),
            ));
        }
    }
    Ok(())
}

fn repeat(values: &[i32], copy: usize) -> Option<i32> {
    values.get(copy).or_else(|| values.first()).copied()
}

fn basename(path: &str) -> &str {
    path.rsplit(['/', '\\']).next().unwrap()
}

fn read_stl(file: AssemblyFile<'_>, name: &str, id: u32) -> Result<ProjectVolume, AssemblyError> {
    stl_volume(file.bytes, name, id)
        .map(|(volume, _)| volume)
        .map_err(|error| AssemblyError::Data(format!("{}: {error}", file.path)))
}

fn set_volume_extruder(volume: &mut ProjectVolume, filament: i32) {
    volume.region_overrides.extruder = Some(OrcaInt(filament));
}

fn object(
    id: u32,
    name: String,
    volumes: Vec<ProjectVolume>,
    filament: Option<i32>,
) -> ProjectObject {
    let region = crate::options::RegionOptionOverrides {
        extruder: filament.map(OrcaInt),
        ..Default::default()
    };
    ProjectObject::new(
        name.clone(),
        id,
        (name, String::new(), Default::default(), region),
        volumes,
        vec![ProjectInstance::new(
            identity(id),
            true,
            true,
            Transform3d::IDENTITY,
        )],
    )
}

fn identity(id: u32) -> [u32; 3] {
    [id, 0, id]
}

fn model(objects: Vec<ProjectObject>, membership: Vec<Vec<u32>>, names: Vec<String>) -> Project {
    use crate::project::{
        filament_sequence::FilamentSequences,
        model_settings::{Metadata, ModelSettings, PlateSettings},
        slice_info::{SliceInfo, SliceInfoHeader},
    };
    Project::new(
        vec![ProjectModel::new(
            String::new(),
            objects.iter().map(ProjectObject::id).collect(),
        )],
        objects,
        membership
            .into_iter()
            .enumerate()
            .map(|(index, ids)| {
                PlateMetadata::new(index as u32 + 1, ids.into_iter().map(identity).collect())
            })
            .collect(),
        Default::default(),
        ProjectDocuments {
            model_settings: ModelSettings {
                objects: Vec::new(),
                plates: names
                    .into_iter()
                    .map(|value| PlateSettings {
                        metadata: vec![Metadata {
                            key: "plater_name".into(),
                            value,
                        }],
                        model_instances: Vec::new(),
                    })
                    .collect(),
                assemble: None,
            },
            project_settings_raw: Default::default(),
            slice_info: SliceInfo {
                header: SliceInfoHeader { items: Vec::new() },
            },
            filament_sequences: FilamentSequences(Default::default()),
            plate_documents: Vec::new(),
            has_painted_layer_height_profile: false,
            bbl_family_3mf: false,
            preset_file_version: None,
            preset_selection: None,
            config_composer: crate::project::domain::ConfigComposer::CliRun,
        },
    )
}

impl Project {
    pub(crate) fn assemble_loaded_archive(&mut self) -> Result<(), SliceError> {
        let mut volumes = Vec::new();
        for source in std::mem::take(&mut self.objects) {
            let extruder = source.region_overrides.extruder.map_or(0, |value| value.0);
            for mut volume in source.volumes {
                volume.id = volumes.len() as u32;
                volume.mesh_preparation.key = volume.id as i32;
                set_volume_extruder(&mut volume, extruder);
                volumes.push(volume);
            }
        }
        if volumes.is_empty() {
            return Err(SliceError::InvalidInput(
                "--assemble needs at least one input model".into(),
            ));
        }
        self.objects
            .push(object(1, "Assembly".into(), volumes, None));
        self.models = vec![ProjectModel::new(String::new(), vec![1])];
        self.initialize_plate(vec![identity(1)]);
        self.documents.model_settings.plates.clear();
        self.documents.plate_documents.clear();
        self.documents.has_painted_layer_height_profile = false;
        self.plates_custom_gcodes.clear();
        Ok(())
    }

    pub(crate) fn place_assembly_plates(&mut self) {
        let (cols, dimensions) = (
            plate_selection::compute_colum_count(self.plates.len()),
            plate_selection::plate_dimensions(&self.settings),
        );
        for (index, plate) in self.plates.iter().enumerate() {
            let (x, y) = plate_selection::plate_origin_at(index, cols, dimensions);
            for &[id, _, _] in plate.instances() {
                let object = &mut self.objects[id as usize - 1];
                object.instances[0].transform =
                    Transform3d::IDENTITY.pretranslated(Point3d::new(x, y, 0.0));
                object.import_instances.clone_from(&object.instances);
            }
        }
    }
}
