use super::{LayerPlan, SeamPerimeter, Vec3};

#[derive(Clone, Copy)]
struct Edge {
    index: usize,
    vector: Vec3,
    length: f32,
}

pub(super) fn apply(layers: &mut [LayerPlan]) {
    for layer in layers {
        for perimeter_index in 0..layer.candidates.perimeters.len() {
            let (seam_index, position) = select(layer, perimeter_index);
            layer.choices[perimeter_index].seam_index = seam_index;
            layer.choices[perimeter_index].final_position = Some(position);
        }
    }
}

fn select(layer: &LayerPlan, perimeter_index: usize) -> (usize, Vec3) {
    let perimeter = &layer.candidates.perimeters[perimeter_index];
    let seed = layer.positions[perimeter.start_index];
    let mut random = (seed.dot(Vec3::new(12.9898, 78.233, 133.3333)).sin() * 43_758.547).abs();
    random -= random as i32 as f32;
    let viable = viable_edges(layer, perimeter);
    let mut picked = viable.iter().map(|edge| edge.length).sum::<f32>() * random;
    for (viable_index, edge) in viable.iter().enumerate() {
        if picked - edge.length <= 0.0 || viable_index + 1 == viable.len() {
            let ratio = if edge.length > 0.0 {
                picked / edge.length
            } else {
                0.0
            };
            return (
                edge.index,
                layer.positions[edge.index] + edge.vector * ratio,
            );
        }
        picked -= edge.length;
    }
    unreachable!("every perimeter contributes a viable edge")
}

fn viable_edges(layer: &LayerPlan, perimeter: &SeamPerimeter) -> Vec<Edge> {
    let mut example = perimeter.start_index;
    let mut viable = Vec::new();
    for index in perimeter.start_index..perimeter.end_index {
        if not_much_worse(layer, index, example) && not_much_worse(layer, example, index) {
            viable.push(edge(layer, perimeter, index));
        } else if !not_much_worse(layer, example, index) {
            example = index;
            viable.clear();
            viable.push(edge(layer, perimeter, index));
        }
    }
    viable
}

fn edge(layer: &LayerPlan, perimeter: &SeamPerimeter, index: usize) -> Edge {
    let next = if index + 1 == perimeter.end_index {
        perimeter.start_index
    } else {
        index + 1
    };
    let vector = layer.positions[next] - layer.positions[index];
    Edge {
        index,
        vector,
        length: vector.norm(),
    }
}

fn not_much_worse(layer: &LayerPlan, first: usize, second: usize) -> bool {
    let overhang_difference = (layer.overhangs[first] - layer.overhangs[second]).abs();
    let flow_width =
        layer.candidates.perimeters[layer.candidates.points[first].perimeter_index].flow_width;
    if (layer.overhangs[first] > 0.0 || layer.overhangs[second] > 0.0)
        && overhang_difference > 0.1 * flow_width
    {
        return layer.overhangs[first] < layer.overhangs[second];
    }
    if layer.embedded_distances[first] < -0.5 && layer.embedded_distances[second] > -0.5 {
        return true;
    }
    if layer.embedded_distances[second] < -0.5 && layer.embedded_distances[first] > -0.5 {
        return false;
    }
    true
}
