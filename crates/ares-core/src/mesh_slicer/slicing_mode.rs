use crate::geometry::Polygon;

use super::LoopedLayer;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SlicingMode {
    Regular,
    EvenOdd,
    Positive,
    PositiveLargestContour,
}

pub(crate) fn apply_slicing_mode(layer: &mut LoopedLayer, mode: SlicingMode) {
    match mode {
        SlicingMode::Regular | SlicingMode::EvenOdd => {}
        SlicingMode::Positive => make_positive(layer),
        SlicingMode::PositiveLargestContour => keep_positive_largest(layer),
    }
}

fn make_positive(layer: &mut LoopedLayer) {
    for polygon in layer.polygons_mut() {
        if signed_area(polygon) < 0.0 {
            polygon.reverse();
        }
    }
}

fn keep_positive_largest(layer: &mut LoopedLayer) {
    let polygons = layer.polygons_mut();
    if polygons.is_empty() {
        return;
    }

    let mut selected = None;
    let mut selected_area = 0.0_f64;
    for (index, polygon) in polygons.iter().enumerate() {
        let area = signed_area(polygon);
        if area.abs() > selected_area.abs() {
            selected = Some(index);
            selected_area = area;
        }
    }

    let mut polygon = polygons
        .swap_remove(selected.expect("positive-largest contour requires a nonzero-area polygon"));
    polygons.clear();
    if selected_area < 0.0 {
        polygon.reverse();
    }
    polygons.push(polygon);
}

fn signed_area(polygon: &Polygon) -> f64 {
    let points = polygon.points();
    let mut area = 0.0;
    for index in 0..points.len() {
        let previous = if index == 0 {
            points.len() - 1
        } else {
            index - 1
        };
        area += points[previous].x() as f64 * points[index].y() as f64
            - points[previous].y() as f64 * points[index].x() as f64;
    }
    0.5 * area
}
