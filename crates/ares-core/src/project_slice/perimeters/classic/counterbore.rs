//! Classic `PerimeterGenerator.cpp:1728–1924`, chbBridges only.
mod detector;
use super::super::types::{PerimeterDispatch, PerimeterInputRecord, PostPerimeterInputPrintObject};
use crate::geometry::{
    ClipperError, CoordinateScale, ExPolygon, FillRule, JoinType, append_simplified_expolygon,
    difference_ex, difference_ex_with_safety_offset, intersection_ex, offset_expolygons,
    offset2_ex, opening_ex, union_ex, union_expolygons,
};
use crate::project_slice::region_slices::RegionSurface;
use crate::{ProcessCounterboreHoleBridging, RegionOptions, SliceError};

pub(super) fn process(
    object: &mut PostPerimeterInputPrintObject,
    record: &PerimeterInputRecord,
    spacing: i64,
    external_width: i64,
    scale: CoordinateScale,
) -> Result<Vec<ExPolygon>, SliceError> {
    let config = object.region_options(record);
    if record.dispatch != PerimeterDispatch::Classic
        || config.counterbore_hole_bridging != ProcessCounterboreHoleBridging::PartiallyBridged
    {
        return Ok(Vec::new());
    }
    let Some(lower) = object
        .lower_slices(record)
        .filter(|lower| !lower.is_empty())
    else {
        return Ok(Vec::new());
    };
    let mut surfaces = object.perimeter_surfaces(record).to_vec();
    let fill = process_surfaces(
        &mut surfaces,
        lower,
        config,
        record.model_rotation_rad,
        spacing,
        external_width,
        scale,
    )
    .map_err(geometry_error)?;
    let slot = object.region_layer_record_index(record.current);
    let group = &mut object.groups[object.record_groups[slot].unwrap()];
    if group.owner == record.current {
        group.merged_surfaces = Some(surfaces);
    }
    Ok(fill)
}

fn process_surfaces(
    surfaces: &mut Vec<RegionSurface>,
    lower: &[ExPolygon],
    config: &RegionOptions,
    model_rotation: f64,
    spacing: i64,
    width: i64,
    scale: CoordinateScale,
) -> Result<Vec<ExPolygon>, ClipperError> {
    // libslic3r.h:89 BRIDGE_INFILL_MARGIN = 1 mm.
    let anchor = (1.0 / scale.factor()).min(spacing as f64).min(width as f64);
    let mut fill = Vec::new();
    let mut index = 0;
    while index < surfaces.len() {
        let original = surfaces[index].clone();
        let mut last = vec![original.as_parts().1.clone()];
        let unsupported = difference_ex_with_safety_offset(&last, lower)?;
        if !unsupported.is_empty() {
            let filtered = offset2_ex(
                &unsupported,
                (-spacing) as f32,
                spacing as f32,
                JoinType::Miter,
                3.0,
            )?;
            if !filtered.is_empty() {
                let support = difference_ex_with_safety_offset(&last, &unsupported)?;
                let custom = config.bridge_angle.0;
                let angle = if custom > 0.0 && !config.relative_bridge_angle.0 {
                    custom.to_radians()
                        + if config.align_infill_direction_to_model.0 {
                            model_rotation
                        } else {
                            0.0
                        }
                } else {
                    0.0
                };
                let mut bridgeable = Vec::new();
                for polygon in &filtered {
                    let coverage = detector::bridgeable(polygon, &support, spacing / 4, angle)?;
                    bridgeable.extend(union_ex(&coverage, FillRule::NonZero)?);
                }
                if !bridgeable.is_empty() && !original.as_parts().1.holes().is_empty() {
                    last = difference_ex_with_safety_offset(&last, &filtered)?;
                    let mask = partial_mask(&last, &filtered, &bridgeable, spacing, width, anchor)?;
                    if !mask.is_empty() {
                        fill.extend(mask.iter().cloned());
                        last = difference_ex_with_safety_offset(&last, &mask)?;
                        let mut rest = Vec::new();
                        for polygon in last {
                            if intersection_ex(&support, std::slice::from_ref(&polygon))?.is_empty()
                            {
                                fill.push(polygon);
                            } else {
                                rest.push(polygon);
                            }
                        }
                        last = rest;
                    }
                }
            }
        }
        if last.is_empty() {
            surfaces.remove(index);
        } else {
            let mut parts = last.into_iter();
            surfaces[index] =
                original.clone_with_expolygon(parts.next().expect("nonempty preserved surface"));
            surfaces.extend(parts.map(|polygon| original.clone_with_expolygon(polygon)));
            index += 1;
        }
    }
    Ok(fill)
}

// PerimeterGenerator.cpp:1865–1891: preserve operation and safety-offset order.
fn partial_mask(
    last: &[ExPolygon],
    unsupported: &[ExPolygon],
    bridgeable: &[ExPolygon],
    spacing: i64,
    width: i64,
    anchor: f64,
) -> Result<Vec<ExPolygon>, ClipperError> {
    let mut filtered = Vec::new();
    for polygon in bridgeable {
        append_simplified_expolygon(polygon.clone(), spacing as f64, &mut filtered)?;
    }
    filtered = opening_ex(&filtered, width as f32, JoinType::Miter, 3.0)?;
    filtered = union_expolygons(
        &offset_expolygons(last, spacing as f32, JoinType::Miter, 3.0)?,
        &filtered,
    )?;
    filtered = offset_expolygons(&filtered, (-spacing) as f32, JoinType::Miter, 3.0)?;
    filtered = difference_ex_with_safety_offset(&filtered, last)?;
    filtered = opening_ex(&filtered, spacing as f32, JoinType::Miter, 3.0)?;
    filtered = offset_expolygons(&filtered, spacing as f32, JoinType::Miter, 3.0)?;
    filtered = difference_ex(
        &filtered,
        &offset_expolygons(last, (-anchor) as f32, JoinType::Miter, 3.0)?,
    )?;
    let anchors = intersection_ex(
        last,
        &offset_expolygons(unsupported, anchor as f32, JoinType::Miter, 3.0)?,
    )?;
    opening_ex(
        &union_expolygons(&filtered, &anchors)?,
        anchor as f32,
        JoinType::Miter,
        3.0,
    )
}

fn geometry_error(_: ClipperError) -> SliceError {
    SliceError::InvalidInput("invalid classic counterbore bridging geometry".to_owned())
}
