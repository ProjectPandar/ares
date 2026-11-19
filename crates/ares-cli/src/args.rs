use std::path::PathBuf;

use ares_core::{CliProjectTransforms, CliVolumeTransform};
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
#[command(group(clap::ArgGroup::new("inspection").args(["inspect_mesh", "inspect_paint", "info"]).multiple(true)))]
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
    /// Arrange the project: 0 disables, 1 forces, other integers retain automatic eligibility
    #[arg(long, allow_hyphen_values = true)]
    arrange: Vec<i32>,
    /// Scale project volumes by this factor
    #[arg(long, allow_hyphen_values = true)]
    scale: Vec<f64>,
    /// Rotate project volumes around Z by this angle in degrees
    #[arg(long, allow_hyphen_values = true)]
    rotate: Vec<f64>,
    /// Ground each instance on the largest face of its convex hull
    #[arg(long, num_args = 0..=1, require_equals = true, default_missing_value = "true", value_parser = parse_ground_bool)]
    ground_largest_face: Vec<bool>,
    /// Ground each instance on the face closest to object-space normal NX,NY,NZ
    #[arg(long, allow_hyphen_values = true)]
    ground_face_normal: Vec<String>,
    /// Ground each instance on the face containing object-space point X,Y,Z
    #[arg(long, allow_hyphen_values = true)]
    ground_face_point: Vec<String>,
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

    /// Print model bounds and lay-on-face candidates as JSON without slicing
    #[arg(long)]
    pub(crate) inspect_mesh: bool,
    /// Print per-volume facet paint statistics as JSON without slicing
    #[arg(long)]
    pub(crate) inspect_paint: bool,
    /// Print model geometry and mesh statistics
    #[arg(long)]
    pub(crate) info: bool,
    /// Directory for the inspection result report
    #[arg(long, requires = "inspection")]
    pub(crate) outputdir: Option<PathBuf>,
    #[arg(long = "slice", requires = "inspection")]
    pub(crate) slice_plate: Option<u32>,
    #[arg(long, requires = "inspection", hide = true)]
    pub(crate) export_stl: bool,
    #[arg(long, requires = "inspection", hide = true)]
    pub(crate) export_3mf: Option<PathBuf>,
    #[arg(short = 'o', default_value = ".")]
    pub(crate) output: PathBuf,
    #[arg(default_value = ".")]
    pub(crate) input: PathBuf,
}

impl SliceArgs {
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
            (
                args.index_of("ground_largest_face"),
                self.ground_largest_face
                    .last()
                    .copied()
                    .map(CliVolumeTransform::GroundLargestFace),
            ),
            (
                args.index_of("ground_face_normal"),
                self.ground_face_normal
                    .pop()
                    .map(CliVolumeTransform::GroundFaceNormal),
            ),
            (
                args.index_of("ground_face_point"),
                self.ground_face_point
                    .pop()
                    .map(CliVolumeTransform::GroundFacePoint),
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
        };
    }
}

// `Config.cpp:1758–1785,1900–1905`: bare flags enable the transform;
// explicit boolean values follow the native spelling and require `=`.
fn parse_ground_bool(raw: &str) -> Result<bool, String> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "" | "1" | "true" | "yes" | "on" | "enabled" => Ok(true),
        "0" | "false" | "no" | "off" | "disabled" => Ok(false),
        _ => Err(format!("invalid boolean: {raw}")),
    }
}
