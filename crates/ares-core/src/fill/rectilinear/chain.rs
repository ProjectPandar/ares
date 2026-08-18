use crate::geometry::CoordinateScale;

use super::path_matrix::MonotonicPathMatrix;
use super::regions::MonotonicRegion;
use super::rng::Mt19937_64;
use super::segments::RectilinearSlice;

const SOURCE_EPSILON: f32 = 1.0e-4;
const INITIAL_PHEROMONE: f32 = 0.5;
const EVAPORATION: f32 = 0.1;
const DIVERSIFICATION: f32 = 0.1;
const TAKE_BEST_PROBABILITY: f32 = 0.9;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct MonotonicRegionLink {
    pub(crate) region: usize,
    pub(crate) flipped: bool,
}

#[derive(Clone, Copy)]
struct Candidate {
    region: usize,
    flipped: bool,
    probability: f32,
    direct: bool,
}

#[expect(
    clippy::excessive_nesting,
    reason = "source ant optimization nests rounds, ants, strict improvement, and zero exit"
)]
pub(crate) fn chain_monotonic_regions(
    regions: &[MonotonicRegion],
    slice: &RectilinearSlice,
    scale: CoordinateScale,
) -> Vec<MonotonicRegionLink> {
    if regions.is_empty() {
        return Vec::new();
    }
    let (dependencies, queue) = initial_state(regions);
    let mut matrix = MonotonicPathMatrix::new(regions, slice, scale, INITIAL_PHEROMONE);
    let initial_deposit =
        greedy_initial_deposit(regions, &mut matrix, dependencies.clone(), queue.clone());
    matrix.update_initial_pheromone(initial_deposit);

    let mut rng = Mt19937_64::default();
    let mut best = Vec::new();
    let mut best_length = f32::MAX;
    let mut rounds_without_change = 0;
    let ants = regions.len().min(10);
    for _ in 0..25 {
        if rounds_without_change >= 8 {
            break;
        }
        let mut improved = false;
        let mut reinforcement_path = Vec::new();
        for _ in 0..ants {
            let mut path = build_ant_path(
                regions,
                &mut matrix,
                dependencies.clone(),
                queue.clone(),
                initial_deposit,
                &mut rng,
            );
            monotonic_three_opt(&mut path, regions, &mut matrix);
            let length = path_length(&path, regions, &mut matrix);
            if length < best_length {
                best_length = length;
                std::mem::swap(&mut best, &mut path);
                improved = true;
                if length == 0.0 {
                    return best;
                }
            }
            reinforcement_path = path;
        }
        reinforce(&reinforcement_path, best_length, &mut matrix);
        if improved {
            rounds_without_change = 0;
        } else {
            rounds_without_change += 1;
        }
    }
    best
}

fn initial_state(regions: &[MonotonicRegion]) -> (Vec<i32>, Vec<usize>) {
    let mut dependencies = vec![1; regions.len()];
    let mut queue = Vec::new();
    for (index, region) in regions.iter().enumerate() {
        if region.left_neighbors.is_empty() {
            queue.push(index);
        } else {
            dependencies[index] += region.left_neighbors.len() as i32;
        }
    }
    (dependencies, queue)
}

#[expect(
    clippy::excessive_nesting,
    reason = "source greedy seed nests path growth, neighbors, and two orientations"
)]
fn greedy_initial_deposit(
    regions: &[MonotonicRegion],
    matrix: &mut MonotonicPathMatrix<'_>,
    mut dependencies: Vec<i32>,
    mut queue: Vec<usize>,
) -> f32 {
    let first = queue.pop().expect("precedence graph has a root");
    dependencies[first] -= 1;
    let mut current = MonotonicRegionLink {
        region: first,
        flipped: false,
    };
    let mut total = regions[first].length(false);
    while !queue.is_empty() || !regions[current.region].right_neighbors.is_empty() {
        let mut selected = None;
        let mut best_visibility = 0.0;
        for &next in &regions[current.region].right_neighbors {
            if dependencies[next] == 2 {
                for flipped in [false, true] {
                    let visibility = matrix
                        .edge(current.region, current.flipped, next, flipped)
                        .visibility;
                    if visibility > best_visibility {
                        selected = Some((next, flipped));
                        best_visibility = visibility;
                    }
                }
            }
        }
        let from_queue = selected.is_none();
        if from_queue {
            for &next in &queue {
                for flipped in [false, true] {
                    let visibility = matrix
                        .edge(current.region, current.flipped, next, flipped)
                        .visibility;
                    if visibility > best_visibility {
                        selected = Some((next, flipped));
                        best_visibility = visibility;
                    }
                }
            }
        }
        let (next, flipped) = selected.expect("unprocessed region is reachable");
        for &neighbor in &regions[current.region].right_neighbors {
            dependencies[neighbor] -= 1;
            if dependencies[neighbor] == 1 && neighbor != next {
                queue.push(neighbor);
            }
        }
        if from_queue {
            remove_queue_item(&mut queue, next);
        }
        total += regions[next].length(flipped)
            + matrix
                .edge(current.region, current.flipped, next, flipped)
                .length;
        current = MonotonicRegionLink {
            region: next,
            flipped,
        };
        dependencies[next] = 0;
    }
    0.1 / total
}

#[expect(
    clippy::too_many_arguments,
    reason = "source ant carries graph, matrix, mutable queue/dependencies, deposit, and RNG"
)]
fn build_ant_path(
    regions: &[MonotonicRegion],
    matrix: &mut MonotonicPathMatrix<'_>,
    mut dependencies: Vec<i32>,
    mut queue: Vec<usize>,
    initial_deposit: f32,
    rng: &mut Mt19937_64,
) -> Vec<MonotonicRegionLink> {
    let first_index = rng.index(queue.len());
    let first = queue.swap_remove(first_index);
    let mut path = vec![MonotonicRegionLink {
        region: first,
        flipped: rng.next() > u64::MAX / 2,
    }];
    dependencies[first] = 0;

    while path.len() < regions.len() {
        let current = *path.last().expect("path is nonempty");
        let mut direct = Vec::new();
        for &next in &regions[current.region].right_neighbors {
            dependencies[next] -= 1;
            if dependencies[next] == 1 {
                direct.push(next);
            }
        }
        let candidate_regions = if direct.is_empty() { &queue } else { &direct };
        let mut candidates = Vec::with_capacity(candidate_regions.len() * 2);
        for &next in candidate_regions {
            for flipped in [false, true] {
                let edge = *matrix.edge(current.region, current.flipped, next, flipped);
                candidates.push(Candidate {
                    region: next,
                    flipped,
                    probability: edge.pheromone.powf(1.0) * edge.visibility.powf(2.0),
                    direct: !direct.is_empty(),
                });
            }
        }
        let selected_index = select_candidate(&candidates, rng);
        let selected = candidates[selected_index];
        for next in direct {
            if next != selected.region && !queue.contains(&next) {
                queue.push(next);
            }
        }
        if !selected.direct {
            remove_queue_item(&mut queue, selected.region);
        }
        let edge = matrix.edge(
            current.region,
            current.flipped,
            selected.region,
            selected.flipped,
        );
        edge.pheromone =
            (1.0 - DIVERSIFICATION) * edge.pheromone + DIVERSIFICATION * initial_deposit;
        dependencies[selected.region] = 0;
        path.push(MonotonicRegionLink {
            region: selected.region,
            flipped: selected.flipped,
        });
    }
    path
}

fn monotonic_three_opt(
    path: &mut [MonotonicRegionLink],
    regions: &[MonotonicRegion],
    matrix: &mut MonotonicPathMatrix<'_>,
) {
    for index in 0..path.len().saturating_sub(3) {
        let [path0, path1, path2, path3] = [
            path[index],
            path[index + 1],
            path[index + 2],
            path[index + 3],
        ];
        if regions[path2.region]
            .right_neighbors
            .contains(&path1.region)
        {
            continue;
        }
        let before = transition_length(matrix, path0, path1)
            + transition_length(matrix, path1, path2)
            + transition_length(matrix, path2, path3);
        let after = transition_length(matrix, path0, path2)
            + transition_length(matrix, path2, path1)
            + transition_length(matrix, path1, path3);
        if after < before {
            path.swap(index + 1, index + 2);
        }
    }
}

fn transition_length(
    matrix: &mut MonotonicPathMatrix<'_>,
    from: MonotonicRegionLink,
    to: MonotonicRegionLink,
) -> f32 {
    matrix
        .edge(from.region, from.flipped, to.region, to.flipped)
        .length
}

fn select_candidate(candidates: &[Candidate], rng: &mut Mt19937_64) -> usize {
    if rng.unit_f32() < TAKE_BEST_PROBABILITY {
        let mut best = 0;
        for index in 1..candidates.len() {
            if candidates[index].probability > candidates[best].probability {
                best = index;
            }
        }
        best
    } else {
        let total = candidates
            .iter()
            .map(|candidate| candidate.probability)
            .sum::<f32>();
        let mut threshold = rng.unit_f32() * total;
        candidates
            .iter()
            .position(|candidate| {
                threshold -= candidate.probability;
                threshold <= 0.0
            })
            .unwrap_or(candidates.len() - 1)
    }
}

fn path_length(
    path: &[MonotonicRegionLink],
    regions: &[MonotonicRegion],
    matrix: &mut MonotonicPathMatrix<'_>,
) -> f32 {
    let mut total = 0.0;
    for (index, link) in path.iter().enumerate() {
        total += regions[link.region].length(link.flipped);
        if let Some(next) = path.get(index + 1) {
            total += matrix
                .edge(link.region, link.flipped, next.region, next.flipped)
                .length;
        }
    }
    total
}

fn reinforce(path: &[MonotonicRegionLink], best_length: f32, matrix: &mut MonotonicPathMatrix<'_>) {
    let total = best_length + SOURCE_EPSILON;
    for pair in path.windows(2) {
        let edge = matrix.edge(
            pair[0].region,
            pair[0].flipped,
            pair[1].region,
            pair[1].flipped,
        );
        edge.pheromone = (1.0 - EVAPORATION) * edge.pheromone + EVAPORATION / total;
    }
}

fn remove_queue_item(queue: &mut Vec<usize>, region: usize) {
    let index = queue
        .iter()
        .position(|candidate| *candidate == region)
        .expect("selected queued region is present");
    queue.swap_remove(index);
}
