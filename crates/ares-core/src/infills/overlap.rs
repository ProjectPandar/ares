use crate::{
    InfillOptions, LayerContours, Point2,
    options::{InfillLayerRole, InfillWallBoundaryOptions, InfillWallOverlapOptions},
};

pub(super) fn adjusted_contours(
    layer: &LayerContours,
    role: InfillLayerRole,
    layer_index: usize,
    layer_count: usize,
    options: &InfillOptions,
) -> Vec<Vec<Point2>> {
    let [contour] = layer.contours() else {
        return layer
            .contours()
            .iter()
            .map(|contour| contour.points().to_vec())
            .collect();
    };
    let points = contour.points();
    match adjusted_rectangle(
        points,
        role,
        LayerPosition {
            id: layer.layer_id(),
            index: layer_index,
            count: layer_count,
        },
        options,
    ) {
        RectangleAdjustment::Adjusted(points) => vec![points],
        RectangleAdjustment::Unsupported => vec![points.to_vec()],
        RectangleAdjustment::Collapsed => Vec::new(),
    }
}

enum RectangleAdjustment {
    Unsupported,
    Collapsed,
    Adjusted(Vec<Point2>),
}

#[derive(Clone, Copy)]
struct LayerPosition {
    id: usize,
    index: usize,
    count: usize,
}

fn adjusted_rectangle(
    points: &[Point2],
    role: InfillLayerRole,
    layer: LayerPosition,
    options: &InfillOptions,
) -> RectangleAdjustment {
    let Some((min_x, min_y, max_x, max_y)) = bounds(points) else {
        return RectangleAdjustment::Unsupported;
    };
    let boundary = options.wall_boundary();
    let overlap = options.wall_overlap();
    let wall_loops = effective_wall_loops(boundary, options, layer.id, layer.index, layer.count);
    if wall_loops == 0 {
        return RectangleAdjustment::Adjusted(points.to_vec());
    }
    let base_inset = fill_boundary_inset(wall_loops, boundary);
    let overlap_reference = overlap_reference(wall_loops, boundary, options.solid_line_width());
    let percent = selected_overlap_percent(role, layer.index, layer.count, overlap);
    let inset = (base_inset - percent / 100.0 * overlap_reference).max(0.0);
    let inner_min_x = min_x + inset;
    let inner_min_y = min_y + inset;
    let inner_max_x = max_x - inset;
    let inner_max_y = max_y - inset;
    if inner_min_x >= inner_max_x || inner_min_y >= inner_max_y {
        return RectangleAdjustment::Collapsed;
    }
    RectangleAdjustment::Adjusted(vec![
        Point2::new(inner_min_x, inner_min_y),
        Point2::new(inner_max_x, inner_min_y),
        Point2::new(inner_max_x, inner_max_y),
        Point2::new(inner_min_x, inner_max_y),
    ])
}

fn bounds(points: &[Point2]) -> Option<(f64, f64, f64, f64)> {
    if points.len() != 4 {
        return None;
    }
    let min_x = points
        .iter()
        .map(|point| point.x())
        .fold(f64::INFINITY, f64::min);
    let min_y = points
        .iter()
        .map(|point| point.y())
        .fold(f64::INFINITY, f64::min);
    let max_x = points
        .iter()
        .map(|point| point.x())
        .fold(f64::NEG_INFINITY, f64::max);
    let max_y = points
        .iter()
        .map(|point| point.y())
        .fold(f64::NEG_INFINITY, f64::max);
    points
        .iter()
        .all(|point| {
            (point.x() == min_x || point.x() == max_x) && (point.y() == min_y || point.y() == max_y)
        })
        .then_some((min_x, min_y, max_x, max_y))
}

fn effective_wall_loops(
    boundary: InfillWallBoundaryOptions,
    options: &InfillOptions,
    layer_id: usize,
    layer_index: usize,
    layer_count: usize,
) -> u32 {
    let mut wall_loops = boundary.wall_loops();
    if wall_loops == 0 {
        return 0;
    }
    if layer_id == 0 && boundary.only_one_wall_first_layer() {
        return 1;
    }
    if boundary.alternate_extra_wall()
        && layer_id % 2 == 1
        && options.sparse_density_percent() > 0.0
    {
        wall_loops += 1;
    }
    if layer_index + 1 == layer_count && boundary.only_one_wall_top() && wall_loops > 1 {
        1
    } else {
        wall_loops
    }
}

fn fill_boundary_inset(wall_loops: u32, boundary: InfillWallBoundaryOptions) -> f64 {
    if wall_loops == 1 {
        boundary.external_wall_line_width() / 2.0
    } else {
        (boundary.external_wall_line_width() + boundary.internal_wall_line_width()) / 2.0
            + f64::from(wall_loops - 2) * boundary.internal_wall_line_width()
            + boundary.internal_wall_line_width() / 2.0
    }
}

fn overlap_reference(
    wall_loops: u32,
    boundary: InfillWallBoundaryOptions,
    solid_line_width: f64,
) -> f64 {
    let wall_half_width = if wall_loops == 1 {
        boundary.external_wall_line_width()
    } else {
        boundary.internal_wall_line_width()
    } / 2.0;
    wall_half_width + solid_line_width / 2.0
}

fn selected_overlap_percent(
    role: InfillLayerRole,
    layer_index: usize,
    layer_count: usize,
    overlap: InfillWallOverlapOptions,
) -> f64 {
    if layer_index == 0
        || layer_index + 1 == layer_count
        || matches!(
            role,
            InfillLayerRole::BottomSurface | InfillLayerRole::TopSurface
        )
    {
        overlap.top_bottom_percent()
    } else {
        overlap.infill_percent()
    }
}
