use crate::{
    SliceError,
    geometry::{
        BoundingBox, ExPolygon, JoinType, clip_clipper_expolygons_with_subject_bbox, difference_ex,
        difference_ex_polygons_with_safety_offset, difference_ex_with_safety_offset,
        intersection_ex, offset_expolygons, offset_paths, union_expolygons,
    },
};

use super::{super::types::ClassicPreludeRecord, config::ValidatedTopSplitConfig};

const MITER_LIMIT: f64 = 3.0;
const BRIDGE_INFILL_MARGIN: f64 = 1.0;
const SLIC3R_EPSILON_MM: f64 = 0.0001;

pub(super) struct SplitResult {
    pub(super) top_fills: Vec<ExPolygon>,
    pub(super) non_top_polygons: Vec<ExPolygon>,
    pub(super) fill_clip: Vec<ExPolygon>,
}

pub(super) struct SplitContext<'a> {
    pub(super) upper_slices: &'a [ExPolygon],
    pub(super) lower_slices: Option<&'a [ExPolygon]>,
    pub(super) record: &'a ClassicPreludeRecord,
    pub(super) config: ValidatedTopSplitConfig,
    pub(super) scale: crate::geometry::CoordinateScale,
}

pub(super) fn apply(
    orig_polygons: &[ExPolygon],
    context: SplitContext<'_>,
) -> Result<SplitResult, SliceError> {
    let SplitContext {
        upper_slices,
        lower_slices,
        record,
        config,
        scale,
    } = context;
    if orig_polygons.is_empty() {
        return Ok(SplitResult {
            top_fills: Vec::new(),
            non_top_polygons: Vec::new(),
            fill_clip: Vec::new(),
        });
    }

    let wall_span = if config.wall_loops == 0 {
        0
    } else {
        record.external_width + record.perimeter_spacing * i64::from(config.wall_loops - 1)
    };
    let mut offset_top_surface = checked_scale(scale, 0.9 * scale.unscale(wall_span))?;
    let inner_span = if config.wall_loops <= 1 {
        0
    } else {
        record.perimeter_spacing * i64::from(config.wall_loops - 1)
    };
    if offset_top_surface as f64 > 0.9 * inner_span as f64 {
        offset_top_surface -= (0.9 * inner_span as f64) as i64;
    } else {
        offset_top_surface = 0;
    }

    let mut bounds = BoundingBox::from_expolygons(orig_polygons)
        .expect("nonempty split input must contain nonempty contours");
    bounds.offset(checked_scale(scale, SLIC3R_EPSILON_MM)?);
    let min_width_top_surface =
        (record.external_spacing as f64 / 2.0 + 10.0).max(config.min_width_top_surface);

    let upper_polygons = clip_clipper_expolygons_with_subject_bbox(upper_slices, bounds);
    let grown_upper = offset_paths(
        &upper_polygons,
        min_width_top_surface as f32,
        JoinType::Miter,
        MITER_LIMIT,
    )
    .map_err(|_| geometry_error())?;

    let virtual_fill_clip = offset(orig_polygons, -(record.external_spacing as f64) as f32)?;
    let bridge_checker = if let Some(lower_slices) = lower_slices {
        let lower_polygons = clip_clipper_expolygons_with_subject_bbox(lower_slices, bounds);
        let unsupported = difference_ex_polygons_with_safety_offset(orig_polygons, &lower_polygons)
            .map_err(|_| geometry_error())?;
        let bridge_offset = record.external_spacing.max(record.perimeter_width) as f64;
        let standard_margin = scaled_f32(scale, BRIDGE_INFILL_MARGIN)?;
        let nozzle_margin = scaled_f32(
            scale,
            config.outer_nozzle_diameter * BRIDGE_INFILL_MARGIN / 0.4,
        )?;
        offset(
            &unsupported,
            (1.5 * bridge_offset
                + f64::from(standard_margin.min(nozzle_margin))
                + record.perimeter_spacing as f64 / 2.0) as f32,
        )?
    } else {
        Vec::new()
    };

    let delete_bridge = difference_ex_with_safety_offset(orig_polygons, &bridge_checker)
        .map_err(|_| geometry_error())?;
    let mut top_polygons = difference_ex_polygons_with_safety_offset(&delete_bridge, &grown_upper)
        .map_err(|_| geometry_error())?;
    let temp_gap =
        difference_ex(&top_polygons, &virtual_fill_clip).map_err(|_| geometry_error())?;
    let grown_top = offset(
        &top_polygons,
        (offset_top_surface as f64 + min_width_top_surface - record.external_spacing as f64 / 2.0)
            as f32,
    )?;
    let inner_polygons = difference_ex_with_safety_offset(orig_polygons, &grown_top)
        .map_err(|_| geometry_error())?;
    top_polygons = difference_ex_with_safety_offset(&virtual_fill_clip, &inner_polygons)
        .map_err(|_| geometry_error())?;
    let top_fills = union_expolygons(&[], &top_polygons).map_err(|_| geometry_error())?;
    let fill_clip = offset(
        orig_polygons,
        (record.external_spacing as f64 / 2.0 - config.sparse_infill_width / 2.0) as f32,
    )?;
    let mut non_top_polygons =
        intersection_ex(&inner_polygons, orig_polygons).map_err(|_| geometry_error())?;
    if config.has_gap_fill {
        non_top_polygons =
            union_expolygons(&non_top_polygons, &temp_gap).map_err(|_| geometry_error())?;
    }

    Ok(SplitResult {
        top_fills,
        non_top_polygons,
        fill_clip,
    })
}

fn offset(input: &[ExPolygon], delta: f32) -> Result<Vec<ExPolygon>, SliceError> {
    offset_expolygons(input, delta, JoinType::Miter, MITER_LIMIT).map_err(|_| geometry_error())
}

fn checked_scale(scale: crate::geometry::CoordinateScale, value: f64) -> Result<i64, SliceError> {
    scale.checked_scale(value).ok_or_else(geometry_error)
}

fn scaled_f32(scale: crate::geometry::CoordinateScale, value: f64) -> Result<f32, SliceError> {
    scale.checked_scale(value).ok_or_else(geometry_error)?;
    Ok((value / scale.factor()) as f32)
}

fn geometry_error() -> SliceError {
    SliceError::InvalidInput(
        "Classic top split geometry is outside the supported Clipper range".to_owned(),
    )
}

#[cfg(test)]
mod tests;
