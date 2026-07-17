use std::collections::BTreeMap;

use serde_json::Value;

use crate::{
    ExtrusionOptions, SliceError,
    extrusions::{ExtrusionWidthSpec, RoleExtrusionHardware, RoleHardwareValues},
};

pub fn parse_sparse_infill_flow_ratio(values: &BTreeMap<String, Value>) -> Result<f64, SliceError> {
    parse_flow_ratio(values, "sparse_infill_flow_ratio")
}

pub fn parse_internal_solid_infill_flow_ratio(
    values: &BTreeMap<String, Value>,
) -> Result<f64, SliceError> {
    parse_flow_ratio(values, "internal_solid_infill_flow_ratio")
}

pub fn parse_support_flow_ratio(values: &BTreeMap<String, Value>) -> Result<f64, SliceError> {
    parse_flow_ratio(values, "support_flow_ratio")
}

pub fn parse_support_interface_flow_ratio(
    values: &BTreeMap<String, Value>,
) -> Result<f64, SliceError> {
    parse_flow_ratio(values, "support_interface_flow_ratio")
}

pub fn parse_top_solid_infill_flow_ratio(
    values: &BTreeMap<String, Value>,
) -> Result<f64, SliceError> {
    parse_flow_ratio(values, "top_solid_infill_flow_ratio")
}

pub fn parse_bottom_solid_infill_flow_ratio(
    values: &BTreeMap<String, Value>,
) -> Result<f64, SliceError> {
    parse_flow_ratio(values, "bottom_solid_infill_flow_ratio")
}

pub fn parse_first_layer_flow_ratio(values: &BTreeMap<String, Value>) -> Result<f64, SliceError> {
    parse_flow_ratio(values, "first_layer_flow_ratio")
}

pub fn parse_gap_fill_flow_ratio(values: &BTreeMap<String, Value>) -> Result<f64, SliceError> {
    parse_flow_ratio(values, "gap_fill_flow_ratio")
}

pub fn parse_overhang_flow_ratio(values: &BTreeMap<String, Value>) -> Result<f64, SliceError> {
    parse_flow_ratio(values, "overhang_flow_ratio")
}

pub fn parse_print_flow_ratio(values: &BTreeMap<String, Value>) -> Result<f64, SliceError> {
    parse_flow_ratio_in_range(values, "print_flow_ratio", 0.01..=2.0)
}

pub fn parse_filament_flow_ratio(values: &BTreeMap<String, Value>) -> Result<f64, SliceError> {
    let Some(value) = values.get("filament_flow_ratio") else {
        return Ok(1.0);
    };
    let ratios = crate::options::parsing::parse_numeric_vector("filament_flow_ratio", value)?;
    if ratios.iter().all(|ratio| ratio.is_finite() && *ratio > 0.0) {
        Ok(ratios[0])
    } else {
        Err(SliceError::InvalidInput(
            "filament_flow_ratio contains invalid value".to_owned(),
        ))
    }
}

pub fn parse_extrusion_options(
    options: &crate::SliceOptions,
) -> Result<ExtrusionOptions, SliceError> {
    let nozzle_diameters = options.nozzle_diameters()?;
    let filament_diameters = options.filament_diameters()?;
    let nozzle_diameter = nozzle_diameters[0];
    let filament_diameter = filament_diameters[0];
    let wall_filament = parse_role_filament_selector(options.values(), "wall_filament")?;
    let sparse_infill_filament =
        parse_role_filament_selector(options.values(), "sparse_infill_filament")?;
    let solid_infill_filament =
        parse_role_filament_selector(options.values(), "solid_infill_filament")?;
    let support_filament = parse_support_filament_selector(options.values(), "support_filament")?;
    let support_interface_filament =
        parse_support_filament_selector(options.values(), "support_interface_filament")?;
    let support_interface_not_for_body = options.support_interface_not_for_body_options()?.not_for_body();
    let selector_count = nozzle_diameters.len().max(filament_diameters.len());
    let support_selector = support_filament.selector();
    let support_extrusion_selector = support_filament.support_selector(
        support_interface_filament,
        support_interface_not_for_body,
        selector_count,
    );
    let support_interface_selector = support_interface_filament.selector();
    let default_hardware = RoleHardwareValues::new(nozzle_diameter, filament_diameter);
    let bridge_options = options.bridge_options()?;
    let support_line_width = parse_extrusion_width_spec(
        options.values(),
        "support_line_width",
        ExtrusionWidthSpec::auto(),
    )?;
    let parsed = apply_supported_other_flow_ratios(
        ExtrusionOptions::new_for_tests(
            nozzle_diameter,
            filament_diameter,
            0.0,
            (0.0, 0.0),
            0.0,
        )
        .with_role_hardware(
            RoleExtrusionHardware::from_default(default_hardware)
                .with_wall(role_hardware(
                    &nozzle_diameters,
                    &filament_diameters,
                    wall_filament,
                ))
                .with_sparse_infill(role_hardware(
                    &nozzle_diameters,
                    &filament_diameters,
                    sparse_infill_filament,
                ))
                .with_solid_infill(role_hardware(
                    &nozzle_diameters,
                    &filament_diameters,
                    solid_infill_filament,
                ))
                .with_support(role_hardware(
                    &nozzle_diameters,
                    &filament_diameters,
                    support_selector,
                ))
                .with_support_interface(role_hardware(
                    &nozzle_diameters,
                    &filament_diameters,
                    support_interface_selector,
                )),
        )
        .with_line_width_spec(parse_extrusion_width_spec(
            options.values(),
            "line_width",
            ExtrusionWidthSpec::auto(),
        )?)
        .with_outer_wall_line_width_spec(parse_extrusion_width_spec(
            options.values(),
            "outer_wall_line_width",
            ExtrusionWidthSpec::absolute(crate::options::defaults::DEFAULT_OUTER_WALL_LINE_WIDTH),
        )?)
        .with_inner_wall_line_width_spec(parse_extrusion_width_spec(
            options.values(),
            "inner_wall_line_width",
            ExtrusionWidthSpec::auto(),
        )?)
        .with_sparse_infill_line_width_spec(parse_extrusion_width_spec(
            options.values(),
            "sparse_infill_line_width",
            ExtrusionWidthSpec::auto(),
        )?)
        .with_internal_solid_infill_line_width_spec(parse_extrusion_width_spec(
            options.values(),
            "internal_solid_infill_line_width",
            ExtrusionWidthSpec::auto(),
        )?)
        .with_top_surface_line_width_spec(parse_extrusion_width_spec(
            options.values(),
            "top_surface_line_width",
            ExtrusionWidthSpec::auto(),
        )?)
        .with_initial_layer_line_width(options.extrusion_width(
            "initial_layer_line_width",
            0.0,
            nozzle_diameter,
        )?)
        .with_bridge_flow(bridge_options.bridge_flow())
        .with_thick_bridges(bridge_options.thick_bridges())
        .with_thick_internal_bridges(bridge_options.thick_internal_bridges())
        .with_internal_bridge_flow(bridge_options.internal_bridge_flow())
        .with_brim_flow_ratio(options.range_f64("brim_flow_ratio", 1.0, 0.0, 2.0)?)
        .with_print_flow_ratio(parse_print_flow_ratio(options.values())?)
        .with_filament_flow_ratio(parse_filament_flow_ratio(options.values())?)
        .with_small_area_infill_flow_compensation(
            crate::options::small_area_infill_flow::parse(options)?,
        ),
        options.values(),
    )?;
    let parsed = if support_extrusion_selector == support_selector {
        parsed
    } else {
        parsed.with_support_material_extrusion_hardware(role_hardware(
            &nozzle_diameters,
            &filament_diameters,
            support_extrusion_selector,
        ))
    };
    Ok(parsed
        .with_support_line_width_spec(support_line_width)
        .with_ironing_flow_ratio(super::ironing_flow::parse(options.values())?)
        .with_top_solid_infill_flow_ratio(parse_top_solid_infill_flow_ratio(options.values())?)
        .with_bottom_solid_infill_flow_ratio(parse_bottom_solid_infill_flow_ratio(
            options.values(),
        )?))
}

pub fn apply_supported_other_flow_ratios(
    options: ExtrusionOptions,
    values: &BTreeMap<String, Value>,
) -> Result<ExtrusionOptions, SliceError> {
    if !parse_set_other_flow_ratios(values)? {
        parse_outer_wall_flow_ratio(values)?;
        parse_inner_wall_flow_ratio(values)?;
        parse_sparse_infill_flow_ratio(values)?;
        parse_internal_solid_infill_flow_ratio(values)?;
        parse_support_flow_ratio(values)?;
        parse_support_interface_flow_ratio(values)?;
        parse_gap_fill_flow_ratio(values)?;
        parse_first_layer_flow_ratio(values)?;
        parse_overhang_flow_ratio(values)?;
        return Ok(options);
    }
    Ok(options
        .with_outer_wall_flow_ratio(parse_outer_wall_flow_ratio(values)?)
        .with_inner_wall_flow_ratio(parse_inner_wall_flow_ratio(values)?)
        .with_overhang_flow_ratio(parse_overhang_flow_ratio(values)?)
        .with_sparse_infill_flow_ratio(parse_sparse_infill_flow_ratio(values)?)
        .with_internal_solid_infill_flow_ratio(parse_internal_solid_infill_flow_ratio(values)?)
        .with_support_flow_ratio(parse_support_flow_ratio(values)?)
        .with_support_interface_flow_ratio(parse_support_interface_flow_ratio(values)?)
        .with_gap_fill_flow_ratio(parse_gap_fill_flow_ratio(values)?)
        .with_first_layer_flow_ratio(parse_first_layer_flow_ratio(values)?))
}

fn parse_extrusion_width_spec(
    values: &BTreeMap<String, Value>,
    key: &str,
    default: ExtrusionWidthSpec,
) -> Result<ExtrusionWidthSpec, SliceError> {
    let Some(value) = values.get(key) else {
        return Ok(default);
    };
    let spec = match value {
        Value::Number(number) => number.as_f64().map(ExtrusionWidthSpec::absolute),
        Value::String(text) => parse_extrusion_width_spec_text(text),
        _ => None,
    }
    .ok_or_else(|| SliceError::InvalidInput(format!("{key} must be a number")))?;
    match spec {
        ExtrusionWidthSpec::Absolute(value) if value.is_finite() && value >= 0.0 => Ok(spec),
        ExtrusionWidthSpec::Percent(value) if value.is_finite() && value >= 0.0 => Ok(spec),
        _ => Err(SliceError::InvalidInput(format!(
            "{key} contains invalid value"
        ))),
    }
}

fn parse_extrusion_width_spec_text(text: &str) -> Option<ExtrusionWidthSpec> {
    let text = text.trim();
    if let Some(percent) = text.strip_suffix('%') {
        percent
            .trim()
            .parse::<f64>()
            .ok()
            .map(ExtrusionWidthSpec::percent)
    } else {
        text.parse::<f64>().ok().map(ExtrusionWidthSpec::absolute)
    }
}

fn parse_role_filament_selector(
    values: &BTreeMap<String, Value>,
    key: &str,
) -> Result<u64, SliceError> {
    let Some(value) = values.get(key) else {
        return Ok(0);
    };
    let parsed = match value {
        Value::Number(number) => parse_role_filament_number(number),
        Value::String(text) => text.trim().parse::<u64>().ok(),
        _ => None,
    }
    .ok_or_else(|| SliceError::InvalidInput(format!("{key} must be a positive integer")))?;
    if parsed == 0 {
        Err(SliceError::InvalidInput(format!("{key} must be positive")))
    } else {
        Ok(parsed - 1)
    }
}

fn parse_support_filament_selector(
    values: &BTreeMap<String, Value>,
    key: &str,
) -> Result<SupportFilamentSelector, SliceError> {
    let Some(value) = values.get(key) else {
        return Ok(SupportFilamentSelector::Auto);
    };
    let parsed = match value {
        Value::Number(number) => parse_role_filament_number(number),
        Value::String(text) => text.trim().parse::<u64>().ok(),
        _ => None,
    }
    .ok_or_else(|| SliceError::InvalidInput(format!("{key} must be a non-negative integer")))?;
    if parsed == 0 {
        Ok(SupportFilamentSelector::Auto)
    } else {
        Ok(SupportFilamentSelector::Fixed(parsed - 1))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SupportFilamentSelector {
    Auto,
    Fixed(u64),
}

impl SupportFilamentSelector {
    fn selector(self) -> u64 {
        match self {
            Self::Auto => 0,
            Self::Fixed(selector) => selector,
        }
    }

    fn support_selector(self, interface: Self, not_for_body: bool, selector_count: usize) -> u64 {
        match self {
            Self::Fixed(selector) => selector,
            Self::Auto if not_for_body && selector_count > 1 && interface == Self::Fixed(0) => 1,
            Self::Auto => 0,
        }
    }
}

fn parse_role_filament_number(number: &serde_json::Number) -> Option<u64> {
    let value = number.as_f64()?;
    if value.is_finite() && value.fract() == 0.0 && value >= 0.0 && value <= u64::MAX as f64 {
        Some(value as u64)
    } else {
        None
    }
}

fn hardware_value(values: &[f64], index: u64) -> f64 {
    usize::try_from(index)
        .ok()
        .and_then(|index| values.get(index))
        .copied()
        .unwrap_or(values[0])
}

fn role_hardware(
    nozzle_diameters: &[f64],
    filament_diameters: &[f64],
    selector: u64,
) -> RoleHardwareValues {
    RoleHardwareValues::new(
        hardware_value(nozzle_diameters, selector),
        hardware_value(filament_diameters, selector),
    )
}

fn parse_set_other_flow_ratios(values: &BTreeMap<String, Value>) -> Result<bool, SliceError> {
    let Some(value) = values.get("set_other_flow_ratios") else {
        return Ok(false);
    };
    value
        .as_bool()
        .ok_or_else(|| SliceError::InvalidInput("set_other_flow_ratios must be a boolean".into()))
}

fn parse_outer_wall_flow_ratio(values: &BTreeMap<String, Value>) -> Result<f64, SliceError> {
    parse_flow_ratio(values, "outer_wall_flow_ratio")
}

fn parse_inner_wall_flow_ratio(values: &BTreeMap<String, Value>) -> Result<f64, SliceError> {
    parse_flow_ratio(values, "inner_wall_flow_ratio")
}

fn parse_flow_ratio(values: &BTreeMap<String, Value>, key: &str) -> Result<f64, SliceError> {
    parse_flow_ratio_in_range(values, key, 0.0..=2.0)
}

fn parse_flow_ratio_in_range(
    values: &BTreeMap<String, Value>,
    key: &str,
    range: std::ops::RangeInclusive<f64>,
) -> Result<f64, SliceError> {
    let Some(value) = values.get(key) else {
        return Ok(1.0);
    };
    let value = match value {
        Value::Number(number) => number.as_f64(),
        Value::String(text) => text.parse().ok(),
        _ => None,
    }
    .ok_or_else(|| SliceError::InvalidInput(format!("{key} must be a number")))?;
    if value.is_finite() && range.contains(&value) {
        Ok(value)
    } else {
        Err(SliceError::InvalidInput(format!("{key} is out of range")))
    }
}
