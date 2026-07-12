use crate::{LayerExtrusionMoves, Point2, PrintPathRole, ToolpathMoveKind};

mod config;
mod kinematics;
mod layer_time;
mod resonance_avoidance;
mod slow_down_layers;
mod small_perimeter;
mod volumetric;

pub use config::{OverhangSpeedBands, SpeedOptions};
pub use kinematics::{AccelerationOptions, JerkOptions};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpeedMove {
    kind: ToolpathMoveKind,
    role: PrintPathRole,
    extrusion_role: Option<PrintPathRole>,
    point: Point2,
    e_position: Option<f64>,
    speed_mm_s: f64,
    feedrate_mm_min: f64,
    acceleration_mm_s2: Option<f64>,
    jerk_mm_s: Option<f64>,
    effective_line_width_mm: Option<f64>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpeedMoveKinematics {
    speed_mm_s: f64,
    acceleration_mm_s2: Option<f64>,
    jerk_mm_s: Option<f64>,
}

impl SpeedMoveKinematics {
    pub const fn new(
        speed_mm_s: f64,
        acceleration_mm_s2: Option<f64>,
        jerk_mm_s: Option<f64>,
    ) -> Self {
        Self {
            speed_mm_s,
            acceleration_mm_s2,
            jerk_mm_s,
        }
    }
}

impl SpeedMove {
    pub const fn new(
        kind: ToolpathMoveKind,
        role: PrintPathRole,
        point: Point2,
        e_position: Option<f64>,
        kinematics: SpeedMoveKinematics,
    ) -> Self {
        Self {
            kind,
            role,
            extrusion_role: None,
            point,
            e_position,
            speed_mm_s: kinematics.speed_mm_s,
            feedrate_mm_min: kinematics.speed_mm_s * 60.0,
            acceleration_mm_s2: kinematics.acceleration_mm_s2,
            jerk_mm_s: kinematics.jerk_mm_s,
            effective_line_width_mm: None,
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

    pub const fn speed_mm_s(&self) -> f64 {
        self.speed_mm_s
    }

    pub const fn feedrate_mm_min(&self) -> f64 {
        self.feedrate_mm_min
    }

    pub const fn acceleration_mm_s2(&self) -> Option<f64> {
        self.acceleration_mm_s2
    }

    pub const fn jerk_mm_s(&self) -> Option<f64> {
        self.jerk_mm_s
    }

    pub const fn effective_line_width_mm(&self) -> Option<f64> {
        self.effective_line_width_mm
    }

    pub const fn with_effective_line_width_mm(
        mut self,
        effective_line_width_mm: Option<f64>,
    ) -> Self {
        self.effective_line_width_mm = effective_line_width_mm;
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LayerSpeedMoves {
    layer_id: usize,
    print_z: f64,
    moves: Vec<SpeedMove>,
}

impl LayerSpeedMoves {
    pub fn new(layer_id: usize, print_z: f64, moves: Vec<SpeedMove>) -> Self {
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

    pub fn moves(&self) -> &[SpeedMove] {
        &self.moves
    }
}

pub fn generate_speed_moves(
    layers: &[LayerExtrusionMoves],
    options: SpeedOptions,
) -> Vec<LayerSpeedMoves> {
    volumetric::generate_capped_speed_moves(layers, options)
}

#[cfg(test)]
mod tests;
