use std::{cell::RefCell, rc::Rc};

use crate::{arachne::extrusion_line::ExtrusionJunction, geometry::Point};

use super::super::SkeletalTrapezoidation;

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
