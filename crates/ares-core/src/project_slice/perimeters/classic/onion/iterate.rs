use crate::{
    SliceError,
    geometry::{
        ExPolygon, JoinType, Polygon, difference_polygons_ex, offset_expolygons_paths, offset2_ex,
    },
};

use super::{config::ValidatedOnionConfig, types::RawShellDepth};

const MITER_LIMIT: f64 = 3.0;

pub(super) struct IterationInput<'a> {
    pub(super) initial_loop_number: i32,
    pub(super) loop_number: i32,
    pub(super) normal_first_offset: &'a [ExPolygon],
    pub(super) smaller_first_offset: &'a [ExPolygon],
    pub(super) remaining: &'a [ExPolygon],
    pub(super) config: ValidatedOnionConfig,
}

pub(super) struct IterationResult {
    pub(super) effective_loop_number: i32,
    pub(super) shells: Vec<RawShellDepth>,
    pub(super) last: Vec<ExPolygon>,
    pub(super) gaps: Vec<ExPolygon>,
}

pub(super) fn apply(input: IterationInput<'_>) -> Result<IterationResult, SliceError> {
    let IterationInput {
        initial_loop_number,
        loop_number,
        normal_first_offset,
        smaller_first_offset,
        remaining,
        config,
    } = input;
    if initial_loop_number < 0
        || (normal_first_offset.is_empty() && smaller_first_offset.is_empty())
    {
        return Ok(IterationResult {
            effective_loop_number: if initial_loop_number < 0 {
                loop_number
            } else {
                -1
            },
            shells: Vec::new(),
            last: if initial_loop_number < 0 {
                remaining.to_vec()
            } else {
                Vec::new()
            },
            gaps: Vec::new(),
        });
    }

    let mut result = IterationResult {
        effective_loop_number: loop_number,
        shells: vec![RawShellDepth {
            depth: 0,
            normal: normal_first_offset.to_vec(),
            smaller_width: smaller_first_offset.to_vec(),
        }],
        last: remaining.to_vec(),
        gaps: Vec::new(),
    };
    if loop_number == 0 && (!config.has_gap_fill || config.sparse_infill_density == 0) {
        return Ok(result);
    }

    let mut depth = 1;
    loop {
        let distance = if depth == 1 {
            config.external_to_internal_spacing
        } else {
            config.perimeter_spacing
        };
        let (first_delta, second_delta) = offset2_deltas(distance, config.minimum_spacing);
        let offsets = offset2_ex(
            &result.last,
            first_delta,
            second_delta,
            JoinType::Miter,
            MITER_LIMIT,
        )
        .map_err(|_| geometry_error())?;

        if config.has_gap_fill {
            let (outer_delta, inner_delta) = gap_deltas(distance);
            let outer = offset(&result.last, outer_delta)?;
            let inner = offset(&offsets, inner_delta)?;
            let collected = difference_polygons_ex(&outer, &inner).map_err(|_| geometry_error())?;
            result.gaps.extend(collected);
        }

        if offsets.is_empty() {
            result.effective_loop_number = depth - 1;
            result.last.clear();
            break;
        }
        // Upstream loops one extra round after the last perimeter purely to
        // collect the gaps between the final offset shells; that round stores
        // no shell and keeps `last` unchanged (PerimeterGenerator.cpp:1240-1245,
        // 1341-1343).
        if depth > loop_number {
            break;
        }

        result.shells.push(RawShellDepth {
            depth,
            normal: offsets.clone(),
            smaller_width: Vec::new(),
        });
        result.last = offsets;
        if depth == loop_number && (!config.has_gap_fill || config.sparse_infill_density == 0) {
            break;
        }
        depth += 1;
    }
    Ok(result)
}

fn offset2_deltas(distance: i64, minimum_spacing: i64) -> (f32, f32) {
    (
        -((distance as f64 + minimum_spacing as f64 / 2.0 - 1.0) as f32),
        (minimum_spacing as f64 / 2.0 - 1.0) as f32,
    )
}

fn gap_deltas(distance: i64) -> (f32, f32) {
    (
        -((0.5_f64 * distance as f64) as f32),
        (0.5_f64 * distance as f64 + 10.0) as f32,
    )
}

fn offset(input: &[ExPolygon], delta: f32) -> Result<Vec<Polygon>, SliceError> {
    offset_expolygons_paths(input, delta, JoinType::Miter, MITER_LIMIT)
        .map_err(|_| geometry_error())
}

fn geometry_error() -> SliceError {
    SliceError::InvalidInput(
        "Classic onion geometry is outside the supported Clipper range".to_owned(),
    )
}

#[cfg(test)]
mod tests;
