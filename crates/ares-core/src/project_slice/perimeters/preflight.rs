use crate::{
    FloatOrPercent, ObjectOptions, OrcaFloats, RegionOptions, SliceError,
    project::effective_config::types::ResolvedProjectObject,
};

use super::{
    super::compensation::PostCompensationPrintObject, flow::resolve_perimeter_flows,
    types::PreparedObjectFlows,
};

pub(in crate::project_slice) fn preflight_perimeter_flows(
    objects: &[PostCompensationPrintObject],
    resolved_objects: &[ResolvedProjectObject],
    initial_layer_width: FloatOrPercent,
    nozzle_diameters: &OrcaFloats,
) -> Result<Vec<PreparedObjectFlows>, SliceError> {
    let contexts = resolved_objects
        .iter()
        .flat_map(|resolved| {
            resolved
                .print_objects
                .iter()
                .enumerate()
                .map(move |(transform_index, _)| (resolved, transform_index))
        })
        .collect::<Vec<_>>();
    assert_eq!(objects.len(), contexts.len());

    for (object, &(resolved, transform_index)) in objects.iter().zip(&contexts) {
        let (post_region, _) = object.as_parts();
        let (plan, _, regions) = post_region.as_parts();
        assert_eq!(plan.source_object_index, resolved.source_object_index);
        assert_eq!(plan.transform_index, transform_index);
        let [region] = regions else {
            return Err(unsupported("multi_region_layer_slices"));
        };
        let (_, region_options, layers) = region.as_parts();
        assert_eq!(plan.layers.len(), layers.len());
        validate_options(
            initial_layer_width,
            region_options,
            &resolved.object,
            nozzle_diameters,
        )?;
        for layer in &plan.layers {
            validate_layer_height(layer.height)?;
        }
    }

    objects
        .iter()
        .zip(contexts)
        .map(|(object, (resolved, _))| {
            prepare_object_flows(
                object,
                &resolved.object,
                initial_layer_width,
                nozzle_diameters,
            )
        })
        .collect()
}

fn prepare_object_flows(
    object: &PostCompensationPrintObject,
    object_options: &ObjectOptions,
    initial_layer_width: FloatOrPercent,
    nozzle_diameters: &OrcaFloats,
) -> Result<PreparedObjectFlows, SliceError> {
    let (post_region, _) = object.as_parts();
    let (plan, _, regions) = post_region.as_parts();
    let [region] = regions else {
        unreachable!("perimeter preflight already validated the region boundary")
    };
    let (_, region_options, layers) = region.as_parts();
    let layers = plan
        .layers
        .iter()
        .zip(layers)
        .map(|(planned, region_layer)| {
            if region_layer.surfaces().is_empty() {
                Ok(None)
            } else {
                resolve_perimeter_flows(
                    planned,
                    initial_layer_width,
                    region_options,
                    object_options,
                    nozzle_diameters,
                )
                .map(Some)
            }
        })
        .collect::<Result<_, _>>()?;
    Ok(PreparedObjectFlows { layers })
}

fn validate_options(
    initial_layer_width: FloatOrPercent,
    region: &RegionOptions,
    object: &ObjectOptions,
    nozzle_diameters: &OrcaFloats,
) -> Result<(), SliceError> {
    validate_width(initial_layer_width, "initial_layer_line_width")?;
    validate_width(region.outer_wall_line_width, "outer_wall_line_width")?;
    validate_width(region.inner_wall_line_width, "inner_wall_line_width")?;
    validate_width(
        region.internal_solid_infill_line_width,
        "internal_solid_infill_line_width",
    )?;
    validate_width(object.line_width, "line_width")?;
    validate_width(region.bridge_line_width, "bridge_line_width")?;
    if nozzle_diameters.0.is_empty()
        || nozzle_diameters
            .0
            .iter()
            .any(|nozzle| !nozzle.0.is_finite() || nozzle.0 <= 0.0)
    {
        return Err(invalid("invalid Orca option nozzle_diameter"));
    }
    if !region.bridge_flow.0.is_finite() || region.bridge_flow.0 <= 0.0 {
        return Err(invalid("invalid Orca option bridge_flow"));
    }
    Ok(())
}

fn validate_width(value: FloatOrPercent, key: &str) -> Result<(), SliceError> {
    let value = match value {
        FloatOrPercent::Float(value) => value,
        FloatOrPercent::Percent(percent) => percent.0,
    };
    if !value.is_finite() {
        return Err(SliceError::InvalidInput(format!(
            "invalid Orca option {key}"
        )));
    }
    Ok(())
}

fn validate_layer_height(height: f64) -> Result<(), SliceError> {
    let height = height as f32;
    if !height.is_finite() || height <= 0.0 {
        return Err(invalid("invalid Orca option layer_height"));
    }
    Ok(())
}

fn invalid(message: &str) -> SliceError {
    SliceError::InvalidInput(message.to_owned())
}

fn unsupported(key: &str) -> SliceError {
    SliceError::UnsupportedProjectFeature(key.to_owned())
}
