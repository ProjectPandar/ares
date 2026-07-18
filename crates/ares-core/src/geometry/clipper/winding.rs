use super::types::{Edge, EdgeId, ExecutionConfig};
use super::{ClipOperation, ClosedClipper, FillRule, PathRole};

impl ClosedClipper {
    pub(super) fn set_winding_count(&mut self, edge_id: EdgeId, config: ExecutionConfig) {
        let edge = *self.edges.edge(edge_id);
        let mut previous = edge.previous_in_ael;
        while let Some(id) = previous {
            let candidate = self.edges.edge(id);
            if candidate.role == edge.role && candidate.wind_delta != 0 {
                break;
            }
            previous = candidate.previous_in_ael;
        }

        let (wind_count, alternate_wind_count, scan) = self.initial_winding(edge, previous, config);
        let alternate_wind_count =
            self.accumulate_alternate_winding(edge_id, edge, config, (alternate_wind_count, scan));
        let edge = self.edges.edge_mut(edge_id);
        edge.wind_count = wind_count;
        edge.alternate_wind_count = alternate_wind_count;
    }

    fn initial_winding(
        &self,
        edge: Edge,
        previous: Option<EdgeId>,
        config: ExecutionConfig,
    ) -> (i32, i32, Option<EdgeId>) {
        let Some(previous) = previous else {
            return (first_winding(edge, config), 0, self.active_edges);
        };
        let previous_edge = *self.edges.edge(previous);
        let wind_count = if edge.wind_delta == 0 && config.operation != ClipOperation::Union {
            1
        } else if own_fill(edge, config) == FillRule::EvenOdd {
            self.even_odd_winding(edge, previous_edge)
        } else {
            nonzero_winding(edge, previous_edge)
        };
        (
            wind_count,
            previous_edge.alternate_wind_count,
            previous_edge.next_in_ael,
        )
    }

    fn even_odd_winding(&self, edge: Edge, previous: Edge) -> i32 {
        if edge.wind_delta != 0 {
            return edge.wind_delta;
        }
        let mut inside = true;
        let mut earlier = previous.previous_in_ael;
        while let Some(id) = earlier {
            let candidate = self.edges.edge(id);
            if candidate.role == previous.role && candidate.wind_delta != 0 {
                inside = !inside;
            }
            earlier = candidate.previous_in_ael;
        }
        i32::from(!inside)
    }

    fn accumulate_alternate_winding(
        &self,
        edge_id: EdgeId,
        edge: Edge,
        config: ExecutionConfig,
        (mut count, mut scan): (i32, Option<EdgeId>),
    ) -> i32 {
        let even_odd = alternate_fill(edge, config) == FillRule::EvenOdd;
        while scan != Some(edge_id) {
            let id = scan.expect("winding scan reaches inserted edge");
            let candidate = self.edges.edge(id);
            count = if even_odd && candidate.wind_delta != 0 {
                toggle_count(count)
            } else if even_odd {
                count
            } else {
                count + candidate.wind_delta
            };
            scan = candidate.next_in_ael;
        }
        count
    }

    pub(super) fn is_contributing(&self, edge_id: EdgeId, config: ExecutionConfig) -> bool {
        let edge = *self.edges.edge(edge_id);
        let primary = own_fill(edge, config);
        let alternate = alternate_fill(edge, config);
        let primary_contributes = match primary {
            FillRule::EvenOdd => edge.wind_delta != 0 || edge.wind_count == 1,
            FillRule::NonZero => edge.wind_count.abs() == 1,
            FillRule::Positive => edge.wind_count == 1,
            FillRule::Negative => edge.wind_count == -1,
        };
        if !primary_contributes {
            return false;
        }

        match config.operation {
            ClipOperation::Intersection => fill_inside(alternate, edge.alternate_wind_count),
            ClipOperation::Union => !fill_inside(alternate, edge.alternate_wind_count),
            ClipOperation::Difference => {
                if edge.role == PathRole::Subject {
                    !fill_inside(alternate, edge.alternate_wind_count)
                } else {
                    fill_inside(alternate, edge.alternate_wind_count)
                }
            }
            ClipOperation::Xor => {
                edge.wind_delta != 0 || !fill_inside(alternate, edge.alternate_wind_count)
            }
        }
    }
}

fn first_winding(edge: Edge, config: ExecutionConfig) -> i32 {
    if edge.wind_delta != 0 {
        edge.wind_delta
    } else if own_fill(edge, config) == FillRule::Negative {
        -1
    } else {
        1
    }
}

fn nonzero_winding(edge: Edge, previous: Edge) -> i32 {
    if previous.wind_count * previous.wind_delta < 0 {
        decreasing_winding(edge, previous)
    } else if edge.wind_delta == 0 {
        previous.wind_count + if previous.wind_count < 0 { -1 } else { 1 }
    } else if previous.wind_delta * edge.wind_delta < 0 {
        previous.wind_count
    } else {
        previous.wind_count + edge.wind_delta
    }
}

fn decreasing_winding(edge: Edge, previous: Edge) -> i32 {
    if previous.wind_count.abs() <= 1 {
        return if edge.wind_delta == 0 {
            1
        } else {
            edge.wind_delta
        };
    }
    if previous.wind_delta * edge.wind_delta < 0 {
        previous.wind_count
    } else {
        previous.wind_count + edge.wind_delta
    }
}

fn own_fill(edge: Edge, config: ExecutionConfig) -> FillRule {
    match edge.role {
        PathRole::Subject => config.subject_fill,
        PathRole::Clip => config.clip_fill,
    }
}

fn alternate_fill(edge: Edge, config: ExecutionConfig) -> FillRule {
    match edge.role {
        PathRole::Subject => config.clip_fill,
        PathRole::Clip => config.subject_fill,
    }
}

fn fill_inside(fill: FillRule, count: i32) -> bool {
    match fill {
        FillRule::EvenOdd | FillRule::NonZero => count != 0,
        FillRule::Positive => count > 0,
        FillRule::Negative => count < 0,
    }
}

fn toggle_count(count: i32) -> i32 {
    i32::from(count == 0)
}
