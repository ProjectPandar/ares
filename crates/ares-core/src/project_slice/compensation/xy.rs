use crate::geometry::{
    ClipperError, ExPolygon, FillRule, JoinType, Polygon, difference_ex, raw_offset_paths,
    union_ex, union_expolygons,
};

pub(super) fn apply(
    source: &[ExPolygon],
    contour_delta: f32,
    hole_delta: f32,
) -> Result<Vec<ExPolygon>, ClipperError> {
    let mut current = source.to_vec();
    if contour_delta > 0.0 || hole_delta > 0.0 {
        current = shrink_contour_holes(contour_delta.max(0.0), hole_delta.max(0.0), &current)?;
    }
    if contour_delta < 0.0 || hole_delta < 0.0 {
        current = shrink_contour_holes(contour_delta.min(0.0), hole_delta.min(0.0), &current)?;
    }
    Ok(current)
}

fn shrink_contour_holes(
    contour_delta: f32,
    hole_delta: f32,
    source: &[ExPolygon],
) -> Result<Vec<ExPolygon>, ClipperError> {
    let mut output = Vec::new();
    for expolygon in source {
        let contours = offset_or_original(expolygon.contour(), contour_delta)?;
        if contours.is_empty() {
            continue;
        }
        let holes = expolygon
            .holes()
            .iter()
            .map(|hole| offset_or_original(hole, -hole_delta))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();
        let contours = union_ex(&contours, FillRule::NonZero)?;
        let holes = union_ex(&holes, FillRule::NonZero)?;
        let adjusted = difference_ex(&contours, &holes)?;
        output = union_expolygons(&output, &adjusted)?;
    }
    Ok(output)
}

fn offset_or_original(polygon: &Polygon, delta: f32) -> Result<Vec<Polygon>, ClipperError> {
    if delta == 0.0 {
        Ok(vec![polygon.clone()])
    } else {
        raw_offset_paths(std::slice::from_ref(polygon), delta, JoinType::Miter, 3.0)
    }
}

#[cfg(test)]
mod tests;
