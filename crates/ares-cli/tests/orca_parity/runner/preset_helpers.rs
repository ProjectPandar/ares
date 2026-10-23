use super::{FailureKind, Map, Path, PathBuf, Stage, StageError, Value};

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum PresetKind {
    Machine,
    Process,
    Filament,
}

pub(crate) fn apply_override(
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

pub(crate) fn write_preset(
    path: &Path,
    kind: &str,
    fields: &Map<String, Value>,
) -> Result<(), StageError> {
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

pub(crate) fn join_paths<'a>(paths: impl Iterator<Item = &'a PathBuf>) -> String {
    let mut joined = String::new();
    for path in paths {
        if !joined.is_empty() {
            joined.push(';');
        }
        joined.push_str(&path.to_string_lossy());
    }
    joined
}

pub(crate) fn digest_slug(
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

pub(crate) fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}
