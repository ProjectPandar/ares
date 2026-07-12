use crate::{PrintPathRole, SliceError, SpeedOptions};

pub(crate) fn parse_speed_options(
    options: &crate::SliceOptions,
) -> Result<SpeedOptions, SliceError> {
    let values = options.values();
    let travel_speed = positive_number_or_string(
        values,
        "travel_speed",
        crate::options::defaults::DEFAULT_TRAVEL_SPEED,
    )?;
    let outer_wall_speed = positive_number_or_string(
        values,
        "outer_wall_speed",
        crate::options::defaults::DEFAULT_OUTER_WALL_SPEED,
    )?;
    let skirt_speed = options.range_f64("skirt_speed", 50.0, 0.0, f64::INFINITY)?;
    let bridge_options = options.bridge_options()?;
    let external_line_width_mm = options
        .extrusion_options()?
        .width_for_role(PrintPathRole::ExternalPerimeter);
    let initial_layer_travel_speed = match options.values().get("initial_layer_travel_speed") {
        Some(value) => crate::options::parsing::parse_positive_numeric_or_percent_over_base(
            "initial_layer_travel_speed",
            value,
            travel_speed,
        )?,
        None => travel_speed,
    };
    let travel_speed_z = options.range_f64("travel_speed_z", 0.0, 0.0, f64::INFINITY)?;

    Ok(SpeedOptions::new(
        travel_speed,
        outer_wall_speed,
        positive_number_or_string(
            values,
            "sparse_infill_speed",
            crate::options::defaults::DEFAULT_SPARSE_INFILL_SPEED,
        )?,
    )
    .with_internal_solid_infill_speed(positive_number_or_string(
        values,
        "internal_solid_infill_speed",
        100.0,
    )?)
    .with_support_speed(crate::options::parsing::parse_range_f64(
        "support_speed",
        values.get("support_speed"),
        80.0,
        1.0,
        f64::INFINITY,
    )?)
    .with_support_interface_speed(crate::options::parsing::parse_range_f64(
        "support_interface_speed",
        values.get("support_interface_speed"),
        80.0,
        1.0,
        f64::INFINITY,
    )?)
    .with_top_surface_speed(positive_number_or_string(
        values,
        "top_surface_speed",
        100.0,
    )?)
    .with_ironing_speed(effective_ironing_speed(values)?)
    .with_first_layer_speed(positive_number_or_string(
        values,
        "initial_layer_speed",
        30.0,
    )?)
    .with_first_layer_infill_speed(positive_number_or_string(
        values,
        "initial_layer_infill_speed",
        60.0,
    )?)
    .with_first_layer_travel_speed(initial_layer_travel_speed)
    .with_travel_speed_z(travel_speed_z)
    .with_internal_perimeter_speed(positive_number_or_string(
        values,
        "inner_wall_speed",
        outer_wall_speed,
    )?)
    .with_bridge_speed(bridge_options.bridge_speed_mm_s())
    .with_overhang_perimeter_speed(
        crate::options::overhang_speed::parse_overhang_perimeter_speed(
            values,
            outer_wall_speed,
            bridge_options.bridge_speed_mm_s(),
        )?,
    )
    .with_overhang_speed_bands(crate::options::overhang_speed::parse_overhang_speed_bands(
        values,
        external_line_width_mm,
        outer_wall_speed,
        bridge_options.bridge_speed_mm_s(),
    )?)
    .with_internal_bridge_speed(bridge_options.internal_bridge_speed_mm_s())
    .with_gap_infill_speed(non_negative_number_or_string(
        values,
        "gap_infill_speed",
        30.0,
    )?)
    .with_skirt_speed(if skirt_speed > 0.0 {
        skirt_speed
    } else {
        outer_wall_speed
    })
    .with_small_perimeter_threshold(
        crate::options::small_perimeter::parse_small_perimeter_threshold(options.values())?,
    )
    .with_small_perimeter_speed(
        crate::options::small_perimeter::parse_small_perimeter_speed(
            options.values(),
            outer_wall_speed,
        )?,
    )
    .with_filament_diameter(options.filament_diameters()?[0])
    .with_filament_max_volumetric_speed(
        crate::options::volumetric_speed::parse_filament_max_volumetric_speed(options.values())?,
    )
    .with_resonance_avoidance(
        bool_option(values, "resonance_avoidance", false)?,
        non_negative_number_or_string(values, "min_resonance_avoidance_speed", 70.0)?,
        non_negative_number_or_string(values, "max_resonance_avoidance_speed", 120.0)?,
    )
    .with_filament_adaptive_volumetric_speed(
        crate::options::volumetric_speed::parse_filament_adaptive_volumetric_speed(
            options.values(),
        )?,
    )
    .with_volumetric_speed_coefficients(
        crate::options::volumetric_speed::parse_volumetric_speed_coefficients(options.values()),
    )
    .with_max_volumetric_extrusion_rate_slope(
        crate::options::volumetric_speed::parse_max_volumetric_extrusion_rate_slope(
            options.values(),
        )?,
    )
    .with_max_volumetric_extrusion_rate_slope_segment_length(
        crate::options::volumetric_speed::parse_max_volumetric_extrusion_rate_slope_segment_length(
            options.values(),
        )?,
    )
    .with_extrusion_rate_smoothing_external_perimeter_only(
        crate::options::volumetric_speed::parse_extrusion_rate_smoothing_external_perimeter_only(
            options.values(),
        )?,
    )
    .with_slow_down_layers(crate::options::slow_down_layers::parse_slow_down_layers(
        options.values(),
    )?)
    .with_dont_slow_down_outer_wall(
        crate::options::slow_down_layers::parse_dont_slow_down_outer_wall(options.values())?,
    )
    .with_slow_down_for_layer_cooling(
        crate::options::slow_down_layers::parse_slow_down_for_layer_cooling(options.values())?,
    )
    .with_slow_down_layer_time(
        crate::options::slow_down_layers::parse_slow_down_layer_time(options.values())?,
    )
    .with_slow_down_min_speed(crate::options::slow_down_layers::parse_slow_down_min_speed(
        options.values(),
    )?)
    .with_acceleration_options(options.acceleration_options()?)
    .with_jerk_options(options.jerk_options()?))
}

fn positive_number_or_string(
    values: &std::collections::BTreeMap<String, serde_json::Value>,
    key: &str,
    default: f64,
) -> Result<f64, SliceError> {
    crate::options::parsing::parse_positive_number_or_string(key, values.get(key), default)
}

fn non_negative_number_or_string(
    values: &std::collections::BTreeMap<String, serde_json::Value>,
    key: &str,
    default: f64,
) -> Result<f64, SliceError> {
    crate::options::parsing::parse_range_f64(key, values.get(key), default, 0.0, f64::INFINITY)
}

fn effective_ironing_speed(
    values: &std::collections::BTreeMap<String, serde_json::Value>,
) -> Result<f64, SliceError> {
    let ironing_speed = crate::options::parsing::parse_range_f64(
        "ironing_speed",
        values.get("ironing_speed"),
        20.0,
        1.0,
        f64::INFINITY,
    )?;
    let Some(value) = values.get("filament_ironing_speed") else {
        return Ok(ironing_speed);
    };
    first_nullable_filament_ironing_speed(value)?.map_or(Ok(ironing_speed), Ok)
}

fn first_nullable_filament_ironing_speed(
    value: &serde_json::Value,
) -> Result<Option<f64>, SliceError> {
    match value {
        serde_json::Value::Array(values) => {
            let Some(first) = values.first() else {
                return Err(SliceError::InvalidInput(
                    "filament_ironing_speed must not be empty".to_owned(),
                ));
            };
            nullable_filament_ironing_speed_value(first)
        }
        value => nullable_filament_ironing_speed_value(value),
    }
}

fn nullable_filament_ironing_speed_value(
    value: &serde_json::Value,
) -> Result<Option<f64>, SliceError> {
    if value.as_str() == Some("nil") {
        return Ok(None);
    }
    let speed = match value {
        serde_json::Value::Number(number) => number.as_f64(),
        serde_json::Value::String(text) => text.parse().ok(),
        _ => None,
    }
    .ok_or_else(|| {
        SliceError::InvalidInput("filament_ironing_speed must be a number or nil".to_owned())
    })?;
    if speed.is_finite() && speed >= 1.0 {
        Ok(Some(speed))
    } else {
        Err(SliceError::InvalidInput(
            "filament_ironing_speed is out of range".to_owned(),
        ))
    }
}

fn bool_option(
    values: &std::collections::BTreeMap<String, serde_json::Value>,
    key: &str,
    default: bool,
) -> Result<bool, SliceError> {
    let Some(value) = values.get(key) else {
        return Ok(default);
    };
    value
        .as_bool()
        .ok_or_else(|| SliceError::InvalidInput(format!("{key} must be a boolean")))
}
