//! STL Model import (`Format/STL.cpp:17-40`, `Model.cpp:485-500,1254-1264,2600-2615`).

mod stl;

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

// PrintConfig.cpp:11475-11488: all seven members share one typed resize.
fn resize_mixed_filament_metadata(
    raw: &mut ProjectSettingsRaw,
    old_count: usize,
    new_count: usize,
) {
    use crate::options::registry::{OptionValueKind, option_definition};

    for key in [
        "filament_is_mixed",
        "filament_mixed_components",
        "filament_mixed_sublayer_ratios",
        "filament_mixed_gradient",
        "filament_mixed_gradient_range",
        "filament_mixed_gradient_curve",
        "filament_mixed_gradient_per_part",
    ] {
        let definition = option_definition(key).unwrap();
        let empty = match definition.kind {
            OptionValueKind::Bools => RawConfigValue::Flag(false),
            OptionValueKind::Strings => RawConfigValue::Text(String::new()),
            _ => unreachable!("mixed metadata members are coBools or coStrings"),
        };
        let mut values = match raw.get(key) {
            Some(RawConfigValue::List(values)) => values.clone(),
            Some(_) => unreachable!("normalized vectors contain logical slots"),
            None if definition.kind == OptionValueKind::Strings => {
                crate::options::unescape_strings_cstyle(definition.default_wire_value)
                    .unwrap()
                    .into_iter()
                    .map(RawConfigValue::Text)
                    .collect()
            }
            None => definition
                .default_wire_value
                .split(',')
                .map(|value| RawConfigValue::Text(value.to_owned()))
                .collect(),
        };
        values.truncate(old_count);
        values.resize(new_count, empty);
        raw.insert(key.to_owned(), RawConfigValue::List(values));
    }
}

impl Project {
    /// The CLI `filament_count` variable of `CLI::run`: the embedded
    /// project's `filament_settings_id` size (`current_filaments_name`,
    /// `OrcaSlicer.cpp:1768`) — the load-settings overlay never
    /// recomputes it (`:2272-2273` only fills a zero count).
    pub(crate) fn cli_filament_count(&self) -> usize {
        self.documents
            .project_settings_raw
            .get("filament_settings_id")
            .map_or(0, crate::project::raw_settings::RawConfigValue::len)
    }

    pub(crate) fn initialize_cli_loaded_project_config(&mut self, filament_count: usize) {
        // OrcaSlicer.cpp:1769: the CLI's 3MF import reads this dynamic option
        // with create=true. STL --load-settings does not perform that read.
        let raw = &mut self.documents.project_settings_raw;
        // OrcaSlicer.cpp:4243-4244: both arguments are loaded logical slots.
        if filament_count > 0 {
            resize_mixed_filament_metadata(raw, filament_count, filament_count);
        }
        if !raw.contains_key("upward_compatible_machine") {
            raw.insert_strings(
                "upward_compatible_machine",
                &self.settings.printer.remaining.upward_compatible_machine.0,
            );
        }
        // OrcaSlicer.cpp:2815-2818 resizes both CLI bookkeeping vectors
        // on m_print_config to filament_count + 2 before the preset sync
        // and GCode::append_full_config serialize them (`std::vector::
        // resize` semantics: grow with empty strings, truncate longer
        // vectors). `filament_count` is the embedded count the CLI loaded
        // in `:1768`, never the post-overlay vector length.
        for key in ["different_settings_to_system", "inherits_group"] {
            let Some(value) = raw.get(key) else {
                continue;
            };
            let mut strings: Vec<String> = match value {
                crate::project::raw_settings::RawConfigValue::List(items) => items
                    .iter()
                    .map(|item| match item {
                        crate::project::raw_settings::RawConfigValue::Text(text) => text.clone(),
                        other => other.token().unwrap_or_default(),
                    })
                    .collect(),
                crate::project::raw_settings::RawConfigValue::Text(token) => {
                    token.split(';').map(str::to_owned).collect()
                }
                _ => continue,
            };
            strings.resize(filament_count + 2, String::new());
            raw.insert_strings(key, &strings);
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
        plate: Option<u32>,
        edits: &crate::plater::PlaterEdits,
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
        project.apply_plater_edits(edits);
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

        let switch = machine_switch(&embedded_raw, settings, project.documents.bbl_family_3mf);
        let mut merged = merged_profile_raw(
            &embedded_raw,
            embedded_raw.clone(),
            &SelectedProfile {
                settings,
                profile_raw,
                filament_raws,
                validate_print,
            },
            OverlayContext {
                selected,
                embedded_count,
                machine_switch: switch,
            },
        )?;
        // M854 edit order: user overrides apply last (`Config.cpp:461`).
        overrides.apply(&mut merged)?;
        // `set_deserialize`/`handle_legacy` runs on every option token the
        // merged config carries — embedded base and profile overlay alike
        // (`Config.cpp:577-579`) — so the raw view is post-legacy before the
        // typed builder and the `is_value_valid` loop see it.
        merged.handle_legacy();
        // The typed build mirrors upstream's fully parsed `m_print_config`:
        // the bed classification reads it exactly like `opt_float(
        // "printable_height")`/`opt<ConfigOptionPoints>(
        // "printable_area")` over the fully parsed config
        // (`OrcaSlicer.cpp:3711,3708`), `PrintConfigDef` defaults included.
        let mut resolved = crate::options::json_deserialize::deserialize_preset_config::<
            ProjectSettingsBuilder,
        >(merged.clone().into_object())
        .map_err(|error| {
            SliceError::InvalidInput(format!("merged project settings are invalid: {error}"))
        })?
        .resolve();
        // OrcaSlicer.cpp:1640 chooses importer membership before placement.
        project.prepare_import_scope(plate);
        // `shrink_to_new_bed` (`OrcaSlicer.cpp:3766-3796`): a BBL-family 3mf
        // re-centers onto the loaded machine's bed — the embedded bed being
        // smaller sets `shrink = 1`, and every instance plus the wipe tower
        // shifts by the bed-center delta (`:3958-4053 translate_models`).
        let offset = project
            .documents
            .bbl_family_3mf
            .then(|| {
                shrink_to_new_bed_offset(&embedded_raw, &resolved, project.instances_world_bbox())
            })
            .flatten();
        if let Some(offset) = offset {
            project.translate_all_instances(offset);
            shift_wipe_tower(&mut merged, offset);
            // The shifted wipe tower reaches the typed view too.
            resolved = crate::options::json_deserialize::deserialize_preset_config::<
                ProjectSettingsBuilder,
            >(merged.clone().into_object())
            .map_err(|error| {
                SliceError::InvalidInput(format!("merged project settings are invalid: {error}"))
            })?
            .resolve();
        }

        project.settings = resolved;
        project.documents.project_settings_raw = merged;
        project.initialize_cli_loaded_project_config(embedded_count);
        Ok((project, preparation))
    }
}

/// `OrcaSlicer.cpp:3766-3796` classification with the `:3990-4021` plate-0
/// offsets: `Some([dx, dy])` for shrink 1 (bed-center delta) and shrink 2
/// (plain object-bbox centering), `None` otherwise (shrink 0). The
/// shared-center, exclude-area, and wipe-tower-inclusive-bbox refinements
/// of shrink 2 are deferred. Widths/heights truncate to int like upstream
/// (`:1780`, `:1338`). The current bed comes from the fully parsed typed
/// config — `OrcaSlicer.cpp:3708,3711` `opt<ConfigOptionPoints>(
/// "printable_area")`/`opt_float("printable_height")` over a config that
/// carries the `PrintConfigDef` defaults. An unrecorded old bed dimension
/// takes the current one (`:3729-3734`) and the classification proceeds.
fn shrink_to_new_bed_offset(
    old: &ProjectSettingsRaw,
    settings: &crate::ProjectSettings,
    (bbox_min, bbox_max): ([f64; 3], [f64; 3]),
) -> Option<[f64; 2]> {
    let area = &settings.printer.remaining.printable_area.0;
    let new_width = (area[2].x - area[0].x) as i32;
    let new_depth = (area[2].y - area[0].y) as i32;
    let new_height = settings.printer.remaining.printable_height.0 as i32;
    let old_area = old
        .get("printable_area")
        .cloned()
        .unwrap_or(RawConfigValue::List(Vec::new()));
    let (mut old_width, mut old_depth) = printable_bed_size(&old_area);
    let mut old_height = printable_height(old).unwrap_or(0.0);
    if old_width == 0.0 {
        old_width = new_width as f64;
    }
    if old_depth == 0.0 {
        old_depth = new_depth as f64;
    }
    if old_height == 0.0 {
        old_height = new_height as f64;
    }
    let (old_width, old_depth, old_height) =
        (old_width as i32, old_depth as i32, old_height as i32);
    let shrink = if old_width > new_width || old_depth > new_depth || old_height > new_height {
        2
    } else if old_width < new_width || old_depth < new_depth {
        1
    } else {
        0
    };
    if shrink == 0 {
        return None;
    }
    if shrink == 2 {
        // `:4008-4021`: center the object bbox on the new bed center.
        return Some([
            new_width as f64 / 2.0 - (bbox_min[0] + bbox_max[0]) * 0.5,
            new_depth as f64 / 2.0 - (bbox_min[1] + bbox_max[1]) * 0.5,
        ]);
    }
    Some([
        new_width as f64 / 2.0 - old_width as f64 / 2.0,
        new_depth as f64 / 2.0 - old_depth as f64 / 2.0,
    ])
}

/// `OrcaSlicer.cpp:1779-1780`: the diagonal-corner difference of the
/// embedded `printable_area`; fewer than four parsable points leave the old
/// bed size at zero (`:1778` guard) for the caller to substitute.
fn printable_bed_size(area: &RawConfigValue) -> (f64, f64) {
    let points = point_pairs(area);
    if points.len() < 4 {
        return (0.0, 0.0);
    }
    let first = &points[0];
    let diagonal = &points[2];
    (diagonal.0 - first.0, diagonal.1 - first.1)
}

fn printable_height(raw: &ProjectSettingsRaw) -> Option<f64> {
    match raw.get("printable_height") {
        Some(RawConfigValue::Text(token)) => token.trim().parse().ok(),
        Some(RawConfigValue::Number(number)) => number.as_f64(),
        _ => None,
    }
}

/// `ConfigOptionPoints` wire entries (`"0x0"`, `"220x220"`) as coordinate
/// pairs; unparsable entries are skipped (the typed layer owns parse
/// diagnostics for config values).
fn point_pairs(value: &RawConfigValue) -> Vec<(f64, f64)> {
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
        _ => Vec::new(),
    };
    tokens
        .iter()
        .filter_map(|token| {
            let (x, y) = token.trim().split_once('x')?;
            Some((x.trim().parse().ok()?, y.trim().parse().ok()?))
        })
        .collect()
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

pub(crate) use overlay::SelectedProfile;
use overlay::{OverlayContext, machine_switch, merged_profile_raw};
use stl_mesh::index_stl_vertices;
