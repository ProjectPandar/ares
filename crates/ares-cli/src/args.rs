use std::path::{Path, PathBuf};

use clap::{ArgGroup, Parser, Subcommand};

pub(crate) fn require_single_input(inputs: &[PathBuf]) {
    if inputs.len() != 1 {
        clap::Error::raw(
            if inputs.is_empty() {
                clap::error::ErrorKind::MissingRequiredArgument
            } else {
                clap::error::ErrorKind::UnknownArgument
            },
            "slicing requires exactly one input model",
        )
        .exit();
    }
}

#[derive(Parser)]
#[command(
    name = "ares",
    args_conflicts_with_subcommands = true,
    subcommand_negates_reqs = true,
    arg_required_else_help = true
)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Option<Commands>,
    #[command(flatten)]
    pub(crate) direct: DirectSliceArgs,
}

#[derive(Subcommand)]
pub(crate) enum Commands {
    Slice(SliceArgs),
}

#[derive(clap::Args)]
pub(crate) struct DirectSliceArgs {
    /// Slice all plates (0) or the given one-based plate number.
    #[arg(
        long = "slice",
        required_unless_present = "export_settings",
        requires = "outputdir",
        allow_negative_numbers = true
    )]
    pub(crate) selection: Option<i32>,
    /// Export effective settings from a saved 3MF to FILE, or to stdout with '-'
    #[arg(long, value_name = "FILE")]
    pub(crate) export_settings: Option<PathBuf>,
    /// Directory for plate_<n>.gcode and the slice result.
    #[arg(long)]
    pub(crate) outputdir: Option<PathBuf>,
    /// Write a statistics JSON sidecar for each sliced plate.
    #[arg(long)]
    pub(crate) export_statistics: bool,
    /// Infer meters or inches for a single STL and convert to millimeters.
    #[arg(long)]
    pub(crate) convert_unit: bool,
    /// Refuse to export a slice with a noncritical slicing warning.
    #[arg(long)]
    pub(crate) strict: bool,
    #[arg(long, hide = true)]
    pub(crate) no_check: bool,
    #[arg(long)]
    pub(crate) load_settings: Option<String>,
    #[arg(long)]
    pub(crate) load_filaments: Option<String>,
    /// Fill empty --load-filaments entries with the first specified preset for STL slicing.
    #[arg(long)]
    pub(crate) load_defaultfila: bool,
    /// Preserve the input model positions (0).
    #[arg(long, value_parser = clap::value_parser!(u8).range(0..=0))]
    pub(crate) arrange: Option<u8>,
    #[arg(required = true)]
    pub(crate) inputs: Vec<PathBuf>,
}

#[derive(Parser)]
#[command(group(
    ArgGroup::new("action")
        .args(["selection", "export_3mf", "export_stl", "export_stls"])
        .multiple(true)
))]
#[command(group(
    ArgGroup::new("export_action")
        .args(["export_3mf", "export_stl", "export_stls"])
        .multiple(true)
))]
pub(crate) struct SliceArgs {
    /// Native data directory used for AppConfig and selected user presets
    #[arg(long)]
    pub(crate) datadir: Option<PathBuf>,
    /// Complete preset catalog supplied explicitly for project-derived refresh
    #[arg(long, conflicts_with_all = ["load_filaments", "resources_dir"])]
    pub(crate) project_preset_catalog: Option<PathBuf>,
    /// Refresh unlisted project values from the retained system presets
    #[arg(long)]
    pub(crate) uptodate: bool,
    /// Flattened machine/process refresh files, `;` separated
    #[arg(long, conflicts_with = "project_preset_catalog")]
    pub(crate) uptodate_settings: Option<String>,
    /// Flattened refresh files in project filament slot order, `;` separated
    #[arg(long, conflicts_with = "project_preset_catalog")]
    pub(crate) uptodate_filaments: Option<String>,
    #[arg(long)]
    pub(crate) options: Option<PathBuf>,
    /// 1-based plate number to slice for multi-plate 3MF projects
    #[arg(long)]
    pub(crate) plate: Option<u32>,
    /// Orient model objects: 0 disables, 1 forces, other integers retain automatic eligibility
    #[arg(long, allow_hyphen_values = true)]
    pub(crate) orient: Option<i32>,
    /// Collect the positional models' volumes into a single assembly
    #[arg(long)]
    pub(crate) assemble: bool,
    /// Load fixed-placement assembly plates from a JSON manifest
    #[arg(long)]
    pub(crate) load_assemble_list: Option<PathBuf>,
    #[arg(long, hide = true, num_args = 0..=1, default_missing_value = "")]
    pub(crate) assemble_list: Option<String>,
    /// Retain source placement for positional assembly (0)
    #[arg(long, allow_hyphen_values = true)]
    pub(crate) arrange: Option<i32>,
    #[arg(long, value_delimiter = ',', allow_hyphen_values = true)]
    pub(crate) clone_objects: Vec<i32>,
    #[arg(long, value_delimiter = ',', allow_hyphen_values = true)]
    pub(crate) load_filament_ids: Vec<i32>,
    /// Resource root containing profiles/BBL/machine_full (default: resources beside the executable)
    #[arg(long)]
    pub(crate) resources_dir: Option<PathBuf>,
    /// Write the G-code processor move vertices as CSV (3MF input only)
    #[arg(long)]
    pub(crate) dump_moves: Option<PathBuf>,
    /// JSON object of user option overrides `{key: string|string[]}`
    /// applied on the project-settings layer (3MF input only; M854)
    #[arg(long)]
    pub(crate) overrides: Option<String>,
    /// OrcaSlicer-native preset files, `;` separated (machine;process;
    /// `PrintConfig.cpp:10890-10895`)
    #[arg(long)]
    pub(crate) load_settings: Option<String>,
    /// OrcaSlicer-native filament preset files, `;` separated
    /// (`PrintConfig.cpp:10896-10901`); empty slots are dropped like
    /// `OrcaSlicer.cpp:2202-2215` with the CLI-default
    /// `use_first_fila_as_default=false`
    #[arg(long)]
    pub(crate) load_filaments: Option<String>,
    /// Print each slicing status to stderr as `Percent N: text` (`PrintBase.cpp:123-128`)
    #[arg(long)]
    pub(crate) progress: bool,
    #[arg(
        short = 'o',
        required_unless_present_any = [
            "selection",
            "export_3mf",
            "export_stl",
            "export_stls",
            "repair"
        ]
    )]
    pub(crate) output: Option<PathBuf>,
    /// Slice all 3MF plates (0) or the given one-based plate number.
    #[arg(long = "slice", allow_negative_numbers = true, conflicts_with_all = ["output", "plate", "dump_moves"])]
    pub(crate) selection: Option<i32>,
    /// Write an unsliced, headless FFF project instead of G-code.
    #[arg(long, value_name = "3MF")]
    pub(crate) export_3mf: Option<PathBuf>,
    /// Write one binary STL per object under the stl output subdirectory.
    #[arg(long, num_args = 0..=1, require_equals = true, default_missing_value = "true", value_parser = native_bool)]
    pub(crate) export_stl: Option<bool>,
    /// Write one binary STL per object under the supplied directory.
    #[arg(long, value_name = "DIRECTORY")]
    pub(crate) export_stls: Option<PathBuf>,
    /// Apply exact mesh cleanup before STL export.
    #[arg(long, num_args = 0..=1, require_equals = true, default_missing_value = "true", value_parser = native_bool)]
    pub(crate) repair: Option<bool>,
    /// Write plate_<n>.gcode in this directory for the --slice action or an export action.
    #[arg(long, requires = "action")]
    pub(crate) outputdir: Option<PathBuf>,
    #[arg(long)]
    pub(crate) min_save: Option<u32>,
    #[arg(long, requires = "export_action")]
    pub(crate) load_slicedata: Option<PathBuf>,
    #[arg(long, requires = "export_action")]
    pub(crate) export_slicedata: Option<PathBuf>,
    #[arg(value_name = "INPUT")]
    pub(crate) inputs: Vec<PathBuf>,
}

impl SliceArgs {
    pub(crate) fn input(&self) -> &Path {
        &self.inputs[0]
    }

    pub(crate) fn output(&self) -> &std::path::Path {
        self.output
            .as_deref()
            .unwrap_or_else(|| std::path::Path::new(""))
    }
}

fn native_bool(value: &str) -> Result<bool, &'static str> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" | "enabled" => Ok(true),
        "0" | "false" | "no" | "off" | "disabled" => Ok(false),
        _ => Err("expected a boolean value"),
    }
}
