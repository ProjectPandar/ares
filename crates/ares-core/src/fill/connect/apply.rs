use super::{
    contour::{
        complete_arc_attempts, limited_hook_is_clockwise, path_length_along_contour_ccw,
        remaining_arc_is_eligible, skip_sorted_multiline_arc, sorted_arc_takes_full, take_full_arc,
        take_limited,
    },
    types::{Arc, WorkingGraph},
};
use crate::geometry::{Point, Polyline, fixed_gcc_sort_by};

#[expect(
    clippy::too_many_arguments,
    reason = "the source connector keeps its independent scalar thresholds explicit"
)]
pub(super) fn apply_connections(
    mut graph: WorkingGraph,
    anchor_length: f64,
    anchor_length_max: f64,
    scaled_spacing: f64,
    multiline: i32,
    dont_sort: bool,
    scaled_epsilon: f64,
) -> Vec<Polyline> {
    let mut arcs = collect_arcs(&graph, dont_sort);
    sort_arcs(&mut arcs);
    for arc in arcs {
        apply_sorted_arc(
            &mut graph,
            arc,
            anchor_length,
            anchor_length_max,
            scaled_spacing,
            multiline,
            scaled_epsilon,
        );
    }
    apply_remaining_endpoints(&mut graph, anchor_length, anchor_length_max, scaled_epsilon);
    graph
        .paths
        .into_iter()
        .flatten()
        .filter(|points| !points.is_empty())
        .map(Polyline::new)
        .collect()
}

pub(super) fn sort_arcs(arcs: &mut [Arc]) {
    fixed_gcc_sort_by(arcs, |left, right| left.length < right.length);
}

fn collect_arcs(graph: &WorkingGraph, dont_sort: bool) -> Vec<Arc> {
    if dont_sort {
        return Vec::new();
    }
    graph
        .intersections
        .iter()
        .enumerate()
        .filter_map(|(intersection_index, intersection)| {
            let contour_index = intersection.contour_index?;
            let next = intersection.next?;
            if next == intersection_index || !graph.could_connect_next(intersection_index) {
                return None;
            }
            let contour_length = *graph.boundary[contour_index]
                .params
                .last()
                .expect("a contour has a closing parameter");
            Some(Arc {
                intersection_index,
                length: path_length_along_contour_ccw(
                    intersection,
                    &graph.intersections[next],
                    contour_length,
                ),
            })
        })
        .collect()
}

#[expect(
    clippy::too_many_arguments,
    reason = "the source sorted-arc pass keeps both thresholds and ordering controls explicit"
)]
fn apply_sorted_arc(
    graph: &mut WorkingGraph,
    arc: Arc,
    anchor_length: f64,
    anchor_length_max: f64,
    scaled_spacing: f64,
    multiline: i32,
    scaled_epsilon: f64,
) {
    let first = arc.intersection_index;
    let second = graph.intersections[first]
        .next
        .expect("a collected arc has a next intersection");
    if graph.intersections[first].consumed || graph.intersections[second].consumed {
        return;
    }
    let first_path = graph.root(WorkingGraph::path_index_for_intersection(first));
    let second_path = graph.root(WorkingGraph::path_index_for_intersection(second));
    if skip_sorted_multiline_arc(multiline, arc.length, scaled_spacing) || first_path == second_path
    {
        return;
    }
    if sorted_arc_takes_full(arc.length, anchor_length_max) {
        merge_paths(graph, first, second, first_path, second_path, false);
    } else if anchor_length > scaled_epsilon {
        take_hook(
            graph,
            first_path,
            first,
            second,
            false,
            anchor_length,
            scaled_epsilon,
        );
        take_hook(
            graph,
            second_path,
            second,
            first,
            true,
            anchor_length,
            scaled_epsilon,
        );
    }
}

#[expect(
    clippy::excessive_nesting,
    reason = "the source remaining-endpoint pass has ordered merge and hook fallbacks"
)]
fn apply_remaining_endpoints(
    graph: &mut WorkingGraph,
    anchor_length: f64,
    anchor_length_max: f64,
    scaled_epsilon: f64,
) {
    for intersection_index in 0..graph.intersections.len() {
        let Some(contour_index) = graph.intersections[intersection_index].contour_index else {
            continue;
        };
        if graph.intersections[intersection_index].consumed {
            continue;
        }
        let contour_length = *graph.boundary[contour_index]
            .params
            .last()
            .expect("a contour has a closing parameter");
        let previous_length = graph.could_connect_prev(intersection_index).then(|| {
            let previous = graph.intersections[intersection_index]
                .prev
                .expect("a connected point has a previous link");
            path_length_along_contour_ccw(
                &graph.intersections[previous],
                &graph.intersections[intersection_index],
                contour_length,
            )
        });
        let next_length = graph.could_connect_next(intersection_index).then(|| {
            let next = graph.intersections[intersection_index]
                .next
                .expect("a connected point has a next link");
            path_length_along_contour_ccw(
                &graph.intersections[intersection_index],
                &graph.intersections[next],
                contour_length,
            )
        });
        let previous_length = previous_length.unwrap_or(f64::MAX);
        let next_length = next_length.unwrap_or(f64::MAX);
        let path_index = graph.root(WorkingGraph::path_index_for_intersection(
            intersection_index,
        ));
        let mut connected = false;
        for (length, clockwise) in complete_arc_attempts(previous_length, next_length) {
            if !remaining_arc_is_eligible(length, anchor_length_max) {
                break;
            }
            let other = if clockwise {
                graph.intersections[intersection_index]
                    .prev
                    .expect("a previous arc is connectable")
            } else {
                graph.intersections[intersection_index]
                    .next
                    .expect("a next arc is connectable")
            };
            let other_path = graph.root(WorkingGraph::path_index_for_intersection(other));
            if path_index == other_path {
                continue;
            }
            merge_paths(
                graph,
                intersection_index,
                other,
                path_index,
                other_path,
                clockwise,
            );
            connected = true;
            break;
        }
        if !connected && anchor_length > scaled_epsilon {
            let previous = graph.intersections[intersection_index].not_taken_prev;
            let next = graph.intersections[intersection_index].not_taken_next;
            if previous.max(next) > scaled_epsilon {
                let clockwise = limited_hook_is_clockwise(previous, next);
                let end = if clockwise {
                    graph.intersections[intersection_index]
                        .prev
                        .expect("a connected point has a previous link")
                } else {
                    graph.intersections[intersection_index]
                        .next
                        .expect("a connected point has a next link")
                };
                take_hook(
                    graph,
                    path_index,
                    intersection_index,
                    end,
                    clockwise,
                    anchor_length,
                    scaled_epsilon,
                );
            }
        }
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "the source merge identifies two endpoints and their two stable roots"
)]
fn merge_paths(
    graph: &mut WorkingGraph,
    first: usize,
    second: usize,
    first_path: usize,
    second_path: usize,
    clockwise: bool,
) {
    let mut first_points = graph.paths[first_path]
        .take()
        .expect("a root path is present");
    let mut second_points = graph.paths[second_path]
        .take()
        .expect("a distinct root path is present");
    orient_first(&mut first_points, graph.point(first));
    orient_second(&mut second_points, graph.point(second));
    let contour_index = graph.intersections[first]
        .contour_index
        .expect("a merge endpoint is connected");
    take_full_arc(
        &mut first_points,
        &second_points,
        &graph.boundary[contour_index].points,
        &mut graph.intersections,
        first,
        second,
        clockwise,
    );
    if second_path < first_path {
        graph.paths[second_path] = Some(first_points);
        graph.parents[first_path] = graph.parents[second_path];
    } else {
        graph.paths[first_path] = Some(first_points);
        graph.parents[second_path] = graph.parents[first_path];
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "the source hook operation keeps endpoint, direction, and length state explicit"
)]
fn take_hook(
    graph: &mut WorkingGraph,
    path_index: usize,
    start: usize,
    end: usize,
    clockwise: bool,
    anchor_length: f64,
    scaled_epsilon: f64,
) {
    let contour_index = graph.intersections[start]
        .contour_index
        .expect("a hook endpoint is connected");
    let mut path = graph.paths[path_index]
        .take()
        .expect("a hook root path is present");
    take_limited(
        &mut path,
        &graph.boundary[contour_index],
        &mut graph.intersections,
        start,
        end,
        clockwise,
        anchor_length,
        graph.line_half_width,
        scaled_epsilon,
    );
    graph.paths[path_index] = Some(path);
}

fn orient_first(path: &mut [Point], boundary_point: Point) {
    debug_assert!(path.first() == Some(&boundary_point) || path.last() == Some(&boundary_point));
    if path.first() == Some(&boundary_point) {
        path.reverse();
    }
}

fn orient_second(path: &mut [Point], boundary_point: Point) {
    debug_assert!(path.first() == Some(&boundary_point) || path.last() == Some(&boundary_point));
    if path.last() == Some(&boundary_point) {
        path.reverse();
    }
}
