//! Drives the OrcaSlicer CLI for the parity harness: builds a 3MF from
//! flattened presets and the shared cube model, slices it for the reference
//! G-code, and caches reference output keyed by the input digest.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use serde_json::{Map, Value};

#[cfg(test)]
#[path = "runner/tests.rs"]
mod tests;

pub(super) struct OrcaRunner {
    bin: PathBuf,
    work: PathBuf,
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
    /// Returns None when no OrcaSlicer binary is configured so the parity
    /// suite can skip.
    pub(super) fn from_env() -> Option<Self> {
        let bin = std::env::var_os("ARES_ORCA_BIN")
            .map(PathBuf::from)
            .or_else(|| {
                let script = repo_root().join("scripts/orca-parity.sh");
                script.exists().then_some(script)
            })?;
        if !bin.exists() {
            eprintln!("ares-parity: ARES_ORCA_BIN {:?} not found; skipping", bin);
            return None;
        }
        let work = std::env::temp_dir().join(format!("ares-parity-{}", std::process::id()));
        std::fs::create_dir_all(&work).ok()?;
        Some(Self { bin, work })
    }

    /// Flattened preset overrides applied on top of the base presets before
    /// the 3MF export; the same map is visible to both slicers.
    pub(super) fn build_case(
        &self,
        inputs: &CaseInputs<'_>,
        overrides: &Map<String, Value>,
        model: &Path,
    ) -> Result<ParityCase, String> {
        let label = inputs.label;
        let machine = apply_override(inputs.machine, overrides, PresetKind::Machine);
        let mut process = apply_override(inputs.process, overrides, PresetKind::Process);
        // The CLI's machine-switch compatibility check matches the
        // process's compatible_printers against the machine name
        // (`OrcaSlicer.cpp:2579-2585`); presets without a non-empty list
        // (e.g. the Prusa CORE One chain) fail with
        // CLI_PROCESS_NOT_COMPATIBLE (-17). Inject the machine name when
        // the process omits it or resolves it empty.
        let compatible_empty = process
            .get("compatible_printers")
            .and_then(Value::as_array)
            .is_none_or(|printers| printers.is_empty());
        if compatible_empty {
            if let Some(name) = machine.get("name").and_then(Value::as_str) {
                process.insert(
                    "compatible_printers".to_owned(),
                    Value::Array(vec![Value::String(name.to_owned())]),
                );
            }
        }
        let filaments: Vec<Map<String, Value>> = inputs
            .filaments
            .iter()
            .map(|filament| apply_override(filament, overrides, PresetKind::Filament))
            .collect();

        let slug = digest_slug(label, &machine, &process, &filaments, model);
        let project_path = self.work.join(format!("{slug}.3mf"));
        let output_dir = self.work.join(&slug);
        let reference_path = output_dir.join("plate_1.gcode");

        if !project_path.exists() {
            let machine_file = self.work.join(format!("{slug}-machine.json"));
            let process_file = self.work.join(format!("{slug}-process.json"));
            write_preset(&machine_file, "machine", &machine)?;
            write_preset(&process_file, "process", &process)?;
            let mut filament_files = Vec::new();
            for (index, filament) in filaments.iter().enumerate() {
                let file = self.work.join(format!("{slug}-filament-{index}.json"));
                write_preset(&file, "filament", filament)?;
                filament_files.push(file);
            }

            run_orca(
                &self.bin,
                [
                    "--load-settings",
                    &join_paths([&machine_file, &process_file].into_iter()),
                    "--load-filaments",
                    &join_paths(filament_files.iter()),
                    "--arrange",
                    "1",
                    "--export-3mf",
                ]
                .into_iter()
                .chain([project_path.to_str().unwrap(), model.to_str().unwrap()])
                .map(std::borrow::ToOwned::to_owned),
            )?;
        }

        if !reference_path.exists() {
            std::fs::create_dir_all(&output_dir).map_err(|e| e.to_string())?;
            run_orca(
                &self.bin,
                [
                    "--slice",
                    "0",
                    "--outputdir",
                    output_dir.to_str().unwrap(),
                    project_path.to_str().unwrap(),
                ]
                .into_iter()
                .map(std::borrow::ToOwned::to_owned),
            )?;
        }

        let project = std::fs::read(&project_path).map_err(|e| format!("{project_path:?}: {e}"))?;
        let reference =
            std::fs::read(&reference_path).map_err(|e| format!("{reference_path:?}: {e}"))?;
        Ok(ParityCase {
            label: label.to_owned(),
            project,
            reference,
        })
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

fn write_preset(path: &Path, kind: &str, fields: &Map<String, Value>) -> Result<(), String> {
    // Keep the preset's own name: the CLI matches it against
    // compatible_printers when both machine and process are loaded.
    let mut preset = fields.clone();
    preset.insert("type".into(), Value::String(kind.to_owned()));
    preset.insert("from".into(), Value::String("system".to_owned()));
    preset.insert("instantiation".into(), Value::String("true".to_owned()));
    let text = serde_json::to_string(&preset).map_err(|e| e.to_string())?;
    std::fs::write(path, text).map_err(|e| format!("{path:?}: {e}"))
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

fn run_orca(bin: &Path, args: impl IntoIterator<Item = String>) -> Result<(), String> {
    let args: Vec<String> = args.into_iter().collect();
    use std::io::Read;
    let mut child = Command::new(bin)
        .args(&args)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|error| format!("spawn {:?}: {error}", bin))?;
    let deadline = std::time::Instant::now() + ORCA_TIMEOUT;
    loop {
        match child.try_wait().map_err(|e| e.to_string())? {
            Some(status) if status.success() => return Ok(()),
            Some(status) => {
                let mut stdout = String::new();
                let mut stderr = String::new();
                if let Some(mut pipe) = child.stdout.take() {
                    pipe.read_to_string(&mut stdout).ok();
                }
                if let Some(mut pipe) = child.stderr.take() {
                    pipe.read_to_string(&mut stderr).ok();
                }
                return Err(format!(
                    "orca-slicer failed ({status}): {}",
                    stdout
                        .lines()
                        .chain(stderr.lines())
                        .last()
                        .unwrap_or("no output")
                ));
            }
            None if std::time::Instant::now() > deadline => {
                let _ = child.kill();
                return Err("orca-slicer timed out".to_owned());
            }
            None => std::thread::sleep(Duration::from_millis(100)),
        }
    }
}

const ORCA_TIMEOUT: Duration = Duration::from_secs(300);

fn digest_slug(
    label: &str,
    machine: &Map<String, Value>,
    process: &Map<String, Value>,
    filaments: &[Map<String, Value>],
    model: &Path,
) -> String {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    label.hash(&mut hasher);
    format!("{:016x}", hasher.finish()) + &content_digest(machine, process, filaments, model)
}

fn content_digest(
    machine: &Map<String, Value>,
    process: &Map<String, Value>,
    filaments: &[Map<String, Value>],
    model: &Path,
) -> String {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    format!("{:?}", machine).hash(&mut hasher);
    format!("{:?}", process).hash(&mut hasher);
    for filament in filaments {
        format!("{:?}", filament).hash(&mut hasher);
    }
    model.to_string_lossy().hash(&mut hasher);
    format!("-{:016x}", hasher.finish())
}

pub(super) fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}
