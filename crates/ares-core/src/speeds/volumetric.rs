use crate::{
    ExtrusionMove, LayerExtrusionMoves, LayerSpeedMoves, Point2, PrintPathRole, SpeedMove,
    SpeedMoveKinematics, SpeedOptions, ToolpathMoveKind,
};

pub(super) fn generate_capped_speed_moves(
    layers: &[LayerExtrusionMoves],
    options: SpeedOptions,
) -> Vec<LayerSpeedMoves> {
    let mut volumetric_cap = VolumetricSpeedCap::new(options);
    let mut volumetric_rate_smoothing = VolumetricRateSmoothing::new(options);
    let mut layer_time_slowdown = super::layer_time::LayerTimeSlowdown::new(options);
    layers
        .iter()
        .map(|layer| {
            let is_first_layer = layer.layer_id() == 0;
            let configured_speeds =
                super::small_perimeter::speeds_for_layer(layer.moves(), &options, |move_| {
                    let base_speed = super::slow_down_layers::speed_for_layer_id(
                        &options,
                        move_.kind(),
                        move_.role(),
                        layer.layer_id(),
                    );
                    overhang_band_speed(&options, move_, base_speed, is_first_layer)
                });
            let capped_speeds = layer
                .moves()
                .iter()
                .zip(configured_speeds)
                .map(|(move_, configured_speed)| {
                    let capped_speed = volumetric_cap.capped_speed(move_, configured_speed);
                    super::resonance_avoidance::adjusted_speed(&options, move_, capped_speed)
                })
                .collect::<Vec<_>>();
            let smoothed_speeds = layer
                .moves()
                .iter()
                .zip(capped_speeds)
                .map(|(move_, speed)| volumetric_rate_smoothing.apply(move_, speed))
                .collect::<Vec<_>>();
            let final_speeds = layer_time_slowdown.apply(layer.moves(), smoothed_speeds);
            let moves = layer
                .moves()
                .iter()
                .zip(final_speeds)
                .map(|(move_, speed)| {
                    SpeedMove::new(
                        move_.kind(),
                        move_.role(),
                        move_.point(),
                        move_.e_position(),
                        SpeedMoveKinematics::new(
                            speed,
                            options.acceleration_for_layer(
                                move_.kind(),
                                move_.role(),
                                is_first_layer,
                            ),
                            options.jerk_for_layer(move_.kind(), move_.role(), is_first_layer),
                        ),
                    )
                    .with_extrusion_role(move_.extrusion_role())
                    .with_effective_line_width_mm(move_.effective_line_width_mm())
                })
                .collect();
            LayerSpeedMoves::new(layer.layer_id(), layer.print_z(), moves)
        })
        .collect()
}

fn overhang_band_speed(
    options: &SpeedOptions,
    move_: &ExtrusionMove,
    base_speed_mm_s: f64,
    is_first_layer: bool,
) -> f64 {
    if is_first_layer
        || move_.kind() != ToolpathMoveKind::Print
        || move_.role() != PrintPathRole::OverhangPerimeter
    {
        return base_speed_mm_s;
    }
    move_.unsupported_span_mm().map_or(base_speed_mm_s, |span| {
        options
            .overhang_speed_for_unsupported_span_mm(span)
            .map_or(base_speed_mm_s, |speed| speed.min(base_speed_mm_s))
    })
}

#[derive(Clone, Copy)]
struct PrintFlow {
    mm3_per_mm: f64,
    distance_mm: f64,
    rate_mm3_s: f64,
    duration_s: f64,
}

impl PrintFlow {
    fn at_speed(mm3_per_mm: f64, distance_mm: f64, speed_mm_s: f64) -> Option<Self> {
        if mm3_per_mm <= 0.0 || distance_mm <= 0.0 || speed_mm_s <= 0.0 {
            return None;
        }
        Some(Self {
            mm3_per_mm,
            distance_mm,
            rate_mm3_s: mm3_per_mm * speed_mm_s,
            duration_s: distance_mm / speed_mm_s,
        })
    }
}

struct VolumetricRateSmoothing {
    filament_area_mm2: f64,
    max_slope_mm3_s2: f64,
    external_only: bool,
    last_point: Option<Point2>,
    last_print_e: f64,
    previous_eligible: Option<PrintFlow>,
}

impl VolumetricRateSmoothing {
    fn new(options: SpeedOptions) -> Self {
        Self {
            filament_area_mm2: std::f64::consts::PI
                * (options.filament_diameter_mm() / 2.0).powi(2),
            max_slope_mm3_s2: options.max_volumetric_extrusion_rate_slope_mm3_s2(),
            external_only: options.extrusion_rate_smoothing_external_perimeter_only(),
            last_point: None,
            last_print_e: 0.0,
            previous_eligible: None,
        }
    }

    fn apply(&mut self, move_: &ExtrusionMove, speed_mm_s: f64) -> f64 {
        match move_.kind() {
            ToolpathMoveKind::Travel => {
                self.last_point = Some(move_.point());
                speed_mm_s
            }
            ToolpathMoveKind::Print => {
                let flow_at_speed = self.print_flow(move_, speed_mm_s);
                self.last_point = Some(move_.point());
                if self.max_slope_mm3_s2 <= 0.0 || !self.is_eligible(move_) {
                    return speed_mm_s;
                }
                let Some(flow_at_speed) = flow_at_speed else {
                    return speed_mm_s;
                };
                let Some(previous) = self.previous_eligible else {
                    self.previous_eligible = Some(flow_at_speed);
                    return speed_mm_s;
                };
                let allowed_rate =
                    previous.rate_mm3_s + self.max_slope_mm3_s2 * previous.duration_s;
                if flow_at_speed.rate_mm3_s <= allowed_rate {
                    self.previous_eligible = Some(flow_at_speed);
                    return speed_mm_s;
                }
                let smoothed_speed = speed_mm_s * allowed_rate / flow_at_speed.rate_mm3_s;
                if let Some(smoothed_flow) = PrintFlow::at_speed(
                    flow_at_speed.mm3_per_mm,
                    flow_at_speed.distance_mm,
                    smoothed_speed,
                ) {
                    self.previous_eligible = Some(smoothed_flow);
                }
                smoothed_speed
            }
        }
    }

    fn print_flow(&mut self, move_: &ExtrusionMove, speed_mm_s: f64) -> Option<PrintFlow> {
        let point = move_.point();
        let start = self.last_point.unwrap_or(point);
        let distance_mm = distance(start, point);
        let e_position = move_
            .e_position()
            .expect("print speed move must have E position");
        let delta_e = e_position - self.last_print_e;
        self.last_print_e = e_position;
        if distance_mm <= 0.0 || delta_e <= 0.0 {
            return None;
        }
        PrintFlow::at_speed(
            delta_e * self.filament_area_mm2 / distance_mm,
            distance_mm,
            speed_mm_s,
        )
    }

    fn is_eligible(&self, move_: &ExtrusionMove) -> bool {
        !self.external_only
            || matches!(
                move_.role(),
                crate::PrintPathRole::ExternalPerimeter | crate::PrintPathRole::OverhangPerimeter
            )
    }
}

struct VolumetricSpeedCap {
    filament_area_mm2: f64,
    max_mm3_s: f64,
    adaptive_enabled: bool,
    adaptive_coefficients: Option<[f64; 6]>,
    last_point: Option<Point2>,
    last_print_e: f64,
}

impl VolumetricSpeedCap {
    fn new(options: SpeedOptions) -> Self {
        Self {
            filament_area_mm2: std::f64::consts::PI
                * (options.filament_diameter_mm() / 2.0).powi(2),
            max_mm3_s: options.filament_max_volumetric_speed_mm3_s(),
            adaptive_enabled: options.filament_adaptive_volumetric_speed(),
            adaptive_coefficients: options.volumetric_speed_coefficients(),
            last_point: None,
            last_print_e: 0.0,
        }
    }

    fn capped_speed(&mut self, move_: &ExtrusionMove, configured_speed_mm_s: f64) -> f64 {
        match move_.kind() {
            ToolpathMoveKind::Travel => {
                self.last_point = Some(move_.point());
                configured_speed_mm_s
            }
            ToolpathMoveKind::Print => {
                let capped = self
                    .print_cap(move_)
                    .map_or(configured_speed_mm_s, |cap| configured_speed_mm_s.min(cap));
                self.last_point = Some(move_.point());
                capped
            }
        }
    }

    fn print_cap(&mut self, move_: &ExtrusionMove) -> Option<f64> {
        let point = move_.point();
        let e_position = move_
            .e_position()
            .expect("print speed move must have E position");
        if self.max_mm3_s <= 0.0 {
            self.last_print_e = e_position;
            return None;
        }
        let start = self.last_point.unwrap_or(point);
        let distance = distance(start, point);
        let delta_e = e_position - self.last_print_e;
        self.last_print_e = e_position;
        if distance <= 0.0 || delta_e <= 0.0 {
            return None;
        }
        let mm3_per_mm = delta_e * self.filament_area_mm2 / distance;
        if mm3_per_mm > 0.0 {
            Some(self.effective_max_mm3_s(move_) / mm3_per_mm)
        } else {
            None
        }
    }

    fn effective_max_mm3_s(&self, move_: &ExtrusionMove) -> f64 {
        let Some(fitted) = self.adaptive_fitted_max_mm3_s(move_) else {
            return self.max_mm3_s;
        };
        self.max_mm3_s.min(fitted)
    }

    fn adaptive_fitted_max_mm3_s(&self, move_: &ExtrusionMove) -> Option<f64> {
        if !self.adaptive_enabled {
            return None;
        }
        let coefficients = self.adaptive_coefficients?;
        let layer_height = move_.effective_layer_height_mm()?;
        let line_width = move_.effective_line_width_mm()?;
        let fitted = coefficients[0] * layer_height * layer_height
            + coefficients[1] * line_width * line_width
            + coefficients[2] * layer_height * line_width
            + coefficients[3] * layer_height
            + coefficients[4] * line_width
            + coefficients[5];
        (fitted.is_finite() && fitted > 0.0).then_some(fitted)
    }
}

fn distance(start: Point2, end: Point2) -> f64 {
    ((end.x() - start.x()).powi(2) + (end.y() - start.y()).powi(2)).sqrt()
}

#[cfg(test)]
mod tests;
