//! STL Model import (`Format/STL.cpp:17-40`, `Model.cpp:485-500,1254-1264,2600-2615`).

use super::{
    Point3d, Project, ProjectInstance, ProjectMesh, ProjectModel, ProjectObject, ProjectVolume,
    ProjectVolumeType, Transform3d,
    documents::{PlateMetadata, ProjectDocuments},
};
use crate::{
    SliceError,
    options::ProjectSettingsBuilder,
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

    /// 3MF selected-profile entry (`OrcaSlicer.cpp:1624-1640` loads the
    /// embedded config as the full-config base; `:2750-2812 update_full_config`
    /// overlays every key the loaded presets supply and keeps the rest).
    ///
    /// `settings` is the selected profile's fully composed config; `profile_raw`
    /// is the chain-expanded presence view and `filament_raws` carries one
    /// merged raw per selected filament for the `:3301` per-slot `set_at`
    /// overlay. Upstream indexes `different_settings_to_system` and
    /// `inherits_group` by the embedded `filament_count` (`:2817-2818`), so
    /// selecting more filaments than the embedded count is rejected rather
    /// than silently truncated or allowed to write out of bounds.
    pub(crate) fn from_3mf_with_profiles(
        input: &[u8],
        profile: SelectedProfile<'_>,
        overrides: &crate::ConfigOverrides,
    ) -> Result<(Self, crate::options::flush_matrix::PreparationContext), SliceError> {
        let SelectedProfile {
            settings,
            profile_raw,
            filament_raws,
            validate_print,
        } = profile;
        let (mut project, preparation) = crate::project::load_project_with_preparation(
            input,
            &crate::ConfigOverrides::default(),
        )?;
        let preparation = preparation.for_overrides(overrides);
        let embedded_raw = project.documents.project_settings_raw.clone();
        // O5: the CLI composite entry checks the **embedded**
        // post_process before the profile overlay (`OrcaSlicer.cpp:1682-1690`).
        if profile.validate_print {
            crate::project_slice::validate_post_process(&project)?;
        }

        let selected = settings.project.preset.filament_settings_id.0.len();
        let embedded_count = project
            .settings()
            .project
            .preset
            .filament_settings_id
            .0
            .len();
        if embedded_count == 0 {
            return Err(SliceError::InvalidInput(
                "project embeds no filament selection".to_owned(),
            ));
        }
        if selected > embedded_count {
            return Err(SliceError::InvalidInput(format!(
                "selection has {selected} filaments but the project embeds {embedded_count}"
            )));
        }

        let mut merged = merged_profile_raw(
            &embedded_raw,
            embedded_raw.clone(),
            &SelectedProfile {
                settings,
                profile_raw,
                filament_raws,
                validate_print,
            },
            selected,
            embedded_count,
        )?;
        // M854 edit order: user overrides apply last (`Config.cpp:461`).
        overrides.apply(&mut merged);
        // `set_deserialize`/`handle_legacy` runs on every option token the
        // merged config carries — embedded base and profile overlay alike
        // (`Config.cpp:577-579`) — so the raw view is post-legacy before the
        // typed builder and the `is_value_valid` loop see it.
        merged.handle_legacy();

        project.settings = crate::options::json_deserialize::deserialize_preset_config::<
            ProjectSettingsBuilder,
        >(merged.clone().into_object())
        .map_err(|error| {
            SliceError::InvalidInput(format!("merged project settings are invalid: {error}"))
        })?
        .resolve();
        project.documents.project_settings_raw = merged;
        project.initialize_cli_loaded_project_config();
        Ok((project, preparation))
    }

    pub(crate) fn from_stl_with_profiles(
        input: &[u8],
        filename: &str,
        profile: SelectedProfile<'_>,
        overrides: &crate::ConfigOverrides,
    ) -> Result<(Self, crate::options::flush_matrix::PreparationContext), SliceError> {
        let preparation = crate::options::flush_matrix::PreparationContext::capture(
            &ProjectSettingsRaw::default(),
            overrides,
        )?;
        // With user edits the sparse profile raw cannot carry a full config;
        // route through the same overlay machinery as the 3MF entry (an
        // empty embedded view) so `ConfigOverrides` apply last and the typed
        // settings re-resolve. The no-override path stays byte-identical to
        // the proven entry.
        let (settings, raw) = if overrides.is_empty() {
            // The sparse profile raw is the post-`handle_legacy` view of the
            // selected presets (`Config.cpp:577-579` normalizes each option
            // token as it is read).
            let mut raw = profile.profile_raw.clone();
            raw.handle_legacy();
            (profile.settings.clone(), raw)
        } else {
            let count = profile.settings.project.preset.filament_settings_id.0.len();
            let mut merged = merged_profile_raw(
                &ProjectSettingsRaw::default(),
                ProjectSettingsRaw::default(),
                &profile,
                count,
                count,
            )?;
            overrides.apply(&mut merged);
            // `handle_legacy` on the overlay (`Config.cpp:577-579`) — same
            // post-legacy contract as the 3MF branch.
            merged.handle_legacy();
            (
                crate::options::json_deserialize::deserialize_preset_config::<ProjectSettingsBuilder>(merged.clone().into_object())
                    .map_err(|error| {
                        SliceError::InvalidInput(format!(
                            "merged project settings are invalid: {error}"
                        ))
                    })?
                    .resolve(),
                merged,
            )
        };
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
        Ok((
            Self::new(
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
            ),
            preparation,
        ))
    }
}

mod overlay;
mod stl_mesh;
#[cfg(test)]
mod tests;

pub(crate) use overlay::SelectedProfile;
use overlay::merged_profile_raw;
use stl_mesh::index_stl_vertices;
