mod context;
mod preparation;

use crate::project_slice::seam_candidates::{LayerSeamCandidates, SeamPerimeter};

use super::{angle_penalty, mesh::Vec3, spatial::PointKdTree, spline::CubicSpline};

pub(super) use preparation::prepare;

const SCORE_TOLERANCE: f32 = 0.3;
const SEARCH_DISTANCE_FLOW_FACTOR: f32 = 4.0;
const MINIMUM_STRING_SEAMS: usize = 6;
const MILLIMETERS_PER_SEGMENT: f32 = 4.0;
const SHARP_ANGLE_THRESHOLD: f32 = 55.0 * std::f32::consts::PI / 180.0;

pub(super) struct LayerPlan {
    pub(super) candidates: LayerSeamCandidates,
    pub(super) choices: Vec<PerimeterChoice>,
    pub(super) collection_perimeters: Vec<Vec<usize>>,
    scores: Vec<f32>,
    z: f32,
    overhangs: Vec<f32>,
    embedded_distances: Vec<f32>,
    positions: Vec<Vec3>,
    point_tree: PointKdTree,
}

pub(super) struct PerimeterChoice {
    pub(super) seam_index: usize,
    pub(super) final_position: Option<Vec3>,
    finalized: bool,
}

fn is_better(layer: &LayerPlan, first: usize, second: usize) -> bool {
    if layer.overhangs[first] > 0.0 || layer.overhangs[second] > 0.0 {
        return layer.overhangs[first] < layer.overhangs[second];
    }
    if layer.embedded_distances[first] < -0.5 && layer.embedded_distances[second] > -0.5 {
        return true;
    }
    if layer.embedded_distances[second] < -0.5 && layer.embedded_distances[first] > -0.5 {
        return false;
    }
    layer.scores[first] < layer.scores[second]
}

fn is_better_between(layers: &[LayerPlan], first: (usize, usize), second: (usize, usize)) -> bool {
    let first_layer = &layers[first.0];
    let second_layer = &layers[second.0];
    if first_layer.overhangs[first.1] > 0.0 || second_layer.overhangs[second.1] > 0.0 {
        return first_layer.overhangs[first.1] < second_layer.overhangs[second.1];
    }
    if first_layer.embedded_distances[first.1] < -0.5
        && second_layer.embedded_distances[second.1] > -0.5
    {
        return true;
    }
    if second_layer.embedded_distances[second.1] < -0.5
        && first_layer.embedded_distances[first.1] > -0.5
    {
        return false;
    }
    first_layer.scores[first.1] < second_layer.scores[second.1]
}

fn is_not_much_worse(layer: &LayerPlan, first: usize, second: usize) -> bool {
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
    let first_penalty = layer.overhangs[first] + layer.scores[first];
    let second_penalty = layer.overhangs[second] + layer.scores[second];
    first_penalty <= second_penalty || first_penalty - second_penalty < SCORE_TOLERANCE
}

pub(super) fn align(layers: &mut [LayerPlan]) {
    let mut seams = layers
        .iter()
        .enumerate()
        .flat_map(|(layer_index, layer)| {
            layer
                .choices
                .iter()
                .map(move |choice| (layer_index, choice.seam_index))
        })
        .collect::<Vec<_>>();
    seams.sort_by(|&left, &right| {
        if is_better_between(layers, left, right) {
            std::cmp::Ordering::Less
        } else if is_better_between(layers, right, left) {
            std::cmp::Ordering::Greater
        } else {
            std::cmp::Ordering::Equal
        }
    });
    let mut global_index = 0;
    while global_index < seams.len() {
        let start = seams[global_index];
        global_index += 1;
        if choice(layers, start).finalized {
            continue;
        }
        let mut seam_string = find_seam_string(layers, start);
        let step = 1 + seam_string.len() / 20;
        let mut alternative_start = 0;
        while alternative_start < seam_string.len() {
            let point = seam_string[alternative_start];
            let alternative = find_seam_string(layers, {
                let seam_index = choice(layers, point).seam_index;
                (point.0, seam_index)
            });
            if alternative.len() > seam_string.len() {
                seam_string = alternative;
            }
            alternative_start += step;
        }
        if seam_string.len() < MINIMUM_STRING_SEAMS {
            continue;
        }
        seam_string.sort_by_key(|point| point.0);
        global_index -= 1;
        finalize_string(layers, &seam_string);
    }
}

fn find_seam_string(layers: &[LayerPlan], start: (usize, usize)) -> Vec<(usize, usize)> {
    let max_distance = SEARCH_DISTANCE_FLOW_FACTOR * perimeter(layers, start).flow_width;
    let mut next_layer = start.0 as isize + 1;
    let mut step = 1;
    let mut previous = start;
    let mut output = vec![start];
    while next_layer >= 0 {
        if next_layer >= layers.len() as isize {
            step = -1;
            previous = start;
            next_layer = start.0 as isize - 1;
            if next_layer < 0 {
                break;
            }
        }
        let next_layer_index = next_layer as usize;
        let previous_position = candidate_position(layers, previous);
        let projected = Vec3::new(
            previous_position.x,
            previous_position.y,
            layers[next_layer_index].z,
        );
        if let Some(next) = find_next_in_layer(layers, next_layer_index, projected, max_distance) {
            output.push(next);
            previous = next;
        } else if step == 1 {
            step = -1;
            previous = start;
            next_layer = start.0 as isize - 1;
            if next_layer < 0 {
                break;
            }
        } else {
            break;
        }
        next_layer += step;
    }
    output
}

fn find_next_in_layer(
    layers: &[LayerPlan],
    layer_index: usize,
    projected: Vec3,
    max_distance: f32,
) -> Option<(usize, usize)> {
    let layer = &layers[layer_index];
    let nearby = layer
        .point_tree
        .in_radius(&layer.positions, projected, max_distance);
    let &first = nearby.first()?;
    let mut best = first;
    let mut nearest = first;
    for index in nearby {
        if layer.choices[layer.candidates.points[index].perimeter_index].finalized {
            continue;
        }
        if is_better(layer, index, best)
            || layer.choices[layer.candidates.points[best].perimeter_index].finalized
        {
            best = index;
        }
        if distance_squared(layer, index, projected) < distance_squared(layer, nearest, projected)
            || layer.choices[layer.candidates.points[nearest].perimeter_index].finalized
        {
            nearest = index;
        }
    }
    if layer.choices[layer.candidates.points[nearest].perimeter_index].finalized {
        return None;
    }
    let selected = layer.choices[layer.candidates.points[nearest].perimeter_index].seam_index;
    if is_not_much_worse(layer, nearest, selected) {
        return Some((layer_index, nearest));
    }
    is_not_much_worse(layer, best, selected).then_some((layer_index, best))
}

fn distance_squared(layer: &LayerPlan, index: usize, point: Vec3) -> f32 {
    let candidate = layer.candidates.points[index].position;
    (Vec3::new(candidate.x, candidate.y, candidate.z) - point).norm_squared()
}

fn finalize_string(layers: &mut [LayerPlan], seam_string: &[(usize, usize)]) {
    let observations = seam_string
        .iter()
        .map(|&point| {
            let position = candidate_position(layers, point);
            (position.x, position.y)
        })
        .collect::<Vec<_>>();
    let points = seam_string
        .iter()
        .map(|&point| candidate_position(layers, point).z)
        .collect::<Vec<_>>();
    let weights = seam_string
        .iter()
        .map(|&point| 1.0 / (0.1 + angle_penalty(candidate_angle(layers, point))))
        .collect::<Vec<_>>();
    let total_length = string_length(layers, seam_string);
    let segment_count = ((total_length.max(0.0) / MILLIMETERS_PER_SEGMENT) as usize).max(1);
    let curve = CubicSpline::fit(&observations, &points, &weights, segment_count);
    let finals = seam_string
        .iter()
        .map(|&point| {
            let current = candidate_position(layers, point);
            let fitted = curve.value(current.z);
            let angle_ratio = candidate_angle(layers, point).abs() / SHARP_ANGLE_THRESHOLD;
            let blend = angle_ratio.powf(3.0).min(1.0);
            Vec3::new(
                blend * current.x + (1.0 - blend) * fitted.0,
                blend * current.y + (1.0 - blend) * fitted.1,
                current.z,
            )
        })
        .collect::<Vec<_>>();
    for (&point, final_position) in seam_string.iter().zip(finals) {
        let perimeter_index = layers[point.0].candidates.points[point.1].perimeter_index;
        let choice = &mut layers[point.0].choices[perimeter_index];
        choice.seam_index = point.1;
        choice.final_position = Some(final_position);
        choice.finalized = true;
    }
}

fn string_length(layers: &[LayerPlan], seam_string: &[(usize, usize)]) -> f32 {
    let mut total = 0.0;
    let mut last = candidate_position(layers, seam_string[0]);
    for (index, &point) in seam_string.iter().enumerate() {
        let current = candidate_position(layers, point);
        let layer_angle = if index == 0 || index + 1 == seam_string.len() {
            0.0
        } else {
            let previous = current - candidate_position(layers, seam_string[index - 1]);
            let next = candidate_position(layers, seam_string[index + 1]) - current;
            previous.normalized().dot(next.normalized()).acos().abs()
        };
        let influence = if layer_angle > 2.0 * candidate_angle(layers, point).abs() {
            -0.8
        } else {
            1.0
        };
        total += influence * (last - current).norm();
        last = current;
    }
    total
}

fn candidate_position(layers: &[LayerPlan], point: (usize, usize)) -> Vec3 {
    layers[point.0].positions[point.1]
}

fn candidate_angle(layers: &[LayerPlan], point: (usize, usize)) -> f32 {
    layers[point.0].candidates.points[point.1].local_ccw_angle
}

fn choice(layers: &[LayerPlan], point: (usize, usize)) -> &PerimeterChoice {
    &layers[point.0].choices[layers[point.0].candidates.points[point.1].perimeter_index]
}

fn perimeter(layers: &[LayerPlan], point: (usize, usize)) -> &SeamPerimeter {
    &layers[point.0].candidates.perimeters
        [layers[point.0].candidates.points[point.1].perimeter_index]
}
