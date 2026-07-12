use crate::{LayerContours, LayerPrintPaths, Point2, PrintPath, PrintPathRole, SliceOptions};
use serde_json::{Value, json};

pub(crate) fn finalize(
    layers: Vec<LayerPrintPaths>,
    extra: Value,
    contours: Vec<LayerContours>,
) -> Vec<LayerPrintPaths> {
    crate::finalize_print_paths_with_layer_contours(layers, &options(extra), &contours).unwrap()
}

pub(crate) fn layer(layer_id: usize, path: PrintPath) -> LayerPrintPaths {
    layer_paths(layer_id, vec![path])
}

pub(crate) fn layer_paths(layer_id: usize, paths: Vec<PrintPath>) -> LayerPrintPaths {
    LayerPrintPaths::new(layer_id, 0.2 * (layer_id + 1) as f64, paths)
}

pub(crate) fn layer_by_id(layers: &[LayerPrintPaths], layer_id: usize) -> &LayerPrintPaths {
    layers
        .iter()
        .find(|layer| layer.layer_id() == layer_id)
        .unwrap()
}

pub(crate) fn support_rect(
    role: PrintPathRole,
    min_x: f64,
    min_y: f64,
    max_x: f64,
    max_y: f64,
) -> PrintPath {
    PrintPath::new(role, rect_points(min_x, min_y, max_x, max_y).to_vec())
        .unwrap()
        .with_closed(true)
}

pub(crate) fn assert_support_material_bounds(
    layer: &LayerPrintPaths,
    min_x: f64,
    min_y: f64,
    max_x: f64,
    max_y: f64,
) {
    assert_eq!(
        support_material_bounds(layer),
        Some((min_x, min_y, max_x, max_y))
    );
}

pub(crate) fn assert_support_material_metadata(path: &PrintPath) {
    assert_eq!(path.role(), PrintPathRole::SupportMaterial);
    assert_eq!(path.extrusion_role(), Some(PrintPathRole::SupportMaterial));
    assert_eq!(path.effective_layer_height_mm(), Some(0.16));
    assert_eq!(path.effective_line_width_mm(), Some(0.44));
    assert_eq!(path.unsupported_span_mm(), Some(1.2));
    assert_eq!(path.seam_gap_mm(), 0.08);
}

pub(crate) fn contains_exact_path(layer: &LayerPrintPaths, expected: &PrintPath) -> bool {
    layer.paths().iter().any(|path| path == expected)
}

pub(crate) fn empty_contours(count: usize) -> Vec<LayerContours> {
    (0..count)
        .map(|layer_id| LayerContours::new(layer_id, 0.2 * (layer_id + 1) as f64, Vec::new()))
        .collect()
}

pub(crate) fn options(extra: Value) -> SliceOptions {
    let mut value = json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "line_width": 0.4,
        "top_surface_line_width": 0.4,
        "enable_support": true,
        "support_base_pattern_spacing": 0.1,
        "raft_first_layer_density": 80.0,
        "filament_max_volumetric_speed": 0.0,
        "slow_down_for_layer_cooling": false
    });
    for (key, value_extra) in extra.as_object().expect("test options must be an object") {
        value[key] = value_extra.clone();
    }
    serde_json::from_value(value).unwrap()
}

fn support_material_bounds(layer: &LayerPrintPaths) -> Option<(f64, f64, f64, f64)> {
    let mut points = layer
        .paths()
        .iter()
        .filter(|path| path.role() == PrintPathRole::SupportMaterial)
        .flat_map(|path| path.points().iter());
    let first = points.next()?;
    let mut min_x = first.x();
    let mut min_y = first.y();
    let mut max_x = first.x();
    let mut max_y = first.y();
    for point in points {
        min_x = min_x.min(point.x());
        min_y = min_y.min(point.y());
        max_x = max_x.max(point.x());
        max_y = max_y.max(point.y());
    }
    Some((min_x, min_y, max_x, max_y))
}

fn rect_points(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> [Point2; 4] {
    [
        Point2::new(min_x, min_y),
        Point2::new(max_x, min_y),
        Point2::new(max_x, max_y),
        Point2::new(min_x, max_y),
    ]
}
