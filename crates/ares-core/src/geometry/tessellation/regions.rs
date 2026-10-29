//! glu-libtess dict.c:73–111; sweep.c:107–409, active regions.
//! Copyright (C) 1991-2000 Silicon Graphics, Inc. All Rights Reserved.
//! SGI FREE SOFTWARE LICENSE B, Version 2.0:
//! http://oss.sgi.com/projects/FreeB/
use super::{Node, Region, RightEdges, Tess, predicates};
impl Tess {
    fn edge_leq(&self, e1: usize, e2: usize) -> bool {
        let event = self.event;
        if self.dst(e1) == event {
            if self.dst(e2) == event {
                return self.edge_leq_at_event(e1, e2);
            }
            return self.sign(self.dst(e2), event, self.e(e2).org) <= 0.0;
        }
        if self.dst(e2) == event {
            return self.sign(self.dst(e1), event, self.e(e1).org) >= 0.0;
        }
        let t1 = predicates::eval(self.v(self.dst(e1)), self.v(event), self.v(self.e(e1).org));
        let t2 = predicates::eval(self.v(self.dst(e2)), self.v(event), self.v(self.e(e2).org));
        t1 >= t2
    }
    fn edge_leq_at_event(&self, e1: usize, e2: usize) -> bool {
        if self.leq(self.e(e1).org, self.e(e2).org) {
            return self.sign(self.dst(e2), self.e(e1).org, self.e(e2).org) <= 0.0;
        }
        self.sign(self.dst(e1), self.e(e2).org, self.e(e1).org) >= 0.0
    }
    pub(super) fn below(&self, r: usize) -> usize {
        self.nodes[self.nodes[self.r(r).node].prev].key
    }
    pub(super) fn above(&self, r: usize) -> usize {
        self.nodes[self.nodes[self.r(r).node].next].key
    }
    pub(super) fn insert_before(&mut self, mut node: usize, key: usize) -> usize {
        loop {
            node = self.nodes[node].prev;
            if self.nodes[node].key == 0
                || self.edge_leq(self.r(self.nodes[node].key).edge, self.r(key).edge)
            {
                break;
            }
        }
        let new = self.nodes.len();
        let next = self.nodes[node].next;
        self.nodes.push(Node {
            prev: node,
            next,
            key,
        });
        self.nodes[next].prev = new;
        self.nodes[node].next = new;
        new
    }
    pub(super) fn search(&self, key: usize) -> usize {
        let mut node = 0;
        loop {
            node = self.nodes[node].next;
            if self.nodes[node].key == 0 || self.edge_leq(key, self.r(self.nodes[node].key).edge) {
                return self.nodes[node].key;
            }
        }
    }
    pub(super) fn delete_region(&mut self, r: usize) {
        let edge = self.r(r).edge;
        self.mesh.edges[edge].region = 0;
        let node = self.r(r).node;
        let next = self.nodes[node].next;
        let prev = self.nodes[node].prev;
        self.nodes[next].prev = prev;
        self.nodes[prev].next = next;
    }
    pub(super) fn fix_upper(&mut self, r: usize, e: usize) {
        self.mesh.delete(self.r(r).edge);
        self.regions[r].fix_upper = false;
        self.regions[r].edge = e;
        self.mesh.edges[e].region = r;
    }
    pub(super) fn top_left(&mut self, mut r: usize) -> usize {
        let org = self.e(self.r(r).edge).org;
        loop {
            r = self.above(r);
            if self.e(self.r(r).edge).org != org {
                break;
            }
        }
        if self.r(r).fix_upper {
            let e = self.mesh.connect(
                self.e(self.r(self.below(r)).edge).sym,
                self.e(self.r(r).edge).lnext,
            );
            self.fix_upper(r, e);
            r = self.above(r);
        }
        r
    }
    pub(super) fn top_right(&self, mut r: usize) -> usize {
        let dst = self.dst(self.r(r).edge);
        loop {
            r = self.above(r);
            if self.dst(self.r(r).edge) != dst {
                return r;
            }
        }
    }
    pub(super) fn add_below(&mut self, above: usize, edge: usize) -> usize {
        let r = self.regions.len();
        self.regions.push(Region {
            edge,
            ..Region::default()
        });
        let node = self.insert_before(self.r(above).node, r);
        self.regions[r].node = node;
        self.mesh.edges[edge].region = r;
        r
    }
    pub(super) fn compute_winding(&mut self, r: usize) {
        self.regions[r].winding = self.r(self.above(r)).winding + self.e(self.r(r).edge).winding;
        self.regions[r].inside = (self.r(r).winding & 1) != 0;
    }
    pub(super) fn finish_region(&mut self, r: usize) {
        let e = self.r(r).edge;
        let f = self.e(e).face;
        self.mesh.faces[f].inside = self.r(r).inside;
        self.mesh.faces[f].edge = e;
        self.delete_region(r);
    }
    pub(super) fn finish_left(&mut self, first: usize, last: usize) -> usize {
        let mut previous = first;
        let mut eprev = self.r(first).edge;
        while previous != last {
            self.regions[previous].fix_upper = false;
            let r = self.below(previous);
            let e = self.r(r).edge;
            let Some(e) = self.finish_left_fix_upper(previous, r, eprev, e) else {
                break;
            };
            if self.e(eprev).onext != e {
                self.mesh.splice(self.oprev(e), e);
                self.mesh.splice(eprev, e);
            }
            self.finish_region(previous);
            eprev = self.r(r).edge;
            previous = r;
        }
        eprev
    }
    fn finish_left_fix_upper(
        &mut self,
        previous: usize,
        r: usize,
        eprev: usize,
        e: usize,
    ) -> Option<usize> {
        if self.e(e).org != self.e(eprev).org {
            if !self.r(r).fix_upper {
                self.finish_region(previous);
                return None;
            }
            let e = self.mesh.connect(self.lprev(eprev), self.e(e).sym);
            self.fix_upper(r, e);
            return Some(e);
        }
        Some(e)
    }
    pub(super) fn add_right(&mut self, up: usize, edges: RightEdges, clean: bool) {
        let RightEdges {
            first,
            last,
            mut top_left,
        } = edges;
        let mut e = first;
        loop {
            self.add_below(up, self.e(e).sym);
            e = self.e(e).onext;
            if e == last {
                break;
            }
        }
        if top_left == 0 {
            top_left = self.rprev(self.r(self.below(up)).edge);
        }
        let mut previous = up;
        let mut eprev = top_left;
        let mut first_time = true;
        loop {
            let r = self.below(previous);
            e = self.e(self.r(r).edge).sym;
            if self.e(e).org != self.e(eprev).org {
                break;
            }
            if self.e(e).onext != eprev {
                self.mesh.splice(self.oprev(e), e);
                self.mesh.splice(self.oprev(eprev), e);
            }
            self.regions[r].winding = self.r(previous).winding - self.e(e).winding;
            self.regions[r].inside = self.r(r).winding & 1 != 0;
            self.regions[previous].dirty = true;
            if !first_time && self.check_right_splice(previous) {
                self.mesh.add_winding(e, eprev);
                self.delete_region(previous);
                self.mesh.delete(eprev);
            }
            first_time = false;
            previous = r;
            eprev = e;
        }
        self.regions[previous].dirty = true;
        if clean {
            self.walk_dirty(previous);
        }
    }
}
