//! glu-libtess sweep.c:847–1196/1199–1361, sweep events/degenerates.
//! Copyright (C) 1991-2000 Silicon Graphics, Inc. All Rights Reserved.
//! SGI FREE SOFTWARE LICENSE B, Version 2.0:
//! http://oss.sgi.com/projects/FreeB/
use super::{
    Region, RightEdges, Tess,
    mesh::{E_HEAD, F_HEAD},
    queue::Queue,
};
impl Tess {
    fn connect_right(&mut self, mut up: usize, mut bottom_left: usize) {
        let mut top_left = self.e(bottom_left).onext;
        let lo = self.below(up);
        let eu = self.r(up).edge;
        let el = self.r(lo).edge;
        let mut degenerate = false;
        if self.dst(eu) != self.dst(el) {
            self.check_intersection(up);
        }
        if self.eq(self.e(eu).org, self.event) {
            self.mesh.splice(self.oprev(top_left), eu);
            up = self.top_left(up);
            top_left = self.r(self.below(up)).edge;
            self.finish_left(self.below(up), lo);
            degenerate = true;
        }
        if self.eq(self.e(el).org, self.event) {
            self.mesh.splice(bottom_left, self.oprev(el));
            bottom_left = self.finish_left(lo, 0);
            degenerate = true;
        }
        if degenerate {
            self.add_right(
                up,
                RightEdges {
                    first: self.e(bottom_left).onext,
                    last: top_left,
                    top_left,
                },
                true,
            );
            return;
        }
        let target = if self.leq(self.e(el).org, self.e(eu).org) {
            self.oprev(el)
        } else {
            eu
        };
        let new = self.mesh.connect(self.lprev(bottom_left), target);
        self.add_right(
            up,
            RightEdges {
                first: new,
                last: self.e(new).onext,
                top_left: self.e(new).onext,
            },
            false,
        );
        let r = self.e(self.e(new).sym).region;
        self.regions[r].fix_upper = true;
        self.walk_dirty(up);
    }
    fn connect_left_degenerate(&mut self, mut up: usize, event: usize) {
        let e = self.r(up).edge;
        if self.eq(self.e(e).org, event) {
            self.merge_vertices(e, self.v(event).edge);
            return;
        }
        if !self.eq(self.dst(e), event) {
            self.mesh.split_edge(self.e(e).sym);
            if self.r(up).fix_upper {
                self.mesh.delete(self.e(e).onext);
                self.regions[up].fix_upper = false;
            }
            self.mesh.splice(self.v(event).edge, e);
            self.sweep_event(event);
            return;
        }
        up = self.top_right(up);
        let r = self.below(up);
        let mut top_right = self.e(self.r(r).edge).sym;
        let mut top_left = self.e(top_right).onext;
        let last = top_left;
        if self.r(r).fix_upper {
            self.delete_region(r);
            self.mesh.delete(top_right);
            top_right = self.oprev(top_left);
        }
        self.mesh.splice(self.v(event).edge, top_right);
        if !self.leq(self.dst(top_left), self.e(top_left).org) {
            top_left = 0;
        }
        self.add_right(
            up,
            RightEdges {
                first: self.e(top_right).onext,
                last,
                top_left,
            },
            true,
        );
    }
    fn connect_left(&mut self, event: usize) {
        // The source uses a stack ActiveRegion as the dictionary search key.
        let up = self.search(self.e(self.v(event).edge).sym);
        let lo = self.below(up);
        let eu = self.r(up).edge;
        let el = self.r(lo).edge;
        if self.sign(self.dst(eu), event, self.e(eu).org) == 0.0 {
            self.connect_left_degenerate(up, event);
            return;
        }
        let r = if self.leq(self.dst(el), self.dst(eu)) {
            up
        } else {
            lo
        };
        if self.r(up).inside || self.r(r).fix_upper {
            let new = if r == up {
                self.mesh
                    .connect(self.e(self.v(event).edge).sym, self.e(eu).lnext)
            } else {
                let e = self.mesh.connect(self.dnext(el), self.v(event).edge);
                self.e(e).sym
            };
            if self.r(r).fix_upper {
                self.fix_upper(r, new);
            } else {
                let added = self.add_below(up, new);
                self.compute_winding(added);
            }
            self.sweep_event(event);
        } else {
            self.add_right(
                up,
                RightEdges {
                    first: self.v(event).edge,
                    last: self.v(event).edge,
                    top_left: 0,
                },
                true,
            );
        }
    }
    fn sweep_event(&mut self, event: usize) {
        self.event = event;
        let mut e = self.v(event).edge;
        while self.e(e).region == 0 {
            e = self.e(e).onext;
            if e == self.v(event).edge {
                self.connect_left(event);
                return;
            }
        }
        let up = self.top_left(self.e(e).region);
        let r = self.below(up);
        let top_left = self.r(r).edge;
        let bottom_left = self.finish_left(r, 0);
        if self.e(bottom_left).onext == top_left {
            self.connect_right(up, bottom_left);
        } else {
            self.add_right(
                up,
                RightEdges {
                    first: self.e(bottom_left).onext,
                    last: top_left,
                    top_left,
                },
                true,
            );
        }
    }
    fn add_sentinel(&mut self, t: f64) {
        let e = self.mesh.make_edge();
        let org = self.e(e).org;
        let dst = self.dst(e);
        self.mesh.vertices[org].s = 4e150;
        self.mesh.vertices[org].t = t;
        self.mesh.vertices[dst].s = -4e150;
        self.mesh.vertices[dst].t = t;
        self.event = dst;
        let r = self.regions.len();
        self.regions.push(Region {
            edge: e,
            sentinel: true,
            ..Region::default()
        });
        let node = self.insert_before(0, r);
        self.regions[r].node = node;
    }
    fn remove_degenerate_edges(&mut self) {
        let mut e = self.e(E_HEAD).next;
        while e != E_HEAD {
            let mut next = self.e(e).next;
            let mut lnext = self.e(e).lnext;
            if self.eq(self.e(e).org, self.dst(e)) && self.e(self.e(e).lnext).lnext != e {
                self.merge_vertices(lnext, e);
                self.mesh.delete(e);
                e = lnext;
                lnext = self.e(e).lnext;
            }
            if self.e(lnext).lnext == e {
                next = self.remove_degenerate_edge_pair(e, lnext, next);
            }
            e = next;
        }
    }
    fn remove_degenerate_edge_pair(&mut self, e: usize, lnext: usize, mut next: usize) -> usize {
        if lnext != e {
            if lnext == next || lnext == self.e(next).sym {
                next = self.e(next).next;
            }
            self.mesh.delete(lnext);
        }
        if e == next || e == self.e(next).sym {
            next = self.e(next).next;
        }
        self.mesh.delete(e);
        next
    }
    fn remove_degenerate_faces(&mut self) {
        let mut f = self.f(F_HEAD).next;
        while f != F_HEAD {
            let next = self.f(f).next;
            let e = self.f(f).edge;
            if self.e(self.e(e).lnext).lnext == e {
                self.mesh.add_winding(self.e(e).onext, e);
                self.mesh.delete(e);
            }
            f = next;
        }
    }
    fn merge_event_vertices(&mut self, v: usize) {
        loop {
            let next = self.queue.minimum(&self.mesh);
            if next == 0 || !self.eq(next, v) {
                break;
            }
            let next = self.queue.extract(&self.mesh);
            self.merge_vertices(self.v(v).edge, self.v(next).edge);
        }
    }
    pub(super) fn compute_interior(&mut self) {
        self.remove_degenerate_edges();
        self.queue = Queue::new(&mut self.mesh);
        self.add_sentinel(-4e150);
        self.add_sentinel(4e150);
        loop {
            let v = self.queue.extract(&self.mesh);
            if v == 0 {
                break;
            }
            self.merge_event_vertices(v);
            self.sweep_event(v);
        }
        self.event = self.e(self.r(self.nodes[self.nodes[0].next].key).edge).org;
        while self.nodes[0].next != 0 {
            let r = self.nodes[self.nodes[0].next].key;
            debug_assert!(self.r(r).sentinel || self.r(r).fix_upper);
            debug_assert_eq!(self.r(r).winding, 0);
            self.delete_region(r);
        }
        self.remove_degenerate_faces();
    }
}
