//! Drives the OrcaSlicer CLI for the parity harness: builds a 3MF from
//! flattened presets and the shared cube model, slices it for the reference
//! G-code, and caches reference output keyed by the input digest.

use std::path::{Path, PathBuf};

use serde_json::{Map, Value};

#[path = "runner/adjacent_color.rs"]
mod adjacent_color;
#[path = "runner/adjacent_motion.rs"]
mod adjacent_motion;

#[cfg(test)]
#[path = "runner/tiq2_retained_fan.rs"]
mod tiq2_retained_fan;
#[path = "runner/type2_copies.rs"]
pub(crate) mod type2_copies;
#[path = "runner/vg1_inputs.rs"]
pub(crate) mod vg1_inputs;

#[path = "runner/application.rs"]
pub(crate) mod application;
#[cfg(test)]
#[path = "runner/application_tests.rs"]
mod application_tests;
#[cfg(test)]
#[path = "runner/arachne_open_wall_order.rs"]
mod arachne_open_wall_order;
#[cfg(test)]
#[path = "runner/cache_contract_tests.rs"]
mod cache_contract_tests;
#[cfg(test)]
#[path = "runner/clipped_loop_motion.rs"]
mod clipped_loop_motion;
#[cfg(test)]
#[path = "runner/compatible_regions.rs"]
mod compatible_regions;
#[cfg(test)]
#[path = "runner/costrings.rs"]
mod costrings;
#[cfg(test)]
#[path = "runner/direct_stl_override_bookkeeping.rs"]
mod direct_stl_override_bookkeeping;
#[cfg(test)]
#[path = "runner/embedded_script_whole_file.rs"]
mod embedded_script_whole_file;
#[cfg(test)]
#[path = "runner/empty_tools.rs"]
mod empty_tools;
#[path = "runner/filament_map.rs"]
pub(crate) mod filament_map;
#[cfg(test)]
#[path = "runner/filament_map_ramming.rs"]
mod filament_map_ramming;
#[cfg(test)]
#[path = "runner/filament_map_tests.rs"]
mod filament_map_tests;
#[cfg(test)]
#[path = "runner/filament_self_index.rs"]
mod filament_self_index;
#[cfg(test)]
#[path = "runner/generated_printer_ids.rs"]
mod generated_printer_ids;
#[cfg(test)]
#[path = "runner/infill_first_ironing.rs"]
mod infill_first_ironing;
#[cfg(test)]
#[path = "runner/initialization_tests.rs"]
mod initialization_tests;
#[cfg(test)]
#[path = "runner/initialized_loop_chain.rs"]
mod initialized_loop_chain;
#[cfg(test)]
#[path = "runner/initialized_overhang_intersections.rs"]
mod initialized_overhang_intersections;
#[cfg(test)]
#[path = "runner/j1_runtime_used_mask.rs"]
mod j1_runtime_used_mask;
#[cfg(test)]
#[path = "runner/j1_top_surface_lifecycle.rs"]
mod j1_top_surface_lifecycle;
#[cfg(test)]
#[path = "runner/lifecycle_tests.rs"]
mod lifecycle_tests;
#[cfg(test)]
#[path = "runner/multi_object_skirt.rs"]
mod multi_object_skirt;
#[cfg(test)]
#[path = "runner/native_availability_tests.rs"]
mod native_availability_tests;
#[cfg(test)]
#[path = "runner/nearest_seam_gaussian.rs"]
mod nearest_seam_gaussian;
#[cfg(test)]
#[path = "runner/object_layer_height_ranges.rs"]
mod object_layer_height_ranges;
#[cfg(test)]
#[path = "runner/overhang_reverse.rs"]
mod overhang_reverse;
#[cfg(test)]
#[path = "runner/path_tests.rs"]
mod path_tests;
#[cfg(test)]
#[path = "runner/processor_migration.rs"]
mod processor_migration;
#[cfg(test)]
#[path = "runner/profile_entry.rs"]
mod profile_entry;
#[cfg(test)]
#[path = "runner/profile_entry_3mf.rs"]
mod profile_entry_3mf;
#[cfg(test)]
#[path = "runner/profile_inherits_group.rs"]
mod profile_inherits_group;
#[cfg(test)]
#[path = "runner/profile_legacy_overlay.rs"]
mod profile_legacy_overlay;
#[cfg(test)]
#[path = "runner/profile_overlay_default_gcodes.rs"]
mod profile_overlay_default_gcodes;
#[cfg(test)]
#[path = "runner/project_export_metadata.rs"]
mod project_export_metadata;
#[cfg(test)]
#[path = "runner/scarf_default_threshold.rs"]
mod scarf_default_threshold;
#[cfg(test)]
#[path = "runner/seam_layer_height.rs"]
mod seam_layer_height;
#[cfg(test)]
#[path = "runner/seam_object_bias.rs"]
mod seam_object_bias;
#[cfg(test)]
#[path = "runner/small_loop_speed.rs"]
mod small_loop_speed;
#[cfg(test)]
#[path = "runner/stage_tests.rs"]
mod stage_tests;
#[path = "runner/stages.rs"]
pub(crate) mod stages;
#[cfg(test)]
#[path = "runner/stl_model_label.rs"]
mod stl_model_label;
#[cfg(test)]
#[path = "runner/stl_variant_whole_file.rs"]
mod stl_variant_whole_file;
#[cfg(test)]
#[path = "runner/support_annotations.rs"]
mod support_annotations;
#[cfg(test)]
#[path = "runner/support_modifier.rs"]
mod support_modifier;
#[cfg(test)]
#[path = "runner/template_rng.rs"]
mod template_rng;
#[cfg(test)]
#[path = "runner/tests.rs"]
mod tests;
#[cfg(test)]
#[path = "runner/tree_support.rs"]
mod tree_support;
#[cfg(test)]
#[path = "runner/volume_fixture.rs"]
mod volume_fixture;

#[path = "runner/command.rs"]
mod command;
#[cfg(test)]
#[path = "runner/strict_reference_tests.rs"]
mod strict_reference_tests;

#[path = "runner/user_overrides.rs"]
mod user_overrides;

#[cfg(test)]
#[path = "runner/wrapper_tests.rs"]
mod wrapper_tests;
#[cfg(test)]
#[path = "runner/x2d_flush_matrix.rs"]
mod x2d_flush_matrix;

#[cfg(test)]
#[path = "runner/wipe_tags.rs"]
mod wipe_tags;

use stages::{FailureKind, Stage, StageError};
#[path = "runner/environment.rs"]
mod environment;
pub(crate) use environment::reference_enabled;
#[path = "runner/cache.rs"]
mod cache;
#[path = "runner/cache_identity.rs"]
mod cache_identity;

pub(super) struct ExportedCase {
    pub(super) label: String,
    pub(super) project: Vec<u8>,
    project_path: PathBuf,
    output_dir: PathBuf,
}

pub(super) struct OrcaRunner {
    bin: PathBuf,
    work: PathBuf,
    /// Content-keyed export and reference outputs; the private work
    /// directory unless ARES_ORCA_CACHE_DIR shares them across runs.
    cache: PathBuf,
    producer: std::sync::OnceLock<cache_identity::Producer>,
}

pub(super) struct ParityCase {
    /// Human-readable case label (vendor / printer / variant).
    pub(super) label: String,
    pub(super) project: Vec<u8>,
    pub(super) reference: Vec<u8>,
}

/// Flattened preset inputs for one parity case.
pub(super) struct CaseInputs<'a> {
    pub(super) label: &'a str,
    pub(super) machine: &'a Map<String, Value>,
    pub(super) process: &'a Map<String, Value>,
    pub(super) filaments: &'a [Map<String, Value>],
}

impl OrcaRunner {
    /// Path to the executable invoked for reference production — used by
    /// the sweep preflight to verify the wrapper actually runs.
    pub(super) fn bin(&self) -> &PathBuf {
        &self.bin
    }

    /// Only absent optional configuration returns None; initialization errors must fail callers.
    #[track_caller]
    pub(super) fn from_env() -> Result<Option<Self>, StageError> {
        let Some((bin, root)) = environment::available()? else {
            return Ok(None);
        };
        let init_error = |detail| StageError::new(Stage::Initialization, FailureKind::Io, detail);
        let work = tempfile::Builder::new()
            .prefix("orca-runner-")
            .tempdir_in(&root)
            .map_err(|error| init_error(format!("runner work directory under {root:?}: {error}")))?
            .keep();
        // Shared roots store v2 content-addressed entries across runs.
        let cache = match std::env::var_os("ARES_ORCA_CACHE_DIR") {
            Some(cache) => {
                let cache = PathBuf::from(cache);
                std::fs::create_dir_all(&cache).map_err(|error| {
                    init_error(format!("Orca cache directory {cache:?}: {error}"))
                })?;
                cache
            }
            None => work.clone(),
        };
        Ok(Some(Self {
            bin,
            work,
            cache,
            producer: Default::default(),
        }))
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum PresetKind {
    Machine,
    Process,
    Filament,
}

fn apply_override(
    base: &Map<String, Value>,
    overrides: &Map<String, Value>,
    kind: PresetKind,
) -> Map<String, Value> {
    let mut merged = base.clone();
    for (key, value) in overrides {
        if merged.contains_key(key) || smoke_owner(key) == Some(kind) {
            merged.insert(key.clone(), value.clone());
        }
    }
    merged
}

fn smoke_owner(key: &str) -> Option<PresetKind> {
    match key {
        "bed_exclude_area"
        | "extruder_printable_height"
        | "machine_start_gcode"
        | "retraction_distances_when_cut"
        | "use_firmware_retraction" => Some(PresetKind::Machine),
        "before_layer_change_gcode"
        | "bridge_line_width"
        | "detect_thin_wall"
        | "post_process"
        | "wall_generator" => Some(PresetKind::Process),
        _ => None,
    }
}

fn write_preset(path: &Path, kind: &str, fields: &Map<String, Value>) -> Result<(), StageError> {
    // Keep the preset's own name: the CLI matches it against
    // compatible_printers when both machine and process are loaded.
    let mut preset = fields.clone();
    preset.insert("type".into(), Value::String(kind.to_owned()));
    preset.insert("from".into(), Value::String("system".to_owned()));
    preset.insert("instantiation".into(), Value::String("true".to_owned()));
    let text = serde_json::to_string(&preset).unwrap();
    std::fs::write(path, text)
        .map_err(|e| StageError::new(Stage::Preset, FailureKind::Io, format!("{path:?}: {e}")))
}

fn join_paths<'a>(paths: impl Iterator<Item = &'a PathBuf>) -> String {
    let mut joined = String::new();
    for path in paths {
        if !joined.is_empty() {
            joined.push(';');
        }
        joined.push_str(&path.to_string_lossy());
    }
    joined
}

fn digest_slug(
    label: &str,
    machine: &Map<String, Value>,
    process: &Map<String, Value>,
    filaments: &[Map<String, Value>],
    (model, model_bytes): (&Path, &[u8]),
) -> String {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    label.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
        + &content_digest(machine, process, filaments, model, model_bytes)
}

fn content_digest(
    machine: &Map<String, Value>,
    process: &Map<String, Value>,
    filaments: &[Map<String, Value>],
    model: &Path,
    model_bytes: &[u8],
) -> String {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    format!("{:?}", machine).hash(&mut hasher);
    format!("{:?}", process).hash(&mut hasher);
    for filament in filaments {
        format!("{:?}", filament).hash(&mut hasher);
    }
    model.file_name().hash(&mut hasher);
    model_bytes.hash(&mut hasher);
    format!("-{:016x}", hasher.finish())
}

pub(super) fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}
