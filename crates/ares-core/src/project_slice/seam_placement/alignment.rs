mod context;
mod contour_continuation;
mod loop_normal;
mod paint_rules;
mod preparation;
mod random;
mod string_fit;

use crate::{
    ProcessSeamPosition,
    project_slice::seam_candidates::{LayerSeamCandidates, SeamPerimeter},
};

use super::{mesh::Vec3, spatial::PointKdTree};

pub(super) use preparation::prepare;

const SCORE_TOLERANCE: f32 = 0.3;
const SEARCH_DISTANCE_FLOW_FACTOR: f32 = 4.0;
const MINIMUM_STRING_SEAMS: usize = 6;

pub(super) struct LayerPlan {
    placement: ProcessSeamPosition,
    pub(super) candidates: LayerSeamCandidates,
    pub(super) choices: Vec<PerimeterChoice>,
    pub(super) collection_perimeters: Vec<Vec<usize>>,
    pub(super) scores: Vec<f32>,
    z: f32,
    max_nozzle_diameter: f64,
    pub(super) overhangs: Vec<f32>,
    pub(super) unsupported_distances: Option<Vec<f32>>,
    pub(super) embedded_distances: Vec<f32>,
    positions: Vec<Vec3>,
    point_tree: PointKdTree,
}

pub(super) struct PerimeterChoice {
    pub(super) seam_index: usize,
    pub(super) final_position: Option<Vec3>,
    finalized: bool,
}

fn is_better(layer: &LayerPlan, first: usize, second: usize) -> bool {
    if let Some(better) = paint_rules::priority(
        layer.placement,
        &layer.candidates.points[first],
        &layer.candidates.points[second],
    ) {
        return better;
    }
    if layer.overhangs[first] > 0.0 || layer.overhangs[second] > 0.0 {
        return layer.overhangs[first] < layer.overhangs[second];
    }
    if layer.embedded_distances[first] < -0.5 && layer.embedded_distances[second] > -0.5 {
        return true;
    }
    if layer.embedded_distances[second] < -0.5 && layer.embedded_distances[first] > -0.5 {
        return false;
    }
    // SeamPlacer.cpp:777-779 — rear preference follows context priorities.
    if layer.placement == ProcessSeamPosition::Back
        && layer.positions[first].y != layer.positions[second].y
    {
        return layer.positions[first].y > layer.positions[second].y;
    }
    layer.scores[first] < layer.scores[second]
}

fn is_better_between(layers: &[LayerPlan], first: (usize, usize), second: (usize, usize)) -> bool {
    let first_layer = &layers[first.0];
    let second_layer = &layers[second.0];
    if let Some(better) = paint_rules::priority(
        first_layer.placement,
        &first_layer.candidates.points[first.1],
        &second_layer.candidates.points[second.1],
    ) {
        return better;
    }
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
    if first_layer.placement == ProcessSeamPosition::Back
        && first_layer.positions[first.1].y != second_layer.positions[second.1].y
    {
        return first_layer.positions[first.1].y > second_layer.positions[second.1].y;
    }
    first_layer.scores[first.1] < second_layer.scores[second.1]
}

fn is_not_much_worse(layer: &LayerPlan, first: usize, second: usize) -> bool {
    let a = &layer.candidates.points[first];
    let b = &layer.candidates.points[second];
    if let Some(better) = paint_rules::priority(layer.placement, a, b) {
        return better;
    }
    if paint_rules::enforced(a) {
        return true;
    }
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
    // SeamPlacer.cpp:1253-1260 — Random uses no further tie-breaker.
    if layer.placement == ProcessSeamPosition::Random {
        return true;
    }
    // SeamPlacer.cpp:839-841 — strict rear-Y similarity, not score tolerance.
    if layer.placement == ProcessSeamPosition::Back {
        return layer.positions[first].y + SCORE_TOLERANCE * 5.0 > layer.positions[second].y;
    }
    let first_penalty = layer.overhangs[first] + layer.scores[first];
    let second_penalty = layer.overhangs[second] + layer.scores[second];
    first_penalty <= second_penalty || first_penalty - second_penalty < SCORE_TOLERANCE
}

pub(super) fn randomize(layers: &mut [LayerPlan]) {
    random::apply(layers);
}

pub(super) fn align(layers: &mut [LayerPlan]) {
    let mut contours = contour_continuation::Resolver::new(
        preparation::contours(layers),
        contour_continuation::Options::default(),
    );
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
        let mut seam_string = find_seam_string(layers, start, &mut contours);
        let step = 1 + seam_string.len() / 20;
        let mut alternative_start = 0;
        while alternative_start < seam_string.len() {
            let point = seam_string[alternative_start];
            let alternative = find_seam_string(
                layers,
                {
                    let seam_index = choice(layers, point).seam_index;
                    (point.0, seam_index)
                },
                &mut contours,
            );
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
        string_fit::finalize_string(layers, &seam_string);
    }
}

fn find_seam_string(
    layers: &[LayerPlan],
    start: (usize, usize),
    contours: &mut contour_continuation::Resolver,
) -> Vec<(usize, usize)> {
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
        if let Some(next) = find_next_in_layer(
            layers,
            (next_layer_index, projected),
            max_distance,
            previous,
            contours,
        ) {
            output.push(next);
            previous = next;
        } else if step == 1 {
            step = -1;
            previous = start;
            next_layer = start.0 as isize - 1;
            if next_layer < 0 {
                break;
            }
            // v2.5.0-beta SeamPlacer.cpp:1865-1869: visit start-1 before stepping.
            continue;
        } else {
            break;
        }
        next_layer += step;
    }
    output
}

fn find_next_in_layer(
    layers: &[LayerPlan],
    (layer_index, projected): (usize, Vec3),
    max_distance: f32,
    previous: (usize, usize),
    contours: &mut contour_continuation::Resolver,
) -> Option<(usize, usize)> {
    let source = perimeter(layers, previous);
    let source_index = layers[previous.0].candidates.points[previous.1].perimeter_index;
    let continuation = contours.resolve(
        (previous.0, source_index, previous.1 - source.start_index),
        if layer_index > previous.0 { 1 } else { -1 },
        f64::from(max_distance),
    );
    let continuation = continuation.contours();
    let layer = &layers[layer_index];
    let nearby = layer
        .point_tree
        .in_radius(&layer.positions, projected, max_distance);
    let mut selected = None;
    for index in nearby {
        let perimeter_index = layer.candidates.points[index].perimeter_index;
        // SeamPlacer.cpp:1766-1775 filters before initializing either index.
        if !continuation.is_empty() && !continuation.contains(&perimeter_index) {
            continue;
        }
        let (best, nearest) = selected.get_or_insert((index, index));
        if layer.choices[perimeter_index].finalized {
            continue;
        }
        if is_better(layer, index, *best)
            || layer.choices[layer.candidates.points[*best].perimeter_index].finalized
        {
            *best = index;
        }
        if distance_squared(layer, index, projected) < distance_squared(layer, *nearest, projected)
            || layer.choices[layer.candidates.points[*nearest].perimeter_index].finalized
        {
            *nearest = index;
        }
    }
    let (best, nearest) = selected?;
    if layer.choices[layer.candidates.points[nearest].perimeter_index].finalized {
        return None;
    }
    let selected = layer.choices[layer.candidates.points[nearest].perimeter_index].seam_index;
    // SeamPlacer.cpp:1802–1818: the alignment shortcut is separate from comparator priority.
    if layer.placement == ProcessSeamPosition::Back
        && layer.candidates.points[selected].central_enforcer
        && distance_squared(layer, selected, projected) < (3.0 * max_distance).powi(2)
    {
        return Some((layer_index, selected));
    }
    if is_not_much_worse(layer, nearest, selected) {
        return Some((layer_index, nearest));
    }
    is_not_much_worse(layer, best, selected).then_some((layer_index, best))
}

fn distance_squared(layer: &LayerPlan, index: usize, point: Vec3) -> f32 {
    let candidate = layer.candidates.points[index].position;
    (Vec3::new(candidate.x, candidate.y, candidate.z) - point).norm_squared()
}

fn candidate_position(layers: &[LayerPlan], point: (usize, usize)) -> Vec3 {
    layers[point.0].positions[point.1]
}

fn choice(layers: &[LayerPlan], point: (usize, usize)) -> &PerimeterChoice {
    &layers[point.0].choices[layers[point.0].candidates.points[point.1].perimeter_index]
}

fn perimeter(layers: &[LayerPlan], point: (usize, usize)) -> &SeamPerimeter {
    &layers[point.0].candidates.perimeters
        [layers[point.0].candidates.points[point.1].perimeter_index]
}
