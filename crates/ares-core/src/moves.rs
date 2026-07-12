use crate::{LayerPrintPaths, Point2, PrintPathRole};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ToolpathMoveKind {
    Travel,
    Print,
}

impl ToolpathMoveKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Travel => "travel",
            Self::Print => "print",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ToolpathMove {
    kind: ToolpathMoveKind,
    role: PrintPathRole,
    extrusion_role: Option<PrintPathRole>,
    point: Point2,
    effective_layer_height_mm: Option<f64>,
    effective_line_width_mm: Option<f64>,
    unsupported_span_mm: Option<f64>,
}

impl ToolpathMove {
    pub const fn new(kind: ToolpathMoveKind, role: PrintPathRole, point: Point2) -> Self {
        Self {
            kind,
            role,
            extrusion_role: None,
            point,
            effective_layer_height_mm: None,
            effective_line_width_mm: None,
            unsupported_span_mm: None,
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

    pub fn with_effective_layer_height_mm(
        mut self,
        effective_layer_height_mm: Option<f64>,
    ) -> Self {
        self.effective_layer_height_mm = effective_layer_height_mm;
        self
    }

    pub const fn effective_layer_height_mm(&self) -> Option<f64> {
        self.effective_layer_height_mm
    }

    pub const fn with_effective_line_width_mm(
        mut self,
        effective_line_width_mm: Option<f64>,
    ) -> Self {
        self.effective_line_width_mm = effective_line_width_mm;
        self
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
pub struct LayerToolpathMoves {
    layer_id: usize,
    print_z: f64,
    moves: Vec<ToolpathMove>,
}

impl LayerToolpathMoves {
    pub fn new(layer_id: usize, print_z: f64, moves: Vec<ToolpathMove>) -> Self {
        Self {
            layer_id,
            print_z,
            moves,
        }
    }

    pub const fn layer_id(&self) -> usize {
        self.layer_id
    }

    pub const fn print_z(&self) -> f64 {
        self.print_z
    }

    pub fn moves(&self) -> &[ToolpathMove] {
        &self.moves
    }
}

pub fn generate_toolpath_moves(layers: &[LayerPrintPaths]) -> Vec<LayerToolpathMoves> {
    layers
        .iter()
        .map(|layer| {
            let mut moves = Vec::new();
            for path in layer.paths() {
                let points = path.points();
                let first = points[0];
                moves.push(
                    ToolpathMove::new(ToolpathMoveKind::Travel, path.role(), first)
                        .with_extrusion_role(path.extrusion_role())
                        .with_effective_layer_height_mm(path.effective_layer_height_mm())
                        .with_effective_line_width_mm(path.effective_line_width_mm())
                        .with_unsupported_span_mm(path.unsupported_span_mm()),
                );
                moves.extend(points[1..].iter().map(|point| {
                    ToolpathMove::new(ToolpathMoveKind::Print, path.role(), *point)
                        .with_extrusion_role(path.extrusion_role())
                        .with_effective_layer_height_mm(path.effective_layer_height_mm())
                        .with_effective_line_width_mm(path.effective_line_width_mm())
                        .with_unsupported_span_mm(path.unsupported_span_mm())
                }));
                let closing_target = path_closing_target(path, points);
                if let Some(target) = closing_target {
                    moves.push(
                        ToolpathMove::new(ToolpathMoveKind::Print, path.role(), target)
                            .with_extrusion_role(path.extrusion_role())
                            .with_effective_layer_height_mm(path.effective_layer_height_mm())
                            .with_effective_line_width_mm(path.effective_line_width_mm())
                            .with_unsupported_span_mm(path.unsupported_span_mm()),
                    );
                }
            }

            LayerToolpathMoves::new(layer.layer_id(), layer.print_z(), moves)
        })
        .collect()
}

fn path_closing_target(path: &crate::PrintPath, points: &[Point2]) -> Option<Point2> {
    if !path.is_closed() {
        return None;
    }
    match path.role() {
        PrintPathRole::ExternalPerimeter
        | PrintPathRole::OverhangPerimeter
        | PrintPathRole::InternalPerimeter => closing_target(points, path.seam_gap_mm()),
        PrintPathRole::Skirt | PrintPathRole::Brim => Some(points[0]),
        _ => None,
    }
}

fn closing_target(points: &[Point2], seam_gap_mm: f64) -> Option<Point2> {
    let start = points[0];
    let end = *points.last().unwrap();
    let length = distance(end, start);
    if length <= f64::EPSILON {
        return None;
    }
    if seam_gap_mm <= 0.0 {
        return Some(start);
    }
    if seam_gap_mm >= length {
        return None;
    }
    let ratio = (length - seam_gap_mm) / length;
    Some(Point2::new(
        end.x() + (start.x() - end.x()) * ratio,
        end.y() + (start.y() - end.y()) * ratio,
    ))
}

fn distance(a: Point2, b: Point2) -> f64 {
    ((a.x() - b.x()).powi(2) + (a.y() - b.y()).powi(2)).sqrt()
}

#[cfg(test)]
mod tests;
