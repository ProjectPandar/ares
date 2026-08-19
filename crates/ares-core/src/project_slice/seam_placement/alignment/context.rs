use crate::geometry::{CoordinateScale, ExPolygon, Point, Polygon};

use super::LayerPlan;

pub(super) fn populate(
    layers: &mut [LayerPlan],
    compute_embedding: &[bool],
    layer_slices: &[Vec<ExPolygon>],
    scale: CoordinateScale,
) {
    assert_eq!(layers.len(), compute_embedding.len());
    assert_eq!(layers.len(), layer_slices.len());
    for layer_index in 0..layers.len() {
        if compute_embedding[layer_index] {
            populate_embedded_distance(&mut layers[layer_index]);
        }
        if layer_index > 0 {
            let layer_height = layers[layer_index].z - layers[layer_index - 1].z;
            let (_, current) = layers.split_at_mut(layer_index);
            populate_overhang(
                &layer_slices[layer_index - 1],
                &mut current[0],
                layer_height,
                scale,
            );
        }
    }
}

fn populate_overhang(
    previous_slices: &[ExPolygon],
    current: &mut LayerPlan,
    layer_height: f32,
    scale: CoordinateScale,
) {
    for index in 0..current.candidates.points.len() {
        let point = position(current, index);
        let flow_width = flow_width(current, index);
        let distance = signed_distance_to_slices(previous_slices, point, scale);
        current.overhangs[index] = (distance + 0.65 * flow_width - layer_height).max(0.0);
    }
}

fn signed_distance_to_slices(
    slices: &[ExPolygon],
    point: (f32, f32),
    scale: CoordinateScale,
) -> f32 {
    let scaled = Point::new(
        scale
            .checked_scale(f64::from(point.0))
            .expect("candidate X is scalable"),
        scale
            .checked_scale(f64::from(point.1))
            .expect("candidate Y is scalable"),
    );
    let inside = slices.iter().any(|slice| {
        slice.contour().contains(&scaled)
            && !slice.holes().iter().any(|hole| hole.contains(&scaled))
    });
    let distance = slices
        .iter()
        .flat_map(|slice| std::iter::once(slice.contour()).chain(slice.holes()))
        .map(|polygon| polygon_distance(polygon, point, scale))
        .reduce(f32::min)
        .unwrap_or(f32::INFINITY);
    if inside { -distance } else { distance }
}

fn polygon_distance(polygon: &Polygon, point: (f32, f32), scale: CoordinateScale) -> f32 {
    let points = polygon.points();
    points
        .iter()
        .zip(points.iter().cycle().skip(1))
        .take(points.len())
        .map(|(start, end)| {
            point_segment_distance(
                point,
                (
                    scale.unscale(start.x()) as f32,
                    scale.unscale(start.y()) as f32,
                ),
                (scale.unscale(end.x()) as f32, scale.unscale(end.y()) as f32),
            )
        })
        .reduce(f32::min)
        .unwrap_or(f32::INFINITY)
}

fn populate_embedded_distance(layer: &mut LayerPlan) {
    if layer.candidates.perimeters.len() <= 1 {
        return;
    }
    for index in 0..layer.candidates.points.len() {
        let own = layer.candidates.points[index].perimeter_index;
        let point = position(layer, index);
        let distance = distance_to_layer(layer, point, Some(own));
        if !distance.is_finite() {
            continue;
        }
        let inside = layer
            .candidates
            .perimeters
            .iter()
            .enumerate()
            .any(|(perimeter_index, _)| {
                perimeter_index != own && point_inside_perimeter(layer, perimeter_index, point)
            });
        layer.embedded_distances[index] =
            if inside { -distance } else { distance } + 0.65 * flow_width(layer, index);
    }
}

fn flow_width(layer: &LayerPlan, candidate_index: usize) -> f32 {
    let perimeter = layer.candidates.points[candidate_index].perimeter_index;
    layer.candidates.perimeters[perimeter].flow_width
}

fn position(layer: &LayerPlan, candidate_index: usize) -> (f32, f32) {
    let position = layer.candidates.points[candidate_index].position;
    (position.x, position.y)
}

fn distance_to_layer(
    layer: &LayerPlan,
    point: (f32, f32),
    excluded_perimeter: Option<usize>,
) -> f32 {
    layer
        .candidates
        .perimeters
        .iter()
        .enumerate()
        .filter(|(index, _)| Some(*index) != excluded_perimeter)
        .flat_map(|(_, perimeter)| {
            let points = &layer.candidates.points[perimeter.start_index..perimeter.end_index];
            points
                .iter()
                .zip(points.iter().cycle().skip(1))
                .map(move |(start, end)| {
                    point_segment_distance(
                        point,
                        (start.position.x, start.position.y),
                        (end.position.x, end.position.y),
                    )
                })
        })
        .reduce(f32::min)
        .unwrap_or(f32::INFINITY)
}

fn point_segment_distance(point: (f32, f32), start: (f32, f32), end: (f32, f32)) -> f32 {
    let edge = (end.0 - start.0, end.1 - start.1);
    let length_squared = edge.0.mul_add(edge.0, edge.1 * edge.1);
    if length_squared == 0.0 {
        return (point.0 - start.0).hypot(point.1 - start.1);
    }
    let projection = (((point.0 - start.0) * edge.0 + (point.1 - start.1) * edge.1)
        / length_squared)
        .clamp(0.0, 1.0);
    let closest = (start.0 + projection * edge.0, start.1 + projection * edge.1);
    (point.0 - closest.0).hypot(point.1 - closest.1)
}

fn point_inside_perimeter(layer: &LayerPlan, perimeter_index: usize, point: (f32, f32)) -> bool {
    let perimeter = &layer.candidates.perimeters[perimeter_index];
    let points = &layer.candidates.points[perimeter.start_index..perimeter.end_index];
    let mut inside = false;
    let mut previous = points.last().expect("a seam perimeter has candidates");
    for current in points {
        let a = (previous.position.x, previous.position.y);
        let b = (current.position.x, current.position.y);
        if (a.1 > point.1) != (b.1 > point.1)
            && point.0 < (b.0 - a.0) * (point.1 - a.1) / (b.1 - a.1) + a.0
        {
            inside = !inside;
        }
        previous = current;
    }
    inside
}

#[cfg(test)]
mod tests {
    use super::point_segment_distance;

    #[test]
    fn task22o129_segment_distance_clamps_to_endpoints() {
        assert_eq!(
            point_segment_distance((0.5, 1.0), (0.0, 0.0), (1.0, 0.0)),
            1.0
        );
        assert_eq!(
            point_segment_distance((2.0, 0.0), (0.0, 0.0), (1.0, 0.0)),
            1.0
        );
    }
}
