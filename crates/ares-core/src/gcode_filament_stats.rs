use crate::{
    HardwareOptions, LayerExtrusionMoves, LayerSpeedMoves, Point2, SliceError, SliceOptions,
    SpeedMove, ToolpathMoveKind,
};

const FILAMENT_COST_KEY: &str = "filament_cost";
const FILAMENT_DENSITY_KEY: &str = "filament_density";
const MACHINE_MIN_EXTRUDING_RATE_KEY: &str = "machine_min_extruding_rate";
const MACHINE_MIN_TRAVEL_RATE_KEY: &str = "machine_min_travel_rate";
const TIME_COST_KEY: &str = "time_cost";
const DEFAULT_FILAMENT_COST: &[f64] = &[0.0];
const DEFAULT_FILAMENT_DENSITY: &[f64] = &[0.0];
const DEFAULT_MACHINE_MIN_RATE: &[f64] = &[0.0, 0.0];
const DEFAULT_TIME_COST: f64 = 0.0;

pub(crate) fn format_filament_stats(
    layer_extrusion_moves: &[LayerExtrusionMoves],
    layer_speed_moves: &[LayerSpeedMoves],
    hardware_options: &HardwareOptions,
    options: &SliceOptions,
) -> Result<String, SliceError> {
    let used_filament_mm = used_filament_mm(layer_extrusion_moves);
    let filament_diameter = hardware_options.filament_diameters()[0];
    let extruded_volume_mm3 =
        used_filament_mm * std::f64::consts::PI * (filament_diameter * 0.5).powi(2);
    let filament_density =
        non_negative_vector(FILAMENT_DENSITY_KEY, options, DEFAULT_FILAMENT_DENSITY)?[0];
    let filament_cost_per_kg =
        non_negative_vector(FILAMENT_COST_KEY, options, DEFAULT_FILAMENT_COST)?[0];
    let filament_weight_g = extruded_volume_mm3 * filament_density * 0.001;
    let filament_cost = filament_weight_g * filament_cost_per_kg * 0.001;
    let time_cost_per_hour = non_negative_first_value(TIME_COST_KEY, options, DEFAULT_TIME_COST)?;
    let print_time_s = normal_print_time_s(options, layer_speed_moves)?;
    let total_cost = filament_cost + time_cost_per_hour * (print_time_s / 3600.0);

    let mut stats = format!(
        "; filament used [mm] = {:.2}\n; filament used [cm3] = {:.2}\n",
        used_filament_mm,
        extruded_volume_mm3 * 0.001
    );
    if filament_weight_g > 0.0 {
        stats.push_str(&format!("; filament used [g] = {filament_weight_g:.2}\n"));
        if filament_cost > 0.0 {
            stats.push_str(&format!("; filament cost = {filament_cost:.2}\n"));
        }
    }
    if filament_weight_g > 0.0 {
        stats.push_str(&format!(
            "; total filament used [g] = {filament_weight_g:.2}\n"
        ));
    }
    if total_cost > 0.0 {
        stats.push_str(&format!("; total filament cost = {total_cost:.2}\n"));
    }
    Ok(stats)
}

pub(crate) fn used_filament_mm(layer_extrusion_moves: &[LayerExtrusionMoves]) -> f64 {
    layer_extrusion_moves
        .iter()
        .map(|layer| layer.total_extrusion_mm())
        .sum()
}

pub(crate) fn normal_print_time_s(
    options: &SliceOptions,
    layer_speed_moves: &[LayerSpeedMoves],
) -> Result<f64, SliceError> {
    let minimum_rates = MachineMinimumRates::from_options(options)?;
    Ok(layer_speed_moves
        .iter()
        .map(|layer| layer_print_time_s(layer.moves(), minimum_rates))
        .sum())
}

fn layer_print_time_s(moves: &[SpeedMove], minimum_rates: MachineMinimumRates) -> f64 {
    let mut last_point = None;
    let mut had_print = false;
    let mut total = 0.0;
    for move_ in moves {
        let start = last_point.unwrap_or(move_.point());
        if move_.kind() == ToolpathMoveKind::Print {
            had_print = true;
        }
        if had_print {
            let length = distance(start, move_.point());
            let speed_mm_s = minimum_rates.clamp_speed(move_.kind(), move_.speed_mm_s());
            if length > 0.0 && speed_mm_s > 0.0 {
                total += length / speed_mm_s;
            }
        }
        last_point = Some(move_.point());
    }
    total
}

fn distance(start: Point2, end: Point2) -> f64 {
    ((end.x() - start.x()).powi(2) + (end.y() - start.y()).powi(2)).sqrt()
}

#[derive(Clone, Copy)]
struct MachineMinimumRates {
    extruding_mm_s: f64,
    travel_mm_s: f64,
}

impl MachineMinimumRates {
    fn from_options(options: &SliceOptions) -> Result<Self, SliceError> {
        Ok(Self {
            extruding_mm_s: non_negative_vector(
                MACHINE_MIN_EXTRUDING_RATE_KEY,
                options,
                DEFAULT_MACHINE_MIN_RATE,
            )?[0],
            travel_mm_s: non_negative_vector(
                MACHINE_MIN_TRAVEL_RATE_KEY,
                options,
                DEFAULT_MACHINE_MIN_RATE,
            )?[0],
        })
    }

    fn clamp_speed(self, kind: ToolpathMoveKind, speed_mm_s: f64) -> f64 {
        match kind {
            ToolpathMoveKind::Print => speed_mm_s.max(self.extruding_mm_s),
            ToolpathMoveKind::Travel => speed_mm_s.max(self.travel_mm_s),
        }
    }
}

fn non_negative_first_value(
    key: &str,
    options: &SliceOptions,
    default: f64,
) -> Result<f64, SliceError> {
    if !options.values().contains_key(key) {
        return Ok(default);
    }
    let values = non_negative_vector(key, options, &[default])?;
    Ok(values[0])
}

fn non_negative_vector(
    key: &str,
    options: &SliceOptions,
    default: &[f64],
) -> Result<Vec<f64>, SliceError> {
    let Some(value) = options.values().get(key) else {
        return Ok(default.to_vec());
    };
    let values = crate::options::parsing::parse_numeric_vector(key, value)?;
    if values.is_empty() {
        return Err(SliceError::InvalidInput(format!("{key} must not be empty")));
    }
    if values
        .iter()
        .all(|value| value.is_finite() && *value >= 0.0)
    {
        Ok(values)
    } else {
        Err(SliceError::InvalidInput(format!(
            "{key} contains invalid value"
        )))
    }
}
