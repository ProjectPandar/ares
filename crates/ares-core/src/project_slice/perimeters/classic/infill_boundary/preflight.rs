use crate::{
    SliceError,
    geometry::{Coord, CoordinateScale},
    project_slice::perimeters::classic::{
        gap_extrusion::PreparedPostClassicGapExtrusion,
        onion::ClassicOnionRecord,
        top_split::ClassicTopSplitRecord,
        traversal::{ClassicTraversalRecord, PendingPathBranch},
        types::ClassicPreludeRecord,
    },
};

use super::types::{
    InfillBoundaryOverlap, NoOverlapOffset, ValidatedObject, ValidatedProject, ValidatedRecord,
    ValidatedSurface,
};

const EPSILON: f64 = 1e-4;
const INSET_OVERLAP_TOLERANCE: f64 = 0.4;
const RANGE_ERROR: &str =
    "Classic infill-boundary overlap is outside the supported coordinate range";

pub(super) fn validate(
    prepared: &PreparedPostClassicGapExtrusion,
) -> Result<ValidatedProject, SliceError> {
    let scale = prepared.predecessor.scale;
    let resolution = prepared
        .predecessor
        .resolved
        .views
        .full
        .process
        .print
        .resolution
        .0;
    let scaled_resolution = resolution.max(EPSILON) / scale.factor();
    let objects = prepared
        .objects
        .iter()
        .zip(&prepared.predecessor.objects)
        .map(|(source, traversal)| {
            let hierarchy = &traversal.predecessor;
            let onion = &hierarchy.predecessor;
            let top_split = &onion.predecessor;
            let prelude = &top_split.predecessor;
            let input = &prelude.object;
            assert_eq!(source.records.len(), onion.records.len());
            assert_eq!(source.records.len(), top_split.records.len());
            assert_eq!(source.records.len(), prelude.records.len());
            assert_eq!(source.records.len(), input.records.len());
            assert_eq!(source.records.len(), traversal.records.len());
            let records = source
                .records
                .iter()
                .zip(&onion.records)
                .zip(&top_split.records)
                .zip(&prelude.records)
                .zip(&input.records)
                .zip(&traversal.records)
                .map(|packed| {
                    let (((((source, onion), top), prelude), input_record), traversal_record) =
                        packed;
                    match (source, onion, top, prelude, input_record, traversal_record) {
                        (None, None, None, None, None, None) => Ok(None),
                        (
                            Some(source),
                            Some(onion),
                            Some(top),
                            Some(prelude),
                            Some(input_record),
                            Some(traversal_record),
                        ) => {
                            let options = input.region_options(input_record);
                            validate_inactive_extra_perimeters(
                                input_record,
                                traversal_record,
                                options,
                            );
                            validate_record(RecordContext {
                                source_surface_count: source.surfaces.len(),
                                onion,
                                top,
                                prelude,
                                layer_id: input_record.layer_id,
                                has_upper: input_record.upper_layer_index.is_some(),
                                ordinary_percent: options.infill_wall_overlap.0,
                                top_percent: options.top_bottom_infill_wall_overlap.0,
                                scaled_resolution,
                                scale,
                            })
                            .map(Some)
                        }
                        _ => panic!("O15 predecessor record alignment is invariant"),
                    }
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(ValidatedObject { records })
        })
        .collect::<Result<Vec<_>, SliceError>>()?;
    Ok(ValidatedProject { objects })
}

fn validate_record(context: RecordContext<'_>) -> Result<ValidatedRecord, SliceError> {
    assert_eq!(context.source_surface_count, context.onion.surfaces.len());
    assert_eq!(context.source_surface_count, context.top.surfaces.len());
    assert_eq!(context.source_surface_count, context.prelude.surfaces.len());
    let min_spacing = checked_trunc(
        context.prelude.solid_infill_spacing as f64 * (1.0 - INSET_OVERLAP_TOLERANCE),
    )?;
    let surfaces = context
        .onion
        .surfaces
        .iter()
        .zip(&context.top.surfaces)
        .zip(&context.prelude.surfaces)
        .map(|((onion, top), prelude_surface)| {
            assert_eq!(onion.source_index, top.source_index);
            assert_eq!(onion.source_index, prelude_surface.source_index);
            validate_surface(SurfaceContext {
                source_index: onion.source_index,
                loop_number: onion.effective_loop_number,
                external_spacing: context.prelude.external_spacing,
                perimeter_spacing: context.prelude.perimeter_spacing,
                solid_infill_spacing: context.prelude.solid_infill_spacing,
                min_spacing,
                layer_id: context.layer_id,
                has_upper: context.has_upper,
                ordinary_percent: context.ordinary_percent,
                top_percent: context.top_percent,
                scaled_resolution: context.scaled_resolution,
                scale: context.scale,
            })
        })
        .collect::<Result<Vec<_>, SliceError>>()?;
    Ok(ValidatedRecord { surfaces })
}

#[derive(Clone, Copy)]
struct SurfaceContext {
    source_index: usize,
    loop_number: i32,
    external_spacing: Coord,
    perimeter_spacing: Coord,
    solid_infill_spacing: Coord,
    min_spacing: Coord,
    layer_id: usize,
    has_upper: bool,
    ordinary_percent: f64,
    top_percent: f64,
    scaled_resolution: f64,
    scale: CoordinateScale,
}

fn validate_surface(context: SurfaceContext) -> Result<ValidatedSurface, SliceError> {
    let mut inset = match context.loop_number {
        value if value < 0 => 0,
        0 => context.external_spacing / 2,
        _ => context.perimeter_spacing / 2,
    };
    let (ordinary_overlap, top_overlap) = if inset > 0 {
        let basis = inset
            .checked_add(context.solid_infill_spacing / 2)
            .ok_or_else(range_error)?;
        if context.layer_id == 0 || !context.has_upper {
            (
                convert_overlap(basis, context.top_percent, context.scale)?,
                0,
            )
        } else {
            (
                convert_overlap(basis, context.ordinary_percent, context.scale)?,
                convert_overlap(basis, context.top_percent, context.scale)?,
            )
        }
    } else {
        (0, 0)
    };
    inset = inset
        .checked_sub(ordinary_overlap)
        .ok_or_else(range_error)?;
    let negated_inset = inset.checked_neg().ok_or_else(range_error)?;
    let ordinary_first = (negated_inset as f64 - context.min_spacing as f64 / 2.0) as f32;
    let ordinary_second = (context.min_spacing as f64 / 2.0) as f32;
    let no_overlap = if context.min_spacing / 2 > ordinary_overlap {
        let second = (context.min_spacing / 2)
            .checked_sub(ordinary_overlap)
            .ok_or_else(range_error)?;
        NoOverlapOffset::Two {
            first: ordinary_first,
            second: second as f32,
        }
    } else {
        let delta = negated_inset
            .checked_sub(ordinary_overlap)
            .ok_or_else(range_error)?;
        NoOverlapOffset::One {
            delta: delta as f64 as f32,
        }
    };
    Ok(ValidatedSurface {
        overlap: InfillBoundaryOverlap {
            source_index: context.source_index,
            inset,
            infill_peri_overlap: ordinary_overlap,
            top_infill_peri_overlap: top_overlap,
            min_perimeter_infill_spacing: context.min_spacing,
            scaled_resolution: context.scaled_resolution,
        },
        ordinary_first,
        ordinary_second,
        top_offset: (context.external_spacing / 2) as f64 as f32,
        top_overlap: top_overlap as f64 as f32,
        no_overlap,
    })
}

struct RecordContext<'a> {
    source_surface_count: usize,
    onion: &'a ClassicOnionRecord,
    top: &'a ClassicTopSplitRecord,
    prelude: &'a ClassicPreludeRecord,
    layer_id: usize,
    has_upper: bool,
    ordinary_percent: f64,
    top_percent: f64,
    scaled_resolution: f64,
    scale: CoordinateScale,
}

fn convert_overlap(
    basis: Coord,
    percent: f64,
    scale: CoordinateScale,
) -> Result<Coord, SliceError> {
    let unscaled = basis as f64 * scale.factor();
    let absolute = unscaled * percent;
    let percent_value = absolute / 100.0;
    checked_trunc(percent_value / scale.factor())
}

fn checked_trunc(value: f64) -> Result<Coord, SliceError> {
    let upper_exclusive = -(i64::MIN as f64);
    if !value.is_finite() || value < i64::MIN as f64 || value >= upper_exclusive {
        Err(range_error())
    } else {
        Ok(value as Coord)
    }
}

fn validate_inactive_extra_perimeters(
    input: &crate::project_slice::perimeters::types::PerimeterInputRecord,
    traversal: &ClassicTraversalRecord,
    options: &crate::RegionOptions,
) {
    let (detect_overhang_wall, layer_id, raft_layers) = match traversal.branch {
        PendingPathBranch::OverhangClipping {
            detect_overhang_wall,
            layer_id,
            raft_layers,
        }
        | PendingPathBranch::OrdinaryUnsplit {
            detect_overhang_wall,
            layer_id,
            raft_layers,
        } => (detect_overhang_wall, layer_id, raft_layers),
    };
    assert_eq!(input.layer_id, layer_id);
    assert!(!extra_perimeters_active(ExtraPerimeterGuard {
        spiral_mode: input.spiral_mode,
        has_lower: input.lower_layer_index.is_some(),
        detect_overhang_wall,
        enabled: options.extra_perimeters_on_overhangs.0,
        wall_loops: options.wall_loops.0,
        layer_id: i32::try_from(layer_id).expect("validated layer id fits source int"),
        raft_layers,
    }));
}

#[derive(Clone, Copy)]
struct ExtraPerimeterGuard {
    spiral_mode: bool,
    has_lower: bool,
    detect_overhang_wall: bool,
    enabled: bool,
    wall_loops: i32,
    layer_id: i32,
    raft_layers: i32,
}

fn extra_perimeters_active(guard: ExtraPerimeterGuard) -> bool {
    !guard.spiral_mode
        && guard.has_lower
        && guard.detect_overhang_wall
        && guard.enabled
        && guard.wall_loops > 0
        && guard.layer_id > guard.raft_layers
}

fn range_error() -> SliceError {
    SliceError::InvalidInput(RANGE_ERROR.to_owned())
}

#[cfg(test)]
pub(super) fn convert_overlap_for_test(
    basis: Coord,
    percent: f64,
    scale: CoordinateScale,
) -> Result<Coord, SliceError> {
    convert_overlap(basis, percent, scale)
}

#[cfg(test)]
#[derive(Clone, Copy)]
pub(super) struct TestSurfaceInput {
    pub(super) loop_number: i32,
    pub(super) external_spacing: Coord,
    pub(super) perimeter_spacing: Coord,
    pub(super) solid_infill_spacing: Coord,
    pub(super) layer_id: usize,
    pub(super) has_upper: bool,
    pub(super) ordinary_percent: f64,
    pub(super) top_percent: f64,
    pub(super) scale: CoordinateScale,
}

#[cfg(test)]
pub(super) fn validate_surface_for_test(
    input: TestSurfaceInput,
) -> Result<ValidatedSurface, SliceError> {
    let min_spacing =
        checked_trunc(input.solid_infill_spacing as f64 * (1.0 - INSET_OVERLAP_TOLERANCE))?;
    validate_surface(SurfaceContext {
        source_index: 7,
        loop_number: input.loop_number,
        external_spacing: input.external_spacing,
        perimeter_spacing: input.perimeter_spacing,
        solid_infill_spacing: input.solid_infill_spacing,
        min_spacing,
        layer_id: input.layer_id,
        has_upper: input.has_upper,
        ordinary_percent: input.ordinary_percent,
        top_percent: input.top_percent,
        scaled_resolution: 123.25,
        scale: input.scale,
    })
}

#[cfg(test)]
pub(super) fn extra_perimeters_active_for_test(operands: [i32; 7]) -> bool {
    extra_perimeters_active(ExtraPerimeterGuard {
        spiral_mode: operands[0] != 0,
        has_lower: operands[1] != 0,
        detect_overhang_wall: operands[2] != 0,
        enabled: operands[3] != 0,
        wall_loops: operands[4],
        layer_id: operands[5],
        raft_layers: operands[6],
    })
}
