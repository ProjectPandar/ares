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
        raw_settings::{ProjectSettingsRaw, RawConfigValue},
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
        // `shrink_to_new_bed` (`OrcaSlicer.cpp:3766-3796`): a BBL-family 3mf
        // re-centers onto the loaded machine's bed — the embedded bed being
        // smaller sets `shrink = 1`, and every instance plus the wipe tower
        // shifts by the bed-center delta (`:3958-4053 translate_models`).
        if project.documents.bbl_family_3mf
            && let Some(offset) = shrink_to_new_bed_offset(&embedded_raw, &merged)
        {
            project.translate_all_instances(offset);
            shift_wipe_tower(&mut merged, offset);
        }

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
            // The composed profile raw is already the post-`handle_legacy`
            // view (`Config.cpp:577-579` normalizes every loaded preset at
            // compose, before any overlay).
            (profile.settings.clone(), profile.profile_raw.clone())
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
        let instance = ProjectInstance::new([1, 0, label_id], true, false, Transform3d::IDENTITY);
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

/// `OrcaSlicer.cpp:3766-3796` classification with the `:4008-4021` plate-0
/// center offset: `Some([dx, dy])` for `shrink = 1`, `None` otherwise
/// (shrink 2/0 are deferred). Widths/heights truncate to int like upstream
/// (`:1780`, `:1338`). The current height comes from the fully parsed
/// config — `OrcaSlicer.cpp:3711` `opt_float("printable_height")` — with
/// the `PrintConfigDef` default (`PrintConfig.cpp:785`, 100.0) when the
/// presets lack the key. An unrecorded old bed dimension takes the current
/// one (`:3729-3734`) and the classification proceeds.
fn shrink_to_new_bed_offset(
    old: &ProjectSettingsRaw,
    new: &ProjectSettingsRaw,
) -> Option<[f64; 2]> {
    let new_height = printable_height(new)
        .or_else(default_printable_height)
        .unwrap_or(0.0);
    let (new_width, new_depth) = printable_bed_size(new).unwrap_or((0.0, 0.0));
    let (mut old_width, mut old_depth) = printable_bed_size(old).unwrap_or((0.0, 0.0));
    let mut old_height = printable_height(old).unwrap_or(0.0);
    if old_width == 0.0 {
        old_width = new_width;
    }
    if old_depth == 0.0 {
        old_depth = new_depth;
    }
    if old_height == 0.0 {
        old_height = new_height;
    }
    let (old_width, old_depth, old_height) =
        (old_width as i32, old_depth as i32, old_height as i32);
    let (new_width, new_depth, new_height) =
        (new_width as i32, new_depth as i32, new_height as i32);
    let shrink = if old_width > new_width || old_depth > new_depth || old_height > new_height {
        2
    } else if old_width < new_width || old_depth < new_depth {
        1
    } else {
        0
    };
    if shrink != 1 {
        return None;
    }
    Some([
        new_width as f64 / 2.0 - old_width as f64 / 2.0,
        new_depth as f64 / 2.0 - old_depth as f64 / 2.0,
    ])
}

/// `PrintConfig.cpp:785`: the `printable_height` default for configs that
/// lack the key.
fn default_printable_height() -> Option<f64> {
    crate::option_definition("printable_height")
        .and_then(|definition| definition.default_value.parse().ok())
}

/// `OrcaSlicer.cpp:1779-1780` (embedded) and `:3726` (overlaid): the
/// diagonal-corner difference of `printable_area`.
fn printable_bed_size(raw: &ProjectSettingsRaw) -> Option<(f64, f64)> {
    let points = point_pairs(raw.get("printable_area")?)?;
    let first = points.first()?;
    let diagonal = points.get(2)?;
    Some((diagonal.0 - first.0, diagonal.1 - first.1))
}

fn printable_height(raw: &ProjectSettingsRaw) -> Option<f64> {
    match raw.get("printable_height") {
        Some(RawConfigValue::Text(token)) => token.trim().parse().ok(),
        Some(RawConfigValue::Number(number)) => number.as_f64(),
        _ => None,
    }
}

/// `ConfigOptionPoints` wire entries (`"0x0"`, `"220x220"`) as plain
/// coordinate pairs.
fn point_pairs(value: &RawConfigValue) -> Option<Vec<(f64, f64)>> {
    let tokens: Vec<String> = match value {
        RawConfigValue::List(items) => items
            .iter()
            .filter_map(|item| match item {
                RawConfigValue::Text(token) => Some(token.clone()),
                RawConfigValue::Number(number) => Some(number.to_string()),
                _ => None,
            })
            .collect(),
        RawConfigValue::Text(token) => token.split(';').map(str::to_owned).collect(),
        _ => return None,
    };
    let pairs = tokens
        .iter()
        .filter_map(|token| {
            let (x, y) = token.trim().split_once('x')?;
            Some((x.trim().parse().ok()?, y.trim().parse().ok()?))
        })
        .collect::<Vec<_>>();
    (!pairs.is_empty()).then_some(pairs)
}

/// `OrcaSlicer.cpp:4038-4048`: the plate-0 wipe tower rides the
/// re-centering delta (`set_at(…, 0)`); absent keys stay absent.
fn shift_wipe_tower(raw: &mut ProjectSettingsRaw, [offset_x, offset_y]: [f64; 2]) {
    for (key, offset) in [("wipe_tower_x", offset_x), ("wipe_tower_y", offset_y)] {
        let Some(value) = raw.get(key).cloned() else {
            continue;
        };
        let Some(shifted) = shift_first_slot(&value, offset) else {
            continue;
        };
        raw.insert(key.to_owned(), shifted);
    }
}

fn shift_first_slot(value: &RawConfigValue, offset: f64) -> Option<RawConfigValue> {
    let mut items = match value {
        RawConfigValue::List(items) => items.clone(),
        _ => return None,
    };
    let first = items.first()?;
    let position: f64 = first.token()?.trim().parse().ok()?;
    items[0] = RawConfigValue::Text(format!("{}", position + offset));
    Some(RawConfigValue::List(items))
}

mod overlay;
mod stl_mesh;
#[cfg(test)]
mod tests;

pub(crate) use overlay::SelectedProfile;
use overlay::merged_profile_raw;
use stl_mesh::index_stl_vertices;
