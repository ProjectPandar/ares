//! Print.cpp:1524-1556,1627-1671: nozzle-dependent height/width bounds.
use crate::project::effective_config::{types::BoundedResolvedProjectConfig, usage};
use crate::{FloatOrPercent, ObjectOptions, OrcaInts, ProjectSettings, RegionOptions, SliceError};

macro_rules! region_widths {
    ($region:expr) => {
        [
            $region.inner_wall_line_width,
            $region.outer_wall_line_width,
            $region.sparse_infill_line_width,
            $region.internal_solid_infill_line_width,
            $region.top_surface_line_width,
            $region.skin_infill_line_width,
            $region.skeleton_infill_line_width,
        ]
    };
}

pub(super) fn applied(resolved: &BoundedResolvedProjectConfig) -> Result<(), SliceError> {
    let settings = &resolved.views.full;
    let nozzles = nozzle_bounds(
        settings,
        &resolved.applied_filament_map,
        &resolved.usage.supported_used_filaments,
    )?;
    for object in resolved
        .objects
        .iter()
        .filter(|object| !object.print_objects.is_empty())
    {
        layer_heights(settings, &object.object, nozzles.0)?;
        object_widths(&object.object, nozzles)?;
        // Upstream iterates keys before regions, not regions before keys.
        for index in 0..7 {
            for region in &object.all_regions {
                width(
                    region_widths!(region)[index],
                    object.object.layer_height.0,
                    nozzles,
                )?;
            }
        }
    }
    Ok(())
}

pub(super) fn flat(settings: &ProjectSettings) -> Result<(), SliceError> {
    let object = ObjectOptions::from_base(&settings.process.object);
    let region = RegionOptions::from_base(&settings.process.region);
    let used = usage::collect_usage(
        &[usage::ObjectUsageInput {
            object: &object,
            regions: std::slice::from_ref(&region),
            object_extruder: None,
            model_part_extruders: vec![None],
            layer_range_extruders: Vec::new(),
        }],
        settings,
        settings.filament.gcode.filament_diameter.0.len(),
    );
    let nozzles = nozzle_bounds(
        settings,
        &settings.project.gcode.filament_map,
        &used.supported_used_filaments,
    )?;
    layer_heights(settings, &object, nozzles.0)?;
    object_widths(&object, nozzles)?;
    for value in region_widths!(region) {
        width(value, object.layer_height.0, nozzles)?;
    }
    Ok(())
}

fn nozzle_bounds(
    settings: &ProjectSettings,
    filament_map: &OrcaInts,
    used: &[usize],
) -> Result<(f64, f64), SliceError> {
    let mut minimum = f64::MAX;
    let mut maximum = 0.0_f64;
    for &filament in used {
        let extruder = filament_map
            .0
            .get(filament)
            .ok_or_else(|| SliceError::InvalidInput("invalid filament_map".to_owned()))?
            .0;
        let diameter = usize::try_from(extruder)
            .ok()
            .and_then(|index| index.checked_sub(1))
            .and_then(|index| settings.project.print.nozzle_diameter.0.get(index))
            .ok_or_else(|| SliceError::InvalidInput("invalid nozzle_diameter".to_owned()))?;
        minimum = minimum.min(diameter.0);
        maximum = maximum.max(diameter.0);
    }
    Ok((minimum, maximum))
}

fn layer_heights(
    settings: &ProjectSettings,
    object: &ObjectOptions,
    minimum_nozzle: f64,
) -> Result<(), SliceError> {
    let first_layer_nozzle = if object.raft_layers.0 > 0 {
        let selector = if object.raft_layers.0 == 1 {
            object.support_interface_filament.0
        } else {
            object.support_filament.0
        };
        if selector == 0 {
            minimum_nozzle
        } else {
            // Print.cpp uses the filament index directly, without filament_map.
            // Config.hpp:624-628 get_at falls back to the first value, not the last.
            let diameters = &settings.project.print.nozzle_diameter.0;
            usize::try_from(i64::from(selector) - 1)
                .ok()
                .and_then(|index| diameters.get(index))
                .unwrap_or(&diameters[0])
                .0
        }
    } else {
        minimum_nozzle
    };
    if settings.process.print.initial_layer_print_height.0 > first_layer_nozzle {
        return Err(SliceError::InvalidInput(
            "Layer height cannot exceed nozzle diameter.".to_owned(),
        ));
    }
    if object.layer_height.0 > minimum_nozzle {
        return Err(SliceError::InvalidInput(
            "Layer height cannot exceed nozzle diameter.".to_owned(),
        ));
    }
    Ok(())
}

fn has_support(object: &ObjectOptions) -> bool {
    object.enable_support.0 || object.enforce_support_layers.0 > 0 || object.raft_layers.0 > 0
}

fn object_widths(object: &ObjectOptions, nozzles: (f64, f64)) -> Result<(), SliceError> {
    width(object.line_width, object.layer_height.0, nozzles)?;
    if has_support(object) {
        width(object.support_line_width, object.layer_height.0, nozzles)?;
    }
    Ok(())
}

fn width(
    value: FloatOrPercent,
    layer_height: f64,
    (minimum_nozzle, maximum_nozzle): (f64, f64),
) -> Result<(), SliceError> {
    let minimum_width = match value {
        FloatOrPercent::Float(value) => value,
        FloatOrPercent::Percent(value) => minimum_nozzle * value.0 / 100.0,
    };
    if minimum_width == 0.0 {
        return Ok(());
    }
    if minimum_width <= layer_height {
        return Err(SliceError::InvalidInput("Too small line width".to_owned()));
    }
    let maximum_width = match value {
        FloatOrPercent::Float(value) => value,
        FloatOrPercent::Percent(value) => maximum_nozzle * value.0 / 100.0,
    };
    if maximum_width > maximum_nozzle * 5.0 {
        return Err(SliceError::InvalidInput("Too large line width".to_owned()));
    }
    Ok(())
}
