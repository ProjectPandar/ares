use super::{
    LayerPrintPaths, PrintPath, PrintPathRole,
    support_rectangle::{EPSILON, RectangleBounds, rectangle_bounds, rectangle_points},
};
use crate::{
    Point2,
    options::ironing_flow::{SupportIroningConfig, SupportIroningPattern},
};

pub(crate) fn apply_support_ironing(
    layers: Vec<LayerPrintPaths>,
    enabled: bool,
    config: SupportIroningConfig,
) -> Vec<LayerPrintPaths> {
    if !enabled {
        return layers;
    }

    let mut output = Vec::with_capacity(layers.len());
    let mut previous_print_z = 0.0;
    for layer in layers {
        let layer_height = layer.print_z() - previous_print_z;
        let mut paths = Vec::with_capacity(layer.paths().len() + support_interface_count(&layer));
        for path in layer.paths() {
            paths.push(path.clone());
            if path.role() == PrintPathRole::SupportMaterialInterface {
                paths.extend(duplicates_as_ironing(path, layer_height, config));
            }
        }
        previous_print_z = layer.print_z();
        output.push(LayerPrintPaths::new(
            layer.layer_id(),
            layer.print_z(),
            paths,
        ));
    }
    output
}

fn support_interface_count(layer: &LayerPrintPaths) -> usize {
    layer
        .paths()
        .iter()
        .filter(|path| path.role() == PrintPathRole::SupportMaterialInterface)
        .count()
}

fn duplicates_as_ironing(
    path: &PrintPath,
    layer_height: f64,
    config: SupportIroningConfig,
) -> Vec<PrintPath> {
    let effective_layer_height =
        path.effective_layer_height_mm().unwrap_or(layer_height) * config.flow_ratio();
    support_ironing_geometries(path, config.spacing_mm(), config.pattern())
        .into_iter()
        .map(|(points, closed)| duplicate_path(path, points, closed, effective_layer_height))
        .collect()
}

fn duplicate_path(
    path: &PrintPath,
    points: Vec<Point2>,
    closed: bool,
    effective_layer_height: f64,
) -> PrintPath {
    PrintPath::new(PrintPathRole::Ironing, points)
        .expect("existing print path points are non-empty")
        .with_effective_layer_height_mm(round_6(effective_layer_height))
        .with_extrusion_role(PrintPathRole::SupportMaterialInterface)
        .with_unsupported_span_mm(path.unsupported_span_mm())
        .with_seam_gap_mm(path.seam_gap_mm())
        .with_closed(closed)
}

fn support_ironing_geometries(
    path: &PrintPath,
    spacing_mm: f64,
    pattern: SupportIroningPattern,
) -> Vec<(Vec<Point2>, bool)> {
    if path.is_closed()
        && path.points().len() == 4
        && spacing_mm > 0.0
        && let Some(bounds) = rectangle_bounds(path.points())
    {
        return match pattern {
            SupportIroningPattern::Rectilinear => rectilinear_ironing_lines(bounds, spacing_mm)
                .into_iter()
                .map(|points| (points, false))
                .collect(),
            SupportIroningPattern::Concentric => concentric_ironing_loops(bounds, spacing_mm)
                .into_iter()
                .map(|points| (points, true))
                .collect(),
        };
    }
    vec![(path.points().to_vec(), path.is_closed())]
}

fn rectilinear_ironing_lines(bounds: RectangleBounds, spacing_mm: f64) -> Vec<Vec<Point2>> {
    let mut lines = Vec::new();
    let mut y = bounds.min_y;
    while y <= bounds.max_y + EPSILON {
        lines.push(vec![
            Point2::new(bounds.min_x, y),
            Point2::new(bounds.max_x, y),
        ]);
        let next_y = y + spacing_mm;
        if next_y <= y {
            break;
        }
        y = next_y;
    }
    lines
}

fn concentric_ironing_loops(bounds: RectangleBounds, spacing_mm: f64) -> Vec<Vec<Point2>> {
    let mut loops = Vec::new();
    let mut inset = 0.0;
    loop {
        let min_x = bounds.min_x + inset;
        let min_y = bounds.min_y + inset;
        let max_x = bounds.max_x - inset;
        let max_y = bounds.max_y - inset;
        if max_x - min_x <= EPSILON || max_y - min_y <= EPSILON {
            break;
        }
        loops.push(rectangle_points(RectangleBounds {
            min_x,
            min_y,
            max_x,
            max_y,
        }));
        let next_inset = inset + spacing_mm;
        if next_inset <= inset {
            break;
        }
        inset = next_inset;
    }
    loops
}

fn round_6(value: f64) -> f64 {
    (value * 1_000_000.0).round() / 1_000_000.0
}
