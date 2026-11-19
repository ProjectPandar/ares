use std::path::{Path, PathBuf};

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
    #[arg(short = 'o')]
    pub(crate) output: PathBuf,
    #[arg(value_name = "INPUT")]
    pub(crate) inputs: Vec<PathBuf>,
}

impl SliceArgs {
    pub(crate) fn input(&self) -> &Path {
        &self.inputs[0]
    }
}
