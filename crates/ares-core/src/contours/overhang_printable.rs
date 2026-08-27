use crate::{Contour, LayerContours, PerimeterOptions, Point2};

type RectBounds = (f64, f64, f64, f64);

pub(crate) fn apply(
    mut layers: Vec<LayerContours>,
    options: PerimeterOptions,
) -> Vec<LayerContours> {
    if !options.make_overhang_printable()
        || options.make_overhang_printable_angle_degrees() >= 90.0
        || layers.len() < 2
    {
        return layers;
    }

    let shrink = options
        .make_overhang_printable_angle_degrees()
        .to_radians()
        .tan()
        * options.layer_height_mm();
    if !shrink.is_finite() || shrink < 0.0 {
        return layers;
    }

    for lower_index in (0..layers.len() - 1).rev() {
        let Some(lower_rectangles) = rectangular_contours(layers[lower_index].contours()) else {
            continue;
        };
        let Some(upper_rectangles) = rectangular_contours(layers[lower_index + 1].contours())
        else {
            continue;
        };
        let protected_holes = protected_hole_rectangles(
            &lower_rectangles,
            options.make_overhang_printable_hole_size_mm2(),
        );
        let additions = upper_rectangles
            .iter()
            .filter_map(|rectangle| {
                (!covers_protected_hole(*rectangle, &protected_holes))
                    .then(|| projected_rectangle(*rectangle, shrink))
                    .flatten()
            })
            .map(Contour::new)
            .collect::<Vec<_>>();
        if additions.is_empty() {
            continue;
        }
        let mut contours = layers[lower_index].contours().to_vec();
        contours.extend(additions);
        layers[lower_index] = LayerContours::new(
            layers[lower_index].layer_id(),
            layers[lower_index].print_z(),
            contours,
        );
    }

    layers
}

fn rectangular_contours(contours: &[Contour]) -> Option<Vec<RectBounds>> {
    if contours.is_empty() {
        return None;
    }
    contours
        .iter()
        .map(|contour| super::axis_aligned_rectangle_bounds(contour.points()))
        .collect()
}

fn protected_hole_rectangles(rectangles: &[RectBounds], max_area: f64) -> Vec<RectBounds> {
    if max_area <= 0.0 {
        return Vec::new();
    }

    rectangles
        .iter()
        .copied()
        .filter(|candidate| {
            rectangle_area(*candidate) < max_area
                && rectangles.iter().any(|outer| {
                    *outer != *candidate && strictly_contains_rectangle(*outer, *candidate)
                })
        })
        .collect()
}

fn covers_protected_hole(rectangle: RectBounds, protected_holes: &[RectBounds]) -> bool {
    protected_holes
        .iter()
        .any(|hole| contains_rectangle(rectangle, *hole))
}

fn rectangle_area((min_x, min_y, max_x, max_y): RectBounds) -> f64 {
    (max_x - min_x) * (max_y - min_y)
}

fn strictly_contains_rectangle(outer: RectBounds, inner: RectBounds) -> bool {
    outer.0 < inner.0 && inner.2 < outer.2 && outer.1 < inner.1 && inner.3 < outer.3
}

fn contains_rectangle(outer: RectBounds, inner: RectBounds) -> bool {
    outer.0 <= inner.0 && inner.2 <= outer.2 && outer.1 <= inner.1 && inner.3 <= outer.3
}

fn projected_rectangle(bounds: RectBounds, shrink: f64) -> Option<Vec<Point2>> {
    let (min_x, min_y, max_x, max_y) = bounds;
    let min_x = min_x + shrink;
    let min_y = min_y + shrink;
    let max_x = max_x - shrink;
    let max_y = max_y - shrink;
    (min_x < max_x && min_y < max_y).then(|| {
        vec![
            Point2::new(min_x, min_y),
            Point2::new(max_x, min_y),
            Point2::new(max_x, max_y),
            Point2::new(min_x, max_y),
        ]
    })
}
