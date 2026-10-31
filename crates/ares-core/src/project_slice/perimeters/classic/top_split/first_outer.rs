use crate::{
    SliceError,
    geometry::{
        CoordinateScale, ExPolygon, JoinType, ThickPolyline, difference_ex_polygons, medial_axis,
        offset_expolygon, offset_expolygons_paths, offset2_ex, opening_ex,
    },
};

use super::super::types::ClassicPreludeRecord;

const MITER_LIMIT: f64 = 3.0;
const NARROW_LOOP_LENGTH_MM: f64 = 10.0;
const CLIPPER_SAFETY_OFFSET: f32 = 10.0;

pub(super) struct FirstOuterOffset {
    pub(super) normal: Vec<ExPolygon>,
    pub(super) smaller: Vec<ExPolygon>,
    /// `thin_walls` — the `i == 0` medial-axis extraction emitted as
    /// `variable_width` external-perimeter paths in traverse_loops
    /// (PerimeterGenerator.cpp:1243-1266,231). Empty unless `detect_thin_wall`.
    pub(super) thin_walls: Vec<ThickPolyline>,
}

pub(super) fn apply(
    polygons: &[ExPolygon],
    record: &ClassicPreludeRecord,
    scale: CoordinateScale,
) -> Result<FirstOuterOffset, SliceError> {
    if record.detect_thin_wall {
        return thin_wall_offsets(polygons, record, scale);
    }
    let smaller_width = scaled_flow(scale, record.smaller_external_flow.width)?;
    let narrow_length = scale
        .checked_scale(NARROW_LOOP_LENGTH_MM)
        .ok_or_else(geometry_error)?;
    let mut normal = Vec::new();
    let mut smaller = Vec::new();

    for expolygon in polygons {
        let narrow_test = offset2_ex(
            std::slice::from_ref(expolygon),
            -((record.external_width as f64 / 2.0
                + record.smaller_external_minimum_spacing as f64 / 2.0) as f32),
            (record.smaller_external_minimum_spacing as f64 / 2.0) as f32,
            JoinType::Miter,
            MITER_LIMIT,
        )
        .map_err(|_| geometry_error())?;
        let narrow_and_short = narrow_test.is_empty()
            && expolygon.area()
                < (record.external_width + record.smaller_external_minimum_spacing) as f64
                    * narrow_length as f64;
        let width = if narrow_and_short {
            smaller_width
        } else {
            record.external_width
        };
        let output = offset_expolygon(
            expolygon,
            -(width as f64 / 2.0) as f32,
            JoinType::Miter,
            MITER_LIMIT,
        )
        .map_err(|_| geometry_error())?;
        if narrow_and_short {
            smaller.extend(output);
        } else {
            normal.extend(output);
        }
    }

    Ok(FirstOuterOffset {
        normal,
        smaller,
        thin_walls: Vec::new(),
    })
}

/// `detect_thin_wall` `i == 0` branch (PerimeterGenerator.cpp:1251-1266).
/// `offsets` becomes the surviving `last` for the onion; the slivers too thin
/// for a full loop are extracted into `thin_walls` via the medial axis.
fn thin_wall_offsets(
    last: &[ExPolygon],
    record: &ClassicPreludeRecord,
    scale: CoordinateScale,
) -> Result<FirstOuterOffset, SliceError> {
    let ext_w = record.external_width;
    let ext_min_sp = record.external_minimum_spacing;
    // `offset2_ex(last, -(ext_w/2 + ext_min_sp/2 - 1), +(ext_min_sp/2 - 1))`
    let offsets = offset2_ex(
        last,
        -((ext_w as f64 / 2.0 + ext_min_sp as f64 / 2.0 - 1.0) as f32),
        (ext_min_sp as f64 / 2.0 - 1.0) as f32,
        JoinType::Miter,
        MITER_LIMIT,
    )
    .map_err(|_| geometry_error())?;
    // `min_width = coord_t(scale_(nozzle_diameter / 3))` — scale-domain, kept
    // as f64 for `medial_axis` (which compares against scale-domain geometry).
    let nozzle = record.ext_perimeter_flow.nozzle_diameter;
    let min_width = scale
        .checked_scale(f64::from(nozzle / 3.0))
        .ok_or_else(geometry_error)? as f64;
    // `opening_ex(diff_ex(last, offset(offsets, ext_w/2 + ClipperSafetyOffset)), min_width/2)`
    let grown = offset_expolygons_paths(
        &offsets,
        (ext_w as f64 / 2.0) as f32 + CLIPPER_SAFETY_OFFSET,
        JoinType::Miter,
        MITER_LIMIT,
    )
    .map_err(|_| geometry_error())?;
    let thin_area = difference_ex_polygons(last, &grown).map_err(|_| geometry_error())?;
    let candidates = opening_ex(
        &thin_area,
        (min_width / 2.0) as f32,
        JoinType::Miter,
        MITER_LIMIT,
    )
    .map_err(|_| geometry_error())?;
    // `ex.medial_axis(min_width, ext_perimeter_width + ext_perimeter_spacing2)`
    // — `ext_perimeter_spacing2` is `external_to_internal_spacing`.
    let max_width = (ext_w + record.external_to_internal_spacing) as f64;
    let mut thin_walls = Vec::new();
    for ex in &candidates {
        thin_walls
            .extend(medial_axis(ex, min_width, max_width, scale).map_err(|_| geometry_error())?);
    }
    Ok(FirstOuterOffset {
        normal: offsets,
        smaller: Vec::new(),
        thin_walls,
    })
}

fn scaled_flow(scale: CoordinateScale, value: f32) -> Result<i64, SliceError> {
    scale
        .checked_scale(f64::from(value))
        .ok_or_else(geometry_error)?;
    Ok((f64::from(value) / scale.factor()) as i64)
}

fn geometry_error() -> SliceError {
    SliceError::InvalidInput(
        "Classic top split geometry is outside the supported Clipper range".to_owned(),
    )
}
