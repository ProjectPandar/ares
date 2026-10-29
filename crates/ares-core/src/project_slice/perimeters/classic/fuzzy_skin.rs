// OrcaSlicer v2.4.2 `Feature/FuzzySkin/FuzzySkin.cpp::apply_fuzzy_skin`
// classic-polygon seam after surface simplification and before overhang clipping.

#[cfg(test)]
mod tests;

use crate::{
    SliceError,
    geometry::{CoordinateScale, Polygon},
    perimeters::FuzzySkinConfig,
};

use super::super::arachne::fuzzy::{Regions, make_exclusive};
use crate::geometry::{Point, split_line};

#[derive(Clone, Copy)]
pub(super) struct FuzzySkinInput {
    pub(super) layer_id: usize,
    pub(super) slice_z: f64,
    pub(super) loop_index: usize,
    pub(super) is_contour: bool,
    pub(super) scale: CoordinateScale,
}

pub(super) fn apply(
    polygon: &Polygon,
    config: FuzzySkinConfig,
    input: FuzzySkinInput,
    context: &mut crate::SlicerContext,
) -> Result<Polygon, SliceError> {
    if !config.should_fuzzify(input.layer_id, input.loop_index, input.is_contour) {
        return Ok(polygon.clone());
    }

    Ok(Polygon::new(config.fuzzified_scaled_points(
        (polygon.points(), true),
        crate::perimeters::FuzzySkinLayer {
            layer_id: input.layer_id,
            print_z: input.slice_z,
        },
        input.scale,
        context,
    )))
}

// FuzzySkin.cpp561–684: compatible effects, exclusive masks and cyclic LineSplit runs.
pub(super) fn apply_regions(
    polygon: &Polygon,
    regions: &Regions,
    input: FuzzySkinInput,
    context: &mut crate::SlicerContext,
) -> Result<Polygon, SliceError> {
    if regions.entries.len() == 1 {
        return apply(polygon, regions.entries[0].config.base, input, context);
    }
    let mut merged = regions.merged(input.layer_id, input.loop_index, input.is_contour)?;
    if merged.len() == 1 && merged[0].mask.is_empty() {
        return apply(polygon, merged[0].config.base, input, context);
    }
    make_exclusive(&mut merged)?;
    let mut polygon = polygon.clone();
    for region in merged {
        let mut split =
            split_line(polygon.points(), &region.mask, true, input.scale).map_err(|_| {
                SliceError::InvalidInput(
                    "fuzzy region clipping exceeds the coordinate range".to_owned(),
                )
            })?;
        if split.is_empty() {
            continue;
        }
        let config = region.config.base;
        if split.iter().all(|point| point.clipped) {
            polygon = apply(&polygon, config, input, context)?;
            continue;
        }
        let first = split.iter().position(|point| !point.clipped).unwrap();
        split.rotate_left(first);
        let mut output = Vec::with_capacity(split.len());
        let mut segment = Vec::with_capacity(split.len());
        for point in &split {
            if point.clipped {
                segment.push(point.point);
            } else if segment.is_empty() {
                output.push(point.point);
            } else {
                segment.push(point.point);
                append_segment(&mut output, &mut segment, config, input, context);
            }
        }
        if !segment.is_empty() {
            segment.push(split[0].point);
            append_segment(&mut output, &mut segment, config, input, context);
        }
        polygon = Polygon::new(output);
    }
    Ok(polygon)
}

fn append_segment(
    output: &mut Vec<Point>,
    segment: &mut Vec<Point>,
    config: FuzzySkinConfig,
    input: FuzzySkinInput,
    context: &mut crate::SlicerContext,
) {
    output.push(segment[0]);
    let back = *segment.last().unwrap();
    output.extend(config.fuzzified_scaled_points(
        (segment, false),
        crate::perimeters::FuzzySkinLayer {
            layer_id: input.layer_id,
            print_z: input.slice_z,
        },
        input.scale,
        context,
    ));
    output.push(back);
    segment.clear();
}
