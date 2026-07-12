use crate::{BrimOptions, BrimPath, LayerContours, Point2, SliceError};

pub(super) type Bounds = (f64, f64, f64, f64);

pub(super) fn outer_brim_paths(
    layer: &LayerContours,
    options: BrimOptions,
    effective_line_width: f64,
    loop_count: u32,
) -> Result<Vec<BrimPath>, SliceError> {
    let Some(bounds) = layer
        .contours()
        .iter()
        .filter(|contour| super::is_outer_contour(layer, contour))
        .filter_map(super::contour_bounds)
        .filter_map(|bounds| {
            super::efc_outline::bounds_or_adjusted(bounds, options.efc_outline_offset_mm())
        })
        .reduce(merge_bounds)
    else {
        return Ok(Vec::new());
    };

    outer_brim_bounds_paths(bounds, options, effective_line_width, loop_count)
}

pub(super) fn outer_brim_bounds_paths(
    bounds: Bounds,
    options: BrimOptions,
    effective_line_width: f64,
    loop_count: u32,
) -> Result<Vec<BrimPath>, SliceError> {
    let (min_x, min_y, max_x, max_y) = bounds;
    (0..loop_count)
        .map(|loop_index| {
            let brim_offset =
                (f64::from(loop_index + 1) * effective_line_width).min(options.width_mm());
            let expand = options.object_gap_mm() + brim_offset;
            BrimPath::new(vec![
                Point2::new(min_x - expand, min_y - expand),
                Point2::new(max_x + expand, min_y - expand),
                Point2::new(max_x + expand, max_y + expand),
                Point2::new(min_x - expand, max_y + expand),
            ])
        })
        .collect()
}

fn merge_bounds(a: Bounds, b: Bounds) -> Bounds {
    (a.0.min(b.0), a.1.min(b.1), a.2.max(b.2), a.3.max(b.3))
}
