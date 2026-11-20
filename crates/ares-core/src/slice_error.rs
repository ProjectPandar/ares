use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum PresetAdmissionReason {
    NotFound,
    Ambiguous,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PresetAdmissionError {
    pub kind: String,
    pub source: String,
    pub reason: PresetAdmissionReason,
}

impl fmt::Display for PresetAdmissionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.reason {
            PresetAdmissionReason::NotFound => {
                f.write_str("Preset was not found in the loaded bundle")
            }
            PresetAdmissionReason::Ambiguous => f.write_str("Preset identity is ambiguous"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum SliceError {
    EmptyInput,
    MissingProjectPlate(u32),
    InvalidInput(String),
    /// The CLI flush calculation cannot index the support flags for each slot.
    FilamentSupportCount,
    /// `Model::read_from_file` rejected the model in the option-only STL entry.
    OptionStlModelRead(Box<SliceError>),
    PresetAdmission(PresetAdmissionError),
    /// Support/palette admission (`OrcaSlicer.cpp:3929-3933`).
    FilamentSupportPaletteMismatch {
        support_count: usize,
        palette_count: usize,
    },
    /// Resolved non-FFF technology (`OrcaSlicer.cpp:3619,3627-3633`).
    InvalidPrinterTechnology,
    /// `CLI_POSTPROCESS_NOT_SUPPORTED` (`Utils.hpp:44`): a 3MF whose
    /// embedded `post_process` array is non-empty is rejected under the CLI
    /// (`OrcaSlicer.cpp:1682-1690`).
    PostProcessNotSupported,
    /// `CLI_PROCESS_NOT_COMPATIBLE` (`Utils.hpp:42`): a plain CLI 3MF slice
    /// whose embedded process `print_compatible_printers` does not contain
    /// the current printer's system name (`OrcaSlicer.cpp:2653-2654`).
    ProcessNotCompatible,
    /// `CLI_MODIFIED_PARAMS_TO_PRINTER` (`Utils.hpp:66`): retained project
    /// printable dimensions exceed the loaded machine's dimensions
    /// (`OrcaSlicer.cpp:2563-2600`).
    PrintableSizeExceedsDefault {
        width: i32,
        depth: i32,
        height: i32,
    },
    /// `CLI_NO_SUITABLE_OBJECTS` (`Utils.hpp:61`): the selected plate is
    /// empty or has no object fully inside it. The payload keeps the
    /// model-read detail for the stderr channel.
    NoSuitableObjects(String),
    /// `CLI_TRIANGLE_COUNT_EXCEEDS_LIMIT` (`OrcaSlicer.cpp:6637–6641`).
    TriangleCountExceedsLimit,
    /// `CLI_INVALID_VALUES_IN_3MF` (`Utils.hpp:43`): parameter values
    /// embedded in the 3MF/config fail validation; the payload is the
    /// `key: detail` block for the stderr channel.
    InvalidValuesInProject(String),
    /// `CLI_VALIDATE_ERROR` (`Utils.hpp:62`): `Print::validate` rejection.
    /// `sliced_info` is already populated at the validate point
    /// (`OrcaSlicer.cpp:5981-5987`), so the result.json fields ride along:
    /// plate_index, layer_height (the f32-stored value widened to f64),
    /// wall_loops, sparse_infill_density.
    Validate {
        message: String,
        plate_index: u32,
        layer_height: f64,
        wall_loops: i32,
        sparse_infill_density: f64,
    },
    SequentialPrintCollision {
        message: String,
        plate_index: u32,
        layer_height: f64,
        wall_loops: i32,
        sparse_infill_density: f64,
    },
    LayeredPrintCollision {
        message: String,
        plate_index: u32,
        layer_height: f64,
        wall_loops: i32,
        sparse_infill_density: f64,
    },
    CustomGCode {
        message: String,
        plate_index: u32,
        layer_height: f64,
        wall_loops: i32,
        sparse_infill_density: f64,
    },
    /// OrcaSlicer.cpp:6286-6291: a slicing/export exception, returned as CLI -100.
    Slicing {
        message: String,
        plate_index: u32,
        layer_height: f64,
        wall_loops: i32,
        sparse_infill_density: f64,
    },
    ProjectSlicingIncomplete,
    UnsupportedProjectFeature(String),
    ToolpathOutsidePrintableArea {
        error_code: u8,
        gcode: Vec<u8>,
        plate_index: u32,
        layer_height: f64,
        wall_loops: i32,
        sparse_infill_density: f64,
    },
}

impl fmt::Display for SliceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyInput => f.write_str("slice input is empty"),
            Self::MissingProjectPlate(plate) => write!(f, "project has no plate {plate}"),
            Self::InvalidInput(message) => f.write_str(message),
            Self::FilamentSupportCount => f.write_str("filament support and colour counts differ"),
            Self::TriangleCountExceedsLimit => f.write_str("plate triangle count exceeds the limit"),
            Self::OptionStlModelRead(error) => fmt::Display::fmt(error, f),
            Self::PresetAdmission(error) => error.fmt(f),
            Self::FilamentSupportPaletteMismatch {
                support_count,
                palette_count,
            } => write!(
                f,
                "filament_is_support's count {support_count} not equal to filament_colour's size {palette_count}"
            ),
            Self::InvalidPrinterTechnology => f.write_str("invalid printer_technology"),
            Self::PostProcessNotSupported => {
                f.write_str("post_process is not supported under CLI.")
            }
            Self::ProcessNotCompatible => f.write_str(
                "The selected printer is not compatible with the process preset in the 3mf.",
            ),
            Self::PrintableSizeExceedsDefault {
                width,
                depth,
                height,
            } => write!(
                f,
                "Printer Settings: the printable size {{{width}, {depth}, {height}}} exceeds the default size."
            ),
            Self::NoSuitableObjects(message) => f.write_str(message),
            Self::InvalidValuesInProject(message) => f.write_str(message),
            Self::Validate { message, .. }
            | Self::SequentialPrintCollision { message, .. }
            | Self::LayeredPrintCollision { message, .. }
            | Self::CustomGCode { message, .. }
            // OrcaSlicer.cpp:6286-6291 forwards ex.what() without decoration.
            | Self::Slicing { message, .. } => f.write_str(message),
            Self::ProjectSlicingIncomplete => f.write_str("ProjectSlicingIncomplete"),
            Self::UnsupportedProjectFeature(feature) => {
                write!(f, "unsupported project feature: {feature}")
            }
            Self::ToolpathOutsidePrintableArea { .. } => f.write_str("Found G-code in unprintable area of multi-extruder printers after slicing. Please make sure the 3mf file can be successfully sliced in the latest Orca Slicer."),
        }
    }
}

impl std::error::Error for SliceError {}
