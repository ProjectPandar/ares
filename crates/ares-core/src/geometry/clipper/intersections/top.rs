use super::super::Clipper;
use super::super::predicates::{slopes_equal_four, top_x};
use super::super::types::{Edge, EdgeId, ExecutionConfig, Join, OutPointId, OutputIndex};
use crate::geometry::Point;

impl Clipper {
    pub(in crate::geometry::clipper) fn process_edges_at_top(
        &mut self,
        top_y: i64,
        config: ExecutionConfig,
    ) {
        let mut edge = self.active_edges;
        while let Some(current) = edge {
            let snapshot = *self.edges.edge(current);
            edge = self.process_top_edge(current, snapshot, top_y, config);
        }

        self.prepare_strict_maxima();
        self.process_horizontals(config);
        self.clear_strict_maxima();
        let mut edge = self.active_edges;
        while let Some(current) = edge {
            let snapshot = *self.edges.edge(current);
            if snapshot.top.y() == top_y && snapshot.next_in_lml.is_some() {
                let output = matches!(snapshot.output, OutputIndex::Assigned(_))
                    .then(|| self.add_out_point(current, snapshot.top));
                let promoted = self.update_edge_into_ael(current);
                self.join_promoted_edge(promoted, output);
                edge = self.edges.edge(promoted).next_in_ael;
            } else {
                edge = snapshot.next_in_ael;
            }
        }
    }

    fn process_top_edge(
        &mut self,
        current: EdgeId,
        snapshot: Edge,
        top_y: i64,
        config: ExecutionConfig,
    ) -> Option<EdgeId> {
        let maxima = snapshot.top.y() == top_y
            && snapshot.next_in_lml.is_none()
            && self
                .maxima_pair_ex(current)
                .is_none_or(|pair| !self.edges.edge(pair).is_horizontal());
        if maxima {
            self.collect_strict_maximum(current);
            let previous = snapshot.previous_in_ael;
            self.do_maxima(current, config);
            return previous
                .and_then(|id| self.edges.edge(id).next_in_ael)
                .or(self.active_edges.filter(|_| previous.is_none()));
        }
        let horizontal_promotion = snapshot
            .next_in_lml
            .filter(|&next| snapshot.top.y() == top_y && self.edges.edge(next).is_horizontal());
        if horizontal_promotion.is_some() {
            let promoted = self.update_edge_into_ael(current);
            if matches!(self.edges.edge(promoted).output, OutputIndex::Assigned(_)) {
                self.add_out_point(promoted, self.edges.edge(promoted).bottom);
            }
            self.add_edge_to_sel(promoted);
            self.join_strict_top_touch(promoted);
            return self.edges.edge(promoted).next_in_ael;
        }
        let x = top_x(snapshot, top_y);
        self.edges.edge_mut(current).current = Point::new(x, top_y);
        self.join_strict_top_touch(current);
        snapshot.next_in_ael
    }

    fn maxima_pair_ex(&self, edge: EdgeId) -> Option<EdgeId> {
        let edge_state = *self.edges.edge(edge);
        let next = self.edges.edge(edge_state.next);
        let pair = if next.top == edge_state.top && next.next_in_lml.is_none() {
            Some(edge_state.next)
        } else {
            let previous = self.edges.edge(edge_state.previous);
            (previous.top == edge_state.top && previous.next_in_lml.is_none())
                .then_some(edge_state.previous)
        }?;
        let pair_edge = self.edges.edge(pair);
        if pair_edge.output == OutputIndex::Skipped
            || pair_edge.next_in_ael == pair_edge.previous_in_ael && !pair_edge.is_horizontal()
        {
            None
        } else {
            Some(pair)
        }
    }

    fn do_maxima(&mut self, edge: EdgeId, config: ExecutionConfig) {
        let point = self.edges.edge(edge).top;
        let Some(pair) = self.maxima_pair_ex(edge) else {
            if matches!(self.edges.edge(edge).output, OutputIndex::Assigned(_)) {
                self.add_out_point(edge, point);
            }
            self.delete_from_ael(edge);
            return;
        };
        while let Some(next) = self.edges.edge(edge).next_in_ael {
            if next == pair {
                break;
            }
            self.intersect_edges(edge, next, point, config);
            self.swap_positions_in_ael(edge, next);
        }
        let first_output = self.edges.edge(edge).output;
        let second_output = self.edges.edge(pair).output;
        if first_output == OutputIndex::Unassigned && second_output == OutputIndex::Unassigned {
            self.delete_from_ael(edge);
            self.delete_from_ael(pair);
        } else if matches!(first_output, OutputIndex::Assigned(_))
            && matches!(second_output, OutputIndex::Assigned(_))
        {
            self.add_local_max_polygon(edge, pair, point);
            self.delete_from_ael(edge);
            self.delete_from_ael(pair);
        } else if self.edges.edge(edge).wind_delta == 0 {
            if matches!(self.edges.edge(edge).output, OutputIndex::Assigned(_)) {
                self.add_out_point(edge, point);
                self.edges.edge_mut(edge).output = OutputIndex::Unassigned;
            }
            self.delete_from_ael(edge);
            if matches!(self.edges.edge(pair).output, OutputIndex::Assigned(_)) {
                self.add_out_point(pair, point);
                self.edges.edge_mut(pair).output = OutputIndex::Unassigned;
            }
            self.delete_from_ael(pair);
        } else {
            unreachable!("maxima output state is paired");
        }
    }

    fn join_promoted_edge(&mut self, edge: EdgeId, output: Option<OutPointId>) {
        let Some(output) = output else { return };
        let snapshot = *self.edges.edge(edge);
        for neighbour in [snapshot.previous_in_ael, snapshot.next_in_ael]
            .into_iter()
            .flatten()
        {
            let other = *self.edges.edge(neighbour);
            if other.current == snapshot.bottom
                && matches!(other.output, OutputIndex::Assigned(_))
                && other.current.y() > other.top.y()
                && slopes_equal_four(
                    snapshot.current,
                    snapshot.top,
                    other.current,
                    other.top,
                    self.use_full_range,
                )
                && snapshot.wind_delta != 0
                && other.wind_delta != 0
            {
                let second = self.add_out_point(neighbour, snapshot.bottom);
                self.joins.push(Join {
                    first: output,
                    second,
                    offset: snapshot.top,
                });
                break;
            }
        }
    }
}
