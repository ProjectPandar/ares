use crate::{
    ObjectOptions, ProcessBrimType, ProcessCounterboreHoleBridging, ProcessFuzzySkinType,
    ProcessWallSequence, RegionOptions, SliceError,
};

use super::super::types::{PerimeterDispatch, PerimeterInputRecord, PostPerimeterInputPrintObject};

#[derive(Clone, Copy)]
pub(super) struct ValidatedClassicConfig {
    pub(super) wall_loops: i32,
    pub(super) precise_outer_wall: bool,
    pub(super) detect_overhang_wall: bool,
    pub(super) only_one_wall_top: bool,
    pub(super) gap_infill_speed: f64,
    pub(super) surface_simplify_resolution: f64,
    pub(super) support_nozzle_diameter: f64,
}

pub(super) struct ValidatedClassicObject {
    pub(super) records: Vec<Option<ValidatedClassicConfig>>,
}

pub(super) struct ClassicValidationContext<'a> {
    pub(super) resolved_objects:
        &'a [crate::project::effective_config::types::ResolvedProjectObject],
    pub(super) enable_arc_fitting: bool,
    pub(super) resolution: f64,
    pub(super) nozzle_diameters: &'a crate::OrcaFloats,
    pub(super) scale: crate::geometry::CoordinateScale,
}

struct RecordValidationContext<'a> {
    object_options: &'a ObjectOptions,
    region: &'a RegionOptions,
    simplify_resolution: f64,
    nozzle_diameters: &'a crate::OrcaFloats,
}

pub(super) fn validate_project(
    objects: &[PostPerimeterInputPrintObject],
    context: ClassicValidationContext<'_>,
) -> Result<Vec<ValidatedClassicObject>, SliceError> {
    if !context.resolution.is_finite() {
        return Err(invalid("invalid Orca option resolution"));
    }
    let effective_resolution = if context.resolution > 1e-4 {
        context.resolution
    } else {
        1e-4
    };
    let scaled_resolution = effective_resolution / context.scale.factor();
    let simplify_resolution = if context.enable_arc_fitting {
        0.2 * scaled_resolution
    } else {
        scaled_resolution
    };

    objects
        .iter()
        .map(|object| {
            let (source_object_index, _) = object.identity();
            let resolved = context
                .resolved_objects
                .iter()
                .find(|resolved| resolved.source_object_index == source_object_index)
                .expect("Task 22N object must retain its resolved source object");
            let records = object
                .as_parts()
                .1
                .iter()
                .map(|record| {
                    record
                        .as_ref()
                        .map(|record| {
                            validate_record(
                                object,
                                record,
                                RecordValidationContext {
                                    object_options: &resolved.object,
                                    region: object.region_options(record),
                                    simplify_resolution,
                                    nozzle_diameters: context.nozzle_diameters,
                                },
                            )
                        })
                        .transpose()
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(ValidatedClassicObject { records })
        })
        .collect()
}

fn validate_record(
    object: &PostPerimeterInputPrintObject,
    record: &PerimeterInputRecord,
    context: RecordValidationContext<'_>,
) -> Result<ValidatedClassicConfig, SliceError> {
    let RecordValidationContext {
        object_options,
        region,
        simplify_resolution,
        nozzle_diameters,
    } = context;
    if record.dispatch == PerimeterDispatch::Arachne {
        return Err(unsupported("wall_generator"));
    }
    if record.spiral_mode {
        return Err(unsupported("spiral_mode"));
    }
    if fuzzy_is_active(region, record.layer_id) {
        return Err(unsupported("fuzzy_skin"));
    }
    if region.detect_thin_wall.0 {
        return Err(unsupported("detect_thin_wall"));
    }
    if region.alternate_extra_wall.0
        && record.layer_id % 2 == 1
        && region.sparse_infill_density.0 > 0.0
    {
        return Err(unsupported("alternate_extra_wall"));
    }
    if region.only_one_wall_first_layer.0
        && i32::try_from(record.layer_id).ok() == Some(object_options.raft_layers.0)
    {
        return Err(unsupported("only_one_wall_first_layer"));
    }
    if region.overhang_reverse.0 {
        return Err(unsupported("overhang_reverse"));
    }
    if region.wall_sequence != ProcessWallSequence::InnerOuter {
        return Err(unsupported("wall_sequence"));
    }
    if object_options.brim_type == ProcessBrimType::OuterOnly
        && object_options.brim_width.0 > 0.0
        && record.layer_id == 0
    {
        return Err(unsupported("brim_type"));
    }
    if region.extra_perimeters_on_overhangs.0
        && object.lower_slices(record).is_some()
        && region.detect_overhang_wall.0
        && region.wall_loops.0 > 0
        && i32::try_from(record.layer_id).is_ok_and(|layer| layer > object_options.raft_layers.0)
    {
        return Err(unsupported("extra_perimeters_on_overhangs"));
    }
    if region.counterbore_hole_bridging != ProcessCounterboreHoleBridging::None {
        return Err(unsupported("counterbore_hole_bridging"));
    }

    let nozzle_index = region
        .outer_wall_filament_id
        .0
        .checked_sub(1)
        .and_then(|index| usize::try_from(index).ok())
        .filter(|index| *index < nozzle_diameters.0.len())
        .unwrap_or(0);
    let support_nozzle_diameter = nozzle_diameters
        .0
        .get(nozzle_index)
        .map(|value| value.0)
        .ok_or_else(|| invalid("invalid Orca option nozzle_diameter"))?;

    Ok(ValidatedClassicConfig {
        wall_loops: region.wall_loops.0,
        precise_outer_wall: region.precise_outer_wall.0,
        detect_overhang_wall: region.detect_overhang_wall.0,
        only_one_wall_top: region.only_one_wall_top.0,
        gap_infill_speed: region.gap_infill_speed.0,
        surface_simplify_resolution: simplify_resolution,
        support_nozzle_diameter,
    })
}

fn fuzzy_is_active(region: &RegionOptions, layer_id: usize) -> bool {
    !matches!(
        region.fuzzy_skin,
        ProcessFuzzySkinType::None | ProcessFuzzySkinType::Disabled
    ) && (layer_id > 0 || region.fuzzy_skin_first_layer.0)
}

fn invalid(message: &str) -> SliceError {
    SliceError::InvalidInput(message.to_owned())
}

fn unsupported(key: &str) -> SliceError {
    SliceError::UnsupportedProjectFeature(key.to_owned())
}
