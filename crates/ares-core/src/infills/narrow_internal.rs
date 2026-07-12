use crate::{InfillOptions, InfillPath, InfillPattern, InfillRole, Point2, SliceError};

use crate::options::InfillLayerRole;

pub(super) fn concentric_internal_override(
    role: InfillLayerRole,
    contours: &[&[Point2]],
    options: &InfillOptions,
) -> Option<(InfillPattern, (f64, f64, f64, f64))> {
    if role != InfillLayerRole::InternalSolid || !options.detect_narrow_internal_solid_infill() {
        return None;
    }
    all_narrow_rectangle_bounds(contours, options.solid_line_width())
        .map(|bounds| (InfillPattern::ConcentricInternal, bounds))
}

pub(super) fn concentric_internal_segments(
    bounds: (f64, f64, f64, f64),
    solid_line_width: f64,
    effective_layer_height_mm: f64,
) -> Result<Vec<InfillPath>, SliceError> {
    let (min_x, min_y, max_x, max_y) = bounds;
    let mut inset = solid_line_width / 2.0;
    let mut paths = Vec::new();
    while min_x + inset < max_x - inset && min_y + inset < max_y - inset {
        let left = min_x + inset;
        let right = max_x - inset;
        let bottom = min_y + inset;
        let top = max_y - inset;
        paths.push(segment(
            left,
            bottom,
            right,
            bottom,
            effective_layer_height_mm,
        )?);
        paths.push(segment(
            right,
            bottom,
            right,
            top,
            effective_layer_height_mm,
        )?);
        paths.push(segment(right, top, left, top, effective_layer_height_mm)?);
        paths.push(segment(left, top, left, bottom, effective_layer_height_mm)?);
        inset += solid_line_width;
    }
    Ok(paths)
}

fn all_narrow_rectangle_bounds(
    contours: &[&[Point2]],
    solid_line_width: f64,
) -> Option<(f64, f64, f64, f64)> {
    let [contour] = contours else {
        return None;
    };
    let (min_x, min_y, max_x, max_y) = rectangle_bounds(contour)?;
    let width = max_x - min_x;
    let height = max_y - min_y;
    (width.min(height) <= 2.0 * solid_line_width).then_some((min_x, min_y, max_x, max_y))
}

fn segment(
    start_x: f64,
    start_y: f64,
    end_x: f64,
    end_y: f64,
    effective_layer_height_mm: f64,
) -> Result<InfillPath, SliceError> {
    InfillPath::new(
        InfillRole::Solid,
        vec![Point2::new(start_x, start_y), Point2::new(end_x, end_y)],
        effective_layer_height_mm,
    )
}

fn rectangle_bounds(points: &[Point2]) -> Option<(f64, f64, f64, f64)> {
    let [_, _, _, _] = points else {
        return None;
    };
    let min_x = points.iter().map(Point2::x).min_by(f64::total_cmp)?;
    let max_x = points.iter().map(Point2::x).max_by(f64::total_cmp)?;
    let min_y = points.iter().map(Point2::y).min_by(f64::total_cmp)?;
    let max_y = points.iter().map(Point2::y).max_by(f64::total_cmp)?;
    let mut actual = points.to_vec();
    actual.sort_by(compare_points);
    let mut expected = vec![
        Point2::new(min_x, min_y),
        Point2::new(max_x, min_y),
        Point2::new(max_x, max_y),
        Point2::new(min_x, max_y),
    ];
    expected.sort_by(compare_points);
    (actual == expected).then_some((min_x, min_y, max_x, max_y))
}

fn compare_points(a: &Point2, b: &Point2) -> std::cmp::Ordering {
    a.x()
        .total_cmp(&b.x())
        .then_with(|| a.y().total_cmp(&b.y()))
}
