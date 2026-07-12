mod options;
mod small_area;

use crate::{
    Layer, LayerToolpathMoves, Point2, PrintPathRole, SliceError, ToolpathMove, ToolpathMoveKind,
};

pub(crate) use options::ExplicitExtrusionSegment;
pub use options::ExtrusionOptions;
pub(crate) use options::{ExtrusionWidthSpec, RoleExtrusionHardware, RoleHardwareValues};
pub(crate) use small_area::SmallAreaInfillFlowCompensation;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExtrusionMove {
    kind: ToolpathMoveKind,
    role: PrintPathRole,
    extrusion_role: Option<PrintPathRole>,
    point: Point2,
    e_position: Option<f64>,
    effective_layer_height_mm: Option<f64>,
    effective_line_width_mm: Option<f64>,
    unsupported_span_mm: Option<f64>,
}

impl ExtrusionMove {
    pub const fn new(
        kind: ToolpathMoveKind,
        role: PrintPathRole,
        point: Point2,
        e_position: Option<f64>,
    ) -> Self {
        Self {
            kind,
            role,
            extrusion_role: None,
            point,
            e_position,
            effective_layer_height_mm: None,
            effective_line_width_mm: None,
            unsupported_span_mm: None,
        }
    }

    pub const fn with_adaptive_volumetric_geometry(
        self,
        effective_layer_height_mm: f64,
        effective_line_width_mm: f64,
    ) -> Self {
        Self {
            effective_layer_height_mm: Some(effective_layer_height_mm),
            effective_line_width_mm: Some(effective_line_width_mm),
            ..self
        }
    }

    pub const fn kind(&self) -> ToolpathMoveKind {
        self.kind
    }

    pub const fn role(&self) -> PrintPathRole {
        self.role
    }

    pub(crate) const fn extrusion_role(&self) -> Option<PrintPathRole> {
        self.extrusion_role
    }

    pub(crate) const fn with_extrusion_role(
        mut self,
        extrusion_role: Option<PrintPathRole>,
    ) -> Self {
        self.extrusion_role = extrusion_role;
        self
    }

    pub const fn point(&self) -> Point2 {
        self.point
    }

    pub const fn e_position(&self) -> Option<f64> {
        self.e_position
    }

    pub const fn effective_layer_height_mm(&self) -> Option<f64> {
        self.effective_layer_height_mm
    }

    pub const fn effective_line_width_mm(&self) -> Option<f64> {
        self.effective_line_width_mm
    }

    pub const fn with_unsupported_span_mm(mut self, unsupported_span_mm: Option<f64>) -> Self {
        self.unsupported_span_mm = unsupported_span_mm;
        self
    }

    pub const fn unsupported_span_mm(&self) -> Option<f64> {
        self.unsupported_span_mm
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LayerExtrusionMoves {
    layer_id: usize,
    print_z: f64,
    moves: Vec<ExtrusionMove>,
    total_extrusion_mm: f64,
}

impl LayerExtrusionMoves {
    pub fn new(
        layer_id: usize,
        print_z: f64,
        moves: Vec<ExtrusionMove>,
        total_extrusion_mm: f64,
    ) -> Self {
        Self {
            layer_id,
            print_z,
            moves,
            total_extrusion_mm,
        }
    }

    pub const fn layer_id(&self) -> usize {
        self.layer_id
    }

    pub const fn print_z(&self) -> f64 {
        self.print_z
    }

    pub fn moves(&self) -> &[ExtrusionMove] {
        &self.moves
    }

    pub const fn total_extrusion_mm(&self) -> f64 {
        self.total_extrusion_mm
    }
}

pub fn generate_extrusion_moves(
    layers: &[Layer],
    moves: &[LayerToolpathMoves],
    options: ExtrusionOptions,
) -> Result<Vec<LayerExtrusionMoves>, SliceError> {
    if layers.len() != moves.len() {
        return Err(SliceError::InvalidInput(
            "layer and move counts must match".to_owned(),
        ));
    }

    let mut e_position = 0.0;
    let mut current_point = None;
    let mut output = Vec::with_capacity(layers.len());

    for (layer, layer_moves) in layers.iter().zip(moves.iter()) {
        if layer.id() != layer_moves.layer_id() || layer.print_z() != layer_moves.print_z() {
            return Err(SliceError::InvalidInput(
                "layer and move metadata must match".to_owned(),
            ));
        }

        let layer_start_e = e_position;
        let mut extrusion_moves = Vec::with_capacity(layer_moves.moves().len());
        let is_first_layer = layer.id() == 0;
        for toolpath_move in layer_moves.moves() {
            let point = toolpath_move.point();
            let extrusion_role = toolpath_move
                .extrusion_role()
                .unwrap_or_else(|| toolpath_move.role());
            let effective_layer_height = toolpath_move
                .effective_layer_height_mm()
                .unwrap_or_else(|| layer.height());
            let effective_line_width =
                toolpath_move.effective_line_width_mm().unwrap_or_else(|| {
                    options.width_for_role_and_layer(extrusion_role, is_first_layer)
                });
            let move_e = match toolpath_move.kind() {
                ToolpathMoveKind::Travel => None,
                ToolpathMoveKind::Print => {
                    let start = current_point.unwrap_or(point);
                    let distance = distance(start, point);
                    let delta = extrusion_delta_for_toolpath_move(
                        &options,
                        toolpath_move,
                        ToolpathExtrusionGeometry {
                            effective_layer_height,
                            effective_line_width,
                            is_first_layer,
                            distance,
                        },
                    )?;
                    e_position = round_6(e_position + delta);
                    Some(e_position)
                }
            };
            let extrusion_move =
                ExtrusionMove::new(toolpath_move.kind(), toolpath_move.role(), point, move_e)
                    .with_extrusion_role(toolpath_move.extrusion_role())
                    .with_unsupported_span_mm(toolpath_move.unsupported_span_mm());
            let extrusion_move = match toolpath_move.kind() {
                ToolpathMoveKind::Travel => extrusion_move,
                ToolpathMoveKind::Print => extrusion_move.with_adaptive_volumetric_geometry(
                    effective_layer_height,
                    effective_line_width,
                ),
            };
            extrusion_moves.push(extrusion_move);
            current_point = Some(point);
        }
        output.push(LayerExtrusionMoves::new(
            layer.id(),
            layer.print_z(),
            extrusion_moves,
            round_6(e_position - layer_start_e),
        ));
    }

    Ok(output)
}

fn extrusion_delta_for_toolpath_move(
    options: &ExtrusionOptions,
    toolpath_move: &ToolpathMove,
    geometry: ToolpathExtrusionGeometry,
) -> Result<f64, SliceError> {
    if geometry.effective_layer_height == 0.0 && toolpath_move.extrusion_role().is_some() {
        return Ok(0.0);
    }
    options.extrusion_delta_for_segment_with_width(ExplicitExtrusionSegment {
        role: toolpath_move
            .extrusion_role()
            .unwrap_or_else(|| toolpath_move.role()),
        layer_height: geometry.effective_layer_height,
        is_first_layer: geometry.is_first_layer,
        line_width: geometry.effective_line_width,
        line_length_mm: geometry.distance,
    })
}

#[derive(Clone, Copy)]
struct ToolpathExtrusionGeometry {
    effective_layer_height: f64,
    effective_line_width: f64,
    is_first_layer: bool,
    distance: f64,
}

fn distance(start: Point2, end: Point2) -> f64 {
    ((end.x() - start.x()).powi(2) + (end.y() - start.y()).powi(2)).sqrt()
}

fn round_6(value: f64) -> f64 {
    (value * 1_000_000.0).round() / 1_000_000.0
}

#[cfg(test)]
mod tests;
