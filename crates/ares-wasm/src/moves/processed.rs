use super::{SliceOutput, VertexBuffers};
use crate::MoveBuffers;
use ares_core::ProcessedGCode;
use ares_vgcode::{
    ViewerImpl,
    convert::{ProcessorConversion, move_type_counts},
    filament_legend,
};

impl SliceOutput {
    /// Tool colors are the result's `extruder_colors`: the `filament_colour`
    /// GUI_Preview passes (`GUI_Preview.cpp:680`, `Plater.cpp:16889-16901`).
    pub fn from_processed(mut output: ProcessedGCode) -> Self {
        let result = &output.result;
        let move_type_counts = move_type_counts(&result.moves).to_vec();
        let mut conversion = ProcessorConversion::new(result);
        let mut moves = MoveBuffers::new(result.moves.len());
        for chunk in std::mem::take(&mut output.result.moves).into_chunks() {
            conversion.extend(&chunk);
            moves.extend(&chunk);
        }
        let data = conversion.finish();
        let result = &output.result;
        let palettes = (
            data.spiral_vase_mode,
            data.tools_colors.as_flattened().to_vec(),
            data.color_print_colors.as_flattened().to_vec(),
        );
        let mut viewer = ViewerImpl::default();
        viewer.load(data);
        let filament_legend = filament_legend(&viewer, result);
        // `PrintEstimatedStatistics` passes through as JSON — volumes in
        // mm³ dense-indexed by filament id; `total_used_filament` in mm.
        let print_statistics = serde_json::to_vec(&output.result.print_statistics)
            .expect("PrintEstimatedStatistics serializes");
        let estimation = serde_json::to_string(&crate::estimation::Estimation::from(&output))
            .expect("Estimation serializes");
        let mut vertices = VertexBuffers::pack(viewer, palettes);
        vertices.move_type_counts = move_type_counts;
        vertices.filament_legend = filament_legend;
        Self {
            gcode: output.gcode,
            moves,
            vertices,
            print_statistics,
            estimation,
            extruder_colors: output.result.extruder_colors,
            filament_diameters: output.result.filament_diameters,
            filament_densities: output.result.filament_densities,
            custom_gcode_overview: Vec::new(),
        }
    }
}
