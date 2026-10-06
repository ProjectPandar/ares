//! Print.cpp:1524-1556,1654-1671: lower widths on applied objects/regions.
use crate::project::effective_config::{types::BoundedResolvedProjectConfig, usage};
use crate::{FloatOrPercent, ObjectOptions, ProjectSettings, RegionOptions, SliceError};

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
    let nozzle = minimum_nozzle(settings, &resolved.usage.supported_used_filaments)?;
    for object in resolved
        .objects
        .iter()
        .filter(|object| !object.print_objects.is_empty())
    {
        object_widths(&object.object, nozzle)?;
        // Upstream iterates keys before regions, not regions before keys.
        for index in 0..7 {
            for region in &object.all_regions {
                width(
                    region_widths!(region)[index],
                    object.object.layer_height.0,
                    nozzle,
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
    let nozzle = minimum_nozzle(settings, &used.supported_used_filaments)?;
    object_widths(&object, nozzle)?;
    for value in region_widths!(region) {
        width(value, object.layer_height.0, nozzle)?;
    }
    Ok(())
}

fn minimum_nozzle(settings: &ProjectSettings, used: &[usize]) -> Result<f64, SliceError> {
    let mut nozzle = f64::MAX;
    for &filament in used {
        let extruder = settings
            .project
            .gcode
            .filament_map
            .0
            .get(filament)
            .ok_or_else(|| SliceError::InvalidInput("invalid filament_map".to_owned()))?
            .0;
        let diameter = usize::try_from(extruder)
            .ok()
            .and_then(|index| index.checked_sub(1))
            .and_then(|index| settings.project.print.nozzle_diameter.0.get(index))
            .ok_or_else(|| SliceError::InvalidInput("invalid nozzle_diameter".to_owned()))?;
        nozzle = nozzle.min(diameter.0);
    }
    Ok(nozzle)
}

fn has_support(object: &ObjectOptions) -> bool {
    object.enable_support.0 || object.enforce_support_layers.0 > 0 || object.raft_layers.0 > 0
}

fn object_widths(object: &ObjectOptions, nozzle: f64) -> Result<(), SliceError> {
    width(object.line_width, object.layer_height.0, nozzle)?;
    if has_support(object) {
        width(object.support_line_width, object.layer_height.0, nozzle)?;
    }
    Ok(())
}

fn width(value: FloatOrPercent, layer_height: f64, nozzle: f64) -> Result<(), SliceError> {
    let absolute = match value {
        FloatOrPercent::Float(value) => value,
        FloatOrPercent::Percent(value) => nozzle * value.0 / 100.0,
    };
    if absolute != 0.0 && absolute <= layer_height {
        Err(SliceError::InvalidInput("Too small line width".to_owned()))
    } else {
        Ok(())
    }
}
