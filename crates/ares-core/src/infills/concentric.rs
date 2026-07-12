use crate::{
    InfillOptions, InfillPath, InfillPattern, InfillRole, LayerInfills, Point2, SliceError,
    options::InfillLayerRole,
};

pub(super) struct LayerInput<'a> {
    pub(super) layer_id: usize,
    pub(super) print_z: f64,
    pub(super) role: InfillLayerRole,
    pub(super) path_role: InfillRole,
    pub(super) contours: &'a [&'a [Point2]],
    pub(super) spacing: f64,
    pub(super) effective_layer_height_mm: f64,
}

pub(super) fn try_layer(
    input: LayerInput<'_>,
    options: &InfillOptions,
) -> Result<Option<LayerInfills>, SliceError> {
    if input.role.pattern(options) != InfillPattern::Concentric {
        return Ok(None);
    }
    Ok(Some(LayerInfills::new(
        input.layer_id,
        input.print_z,
        rectangle_segments(
            input.contours,
            input.spacing,
            input.path_role,
            options.solid_line_width(),
            input.effective_layer_height_mm,
        )?,
    )))
}

fn rectangle_segments(
    contours: &[&[Point2]],
    spacing: f64,
    path_role: InfillRole,
    solid_line_width: f64,
    effective_layer_height_mm: f64,
) -> Result<Vec<InfillPath>, SliceError> {
    let (min_x, min_y, max_x, max_y) = rectangle_bounds(contours).ok_or_else(|| {
        SliceError::InvalidInput(
            "concentric infill currently supports one axis-aligned rectangle only".to_owned(),
        )
    })?;
    let mut inset = solid_line_width / 2.0;
    let mut paths = Vec::new();
    while min_x + inset < max_x - inset && min_y + inset < max_y - inset {
        let left = min_x + inset;
        let right = max_x - inset;
        let bottom = min_y + inset;
        let top = max_y - inset;
        paths.push(segment(
            Point2::new(left, bottom),
            Point2::new(right, bottom),
            path_role,
            effective_layer_height_mm,
        )?);
        paths.push(segment(
            Point2::new(right, bottom),
            Point2::new(right, top),
            path_role,
            effective_layer_height_mm,
        )?);
        paths.push(segment(
            Point2::new(right, top),
            Point2::new(left, top),
            path_role,
            effective_layer_height_mm,
        )?);
        paths.push(segment(
            Point2::new(left, top),
            Point2::new(left, bottom),
            path_role,
            effective_layer_height_mm,
        )?);
        inset += spacing;
    }
    Ok(paths)
}

fn rectangle_bounds(contours: &[&[Point2]]) -> Option<(f64, f64, f64, f64)> {
    let [contour] = contours else {
        return None;
    };
    if contour.len() != 4 {
        return None;
    }
    let min_x = contour.iter().map(Point2::x).min_by(f64::total_cmp)?;
    let max_x = contour.iter().map(Point2::x).max_by(f64::total_cmp)?;
    let min_y = contour.iter().map(Point2::y).min_by(f64::total_cmp)?;
    let max_y = contour.iter().map(Point2::y).max_by(f64::total_cmp)?;
    let mut actual = contour.to_vec();
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

fn segment(
    start: Point2,
    end: Point2,
    role: InfillRole,
    effective_layer_height_mm: f64,
) -> Result<InfillPath, SliceError> {
    InfillPath::new(role, vec![start, end], effective_layer_height_mm)
}

fn compare_points(a: &Point2, b: &Point2) -> std::cmp::Ordering {
    a.x()
        .total_cmp(&b.x())
        .then_with(|| a.y().total_cmp(&b.y()))
}
