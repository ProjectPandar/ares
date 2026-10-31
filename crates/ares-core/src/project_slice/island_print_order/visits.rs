//! GCode.cpp:7290–7362 producer and7680–7720 consumer at
//! OrcaSlicer 345efec64c7b95a05ce5cf96df0dde66a158835c.

use super::OrderedExtrusionLayer;
use crate::geometry::{CoordinateScale, ExPolygon, Point};

pub(in crate::project_slice) struct InstanceVisit {
    pub(in crate::project_slice) instance_idx: usize,
    pub(in crate::project_slice) islands: Vec<usize>,
    pub(in crate::project_slice) first_visit: bool,
}

pub(in crate::project_slice) struct InstanceInput<'a> {
    pub(in crate::project_slice) shift: Point,
    pub(in crate::project_slice) slices: Option<&'a [ExPolygon]>,
    pub(in crate::project_slice) occupied: Vec<bool>,
}

// Cache-only ObjectID/instance-id identity stays in the owning instance vector;
// recomputing the pure memo needs only its dense index, island index and position.
struct IslandOrderNode {
    instance_idx: usize,
    island_idx: Option<usize>,
    pos: Point,
}

pub(in crate::project_slice) fn produce(
    instances: &[InstanceInput<'_>],
    print_order: crate::ProcessPrintOrder,
    wt_pos: Point,
    scale: CoordinateScale,
) -> Vec<InstanceVisit> {
    let nodes = nodes(instances, scale);
    let points = nodes.iter().map(|node| node.pos).collect::<Vec<_>>();
    let tour =
        crate::geometry::order_points_with_strategy(&points, print_order, Some(wt_pos), scale);
    let mut visits: Vec<InstanceVisit> = Vec::new();
    let mut seen = vec![false; instances.len()];
    let mut last = vec![None; instances.len()];
    for index in tour.into_iter().rev() {
        let node = &nodes[index];
        if visits
            .last()
            .is_none_or(|visit| visit.instance_idx != node.instance_idx)
        {
            visits.push(InstanceVisit {
                instance_idx: node.instance_idx,
                islands: Vec::new(),
                first_visit: !seen[node.instance_idx],
            });
            seen[node.instance_idx] = true;
        }
        let visit_index = visits.len() - 1;
        if let Some(island) = node.island_idx {
            visits[visit_index].islands.push(island);
        }
        last[node.instance_idx] = Some(visit_index);
    }
    append_catch_all(&mut visits, instances, &last);
    visits
}

fn nodes(instances: &[InstanceInput<'_>], scale: CoordinateScale) -> Vec<IslandOrderNode> {
    let mut nodes = Vec::new();
    for (instance_idx, instance) in instances.iter().enumerate() {
        let first_node = nodes.len();
        append_chainable_nodes(&mut nodes, instance_idx, instance, scale);
        if nodes.len() == first_node {
            nodes.push(IslandOrderNode {
                instance_idx,
                island_idx: None,
                pos: quantize(instance.shift, scale),
            });
        }
    }
    nodes
}

fn append_chainable_nodes(
    nodes: &mut Vec<IslandOrderNode>,
    instance_idx: usize,
    instance: &InstanceInput<'_>,
    scale: CoordinateScale,
) {
    if let Some(slices) = instance.slices
        && instance.occupied.len() == slices.len() + 1
    {
        for (island_idx, slice) in slices.iter().enumerate() {
            if instance.occupied[island_idx] {
                let center = slice.contour().centroid();
                nodes.push(IslandOrderNode {
                    instance_idx,
                    island_idx: Some(island_idx),
                    pos: quantize(
                        Point::new(
                            center.x() + instance.shift.x(),
                            center.y() + instance.shift.y(),
                        ),
                        scale,
                    ),
                });
            }
        }
    }
}

fn append_catch_all(
    visits: &mut [InstanceVisit],
    instances: &[InstanceInput<'_>],
    last: &[Option<usize>],
) {
    for (index, instance) in instances.iter().enumerate() {
        let Some(last_visit) = last[index] else {
            continue;
        };
        let visit = &mut visits[last_visit];
        if !visit.islands.is_empty() && instance.occupied.last().is_some_and(|&occupied| occupied) {
            visit.islands.push(instance.occupied.len() - 1);
        }
    }
}

fn quantize(point: Point, scale: CoordinateScale) -> Point {
    let grid = scale.scaled_delta(1.0) as i64;
    let q = |value: i64| {
        ((if value >= 0 {
            value + grid / 2
        } else {
            value - grid / 2
        }) / grid)
            * grid
    };
    Point::new(q(point.x()), q(point.y()))
}

pub(in crate::project_slice) fn island_order(
    layer: &OrderedExtrusionLayer,
    visit: &InstanceVisit,
    slices: Option<&[ExPolygon]>,
    start_near: Option<Point>,
) -> Vec<usize> {
    if !visit.islands.is_empty() {
        // Indices are generated from this layout; no stale cached tour exists.
        // The source's malformed-cache diagnostic is outside this port boundary.
        return visit.islands.clone();
    }
    let Some(slices) = slices.filter(|slices| layer.islands.len() == slices.len() + 1) else {
        return (0..layer.islands.len()).collect();
    };
    let mut order = (0..slices.len())
        .filter(|&index| !layer.islands[index].entities.is_empty())
        .collect::<Vec<_>>();
    if order.len() > 1 {
        let centroids = order
            .iter()
            .map(|&index| slices[index].contour().centroid())
            .collect::<Vec<_>>();
        order = crate::geometry::chain_points_with_start(&centroids, start_near)
            .into_iter()
            .map(|index| order[index])
            .collect();
    }
    if layer
        .islands
        .last()
        .is_some_and(|island| !island.entities.is_empty())
    {
        order.push(layer.islands.len() - 1);
    }
    order
}
