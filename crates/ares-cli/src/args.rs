use std::path::PathBuf;

use ares_core::{CliObjectAssignment, CliProjectTransforms, CliVolumeTransform};
use clap::{ArgMatches, Parser, Subcommand};

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
    /// Skip saved 3MF instances by their identify IDs (comma-separated)
    #[arg(long, value_delimiter = ',', allow_hyphen_values = true)]
    skip_objects: Vec<i32>,
    /// Clone counts per input; saved 3MF input does not support cloning
    #[arg(long, value_delimiter = ',', allow_hyphen_values = true)]
    clone_objects: Vec<i32>,
    /// Filament IDs per input; saved 3MF input does not support assignment
    #[arg(long, value_delimiter = ',', allow_hyphen_values = true)]
    load_filament_ids: Vec<i32>,
    /// Orient model objects: 0 disables, 1 forces, other integers retain automatic eligibility
    #[arg(long, default_value_t = 0, allow_hyphen_values = true)]
    pub(crate) orient: i32,
    /// Arrange the project: 0 disables, 1 forces, other integers retain automatic eligibility
    #[arg(long, allow_hyphen_values = true)]
    arrange: Vec<i32>,
    /// Scale project volumes by this factor
    #[arg(long, allow_hyphen_values = true)]
    scale: Vec<f64>,
    /// Rotate project volumes around Z by this angle in degrees
    #[arg(long, allow_hyphen_values = true)]
    rotate: Vec<f64>,
    #[arg(skip)]
    pub(crate) transforms: CliProjectTransforms,
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

impl SliceArgs {
    pub(crate) fn object_assignment(&self) -> CliObjectAssignment {
        CliObjectAssignment::new(&self.clone_objects, &self.load_filament_ids)
    }

    pub(crate) fn parse_transforms(&mut self, args: &ArgMatches) {
        // Native retains the first occurrence of each key in the action order,
        // and applies the final scalar value of that key.
        let mut ordered = [
            (
                args.index_of("scale"),
                self.scale.last().copied().map(CliVolumeTransform::Scale),
            ),
            (
                args.index_of("rotate"),
                self.rotate.last().copied().map(CliVolumeTransform::Rotate),
            ),
        ];
        ordered.sort_by_key(|(index, _)| *index);
        self.transforms = CliProjectTransforms {
            volume: ordered
                .into_iter()
                .filter_map(|(_, action)| action)
                .collect(),
            orient: self.orient == 1,
            arrange: self.arrange.last().copied(),
            skip_objects: self.skip_objects.iter().copied().collect(),
        };
    }
}
