use std::{cell::RefCell, rc::Rc};

use crate::{
    arachne::{extrusion_line::ExtrusionJunction, skeletal::EdgeId},
    geometry::Point,
};

use super::{super::SkeletalTrapezoidation, toolpaths::SegmentConditions};

impl SkeletalTrapezoidation<'_> {
    pub(super) fn generate_junctions(&mut self) {
        let edges = self.graph.active_edges().collect::<Vec<_>>();
        for edge in edges {
            let Some(junctions) = self.junctions_for_edge(edge) else {
                continue;
            };
            let storage = Rc::new(RefCell::new(junctions));
            self.graph
                .edge_mut(edge)
                .data
                .set_extrusion_junctions(&storage);
            self.extrusion_junction_storage.push(storage);
        }
    }

    pub(super) fn get_quad_max_r_edge_to(&self, quad_start: EdgeId) -> EdgeId {
        assert!(self.graph.edge(quad_start).prev.is_none());
        let mut selected = None;
        let mut max_radius = -1;
        let mut current = Some(quad_start);
        while let Some(edge) = current {
            let to = self.graph.edge(edge).to.unwrap();
            let radius = self.graph.node(to).data.distance_to_boundary;
            if radius > max_radius {
                max_radius = radius;
                selected = Some(edge);
            }
            current = self.graph.edge(edge).next;
        }
        let mut selected = selected.unwrap();
        let half_edge = self.graph.edge(selected);
        if half_edge.next.is_none() {
            let from = half_edge.from.unwrap();
            let to = half_edge.to.unwrap();
            let epsilon = self.config.coordinate_scale.checked_scale(0.005).unwrap();
            if self.graph.node(to).data.distance_to_boundary - epsilon
                < self.graph.node(from).data.distance_to_boundary
            {
                selected = half_edge.prev.unwrap();
            }
        }
        assert!(self.graph.edge(selected).next.is_some());
        selected
    }

    pub(super) fn connect_even_quad_junctions(&mut self, quad_start: EdgeId, force_new_path: bool) {
        let edge_to_peak = self.get_quad_max_r_edge_to(quad_start);
        let peak = self.graph.edge(edge_to_peak).to.unwrap();
        assert_eq!(self.graph.node(peak).data.bead_count % 2, 0);
        let edge_from_peak = self.graph.edge(edge_to_peak).next.unwrap();
        let opposite_edge = self.graph.edge(edge_from_peak).twin.unwrap();
        self.ensure_edge_junction_storage(edge_to_peak);
        self.ensure_edge_junction_storage(opposite_edge);
        let mut from_junctions = self.edge_junctions(edge_to_peak);
        if let Some(previous) = self.graph.edge(edge_to_peak).prev {
            from_junctions =
                append_adjacent_junctions(from_junctions, self.edge_junctions(previous));
            assert!(self.graph.edge(previous).prev.is_none());
        }
        let mut to_junctions = self.edge_junctions(opposite_edge);
        if let Some(next) = self.graph.edge(edge_from_peak).next {
            let adjacent = self.graph.edge(next).twin.unwrap();
            to_junctions = append_adjacent_junctions(to_junctions, self.edge_junctions(adjacent));
            assert!(self.graph.edge(next).next.is_none());
        }
        self.connect_junction_vectors(
            &from_junctions,
            &to_junctions,
            SegmentConditions {
                is_odd: false,
                force_new_path,
                from_is_three_way: false,
                to_is_three_way: false,
            },
        );
    }

    fn ensure_edge_junction_storage(&mut self, edge: EdgeId) {
        if self.graph.edge(edge).data.extrusion_junctions().is_some() {
            return;
        }
        let storage = Rc::new(RefCell::new(Vec::new()));
        self.graph
            .edge_mut(edge)
            .data
            .set_extrusion_junctions(&storage);
        self.extrusion_junction_storage.push(storage);
    }

    pub(super) fn connect_junction_pair(
        &mut self,
        from_edge: EdgeId,
        to_edge: EdgeId,
        conditions: SegmentConditions,
    ) {
        let from_junctions = self.edge_junctions(from_edge);
        let to_junctions = self.edge_junctions(to_edge);
        self.connect_junction_vectors(&from_junctions, &to_junctions, conditions);
    }

    fn edge_junctions(&self, edge: EdgeId) -> Vec<ExtrusionJunction> {
        self.graph
            .edge(edge)
            .data
            .extrusion_junctions()
            .unwrap()
            .borrow()
            .clone()
    }

    fn connect_junction_vectors(
        &mut self,
        from_junctions: &[ExtrusionJunction],
        to_junctions: &[ExtrusionJunction],
        conditions: SegmentConditions,
    ) {
        assert!(from_junctions.len().abs_diff(to_junctions.len()) <= 1);
        let segment_count = from_junctions.len().min(to_junctions.len());
        for reverse_index in 0..segment_count {
            let from = from_junctions[from_junctions.len() - 1 - reverse_index];
            let to = to_junctions[to_junctions.len() - 1 - reverse_index];
            assert_eq!(from.perimeter_index, to.perimeter_index);
            self.add_toolpath_segment(from, to, conditions);
        }
    }

    fn junctions_for_edge(
        &self,
        edge: crate::arachne::skeletal::EdgeId,
    ) -> Option<Vec<ExtrusionJunction>> {
        let half_edge = self.graph.edge(edge);
        let from = half_edge.from.unwrap();
        let to = half_edge.to.unwrap();
        let end_radius = self.graph.node(from).data.distance_to_boundary;
        let start_radius = self.graph.node(to).data.distance_to_boundary;
        let from_bead_count = self.graph.node(from).data.bead_count;
        let to_bead_count = self.graph.node(to).data.bead_count;
        if end_radius >= start_radius || (from_bead_count == to_bead_count && from_bead_count >= 0)
        {
            return None;
        }
        let beading_storage = self.graph.node(to).data.beading().unwrap();
        let beading = beading_storage.borrow();
        assert!(beading.beading.total_thickness >= start_radius * 2);
        let mut junctions = Vec::new();
        let locations = &beading.beading.toolpath_locations;
        let mut junction_index = (locations.len().max(1) - 1) / 2;
        while junction_index < locations.len() && locations[junction_index] > start_radius + 1 {
            if junction_index == 0 {
                junction_index = locations.len();
                break;
            }
            junction_index -= 1;
        }
        let snap_distance = self.config.coordinate_scale.checked_scale(0.005).unwrap();
        if junction_index + 1 < locations.len()
            && locations[junction_index + 1] <= start_radius + snap_distance
            && beading.beading.total_thickness < start_radius + snap_distance
        {
            junction_index += 1;
        }
        while junction_index < locations.len() {
            let bead_radius = locations[junction_index];
            if bead_radius < end_radius {
                break;
            }
            let point = if bead_radius > start_radius - snap_distance {
                self.graph.node(to).point
            } else {
                interpolate_point(
                    self.graph.node(to).point,
                    self.graph.node(from).point,
                    bead_radius - start_radius,
                    end_radius - start_radius,
                )
            };
            junctions.push(ExtrusionJunction::new(
                point,
                beading.beading.bead_widths[junction_index],
                junction_index,
            ));
            if junction_index == 0 {
                break;
            }
            junction_index -= 1;
        }
        Some(junctions)
    }
}

fn interpolate_point(start: Point, end: Point, numerator: i64, denominator: i64) -> Point {
    let coordinate = |start: i64, end: i64| {
        start + (((end - start) as i128 * numerator as i128) / denominator as i128) as i64
    };
    Point::new(
        coordinate(start.x(), end.x()),
        coordinate(start.y(), end.y()),
    )
}

#[cfg(test)]
mod tests;

fn append_adjacent_junctions(
    mut junctions: Vec<ExtrusionJunction>,
    adjacent: Vec<ExtrusionJunction>,
) -> Vec<ExtrusionJunction> {
    while junctions
        .last()
        .zip(adjacent.first())
        .is_some_and(|(current, next)| current.perimeter_index <= next.perimeter_index)
    {
        junctions.pop();
    }
    junctions.extend(adjacent);
    junctions
}
