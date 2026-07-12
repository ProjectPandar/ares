use super::{LayerPrintPaths, PrintPath, PrintPathRole, support_rectangle};
use crate::{LayerContours, options::support_threshold::SupportThresholdOptions};

pub(crate) fn apply_support_threshold_contacts(
    mut layers: Vec<LayerPrintPaths>,
    layer_contours: &[LayerContours],
    enabled: bool,
    threshold: SupportThresholdOptions,
    external_perimeter_width_mm: f64,
) -> Vec<LayerPrintPaths> {
    if !enabled {
        return layers;
    }

    for current_index in 1..layer_contours.len() {
        let previous_index = current_index - 1;
        let previous_bounds = rectangular_contours(&layer_contours[previous_index]);
        if previous_bounds.is_empty() {
            continue;
        }
        let current_bounds = rectangular_contours(&layer_contours[current_index]);
        if current_bounds.is_empty() {
            continue;
        }

        let offset = threshold_offset(
            threshold,
            previous_layer_height(layer_contours, previous_index),
            external_perimeter_width_mm,
        );
        let expanded_previous = previous_bounds
            .iter()
            .map(|bounds| inflate(*bounds, offset))
            .collect::<Vec<_>>();
        let contacts = current_bounds
            .into_iter()
            .flat_map(|current| {
                contact_pieces(current, &previous_bounds, &expanded_previous, offset)
            })
            .map(contact_path)
            .collect::<Vec<_>>();
        if contacts.is_empty() {
            continue;
        }
        if let Some(layer) = layers
            .iter_mut()
            .find(|layer| layer.layer_id() == layer_contours[previous_index].layer_id())
        {
            let mut paths = layer.paths().to_vec();
            paths.extend(contacts);
            *layer = LayerPrintPaths::new(layer.layer_id(), layer.print_z(), paths);
        }
    }

    layers
}

fn rectangular_contours(layer: &LayerContours) -> Vec<support_rectangle::RectangleBounds> {
    layer
        .contours()
        .iter()
        .filter_map(|contour| support_rectangle::rectangle_bounds(contour.points()))
        .collect()
}

fn previous_layer_height(layer_contours: &[LayerContours], previous_index: usize) -> f64 {
    if previous_index > 0 {
        layer_contours[previous_index].print_z() - layer_contours[previous_index - 1].print_z()
    } else {
        layer_contours[previous_index].print_z()
    }
}

fn threshold_offset(
    threshold: SupportThresholdOptions,
    previous_lower_layer_height_mm: f64,
    external_perimeter_width_mm: f64,
) -> f64 {
    let angle_degrees = threshold.angle_degrees();
    if angle_degrees == 0 {
        return external_perimeter_width_mm
            - threshold.overlap().abs_value(external_perimeter_width_mm);
    }

    let angle = (angle_degrees + 1).min(89) as f64;
    previous_lower_layer_height_mm / angle.to_radians().tan()
}

fn contact_pieces(
    current: support_rectangle::RectangleBounds,
    previous_bounds: &[support_rectangle::RectangleBounds],
    expanded_previous: &[support_rectangle::RectangleBounds],
    offset: f64,
) -> Vec<support_rectangle::RectangleBounds> {
    let mut detected = vec![current];
    for previous in expanded_previous {
        detected = detected
            .into_iter()
            .flat_map(|piece| subtract_rect(piece, *previous))
            .collect();
    }

    let mut contacts = Vec::new();
    for detected_piece in detected {
        let Some(restored) = intersect_rect(inflate(detected_piece, offset), current) else {
            continue;
        };
        let mut pieces = vec![restored];
        for previous in previous_bounds {
            pieces = pieces
                .into_iter()
                .flat_map(|piece| subtract_rect(piece, *previous))
                .collect();
        }
        for piece in pieces {
            let disjoint = contacts.iter().fold(vec![piece], |pieces, contact| {
                pieces
                    .into_iter()
                    .flat_map(|piece| subtract_rect(piece, *contact))
                    .collect()
            });
            contacts.extend(disjoint);
        }
    }
    contacts
}

fn contact_path(piece: support_rectangle::RectangleBounds) -> PrintPath {
    PrintPath::new(
        PrintPathRole::SupportMaterialInterface,
        support_rectangle::rectangle_points(piece),
    )
    .expect("rectangle points are non-empty")
    .with_closed(true)
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

fn intersect_rect(
    left: support_rectangle::RectangleBounds,
    right: support_rectangle::RectangleBounds,
) -> Option<support_rectangle::RectangleBounds> {
    rect(
        left.min_x.max(right.min_x),
        left.min_y.max(right.min_y),
        left.max_x.min(right.max_x),
        left.max_y.min(right.max_y),
    )
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
