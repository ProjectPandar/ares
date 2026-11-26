use super::{SliceOutput, VertexBuffers};
use ares_core::ProcessedGCode;
use ares_vgcode::{
    ViewerImpl,
    convert::{ProcessorConversion, move_type_counts},
    filament_legend,
};

impl SliceOutput {
    /// Tool colors are the result's `extruder_colors`: the `filament_colour`
    /// GUI_Preview passes (`GUI_Preview.cpp:680`, `Plater.cpp:16889-16901`).
    pub fn from_processed(output: ProcessedGCode) -> Self {
        let result = &output.result;
        let move_type_counts = move_type_counts(&result.moves).to_vec();
        let mut conversion = ProcessorConversion::new(result);
        for chunk in result.moves.chunks() {
            conversion.extend(chunk);
        }
        let data = conversion.finish();
        let palettes = (
            data.spiral_vase_mode,
            data.tools_colors.as_flattened().to_vec(),
            data.color_print_colors.as_flattened().to_vec(),
        );
        let mut viewer = ViewerImpl::default();
        viewer.load(data);
        let filament_legend = filament_legend(&viewer, result);
        let estimation = crate::estimation::Estimation::from(&output);
        let mut vertices = VertexBuffers::from_viewer(viewer, palettes);
        vertices.move_type_counts = move_type_counts;
        vertices.filament_legend = filament_legend;
        Self {
            gcode: output.gcode,
            moves: output.result.moves,
            vertices,
            print_statistics: Some(output.result.print_statistics),
            estimation,
            extruder_colors: output.result.extruder_colors,
            filament_diameters: output.result.filament_diameters,
            filament_densities: output.result.filament_densities,
            custom_gcode_overview: Vec::new(),
        }
    }
}
