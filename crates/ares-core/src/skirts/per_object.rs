use crate::{Contour, LayerContours, SliceError};

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
        .filter(|contour| layer.is_outer_contour(contour))
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
