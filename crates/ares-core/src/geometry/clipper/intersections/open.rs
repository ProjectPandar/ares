use super::super::types::{EdgeId, ExecutionConfig, OutputIndex};
use super::super::z::KernelPoint;
use super::super::{ClipOperation, Clipper};

impl Clipper {
    pub(super) fn intersect_open_edges(
        &mut self,
        edges: [EdgeId; 2],
        point: KernelPoint,
        config: ExecutionConfig,
        was_output: [bool; 2],
    ) -> bool {
        let [first, second] = edges;
        let first_edge = *self.edges.edge(first);
        let second_edge = *self.edges.edge(second);
        if first_edge.wind_delta != 0 && second_edge.wind_delta != 0 {
            return false;
        }
        if first_edge.wind_delta == 0 && second_edge.wind_delta == 0 {
            return true;
        }

        if first_edge.role == second_edge.role
            && first_edge.wind_delta != second_edge.wind_delta
            && config.operation == ClipOperation::Union
        {
            self.intersect_same_role_open(edges, point, was_output, first_edge.wind_delta == 0);
        } else if first_edge.role != second_edge.role {
            self.intersect_different_role_open(edges, point, config, was_output);
        }
        true
    }

    fn intersect_same_role_open(
        &mut self,
        [first, second]: [EdgeId; 2],
        point: KernelPoint,
        [first_was_output, second_was_output]: [bool; 2],
        first_is_open: bool,
    ) {
        if first_is_open && second_was_output {
            self.add_out_point(first, point);
            self.unassign_if_output(first, first_was_output);
        } else if !first_is_open && first_was_output {
            self.add_out_point(second, point);
            self.unassign_if_output(second, second_was_output);
        }
    }

    fn intersect_different_role_open(
        &mut self,
        [first, second]: [EdgeId; 2],
        point: KernelPoint,
        config: ExecutionConfig,
        [first_was_output, second_was_output]: [bool; 2],
    ) {
        let first_edge = *self.edges.edge(first);
        let second_edge = *self.edges.edge(second);
        if first_edge.wind_delta == 0
            && second_edge.wind_count.abs() == 1
            && (config.operation != ClipOperation::Union || second_edge.alternate_wind_count == 0)
        {
            self.add_out_point(first, point);
            self.unassign_if_output(first, first_was_output);
        } else if second_edge.wind_delta == 0
            && first_edge.wind_count.abs() == 1
            && (config.operation != ClipOperation::Union || first_edge.alternate_wind_count == 0)
        {
            self.add_out_point(second, point);
            self.unassign_if_output(second, second_was_output);
        }
    }

    fn unassign_if_output(&mut self, edge: EdgeId, was_output: bool) {
        if was_output {
            self.edges.edge_mut(edge).output = OutputIndex::Unassigned;
        }
    }
}
