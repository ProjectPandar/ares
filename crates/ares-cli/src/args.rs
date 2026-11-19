use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "ares")]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Commands,
}

#[derive(Subcommand)]
pub(crate) enum Commands {
    Slice(SliceArgs),
}

#[derive(Parser)]
#[command(group(clap::ArgGroup::new("export_action").multiple(true)))]
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
    /// Write an unsliced, headless FFF project instead of G-code
    #[arg(long, value_name = "3MF", group = "export_action")]
    pub(crate) export_3mf: Option<PathBuf>,
    /// Write one binary STL per object under the stl output subdirectory
    #[arg(long, group = "export_action", num_args = 0..=1, require_equals = true, default_missing_value = "true", value_parser = native_bool)]
    pub(crate) export_stl: Option<bool>,
    /// Write one binary STL per object under the supplied directory
    #[arg(long, value_name = "DIRECTORY", group = "export_action")]
    pub(crate) export_stls: Option<PathBuf>,
    /// Apply exact mesh cleanup before STL export
    #[arg(long, num_args = 0..=1, require_equals = true, default_missing_value = "true", value_parser = native_bool)]
    pub(crate) repair: Option<bool>,
    #[arg(long, requires = "export_action")]
    pub(crate) outputdir: Option<PathBuf>,
    #[arg(long = "slice", requires = "export_action")]
    pub(crate) slice_for_export: Option<u32>,
    #[arg(long, requires = "export_action")]
    pub(crate) arrange: Option<u32>,
    #[arg(long, requires = "export_action")]
    pub(crate) min_save: Option<u32>,
    #[arg(long, requires = "export_action")]
    pub(crate) load_slicedata: Option<PathBuf>,
    #[arg(long, requires = "export_action")]
    pub(crate) export_slicedata: Option<PathBuf>,
    #[arg(
        short = 'o',
        value_name = "OUTPUT",
        required_unless_present_any = ["export_3mf", "export_stl", "export_stls", "repair"]
    )]
    pub(crate) output_path: Option<PathBuf>,
    #[arg(skip)]
    pub(crate) output: PathBuf,
    pub(crate) input: PathBuf,
}

fn native_bool(value: &str) -> Result<bool, &'static str> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" | "enabled" => Ok(true),
        "0" | "false" | "no" | "off" | "disabled" => Ok(false),
        _ => Err("expected a boolean value"),
    }
}
