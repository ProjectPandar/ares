use crate::{ProjectObject, SliceError, project::effective_config::types::ResolvedProjectObject};

pub(super) fn validate(
    source_objects: &[ProjectObject],
    resolved_objects: &[ResolvedProjectObject],
) -> Result<(), SliceError> {
    for resolved in resolved_objects {
        if resolved.object.brim_type == crate::ProcessBrimType::LeadingEdgeOnly {
            return unsupported("brim_type: leading_edge_only");
        }
        if resolved.object.support_interface_pattern
            == crate::ProcessSupportInterfacePattern::SpiralInset
        {
            return unsupported("support_interface_pattern: spiralinset");
        }
        if resolved.object.support_ironing_pattern == crate::ProcessInfillPattern::SpiralInset {
            return unsupported("support_ironing_pattern: spiralinset");
        }
        for region in &resolved.all_regions {
            for (key, pattern) in [
                ("top_surface_pattern", region.top_surface_pattern),
                ("bottom_surface_pattern", region.bottom_surface_pattern),
                (
                    "internal_solid_infill_pattern",
                    region.internal_solid_infill_pattern,
                ),
                ("sparse_infill_pattern", region.sparse_infill_pattern),
                ("ironing_pattern", region.ironing_pattern),
            ] {
                validate_pattern(key, pattern)?;
            }
        }
    }
    if resolved_objects.iter().any(|resolved| {
        source_objects[resolved.source_object_index]
            .layer_height_profile()
            .is_empty()
            && source_objects[resolved.source_object_index]
                .layer_config_ranges()
                .iter()
                .any(|range| range.layer_height().is_some())
            && source_objects[resolved.source_object_index]
                .layer_config_ranges()
                .iter()
                .any(|range| range.layer_height().is_none())
    }) {
        // Slicing.cpp:249 is undefined without height; profile selection retains
        // this rejection if a nonempty stored profile must regenerate.
        return unsupported("layer_config_ranges");
    }
    Ok(())
}

pub(super) fn validate_settings(settings: &crate::ProjectSettings) -> Result<(), SliceError> {
    for (key, pattern) in [
        (
            "top_surface_pattern",
            settings.process.region.top_surface_pattern,
        ),
        (
            "bottom_surface_pattern",
            settings.process.region.bottom_surface_pattern,
        ),
        (
            "internal_solid_infill_pattern",
            settings.process.region.internal_solid_infill_pattern,
        ),
        (
            "sparse_infill_pattern",
            settings.process.region.sparse_infill_pattern,
        ),
        ("ironing_pattern", settings.process.region.ironing_pattern),
        (
            "support_ironing_pattern",
            settings.process.object.support_ironing_pattern,
        ),
    ] {
        validate_pattern(key, pattern)?;
    }
    if settings.process.object.support_interface_pattern
        == crate::ProcessSupportInterfacePattern::SpiralInset
    {
        return Err(SliceError::UnsupportedProjectFeature(
            "support_interface_pattern: spiralinset".to_owned(),
        ));
    }
    if settings.process.object.brim_type == crate::ProcessBrimType::LeadingEdgeOnly {
        return Err(SliceError::UnsupportedProjectFeature(
            "brim_type: leading_edge_only".to_owned(),
        ));
    }
    if matches!(
        settings.process.print.print_order,
        crate::ProcessPrintOrder::BestOfStrategies | crate::ProcessPrintOrder::Snake
    ) {
        let value =
            if settings.process.print.print_order == crate::ProcessPrintOrder::BestOfStrategies {
                "best_of"
            } else {
                "snake"
            };
        return Err(SliceError::UnsupportedProjectFeature(format!(
            "print_order: {value}"
        )));
    }
    if settings.project.gcode.filament_map_mode == crate::ProjectFilamentMapMode::NozzleManual {
        return Err(SliceError::UnsupportedProjectFeature(
            "filament_map_mode: Nozzle Manual".to_owned(),
        ));
    }
    Ok(())
}

fn validate_pattern(key: &str, pattern: crate::ProcessInfillPattern) -> Result<(), SliceError> {
    if pattern == crate::ProcessInfillPattern::SpiralInset {
        return unsupported(&format!("{key}: spiralinset"));
    }
    Ok(())
}

fn unsupported<T>(key: &str) -> Result<T, SliceError> {
    Err(SliceError::UnsupportedProjectFeature(key.to_owned()))
}
