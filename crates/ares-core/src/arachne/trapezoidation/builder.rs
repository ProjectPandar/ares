use boostvoronoi::prelude::{Diagram, EdgeIndex, VertexIndex};

use crate::geometry::{Point, Polygon};

use super::{
    SkeletalTrapezoidation, TrapezoidationError,
    discretize::{discretize_parabola, discretize_point_point},
    index::{PolygonSegmentIndex, collect_segments},
    voronoi,
};
use crate::arachne::skeletal::{EdgeId, SkeletalEdge, SkeletalJoint};

impl SkeletalTrapezoidation<'_> {
    pub(super) fn construct_from_polygons(
        &mut self,
        polygons: &[Polygon],
    ) -> Result<(), TrapezoidationError> {
        let segments = collect_segments(polygons);
        let vd = voronoi::build(polygons, &segments)?;
        for cell in vd.cells() {
            let Some(range) = voronoi::cell_range(&vd, cell.id(), polygons, &segments)? else {
                continue;
            };
            let mut previous = None;
            let first_to = voronoi::vertex1(&vd, range.edge_begin)?
                .ok_or(TrapezoidationError::InvalidTopology)?;
            self.transfer_edge(
                &vd,
                range.source_start,
                first_to,
                range.edge_begin,
                &mut previous,
                range.source_start,
                range.source_end,
                polygons,
                &segments,
            )?;

            let first_vertex = vd
                .edge_get_vertex0(range.edge_begin)
                .map_err(|_| TrapezoidationError::InvalidTopology)?
                .ok_or(TrapezoidationError::InvalidTopology)?;
            let first_node = *self
                .vd_node_to_he_node
                .get(&first_vertex)
                .ok_or(TrapezoidationError::InvalidTopology)?;
            self.graph.node_mut(first_node).data.distance_to_boundary = 0;

            previous = Some(self.graph.make_rib(
                previous.ok_or(TrapezoidationError::InvalidTopology)?,
                range.source_start,
                range.source_end,
            ));

            let mut edge = voronoi::next(&vd, range.edge_begin)?;
            while edge != range.edge_end {
                let from =
                    voronoi::vertex0(&vd, edge)?.ok_or(TrapezoidationError::InvalidTopology)?;
                let to =
                    voronoi::vertex1(&vd, edge)?.ok_or(TrapezoidationError::InvalidTopology)?;
                self.transfer_edge(
                    &vd,
                    from,
                    to,
                    edge,
                    &mut previous,
                    range.source_start,
                    range.source_end,
                    polygons,
                    &segments,
                )?;

                previous = Some(self.graph.make_rib(
                    previous.ok_or(TrapezoidationError::InvalidTopology)?,
                    range.source_start,
                    range.source_end,
                ));
                edge = voronoi::next(&vd, edge)?;
            }

            let last_from = voronoi::vertex0(&vd, range.edge_end)?
                .ok_or(TrapezoidationError::InvalidTopology)?;
            self.transfer_edge(
                &vd,
                last_from,
                range.source_end,
                range.edge_end,
                &mut previous,
                range.source_start,
                range.source_end,
                polygons,
                &segments,
            )?;

            let last = previous.ok_or(TrapezoidationError::InvalidTopology)?;
            let last_node = self
                .graph
                .edge(last)
                .to
                .ok_or(TrapezoidationError::InvalidTopology)?;
            self.graph.node_mut(last_node).data.distance_to_boundary = 0;
        }
        self.separate_pointy_quad_end_nodes()?;
        self.graph.collapse_small_edges(
            self.config
                .coordinate_scale
                .checked_scale(0.02)
                .ok_or(TrapezoidationError::InvalidTopology)?,
        );
        let edges = self.graph.active_edges().collect::<Vec<_>>();
        for edge in edges {
            if self.graph.edge(edge).prev.is_none() {
                let from = self.graph.edge(edge).from.unwrap();
                self.graph.node_mut(from).incident_edge = Some(edge);
            }
        }
        Ok(())
    }

    fn make_node(&mut self, vertex: VertexIndex, point: Point) -> crate::arachne::skeletal::NodeId {
        if let Some(&node) = self.vd_node_to_he_node.get(&vertex) {
            return node;
        }
        let node = self.graph.add_node_front(SkeletalJoint::default(), point);
        self.vd_node_to_he_node.insert(vertex, node);
        node
    }

    #[expect(
        clippy::excessive_nesting,
        clippy::too_many_arguments,
        reason = "keeps the pinned transferEdge topology operation together"
    )]
    fn transfer_edge(
        &mut self,
        vd: &Diagram,
        from: Point,
        to: Point,
        vd_edge: EdgeIndex,
        previous: &mut Option<EdgeId>,
        source_start: Point,
        source_end: Point,
        polygons: &[Polygon],
        segments: &[PolygonSegmentIndex],
    ) -> Result<(), TrapezoidationError> {
        let twin_vd = voronoi::twin(vd, vd_edge)?;
        if let Some(&source_twin) = self.vd_edge_to_he_edge.get(&twin_vd) {
            let end_vertex = vd
                .edge_get_vertex1(vd_edge)
                .map_err(|_| TrapezoidationError::InvalidTopology)?
                .ok_or(TrapezoidationError::InvalidTopology)?;
            let end_node = *self
                .vd_node_to_he_node
                .get(&end_vertex)
                .ok_or(TrapezoidationError::InvalidTopology)?;
            let mut twin = source_twin;
            loop {
                let edge = self.graph.add_edge_front(SkeletalEdge::default());
                let twin_edge = self.graph.edge(twin);
                let edge_from = twin_edge.to.ok_or(TrapezoidationError::InvalidTopology)?;
                let edge_to = twin_edge.from.ok_or(TrapezoidationError::InvalidTopology)?;
                self.graph.edge_mut(edge).from = Some(edge_from);
                self.graph.edge_mut(edge).to = Some(edge_to);
                self.graph.connect_twins(edge, twin);
                self.graph.node_mut(edge_from).incident_edge = Some(edge);
                if let Some(prev) = *previous {
                    self.graph.edge_mut(edge).prev = Some(prev);
                    self.graph.edge_mut(prev).next = Some(edge);
                }
                *previous = Some(edge);
                if edge_to == end_node {
                    return Ok(());
                }
                let Some(forth_rib) = self.graph.edge(twin).prev else {
                    return Ok(());
                };
                let Some(back_rib) = self.graph.edge(forth_rib).twin else {
                    return Ok(());
                };
                let Some(previous_twin) = self.graph.edge(back_rib).prev else {
                    return Ok(());
                };
                twin = previous_twin;
                *previous = Some(self.graph.make_rib(edge, source_start, source_end));
            }
        }

        let discretized = self.discretize(vd, vd_edge, polygons, segments)?;
        if discretized.len() < 2 {
            return Err(TrapezoidationError::InvalidTopology);
        }
        let vertex0 = vd
            .edge_get_vertex0(vd_edge)
            .map_err(|_| TrapezoidationError::InvalidTopology)?
            .ok_or(TrapezoidationError::InvalidTopology)?;
        let mut v0 = if let Some(prev) = *previous {
            self.graph.edge(prev).to.unwrap()
        } else {
            self.make_node(vertex0, from)
        };
        for (index, &point) in discretized.iter().enumerate().skip(1) {
            let last = index + 1 == discretized.len();
            let v1 = if last {
                let vertex1 = vd
                    .edge_get_vertex1(vd_edge)
                    .map_err(|_| TrapezoidationError::InvalidTopology)?
                    .ok_or(TrapezoidationError::InvalidTopology)?;
                self.make_node(vertex1, to)
            } else {
                self.graph.add_node_front(SkeletalJoint::default(), point)
            };
            let edge = self.graph.add_edge_front(SkeletalEdge::default());
            self.graph.edge_mut(edge).from = Some(v0);
            self.graph.edge_mut(edge).to = Some(v1);
            self.graph.node_mut(v0).incident_edge = Some(edge);
            if let Some(prev) = *previous {
                self.graph.edge_mut(edge).prev = Some(prev);
                self.graph.edge_mut(prev).next = Some(edge);
            }
            *previous = Some(edge);
            v0 = v1;
            if !last {
                *previous = Some(self.graph.make_rib(edge, source_start, source_end));
            }
        }
        self.vd_edge_to_he_edge.insert(vd_edge, previous.unwrap());
        Ok(())
    }

    pub(super) fn separate_pointy_quad_end_nodes(&mut self) -> Result<(), TrapezoidationError> {
        let mut visited = std::collections::HashSet::new();
        let starts = self
            .graph
            .active_edges()
            .filter(|&edge| self.graph.edge(edge).prev.is_none())
            .collect::<Vec<_>>();
        for start in starts {
            let old = self.graph.edge(start).from.unwrap();
            if visited.insert(old) {
                continue;
            }
            let data = self.graph.node(old).data.clone();
            let point = self.graph.node(old).point;
            let new = self.graph.add_node(data, point);
            self.graph.node_mut(new).incident_edge = Some(start);
            self.graph.edge_mut(start).from = Some(new);
            let twin = self
                .graph
                .edge(start)
                .twin
                .ok_or(TrapezoidationError::InvalidTopology)?;
            self.graph.edge_mut(twin).to = Some(new);
        }
        Ok(())
    }
}

impl SkeletalTrapezoidation<'_> {
    fn discretize(
        &self,
        vd: &Diagram,
        edge: EdgeIndex,
        polygons: &[Polygon],
        segments: &[PolygonSegmentIndex],
    ) -> Result<Vec<Point>, TrapezoidationError> {
        let start = voronoi::vertex0(vd, edge)?.ok_or(TrapezoidationError::InvalidTopology)?;
        let end = voronoi::vertex1(vd, edge)?.ok_or(TrapezoidationError::InvalidTopology)?;
        let twin = voronoi::twin(vd, edge)?;
        let left_point = voronoi::cell_contains_point(vd, edge)?;
        let right_point = voronoi::cell_contains_point(vd, twin)?;
        if (!left_point && !right_point) || voronoi::is_secondary(vd, edge)? {
            return Ok(vec![start, end]);
        }
        if left_point != right_point {
            let point_edge = if left_point { edge } else { twin };
            let segment_edge = if left_point { twin } else { edge };
            let point = voronoi::source_point(vd, point_edge, polygons, segments)?
                .ok_or(TrapezoidationError::InvalidTopology)?;
            let segment = voronoi::source_segment(vd, segment_edge, segments)?
                .ok_or(TrapezoidationError::InvalidTopology)?;
            return Ok(discretize_parabola(
                point,
                segment,
                polygons,
                start,
                end,
                self.config.discretization_step_size,
                self.config.transitioning_angle,
            ));
        }
        let left = voronoi::source_point(vd, edge, polygons, segments)?
            .ok_or(TrapezoidationError::InvalidTopology)?;
        let right = voronoi::source_point(vd, twin, polygons, segments)?
            .ok_or(TrapezoidationError::InvalidTopology)?;
        Ok(discretize_point_point(
            left,
            right,
            start,
            end,
            self.config.discretization_step_size,
            self.config.transitioning_angle,
        ))
    }
}
