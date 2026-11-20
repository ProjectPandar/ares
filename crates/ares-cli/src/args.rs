use std::path::PathBuf;

use clap::{Parser, Subcommand};

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
    #[command(flatten)]
    pub(crate) diagnostics: crate::diagnostics::DiagnosticLog,
    /// Slice all plates (0) or the given one-based plate number.
    #[arg(
        long = "slice",
        required_unless_present = "export_settings",
        requires = "outputdir",
        allow_negative_numbers = true
    )]
    pub(crate) selection: Option<i32>,
    /// Export effective settings to FILE, or to stdout with '-'.
    #[arg(long, value_name = "FILE")]
    pub(crate) export_settings: Option<PathBuf>,
    /// Maximum triangle count per plate for direct STL slicing (0 means unlimited).
    #[arg(long, allow_negative_numbers = true)]
    pub(crate) mtcpp: Option<i32>,
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
    /// Preserve an enabled prime tower after single-preset remapping in Smooth mode.
    #[arg(
        long,
        action = clap::ArgAction::Set,
        num_args = 0..=1,
        require_equals = true,
        default_value = "0",
        default_missing_value = "1",
        overrides_with = "enable_timelapse",
        value_parser = parse_enable_timelapse
    )]
    pub(crate) enable_timelapse: bool,
    #[arg(long)]
    pub(crate) load_settings: Option<String>,
    #[arg(long)]
    pub(crate) load_filaments: Option<String>,
    /// Replace saved layer G-code events with those from a JSON file for the selected plate.
    #[arg(long, value_name = "FILE", requires = "selection")]
    pub(crate) load_custom_gcodes: Option<PathBuf>,
    /// Fill empty --load-filaments entries with the first specified preset for STL slicing.
    #[arg(long)]
    pub(crate) load_defaultfila: bool,
    /// Preserve the input model positions (0).
    #[arg(long, value_parser = clap::value_parser!(u8).range(0..=0))]
    pub(crate) arrange: Option<u8>,
    #[arg(required = true)]
    pub(crate) inputs: Vec<PathBuf>,
}

fn parse_enable_timelapse(value: &str) -> Result<bool, &'static str> {
    // Config.cpp:1757-1782,1900-1906 and Config.hpp:1927-1945.
    if value.is_empty() {
        return Ok(true);
    }
    match value
        .split(',')
        .map(str::trim)
        .find(|item| !item.is_empty())
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "1" | "true" | "yes" | "on" | "enabled" => Ok(true),
        "0" | "false" | "no" | "off" | "disabled" => Ok(false),
        _ => Err("expected a boolean value"),
    }
}

#[derive(Parser)]
pub(crate) struct SliceArgs {
    /// Complete preset catalog supplied explicitly for project-derived refresh
    #[arg(long, conflicts_with_all = ["load_filaments", "resources_dir"])]
    pub(crate) project_preset_catalog: Option<PathBuf>,
    /// Refresh unlisted project values from the supplied system presets
    #[arg(long, requires = "project_preset_catalog")]
    pub(crate) uptodate: bool,
    #[arg(long)]
    pub(crate) options: Option<PathBuf>,
    /// 1-based plate number to slice for multi-plate 3MF projects
    #[arg(long)]
    pub(crate) plate: Option<u32>,
    /// Orient model objects: 0 disables, 1 forces, other integers retain automatic eligibility
    #[arg(long, default_value_t = 0, allow_hyphen_values = true)]
    pub(crate) orient: i32,
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
    #[arg(short = 'o')]
    pub(crate) output: PathBuf,
    pub(crate) input: PathBuf,
}
