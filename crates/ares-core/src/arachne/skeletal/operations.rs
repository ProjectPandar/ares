use crate::geometry::{Line, Point};

use super::{
    graph::{EdgeId, NodeId, SkeletalGraph, point_distance},
    payload::{EdgeType, SkeletalEdge, SkeletalJoint},
};

impl SkeletalGraph {
    pub(crate) fn remove_edge(&mut self, edge: EdgeId) {
        self.edges[edge.0] = None;
    }

    pub(crate) fn remove_node(&mut self, node: NodeId) {
        self.nodes[node.0] = None;
    }

    pub(crate) fn collapse_small_edges(&mut self, snap_distance: i64) {
        let candidates = self.active_edges().collect::<Vec<_>>();
        for quad_start in candidates {
            if !self.contains_edge(quad_start) || self.edge(quad_start).prev.is_some() {
                continue;
            }
            let mut quad_end = quad_start;
            while let Some(next) = self.edge(quad_end).next {
                quad_end = next;
            }
            let quad_mid = self.edge(quad_start).next.filter(|next| *next != quad_end);
            if let Some(middle) = quad_mid {
                self.collapse_quad_middle_if_close(middle, quad_end, snap_distance);
            }
            if !self.contains_edge(quad_start) || !self.contains_edge(quad_end) {
                continue;
            }
            let start_from = self
                .edge(quad_start)
                .from
                .expect("edge must have from node");
            let start_to = self.edge(quad_start).to.expect("edge must have to node");
            let end_from = self.edge(quad_end).from.expect("edge must have from node");
            let end_to = self.edge(quad_end).to.expect("edge must have to node");
            if self.nodes_are_close(start_from, end_to, snap_distance)
                && self.nodes_are_close(start_to, end_from, snap_distance)
            {
                self.collapse_quad_sides(quad_start, quad_end);
            }
        }
    }

    fn collapse_quad_middle_if_close(
        &mut self,
        middle: EdgeId,
        quad_end: EdgeId,
        snap_distance: i64,
    ) {
        let from = self
            .edge(middle)
            .from
            .expect("middle edge must have from node");
        let to = self.edge(middle).to.expect("middle edge must have to node");
        if self.nodes_are_close(from, to, snap_distance) {
            self.collapse_quad_middle(middle, quad_end);
        }
    }

    fn collapse_quad_middle(&mut self, middle: EdgeId, quad_end: EdgeId) {
        let middle_twin = self.edge(middle).twin.expect("quad middle must have twin");
        let merged_node = self.edge(middle).from.expect("middle must have from node");
        let removed_node = self.edge(middle).to.expect("middle must have to node");
        let mut edge_from_third = quad_end;
        for _ in 0..=1000 {
            if edge_from_third == middle_twin {
                break;
            }
            let twin = self
                .edge(edge_from_third)
                .twin
                .expect("quad edge must have twin");
            self.edge_mut(edge_from_third).from = Some(merged_node);
            self.edge_mut(twin).to = Some(merged_node);
            let Some(next) = self.edge(twin).next else {
                break;
            };
            edge_from_third = next;
        }

        if self.node(merged_node).incident_edge == Some(middle) {
            let replacement = self.edge(middle_twin).next.or_else(|| {
                let previous = self
                    .edge(middle)
                    .prev
                    .expect("middle must have previous edge");
                self.edge(previous).twin
            });
            self.node_mut(merged_node).incident_edge = replacement;
        }
        self.remove_node(removed_node);

        let previous = self
            .edge(middle)
            .prev
            .expect("middle must have previous edge");
        let next = self.edge(middle).next.expect("middle must have next edge");
        let twin_next = self
            .edge(middle_twin)
            .next
            .expect("twin must have next edge");
        let twin_previous = self
            .edge(middle_twin)
            .prev
            .expect("twin must have previous edge");
        self.edge_mut(previous).next = Some(next);
        self.edge_mut(next).prev = Some(previous);
        self.edge_mut(twin_next).prev = Some(twin_previous);
        self.edge_mut(twin_previous).next = Some(twin_next);
        self.remove_edge(middle_twin);
        self.remove_edge(middle);
    }

    fn collapse_quad_sides(&mut self, quad_start: EdgeId, quad_end: EdgeId) {
        let start_twin = self
            .edge(quad_start)
            .twin
            .expect("quad start must have twin");
        let end_twin = self.edge(quad_end).twin.expect("quad end must have twin");
        let end_to = self.edge(quad_end).to.expect("quad end must have to node");
        let end_from = self
            .edge(quad_end)
            .from
            .expect("quad end must have from node");
        let start_from = self
            .edge(quad_start)
            .from
            .expect("quad start must have from node");
        self.edge_mut(start_twin).to = Some(end_to);
        self.node_mut(end_to).incident_edge = Some(end_twin);
        if self.node(end_from).incident_edge == Some(quad_end) {
            let replacement = self.edge(end_twin).next.or_else(|| {
                let previous = self
                    .edge(quad_end)
                    .prev
                    .expect("quad end must have previous edge");
                self.edge(previous).twin
            });
            self.node_mut(end_from).incident_edge = replacement;
        }
        self.remove_node(start_from);
        self.edge_mut(start_twin).twin = Some(end_twin);
        self.edge_mut(end_twin).twin = Some(start_twin);
        self.remove_edge(quad_start);
        self.remove_edge(quad_end);
    }

    pub(crate) fn make_rib(
        &mut self,
        previous_edge: EdgeId,
        source_start: Point,
        source_end: Point,
    ) -> EdgeId {
        let target = self
            .edge(previous_edge)
            .to
            .expect("previous edge must have to node");
        let target_point = self.node(target).point;
        let projection = project(target_point, source_start, source_end, false);
        let distance = point_distance(target_point, projection);
        self.node_mut(target).data.distance_to_boundary = distance;
        let source_node = self.add_node_front(SkeletalJoint::default(), projection);
        self.node_mut(source_node).data.distance_to_boundary = 0;
        let forth = self.add_edge_front(SkeletalEdge::new(EdgeType::ExtraVoronoi));
        let back = self.add_edge_front(SkeletalEdge::new(EdgeType::ExtraVoronoi));
        self.edge_mut(previous_edge).next = Some(forth);
        self.edge_mut(forth).prev = Some(previous_edge);
        self.edge_mut(forth).from = Some(target);
        self.edge_mut(forth).to = Some(source_node);
        self.connect_twins(forth, back);
        self.edge_mut(back).from = Some(source_node);
        self.edge_mut(back).to = Some(target);
        self.node_mut(source_node).incident_edge = Some(back);
        back
    }

    pub(crate) fn insert_node(&mut self, edge: EdgeId, middle: Point, bead_count: i64) -> EdgeId {
        let middle_node = self.add_node(SkeletalJoint::default(), middle);
        let twin = self.edge(edge).twin.expect("edge must have twin");
        self.edge_mut(edge).twin = None;
        self.edge_mut(twin).twin = None;
        let (input_first, input_last) = self.insert_rib(edge, middle_node);
        let (twin_first, twin_last) = self.insert_rib(twin, middle_node);
        self.connect_twins(input_first, twin_last);
        self.connect_twins(input_last, twin_first);
        self.node_mut(middle_node).data.bead_count = bead_count;
        input_last
    }

    pub(crate) fn insert_rib(&mut self, edge: EdgeId, middle_node: NodeId) -> (EdgeId, EdgeId) {
        let before = self.edge(edge).prev;
        let after = self.edge(edge).next;
        let node_before = self.edge(edge).from.expect("edge must have from node");
        let node_after = self.edge(edge).to.expect("edge must have to node");
        let source = self.source(edge);
        let projection = project(self.node(middle_node).point, source.a, source.b, true);
        let distance = point_distance(self.node(middle_node).point, projection);
        assert!(distance > 0);
        self.node_mut(middle_node).data.distance_to_boundary = distance;
        self.node_mut(middle_node).data.transition_ratio = 0.0;
        let source_node = self.add_node(SkeletalJoint::default(), projection);
        self.node_mut(source_node).data.distance_to_boundary = 0;

        let second = self.add_edge(SkeletalEdge::default());
        let outward = self.add_edge(SkeletalEdge::new(EdgeType::TransitionEnd));
        let inward = self.add_edge(SkeletalEdge::new(EdgeType::TransitionEnd));
        if let Some(before) = before {
            self.edge_mut(before).next = Some(edge);
        }
        self.edge_mut(edge).next = Some(outward);
        self.edge_mut(outward).next = None;
        self.edge_mut(inward).next = Some(second);
        self.edge_mut(second).next = after;
        if let Some(after) = after {
            self.edge_mut(after).prev = Some(second);
            self.node_mut(node_after).incident_edge = Some(after);
        }
        self.edge_mut(second).prev = Some(inward);
        self.edge_mut(inward).prev = None;
        self.edge_mut(outward).prev = Some(edge);
        self.edge_mut(edge).prev = before;

        self.edge_mut(edge).from = Some(node_before);
        self.edge_mut(edge).to = Some(middle_node);
        self.edge_mut(outward).from = Some(middle_node);
        self.edge_mut(outward).to = Some(source_node);
        self.edge_mut(inward).from = Some(source_node);
        self.edge_mut(inward).to = Some(middle_node);
        self.edge_mut(second).from = Some(middle_node);
        self.edge_mut(second).to = Some(node_after);
        self.node_mut(node_before).incident_edge = Some(edge);
        self.node_mut(middle_node).incident_edge = Some(outward);
        self.node_mut(source_node).incident_edge = Some(inward);
        self.edge_mut(edge).data.set_is_central(true);
        self.edge_mut(outward).data.set_is_central(false);
        self.edge_mut(inward).data.set_is_central(false);
        self.edge_mut(second).data.set_is_central(true);
        self.connect_twins(outward, inward);
        self.edge_mut(edge).twin = None;
        self.edge_mut(second).twin = None;
        (edge, second)
    }

    pub(crate) fn source(&self, edge: EdgeId) -> Line {
        let mut from_edge = edge;
        while let Some(previous) = self.edge(from_edge).prev {
            from_edge = previous;
        }
        let mut to_edge = edge;
        while let Some(next) = self.edge(to_edge).next {
            to_edge = next;
        }
        Line::new(
            self.node(self.edge(from_edge).from.expect("edge must have from node"))
                .point,
            self.node(self.edge(to_edge).to.expect("edge must have to node"))
                .point,
        )
    }

    fn nodes_are_close(&self, left: NodeId, right: NodeId, distance: i64) -> bool {
        let left = self.node(left).point;
        let right = self.node(right).point;
        let dx = left.x() - right.x();
        let dy = left.y() - right.y();
        if dx > distance || dx < -distance || dy > distance || dy < -distance {
            return false;
        }
        dx * dx + dy * dy <= distance * distance
    }
}

fn project(point: Point, start: Point, end: Point, clamp_to_segment: bool) -> Point {
    let dx = (end.x() - start.x()) as f64;
    let dy = (end.y() - start.y()) as f64;
    let denominator = dx * dx + dy * dy;
    if denominator == 0.0 {
        return start;
    }
    let position =
        ((point.x() - start.x()) as f64 * dx + (point.y() - start.y()) as f64 * dy) / denominator;
    if clamp_to_segment && position <= 0.0 {
        return start;
    }
    if clamp_to_segment && position >= 1.0 {
        return end;
    }
    Point::new(
        (start.x() as f64 + position * dx) as i64,
        (start.y() as f64 + position * dy) as i64,
    )
}
