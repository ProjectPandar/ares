use super::super::Clipper;
use super::super::types::{
    EdgeId, EdgeSide, OutPoint, OutPointArena, OutPointId, OutPointSlot, OutRec, OutRecId,
    OutputIndex,
};
use crate::geometry::Point;

impl OutPointArena {
    fn allocate(&mut self, out_rec: OutRecId, point: Point) -> OutPointId {
        let id = if let Some(id) = self.free_head {
            self.free_head = match self.slots[id.0] {
                OutPointSlot::Free { next } => next,
                OutPointSlot::Live(_) => unreachable!("free-list entry is live"),
            };
            id
        } else {
            let id = OutPointId(self.slots.len());
            self.slots.push(OutPointSlot::Free { next: None });
            id
        };
        self.slots[id.0] = OutPointSlot::Live(OutPoint {
            out_rec,
            point,
            next: id,
            previous: id,
        });
        id
    }

    pub(in crate::geometry::clipper) fn point(&self, id: OutPointId) -> &OutPoint {
        match &self.slots[id.0] {
            OutPointSlot::Live(point) => point,
            OutPointSlot::Free { .. } => unreachable!("freed output point is not traversable"),
        }
    }

    pub(in crate::geometry::clipper) fn point_mut(&mut self, id: OutPointId) -> &mut OutPoint {
        match &mut self.slots[id.0] {
            OutPointSlot::Live(point) => point,
            OutPointSlot::Free { .. } => unreachable!("freed output point is not traversable"),
        }
    }

    fn dispose_point(&mut self, id: OutPointId) {
        self.slots[id.0] = OutPointSlot::Free {
            next: self.free_head,
        };
        self.free_head = Some(id);
    }

    fn dispose_ring(&mut self, start: OutPointId) {
        let last = self.point(start).previous;
        let old_free = self.free_head;
        let mut point = start;
        loop {
            let next = self.point(point).next;
            self.slots[point.0] = OutPointSlot::Free {
                next: if point == last { old_free } else { Some(next) },
            };
            if point == last {
                break;
            }
            point = next;
        }
        self.free_head = Some(start);
    }

    fn clear(&mut self) {
        self.slots.clear();
        self.free_head = None;
    }
}

impl Clipper {
    pub(super) fn create_out_rec(&mut self) -> OutRecId {
        let id = OutRecId(self.out_recs.len());
        self.out_recs.push(OutRec {
            root: id,
            is_hole: false,
            is_open: false,
            first_left: None,
            points: None,
            bottom_point: None,
        });
        id
    }

    pub(in crate::geometry::clipper) fn dispose_all_out_recs(&mut self) {
        self.out_points.clear();
        self.out_recs.clear();
    }

    pub(in crate::geometry::clipper) fn add_out_point(
        &mut self,
        edge_id: EdgeId,
        point: Point,
    ) -> OutPointId {
        match self.edges.edge(edge_id).output {
            OutputIndex::Unassigned | OutputIndex::Skipped => {
                let out_rec = self.create_out_rec();
                let out_point = self.out_points.allocate(out_rec, point);
                self.out_recs[out_rec.0].points = Some(out_point);
                self.out_recs[out_rec.0].is_open = self.edges.edge(edge_id).wind_delta == 0;
                if !self.out_recs[out_rec.0].is_open {
                    self.set_hole_state(edge_id, out_rec);
                }
                self.edges.edge_mut(edge_id).output = OutputIndex::Assigned(out_rec);
                out_point
            }
            OutputIndex::Assigned(out_rec) => {
                let front = self.out_recs[out_rec.0]
                    .points
                    .expect("assigned output record has points");
                let to_front = self.edges.edge(edge_id).side == EdgeSide::Left;
                if to_front && self.out_points.point(front).point == point {
                    return front;
                }
                let back = self.out_points.point(front).previous;
                if !to_front && self.out_points.point(back).point == point {
                    return back;
                }

                let new_point = self.out_points.allocate(out_rec, point);
                self.out_points.point_mut(new_point).next = front;
                self.out_points.point_mut(new_point).previous = back;
                self.out_points.point_mut(back).next = new_point;
                self.out_points.point_mut(front).previous = new_point;
                if to_front {
                    self.out_recs[out_rec.0].points = Some(new_point);
                }
                new_point
            }
        }
    }

    pub(in crate::geometry::clipper) fn last_out_point(&self, edge_id: EdgeId) -> OutPointId {
        let OutputIndex::Assigned(out_rec) = self.edges.edge(edge_id).output else {
            unreachable!("edge without output has no last point");
        };
        let front = self.out_recs[out_rec.0]
            .points
            .expect("assigned output record has points");
        if self.edges.edge(edge_id).side == EdgeSide::Left {
            front
        } else {
            self.out_points.point(front).previous
        }
    }

    pub(super) fn duplicate_out_point(
        &mut self,
        point_id: OutPointId,
        insert_after: bool,
    ) -> OutPointId {
        let point = *self.out_points.point(point_id);
        let duplicate = self.out_points.allocate(point.out_rec, point.point);
        if insert_after {
            let next = point.next;
            self.out_points.point_mut(duplicate).next = next;
            self.out_points.point_mut(duplicate).previous = point_id;
            self.out_points.point_mut(next).previous = duplicate;
            self.out_points.point_mut(point_id).next = duplicate;
        } else {
            let previous = point.previous;
            self.out_points.point_mut(duplicate).previous = previous;
            self.out_points.point_mut(duplicate).next = point_id;
            self.out_points.point_mut(previous).next = duplicate;
            self.out_points.point_mut(point_id).previous = duplicate;
        }
        duplicate
    }

    pub(super) fn dispose_out_point(&mut self, point_id: OutPointId) {
        self.out_points.dispose_point(point_id);
    }

    pub(super) fn dispose_out_ring(&mut self, point_id: OutPointId) {
        self.out_points.dispose_ring(point_id);
    }

    pub(in crate::geometry::clipper) fn reverse_out_ring(&mut self, start: OutPointId) {
        let mut point = start;
        loop {
            let current = *self.out_points.point(point);
            self.out_points.point_mut(point).next = current.previous;
            self.out_points.point_mut(point).previous = current.next;
            point = current.next;
            if point == start {
                break;
            }
        }
    }

    pub(in crate::geometry::clipper) fn out_ring_area(&self, start: OutPointId) -> f64 {
        let mut area = 0.0;
        let mut point = start;
        loop {
            let current = self.out_points.point(point);
            let previous = self.out_points.point(current.previous);
            area += (previous.point.x() + current.point.x()) as f64
                * (previous.point.y() - current.point.y()) as f64;
            point = current.next;
            if point == start {
                return area * 0.5;
            }
        }
    }

    pub(super) fn out_point_count(&self, start: OutPointId) -> usize {
        let mut count = 0;
        let mut point = start;
        loop {
            count += 1;
            point = self.out_points.point(point).next;
            if point == start {
                return count;
            }
        }
    }

    pub(super) fn update_out_point_indices(&mut self, out_rec: OutRecId) {
        let start = self.out_recs[out_rec.0]
            .points
            .expect("output record has points");
        let mut point = start;
        loop {
            self.out_points.point_mut(point).out_rec = out_rec;
            point = self.out_points.point(point).previous;
            if point == start {
                break;
            }
        }
    }
}
