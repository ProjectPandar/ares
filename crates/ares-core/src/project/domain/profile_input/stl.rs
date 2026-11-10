use super::{
    FilamentSequences, ModelSettings, PlateMetadata, Point3d, Project, ProjectDocuments,
    ProjectInstance, ProjectMesh, ProjectModel, ProjectObject, ProjectSettingsBuilder,
    ProjectSettingsRaw, ProjectVolume, ProjectVolumeType, RawConfigValue, SelectedProfile,
    SliceError, SliceInfo, SliceInfoHeader, Transform3d, index_stl_vertices,
};

impl Project {
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
        // STL --load-settings is not the BBL 3MF filament-mapping branch
        // (OrcaSlicer.cpp:2275–2277); apply user edits to the composed profile.
        let mut raw = profile.profile_raw.clone();
        // CLI preset identities are injected after loading settings
        // (OrcaSlicer.cpp:2048,2088,3152–3158,3175–3180), before
        // command-line overrides (:3613–3615). Use the same composed data
        // for both STL paths (profiles/composition.rs:186–211).
        let project = &profile.settings.project;
        for (key, value) in [
            ("print_settings_id", &project.preset.print_settings_id.0),
            ("printer_settings_id", &project.preset.printer_settings_id.0),
        ] {
            raw.insert(key.to_owned(), RawConfigValue::Text(value.clone()));
        }
        for (key, values) in [
            (
                "filament_settings_id",
                &project.preset.filament_settings_id.0,
            ),
            ("filament_ids", &project.gcode.filament_ids.0),
            (
                "print_compatible_printers",
                &project.preset.print_compatible_printers.0,
            ),
        ] {
            raw.insert_strings(key, values);
        }
        for (key, values) in [
            ("filament_map", &project.gcode.filament_map.0),
            ("filament_self_index", &project.preset.filament_self_index.0),
        ] {
            raw.insert(
                key.to_owned(),
                RawConfigValue::List(
                    values
                        .iter()
                        .map(|value| RawConfigValue::Text(value.0.to_string()))
                        .collect(),
                ),
            );
        }
        let (settings, mut raw) = if overrides.is_empty() {
            // The composed profile raw is already the post-`handle_legacy`
            // view (`Config.cpp:577-579` normalizes every loaded preset at
            // compose, before any overlay).
            let mut settings = profile.settings.clone();
            // CLI:4250-4259 adds a missing static default once, not once per
            // loaded filament. Typed composition must not invent raw members.
            settings
                .filament
                .restore_absent_defaults(|key| raw.contains_key(key));
            (settings, raw)
        } else {
            let mut merged = raw;
            overrides.apply(&mut merged)?;
            // `handle_legacy` on user edits (Config.cpp:577–579) precedes
            // the typed settings rebuild.
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
        // CLI:4243-4244 normalizes mixed metadata for the loaded logical slots.
        let filament_count = settings.project.preset.filament_settings_id.0.len();
        super::resize_mixed_filament_metadata(&mut raw, filament_count, filament_count);
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
            ProjectMesh::new(vertices, triangles, Vec::new()),
            transform,
            (
                filename.to_owned(),
                ProjectVolumeType::ModelPart,
                Default::default(),
                Transform3d::IDENTITY,
            ),
        );
        // `ObjectID.hpp:86-87`: every default-constructed `ObjectBase`
        // increments the process-global id counter (`ObjectBase(-1)`
        // members do not). The CLI STL flow constructs, in order: the
        // outer `Model` and its wipe tower (`OrcaSlicer.cpp:1612`), the
        // `read_from_file` `Model` and its wipe tower (`Model.cpp:256`),
        // this object's `ModelObject` + config + layer height profile
        // (`Model.cpp:467-500`), the volume's `ModelVolume` + config + the
        // four facet annotations (`Model.hpp:794,870-879`), and finally the
        // instance (`Model.cpp:361,658`) — 2 + 2 + 3 + 6 + 1.
        // `get_labeled_id` (`Model.hpp:1267-1273`) returns it while
        // `loaded_id` stays 0 for STL inputs; the header emits it at
        // `GCode.cpp:2594`.
        let label_id = 2 + 2 + 3 + 6 + 1;
        // `ModelInstance` drops onto the bed by default (`auto_drop(true)`, `Model.hpp:1392`).
        let instance = ProjectInstance::new([1, 0, label_id], true, true, Transform3d::IDENTITY);
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
                vec![PlateMetadata::new(1, vec![[1, 0, label_id]])],
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
                    bbl_family_3mf: false,
                },
            ),
            preparation,
        ))
    }
}
