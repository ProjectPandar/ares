use crate::{ProjectObject, SliceError, project::effective_config::types::ResolvedProjectObject};

pub(super) fn validate(
    source_objects: &[ProjectObject],
    resolved_objects: &[ResolvedProjectObject],
) -> Result<(), SliceError> {
    for resolved in resolved_objects {
        if resolved.object.brim_type == crate::ProcessBrimType::LeadingEdgeOnly {
            return unsupported("brim_type: leading_edge_only");
        }
        for region in &resolved.all_regions {
            validate_adaptive_infill(
                region.sparse_infill_pattern,
                region.sparse_infill_density.0,
                region.adaptive_infill_mode,
            )?;
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
    validate_adaptive_infill(
        settings.process.region.sparse_infill_pattern,
        settings.process.region.sparse_infill_density.0,
        settings.process.region.adaptive_infill_mode,
    )?;
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

fn validate_adaptive_infill(
    pattern: crate::ProcessInfillPattern,
    density: f64,
    mode: crate::ProcessAdaptiveInfillMode,
) -> Result<(), SliceError> {
    if mode == crate::ProcessAdaptiveInfillMode::Disabled || density <= 0.0 || density >= 100.0 {
        return Ok(());
    }
    match pattern {
        crate::ProcessInfillPattern::Gyroid
            if mode == crate::ProcessAdaptiveInfillMode::SteppedShells =>
        {
            unsupported("adaptive_infill_mode: stepped_shells")
        }
        crate::ProcessInfillPattern::TpmsD => unsupported("adaptive_infill_mode: tpmsd"),
        crate::ProcessInfillPattern::TpmsFk => unsupported("adaptive_infill_mode: tpmsfk"),
        _ => Ok(()),
    }
}

fn unsupported<T>(key: &str) -> Result<T, SliceError> {
    Err(SliceError::UnsupportedProjectFeature(key.to_owned()))
}
