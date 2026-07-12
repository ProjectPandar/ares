use crate::{
    LayerBrims, LayerContours, LayerExtrusionMoves, LayerInfills, LayerPerimeters, LayerPrintPaths,
    LayerSkirts, LayerSlice, LayerSpeedMoves, LayerToolpathMoves,
};

pub(crate) struct LayerDiagnosticEmitCommand<'a> {
    pub(crate) layer_slice: &'a LayerSlice,
    pub(crate) layer_contours: &'a LayerContours,
    pub(crate) layer_perimeters: &'a LayerPerimeters,
    pub(crate) layer_infills: &'a LayerInfills,
    pub(crate) layer_skirts: &'a LayerSkirts,
    pub(crate) layer_brims: &'a LayerBrims,
    pub(crate) layer_print_paths: &'a LayerPrintPaths,
    pub(crate) layer_toolpath_moves: &'a LayerToolpathMoves,
    pub(crate) layer_extrusion_moves: &'a LayerExtrusionMoves,
    pub(crate) layer_speed_moves: &'a LayerSpeedMoves,
}

pub(crate) fn layer_diagnostics(command: LayerDiagnosticEmitCommand<'_>) -> String {
    crate::gcode_layer_diagnostics::layer_diagnostics(
        crate::gcode_layer_diagnostics::LayerDiagnosticCommand {
            layer_slice: command.layer_slice,
            layer_contours: command.layer_contours,
            layer_perimeters: command.layer_perimeters,
            layer_infills: command.layer_infills,
            layer_skirts: command.layer_skirts,
            layer_brims: command.layer_brims,
            layer_print_paths: command.layer_print_paths,
            layer_toolpath_moves: command.layer_toolpath_moves,
            layer_extrusion_moves: command.layer_extrusion_moves,
            layer_speed_moves: command.layer_speed_moves,
        },
    )
}
