use super::{ClipOperation, FillRule, PathRole, predicates::get_dx, z::KernelPoint};
#[cfg(test)]
use crate::geometry::Point;
use crate::geometry::{Polygon, Polyline};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct EdgeId(pub(super) usize);

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct OutRecId(pub(super) usize);

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct OutPointId(pub(super) usize);

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct PolyNodeId(pub(super) usize);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum EdgeSide {
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum OutputIndex {
    Unassigned,
    Skipped,
    Assigned(OutRecId),
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Edge {
    pub(super) current: KernelPoint,
    pub(super) bottom: KernelPoint,
    pub(super) top: KernelPoint,
    pub(super) dx: f64,
    pub(super) role: PathRole,
    pub(super) side: EdgeSide,
    pub(super) previous: EdgeId,
    pub(super) next: EdgeId,
    pub(super) next_in_lml: Option<EdgeId>,
    pub(super) previous_in_ael: Option<EdgeId>,
    pub(super) next_in_ael: Option<EdgeId>,
    pub(super) previous_in_sel: Option<EdgeId>,
    pub(super) next_in_sel: Option<EdgeId>,
    pub(super) wind_delta: i32,
    pub(super) wind_count: i32,
    pub(super) alternate_wind_count: i32,
    pub(super) output: OutputIndex,
}

impl Edge {
    pub(super) fn new(
        current: KernelPoint,
        role: PathRole,
        previous: EdgeId,
        next: EdgeId,
    ) -> Self {
        Self {
            current,
            bottom: current,
            top: current,
            dx: 0.0,
            role,
            side: EdgeSide::Left,
            previous,
            next,
            next_in_lml: None,
            previous_in_ael: None,
            next_in_ael: None,
            previous_in_sel: None,
            next_in_sel: None,
            wind_delta: 0,
            wind_count: 0,
            alternate_wind_count: 0,
            output: OutputIndex::Unassigned,
        }
    }

    pub(super) fn initialize_direction(&mut self, next: KernelPoint) {
        if self.current.y() >= next.y() {
            self.bottom = self.current;
            self.top = next;
        } else {
            self.top = self.current;
            self.bottom = next;
        }
        self.dx = get_dx(self.bottom, self.top);
    }

    pub(super) fn is_horizontal(self) -> bool {
        self.bottom.y() == self.top.y()
    }
}

#[derive(Debug, Default)]
pub(super) struct EdgeArena {
    slots: Vec<Option<Edge>>,
}

impl EdgeArena {
    pub(super) fn len(&self) -> usize {
        self.slots.len()
    }

    pub(super) fn clear(&mut self) {
        self.slots.clear();
    }

    pub(super) fn truncate(&mut self, len: usize) {
        self.slots.truncate(len);
    }

    pub(super) fn push(&mut self, edge: Edge) {
        self.slots.push(Some(edge));
    }

    pub(super) fn edge(&self, id: EdgeId) -> &Edge {
        self.slots[id.0]
            .as_ref()
            .expect("removed edge is not traversable")
    }

    pub(super) fn edge_mut(&mut self, id: EdgeId) -> &mut Edge {
        self.slots[id.0]
            .as_mut()
            .expect("removed edge is not traversable")
    }

    pub(super) fn role(&self, id: EdgeId) -> PathRole {
        self.edge(id).role
    }

    pub(super) fn remove(&mut self, id: EdgeId) -> EdgeId {
        let edge = *self.edge(id);
        self.edge_mut(edge.previous).next = edge.next;
        self.edge_mut(edge.next).previous = edge.previous;
        self.slots[id.0] = None;
        edge.next
    }

    #[cfg(test)]
    pub(super) fn snapshot(&self) -> Vec<EdgeSnapshot> {
        self.slots
            .iter()
            .map(|slot| match slot {
                Some(edge) => EdgeSnapshot {
                    removed: false,
                    current: Some(edge.current.xy),
                    bottom: Some(edge.bottom.xy),
                    top: Some(edge.top.xy),
                    dx: Some(edge.dx),
                    wind_delta: Some(edge.wind_delta),
                    previous: Some(edge.previous.0),
                    next: Some(edge.next.0),
                    next_in_lml: edge.next_in_lml.map(|id| id.0),
                },
                None => EdgeSnapshot {
                    removed: true,
                    current: None,
                    bottom: None,
                    top: None,
                    dx: None,
                    wind_delta: None,
                    previous: None,
                    next: None,
                    next_in_lml: None,
                },
            })
            .collect()
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct LocalMinimum {
    pub(super) y: i64,
    pub(super) left: Option<EdgeId>,
    pub(super) right: Option<EdgeId>,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct IntersectionNode {
    pub(super) first: EdgeId,
    pub(super) second: EdgeId,
    pub(super) point: KernelPoint,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct OutRec {
    pub(super) root: OutRecId,
    pub(super) is_hole: bool,
    pub(super) is_open: bool,
    pub(super) first_left: Option<OutRecId>,
    pub(super) points: Option<OutPointId>,
    pub(super) bottom_point: Option<OutPointId>,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct OutPoint {
    pub(super) out_rec: OutRecId,
    pub(super) point: KernelPoint,
    pub(super) next: OutPointId,
    pub(super) previous: OutPointId,
}

#[derive(Debug, Eq, PartialEq)]
pub(super) enum PolyNodeContour {
    Closed(Polygon),
    Open(Polyline),
}

#[derive(Debug, Eq, PartialEq)]
pub(super) struct PolyNodeRecord {
    pub(super) parent: Option<PolyNodeId>,
    pub(super) children: Vec<PolyNodeId>,
    pub(super) contour: Option<PolyNodeContour>,
    pub(super) z: Option<Vec<i64>>,
}

#[derive(Clone, Copy, Debug)]
pub(super) enum OutPointSlot {
    Live(OutPoint),
    Free { next: Option<OutPointId> },
}

#[derive(Debug, Default)]
pub(super) struct OutPointArena {
    pub(super) slots: Vec<OutPointSlot>,
    pub(super) free_head: Option<OutPointId>,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Join {
    pub(super) first: OutPointId,
    pub(super) second: OutPointId,
    pub(super) offset: KernelPoint,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct GhostJoin {
    pub(super) point: OutPointId,
    pub(super) offset: KernelPoint,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct ExecutionConfig {
    pub(super) operation: ClipOperation,
    pub(super) subject_fill: FillRule,
    pub(super) clip_fill: FillRule,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct EdgeSnapshot {
    pub(crate) removed: bool,
    pub(crate) current: Option<Point>,
    pub(crate) bottom: Option<Point>,
    pub(crate) top: Option<Point>,
    pub(crate) dx: Option<f64>,
    pub(crate) wind_delta: Option<i32>,
    pub(crate) previous: Option<usize>,
    pub(crate) next: Option<usize>,
    pub(crate) next_in_lml: Option<usize>,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LocalMinimumSnapshot {
    pub(crate) y: i64,
    pub(crate) left: Option<usize>,
    pub(crate) right: Option<usize>,
}

#[cfg(test)]
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct InputSnapshot {
    pub(crate) use_full_range: bool,
    pub(crate) edges: Vec<EdgeSnapshot>,
    pub(crate) minima: Vec<LocalMinimumSnapshot>,
}
