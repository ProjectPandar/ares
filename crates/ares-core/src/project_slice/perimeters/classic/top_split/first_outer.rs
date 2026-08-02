use crate::{
    SliceError,
    geometry::{CoordinateScale, ExPolygon, JoinType, offset_expolygon, offset2_ex},
};

use super::super::types::ClassicPreludeRecord;

const MITER_LIMIT: f64 = 3.0;
const NARROW_LOOP_LENGTH_MM: f64 = 10.0;

pub(super) struct FirstOuterOffset {
    pub(super) normal: Vec<ExPolygon>,
    pub(super) smaller: Vec<ExPolygon>,
}

pub(super) fn apply(
    polygons: &[ExPolygon],
    record: &ClassicPreludeRecord,
    scale: CoordinateScale,
) -> Result<FirstOuterOffset, SliceError> {
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

    Ok(FirstOuterOffset { normal, smaller })
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

#[cfg(test)]
mod tests;
