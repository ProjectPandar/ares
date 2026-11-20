use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "ares",
    args_conflicts_with_subcommands = true,
    subcommand_negates_reqs = true
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
    #[arg(long = "slice", required = true, allow_negative_numbers = true)]
    pub(crate) selection: Option<i32>,
    /// Directory for plate_<n>.gcode and the slice result.
    #[arg(long, required = true)]
    pub(crate) outputdir: Option<PathBuf>,
    #[arg(long)]
    pub(crate) load_settings: Option<String>,
    #[arg(long)]
    pub(crate) load_filaments: Option<String>,
    #[command(flatten)]
    pub(crate) arrange: crate::direct_stl::arrange::DirectArrangeArgs,
    #[arg(required = true)]
    pub(crate) inputs: Vec<PathBuf>,
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
