use crate::{Contour, LayerContours, Point2, SliceError};

use super::{SkirtOptions, SkirtPath, generate_bounds_skirt_paths, min_length};

pub(super) fn generate_layer_skirts(
    layer: &LayerContours,
    options: SkirtOptions,
    effective_line_width: f64,
    skirt_extrusion_per_mm: f64,
    apply_min_length: bool,
) -> Result<Vec<SkirtPath>, SliceError> {
    let mut paths = Vec::new();

    for contour in layer
        .contours()
        .iter()
        .filter(|contour| is_outer_contour(layer, contour))
    {
        let Some(bounds) = contour_bounds(contour) else {
            continue;
        };
        let mut contour_paths = generate_bounds_skirt_paths(
            bounds,
            options,
            effective_line_width,
            skirt_extrusion_per_mm,
            apply_min_length && paths.is_empty(),
        )?;
        paths.append(&mut contour_paths);
    }

    Ok(paths)
}

fn is_outer_contour(layer: &LayerContours, contour: &Contour) -> bool {
    let Some(reference) = contour.points().first() else {
        return false;
    };
    layer
        .contours()
        .iter()
        .filter(|candidate| !std::ptr::eq(*candidate, contour))
        .filter(|candidate| point_in_contour(reference, candidate.points()))
        .count()
        % 2
        == 0
}

fn point_in_contour(point: &Point2, contour: &[Point2]) -> bool {
    if contour.len() < 3 {
        return false;
    }

    let mut inside = false;
    let mut previous = contour[contour.len() - 1];
    for &current in contour {
        if (current.y() > point.y()) != (previous.y() > point.y()) {
            let x_intersection = (previous.x() - current.x()) * (point.y() - current.y())
                / (previous.y() - current.y())
                + current.x();
            if point.x() < x_intersection {
                inside = !inside;
            }
        }
        previous = current;
    }
    inside
}

fn contour_bounds(contour: &Contour) -> Option<min_length::Bounds> {
    let first = contour.points().first()?;
    let mut bounds = min_length::Bounds {
        min_x: first.x(),
        min_y: first.y(),
        max_x: first.x(),
        max_y: first.y(),
    };

    for point in &contour.points()[1..] {
        bounds.min_x = bounds.min_x.min(point.x());
        bounds.min_y = bounds.min_y.min(point.y());
        bounds.max_x = bounds.max_x.max(point.x());
        bounds.max_y = bounds.max_y.max(point.y());
    }

    Some(bounds)
}
