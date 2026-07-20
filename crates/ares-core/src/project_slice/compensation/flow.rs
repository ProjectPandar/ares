use crate::{FloatOrPercent, OrcaFloats, OrcaInt, SliceError};

use super::super::layers::PlannedLayer;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::project_slice) struct ExternalPerimeterFlow {
    pub(in crate::project_slice) nozzle_diameter: f32,
    pub(in crate::project_slice) height: f32,
    pub(in crate::project_slice) width: f32,
    pub(in crate::project_slice) spacing: f32,
    pub(in crate::project_slice) minimum_width: f32,
}

#[allow(clippy::too_many_arguments)]
pub(in crate::project_slice) fn resolve_external_perimeter_flow(
    layer: &PlannedLayer,
    initial_layer_width: FloatOrPercent,
    outer_wall_width: FloatOrPercent,
    object_line_width: FloatOrPercent,
    outer_wall_filament_id: OrcaInt,
    nozzle_diameters: &OrcaFloats,
) -> Result<ExternalPerimeterFlow, SliceError> {
    let nozzle_index = outer_wall_filament_id
        .0
        .checked_sub(1)
        .and_then(|index| usize::try_from(index).ok())
        .filter(|index| *index < nozzle_diameters.0.len())
        .unwrap_or(0);
    let nozzle_diameter = nozzle_diameters
        .0
        .get(nozzle_index)
        .map(|diameter| diameter.0 as f32)
        .filter(|diameter| diameter.is_finite() && *diameter > 0.0)
        .ok_or_else(|| invalid("invalid Orca option nozzle_diameter"))?;
    let height = layer.height as f32;
    if !height.is_finite() || height <= 0.0 {
        return Err(invalid("invalid Orca option layer_height"));
    }

    let mut selected_width = if layer.id == 0 && raw(initial_layer_width) > 0.0 {
        initial_layer_width
    } else {
        outer_wall_width
    };
    if raw(selected_width) == 0.0 {
        selected_width = object_line_width;
    }
    let width = match selected_width {
        FloatOrPercent::Float(value) if value <= 0.0 => 1.125_f32 * nozzle_diameter,
        value => absolute(value, nozzle_diameter),
    };
    let spacing = width - height * ((1.0_f64 - 0.25_f64 * std::f64::consts::PI) as f32);
    if !spacing.is_finite() || spacing <= 0.0 {
        return Err(invalid("invalid external perimeter flow spacing"));
    }

    Ok(ExternalPerimeterFlow {
        nozzle_diameter,
        height,
        width,
        spacing,
        minimum_width: width + spacing,
    })
}

fn absolute(value: FloatOrPercent, nozzle_diameter: f32) -> f32 {
    match value {
        FloatOrPercent::Float(value) => value as f32,
        FloatOrPercent::Percent(percent) => (f64::from(nozzle_diameter) * percent.0 / 100.0) as f32,
    }
}

fn raw(value: FloatOrPercent) -> f64 {
    match value {
        FloatOrPercent::Float(value) => value,
        FloatOrPercent::Percent(percent) => percent.0,
    }
}

fn invalid(message: &str) -> SliceError {
    SliceError::InvalidInput(message.to_owned())
}

type ResolveExternalPerimeterFlow = fn(
    &PlannedLayer,
    FloatOrPercent,
    FloatOrPercent,
    FloatOrPercent,
    OrcaInt,
    &OrcaFloats,
) -> Result<ExternalPerimeterFlow, SliceError>;
const _: ResolveExternalPerimeterFlow = resolve_external_perimeter_flow;
