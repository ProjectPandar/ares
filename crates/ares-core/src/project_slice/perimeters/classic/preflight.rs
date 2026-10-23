use crate::{ObjectOptions, RegionOptions, SliceError};

use super::super::types::{PerimeterInputRecord, PostPerimeterInputPrintObject};

#[derive(Clone, Copy)]
pub(super) struct ValidatedClassicConfig {
    pub(super) wall_loops: i32,
    pub(super) precise_outer_wall: bool,
    pub(super) detect_overhang_wall: bool,
    pub(super) detect_thin_wall: bool,
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
    enable_arc_fitting: bool,
    scaled_resolution: f64,
    has_fuzzy_skin: bool,
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

    objects
        .iter()
        .map(|object| {
            let (source_object_index, _) = object.identity();
            let resolved = context
                .resolved_objects
                .iter()
                .find(|resolved| resolved.source_object_index == source_object_index)
                .expect("Task 22N object must retain its resolved source object");
            let records =
                object
                    .as_parts()
                    .1
                    .iter()
                    .map(|record| {
                        let Some(record) = record else {
                            return Ok(None);
                        };
                        let has_fuzzy_skin =
                            object.compatible_fuzzy_regions(record).any(|(options, _)| {
                                crate::perimeters::FuzzySkinConfig::from_region(options)
                                    .should_fuzzify(record.layer_id, 0, true)
                            });
                        validate_record(
                            record,
                            RecordValidationContext {
                                object_options: &resolved.object,
                                region: object.region_options(record),
                                enable_arc_fitting: context.enable_arc_fitting,
                                scaled_resolution,
                                has_fuzzy_skin,
                                nozzle_diameters: context.nozzle_diameters,
                            },
                        )
                        .map(Some)
                    })
                    .collect::<Result<Vec<_>, _>>()?;
            Ok(ValidatedClassicObject { records })
        })
        .collect()
}

fn validate_record(
    record: &PerimeterInputRecord,
    context: RecordValidationContext<'_>,
) -> Result<ValidatedClassicConfig, SliceError> {
    let RecordValidationContext {
        object_options,
        region,
        enable_arc_fitting,
        scaled_resolution,
        has_fuzzy_skin,
        nozzle_diameters,
    } = context;
    // `detect_thin_wall` is supported: `i == 0` medial-axis extraction in
    // `first_outer::thin_wall_offsets` (PerimeterGenerator.cpp:1251-1266),
    // materialized as `variable_width` external-perimeter `ExtrusionPath`s in
    // `traverse_loops` (:231-232).
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

    let simplify_resolution = if enable_arc_fitting && !has_fuzzy_skin {
        0.2 * scaled_resolution
    } else {
        scaled_resolution
    };
    let first_object_layer =
        i32::try_from(record.layer_id).ok() == Some(object_options.raft_layers.0);
    let mut wall_loops = region.wall_loops.0;
    if region.alternate_extra_wall.0
        && record.layer_id % 2 == 1
        && region.sparse_infill_density.0 > 0.0
    {
        wall_loops += 1;
    }
    if region.only_one_wall_first_layer.0 && first_object_layer {
        wall_loops = wall_loops.min(1);
    }
    Ok(ValidatedClassicConfig {
        wall_loops,
        precise_outer_wall: region.precise_outer_wall.0
            && region.wall_sequence == crate::ProcessWallSequence::InnerOuter,
        detect_overhang_wall: region.detect_overhang_wall.0,
        detect_thin_wall: region.detect_thin_wall.0,
        only_one_wall_top: region.only_one_wall_top.0,
        gap_infill_speed: region.gap_infill_speed.0,
        surface_simplify_resolution: simplify_resolution,
        support_nozzle_diameter,
    })
}

fn invalid(message: &str) -> SliceError {
    SliceError::InvalidInput(message.to_owned())
}
