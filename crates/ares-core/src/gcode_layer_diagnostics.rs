use crate::gcode_format::format_decimal;
use crate::{
    LayerBrims, LayerContours, LayerExtrusionMoves, LayerInfills, LayerPerimeters, LayerPrintPaths,
    LayerSkirts, LayerSlice, LayerSpeedMoves, LayerToolpathMoves, Point2,
};

pub(crate) struct LayerDiagnosticCommand<'a> {
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

pub(crate) fn layer_diagnostics(command: LayerDiagnosticCommand<'_>) -> String {
    let mut gcode = String::new();
    gcode.push_str(&format!(
        "; segment_count = {}\n",
        command.layer_slice.segments().len()
    ));
    for segment in command.layer_slice.segments() {
        let start = segment.start();
        let end = segment.end();
        gcode.push_str(&format!(
            ";SEGMENT:{},{} -> {},{}\n",
            format_decimal(start.x()),
            format_decimal(start.y()),
            format_decimal(end.x()),
            format_decimal(end.y())
        ));
    }
    gcode.push_str(&format!(
        "; contour_count = {}\n",
        command.layer_contours.contours().len()
    ));
    for contour in command.layer_contours.contours() {
        gcode.push_str(&format!(";CONTOUR:{}\n", points(contour.points())));
    }
    gcode.push_str(&format!(
        "; perimeter_count = {}\n",
        command.layer_perimeters.paths().len()
    ));
    for perimeter in command.layer_perimeters.paths() {
        gcode.push_str(&format!(
            ";PERIMETER:{}:{}\n",
            perimeter.role().as_str(),
            points(perimeter.points())
        ));
    }
    gcode.push_str(&format!(
        "; infill_count = {}\n",
        command.layer_infills.paths().len()
    ));
    for infill in command.layer_infills.paths() {
        let start = infill.points()[0];
        let end = infill.points()[1];
        gcode.push_str(&format!(
            ";INFILL:{}:{},{} -> {},{}\n",
            infill.role().as_str(),
            format_decimal(start.x()),
            format_decimal(start.y()),
            format_decimal(end.x()),
            format_decimal(end.y())
        ));
    }
    gcode.push_str(&format!(
        "; skirt_count = {}\n",
        command.layer_skirts.paths().len()
    ));
    for skirt in command.layer_skirts.paths() {
        gcode.push_str(&format!(";SKIRT:{}\n", points(skirt.points())));
    }
    gcode.push_str(&format!(
        "; brim_count = {}\n",
        command.layer_brims.paths().len()
    ));
    for brim in command.layer_brims.paths() {
        gcode.push_str(&format!(";BRIM:{}\n", points(brim.points())));
    }
    gcode.push_str(&format!(
        "; print_path_count = {}\n",
        command.layer_print_paths.paths().len()
    ));
    for print_path in command.layer_print_paths.paths() {
        gcode.push_str(&format!(
            ";PRINT_PATH:{}:{}\n",
            crate::print_paths::diagnostic_role_label(
                print_path.role(),
                print_path.extrusion_role()
            ),
            points(print_path.points())
        ));
    }
    gcode.push_str(&format!(
        "; toolpath_move_count = {}\n; extrusion_move_count = {}\n; speed_move_count = {}\n; extrusion_mm = {}\n",
        command.layer_toolpath_moves.moves().len(),
        command.layer_extrusion_moves.moves().len(),
        command.layer_speed_moves.moves().len(),
        format_decimal(command.layer_extrusion_moves.total_extrusion_mm())
    ));
    gcode
}

fn points(points: &[Point2]) -> String {
    points
        .iter()
        .map(|point| {
            format!(
                "{},{}",
                format_decimal(point.x()),
                format_decimal(point.y())
            )
        })
        .collect::<Vec<_>>()
        .join(" -> ")
}
