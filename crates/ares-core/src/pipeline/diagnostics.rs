use crate::InputFormat;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PipelineStage {
    Model,
    Layers,
    Segments,
    Contours,
    Perimeters,
    Infills,
    Skirts,
    Brims,
    PrintPaths,
    Moves,
    Extrusions,
    Speeds,
}

impl PipelineStage {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Model => "model",
            Self::Layers => "layers",
            Self::Segments => "segments",
            Self::Contours => "contours",
            Self::Perimeters => "perimeters",
            Self::Infills => "infills",
            Self::Skirts => "skirts",
            Self::Brims => "brims",
            Self::PrintPaths => "print_paths",
            Self::Moves => "moves",
            Self::Extrusions => "extrusions",
            Self::Speeds => "speeds",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PipelineDiagnostics {
    pub(crate) completed_stages: Vec<PipelineStage>,
    pub(crate) input_format: InputFormat,
    pub(crate) triangle_count: usize,
    pub(crate) layer_count: usize,
    pub(crate) total_segment_count: usize,
    pub(crate) total_contour_count: usize,
    pub(crate) total_perimeter_count: usize,
    pub(crate) total_infill_count: usize,
    pub(crate) total_skirt_path_count: usize,
    pub(crate) total_brim_path_count: usize,
    pub(crate) total_print_path_count: usize,
    pub(crate) total_toolpath_move_count: usize,
    pub(crate) total_extrusion_move_count: usize,
    pub(crate) total_speed_move_count: usize,
    pub(crate) total_extrusion_mm: f64,
    pub(crate) empty_layer_count: usize,
    pub(crate) option_count: usize,
}

impl PipelineDiagnostics {
    pub fn completed_stages(&self) -> &[PipelineStage] {
        &self.completed_stages
    }

    pub const fn input_format(&self) -> InputFormat {
        self.input_format
    }

    pub const fn triangle_count(&self) -> usize {
        self.triangle_count
    }

    pub const fn layer_count(&self) -> usize {
        self.layer_count
    }

    pub const fn total_segment_count(&self) -> usize {
        self.total_segment_count
    }

    pub const fn total_contour_count(&self) -> usize {
        self.total_contour_count
    }

    pub const fn total_perimeter_count(&self) -> usize {
        self.total_perimeter_count
    }

    pub const fn total_infill_count(&self) -> usize {
        self.total_infill_count
    }

    pub const fn total_skirt_path_count(&self) -> usize {
        self.total_skirt_path_count
    }

    pub const fn total_brim_path_count(&self) -> usize {
        self.total_brim_path_count
    }

    pub const fn total_print_path_count(&self) -> usize {
        self.total_print_path_count
    }

    pub const fn total_toolpath_move_count(&self) -> usize {
        self.total_toolpath_move_count
    }

    pub const fn total_extrusion_move_count(&self) -> usize {
        self.total_extrusion_move_count
    }

    pub const fn total_speed_move_count(&self) -> usize {
        self.total_speed_move_count
    }

    pub const fn total_extrusion_mm(&self) -> f64 {
        self.total_extrusion_mm
    }

    pub const fn empty_layer_count(&self) -> usize {
        self.empty_layer_count
    }

    pub const fn option_count(&self) -> usize {
        self.option_count
    }
}
