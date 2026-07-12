use super::{LayerPrintPaths, PrintPath, PrintPathRole, support_rectangle};
use crate::LayerContours;

pub(crate) fn apply_support_object_xy_distance(
    layers: Vec<LayerPrintPaths>,
    layer_contours: &[LayerContours],
    object_xy_distance_mm: f64,
    object_first_layer_gap_mm: f64,
    raft_layers: u32,
) -> Vec<LayerPrintPaths> {
    layers
        .into_iter()
        .map(|layer| {
            if layer.layer_id() < raft_layers as usize {
                return layer;
            }
            let object_bounds = object_bounds_for_layer(layer.layer_id(), layer_contours);
            if object_bounds.is_empty() {
                return layer;
            }
            let object_distance_mm = if layer.layer_id() == 0 {
                object_first_layer_gap_mm
            } else {
                object_xy_distance_mm
            };
            clip_layer(layer, &object_bounds, object_distance_mm)
        })
        .collect()
}

fn object_bounds_for_layer(
    layer_id: usize,
    layer_contours: &[LayerContours],
) -> Vec<support_rectangle::RectangleBounds> {
    layer_contours
        .iter()
        .find(|contours| contours.layer_id() == layer_id)
        .map(|contours| {
            contours
                .contours()
                .iter()
                .filter_map(|contour| support_rectangle::rectangle_bounds(contour.points()))
                .collect()
        })
        .unwrap_or_default()
}

fn clip_layer(
    layer: LayerPrintPaths,
    object_bounds: &[support_rectangle::RectangleBounds],
    object_xy_distance_mm: f64,
) -> LayerPrintPaths {
    let paths = layer
        .paths()
        .iter()
        .flat_map(|path| clip_path(path, object_bounds, object_xy_distance_mm))
        .collect();
    LayerPrintPaths::new(layer.layer_id(), layer.print_z(), paths)
}

fn clip_path(
    path: &PrintPath,
    object_bounds: &[support_rectangle::RectangleBounds],
    object_xy_distance_mm: f64,
) -> Vec<PrintPath> {
    if !path.is_closed() || !support_path(path) {
        return vec![path.clone()];
    }
    let Some(bounds) = support_rectangle::rectangle_bounds(path.points()) else {
        return vec![path.clone()];
    };

    let mut pieces = vec![bounds];
    for object in object_bounds {
        let inflated = inflate(*object, object_xy_distance_mm);
        pieces = pieces
            .into_iter()
            .flat_map(|piece| subtract_rect(piece, inflated))
            .collect();
    }

    pieces
        .into_iter()
        .map(|piece| {
            support_rectangle::rebuild_path(
                path,
                path.role(),
                support_rectangle::rectangle_points(piece),
                true,
            )
        })
        .collect()
}

fn support_path(path: &PrintPath) -> bool {
    matches!(
        path.role(),
        PrintPathRole::SupportMaterial | PrintPathRole::SupportMaterialInterface
    )
}

fn inflate(
    bounds: support_rectangle::RectangleBounds,
    distance: f64,
) -> support_rectangle::RectangleBounds {
    support_rectangle::RectangleBounds {
        min_x: bounds.min_x - distance,
        min_y: bounds.min_y - distance,
        max_x: bounds.max_x + distance,
        max_y: bounds.max_y + distance,
    }
}

fn subtract_rect(
    source: support_rectangle::RectangleBounds,
    cutter: support_rectangle::RectangleBounds,
) -> Vec<support_rectangle::RectangleBounds> {
    let min_x = source.min_x.max(cutter.min_x);
    let min_y = source.min_y.max(cutter.min_y);
    let max_x = source.max_x.min(cutter.max_x);
    let max_y = source.max_y.min(cutter.max_y);
    if max_x - min_x <= support_rectangle::EPSILON || max_y - min_y <= support_rectangle::EPSILON {
        return vec![source];
    }

    [
        rect(source.min_x, source.min_y, source.max_x, min_y),
        rect(source.min_x, max_y, source.max_x, source.max_y),
        rect(source.min_x, min_y, min_x, max_y),
        rect(max_x, min_y, source.max_x, max_y),
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn rect(
    min_x: f64,
    min_y: f64,
    max_x: f64,
    max_y: f64,
) -> Option<support_rectangle::RectangleBounds> {
    (max_x - min_x > support_rectangle::EPSILON && max_y - min_y > support_rectangle::EPSILON)
        .then_some(support_rectangle::RectangleBounds {
            min_x,
            min_y,
            max_x,
            max_y,
        })
}
