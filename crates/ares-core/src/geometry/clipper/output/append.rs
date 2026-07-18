use super::super::ClosedClipper;
use super::super::predicates::{slopes_equal_four, top_x};
use super::super::types::{EdgeId, EdgeSide, Join, OutPointId, OutRecId, OutputIndex};
use crate::geometry::Point;

impl ClosedClipper {
    pub(in crate::geometry::clipper) fn add_local_min_polygon(
        &mut self,
        first: EdgeId,
        second: EdgeId,
        point: Point,
    ) -> OutPointId {
        let first_edge = *self.edges.edge(first);
        let second_edge = *self.edges.edge(second);
        let (result, output_edge, previous) =
            if second_edge.is_horizontal() || first_edge.dx > second_edge.dx {
                let result = self.add_out_point(first, point);
                let output = self.edges.edge(first).output;
                self.edges.edge_mut(second).output = output;
                self.edges.edge_mut(first).side = EdgeSide::Left;
                self.edges.edge_mut(second).side = EdgeSide::Right;
                let previous = if first_edge.previous_in_ael == Some(second) {
                    second_edge.previous_in_ael
                } else {
                    first_edge.previous_in_ael
                };
                (result, first, previous)
            } else {
                let result = self.add_out_point(second, point);
                let output = self.edges.edge(second).output;
                self.edges.edge_mut(first).output = output;
                self.edges.edge_mut(first).side = EdgeSide::Right;
                self.edges.edge_mut(second).side = EdgeSide::Left;
                let previous = if second_edge.previous_in_ael == Some(first) {
                    first_edge.previous_in_ael
                } else {
                    second_edge.previous_in_ael
                };
                (result, second, previous)
            };

        if let Some(previous) = previous {
            self.join_local_minimum_predecessor(previous, output_edge, result, point);
        }
        result
    }

    fn join_local_minimum_predecessor(
        &mut self,
        previous: EdgeId,
        edge: EdgeId,
        result: OutPointId,
        point: Point,
    ) {
        let previous_edge = *self.edges.edge(previous);
        let edge = *self.edges.edge(edge);
        if !matches!(previous_edge.output, OutputIndex::Assigned(_))
            || previous_edge.top.y() >= point.y()
            || edge.top.y() >= point.y()
        {
            return;
        }
        let previous_x = top_x(previous_edge, point.y());
        let edge_x = top_x(edge, point.y());
        if previous_x == edge_x
            && edge.wind_delta != 0
            && previous_edge.wind_delta != 0
            && slopes_equal_four(
                Point::new(previous_x, point.y()),
                previous_edge.top,
                Point::new(edge_x, point.y()),
                edge.top,
                self.use_full_range,
            )
        {
            let previous_point = self.add_out_point(previous, point);
            self.joins.push(Join {
                first: result,
                second: previous_point,
                offset: edge.top,
            });
        }
    }

    pub(in crate::geometry::clipper) fn add_local_max_polygon(
        &mut self,
        first: EdgeId,
        second: EdgeId,
        point: Point,
    ) {
        self.add_out_point(first, point);
        if self.edges.edge(second).wind_delta == 0 {
            self.add_out_point(second, point);
        }
        let first_output = self.edges.edge(first).output;
        let second_output = self.edges.edge(second).output;
        if first_output == second_output {
            self.edges.edge_mut(first).output = OutputIndex::Unassigned;
            self.edges.edge_mut(second).output = OutputIndex::Unassigned;
        } else {
            let (OutputIndex::Assigned(first_rec), OutputIndex::Assigned(second_rec)) =
                (first_output, second_output)
            else {
                unreachable!("local maximum joins assigned outputs");
            };
            if first_rec < second_rec {
                self.append_polygon(first, second);
            } else {
                self.append_polygon(second, first);
            }
        }
    }

    fn append_polygon(&mut self, first_edge: EdgeId, second_edge: EdgeId) {
        let OutputIndex::Assigned(first_rec) = self.edges.edge(first_edge).output else {
            unreachable!("first append edge has output");
        };
        let OutputIndex::Assigned(second_rec) = self.edges.edge(second_edge).output else {
            unreachable!("second append edge has output");
        };
        let hole_state = if self.out_rec_right_of(first_rec, second_rec) {
            second_rec
        } else if self.out_rec_right_of(second_rec, first_rec) {
            first_rec
        } else {
            self.get_lowermost_rec(first_rec, second_rec)
        };

        let first_left = self.out_recs[first_rec.0]
            .points
            .expect("first output has points");
        let first_right = self.out_points.point(first_left).previous;
        let second_left = self.out_recs[second_rec.0]
            .points
            .expect("second output has points");
        let second_right = self.out_points.point(second_left).previous;
        let first_side = self.edges.edge(first_edge).side;
        let second_side = self.edges.edge(second_edge).side;

        match (first_side, second_side) {
            (EdgeSide::Left, EdgeSide::Left) => {
                self.reverse_out_ring(second_left);
                self.link_output_points(second_left, first_left);
                self.link_output_points(first_right, second_right);
                self.out_recs[first_rec.0].points = Some(second_right);
            }
            (EdgeSide::Left, EdgeSide::Right) => {
                self.link_output_points(second_right, first_left);
                self.link_output_points(first_right, second_left);
                self.out_recs[first_rec.0].points = Some(second_left);
            }
            (EdgeSide::Right, EdgeSide::Right) => {
                self.reverse_out_ring(second_left);
                self.link_output_points(first_right, second_right);
                self.link_output_points(second_left, first_left);
            }
            (EdgeSide::Right, EdgeSide::Left) => {
                self.link_output_points(first_right, second_left);
                self.link_output_points(second_right, first_left);
            }
        }

        self.out_recs[first_rec.0].bottom_point = None;
        if hole_state == second_rec {
            let second_first_left = self.out_recs[second_rec.0].first_left;
            if second_first_left != Some(first_rec) {
                self.out_recs[first_rec.0].first_left = second_first_left;
            }
            self.out_recs[first_rec.0].is_hole = self.out_recs[second_rec.0].is_hole;
        }
        self.out_recs[second_rec.0].points = None;
        self.out_recs[second_rec.0].bottom_point = None;
        self.out_recs[second_rec.0].first_left = Some(first_rec);

        self.edges.edge_mut(first_edge).output = OutputIndex::Unassigned;
        self.edges.edge_mut(second_edge).output = OutputIndex::Unassigned;
        let mut edge = self.active_edges;
        while let Some(active) = edge {
            if self.edges.edge(active).output == OutputIndex::Assigned(second_rec) {
                self.edges.edge_mut(active).output = OutputIndex::Assigned(first_rec);
                self.edges.edge_mut(active).side = first_side;
                break;
            }
            edge = self.edges.edge(active).next_in_ael;
        }
        self.out_recs[second_rec.0].root = first_rec;
    }

    fn link_output_points(&mut self, first: OutPointId, second: OutPointId) {
        self.out_points.point_mut(first).next = second;
        self.out_points.point_mut(second).previous = first;
    }

    pub(super) fn replace_out_rec_root(&mut self, obsolete: OutRecId, root: OutRecId) {
        self.out_recs[obsolete.0].root = root;
    }
}
