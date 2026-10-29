//! glu-libtess sweep.c:454–497/606–758, intersections and combine.
//! Copyright (C) 1991-2000 Silicon Graphics, Inc. All Rights Reserved.
//! SGI FREE SOFTWARE LICENSE B, Version 2.0:
//! http://oss.sgi.com/projects/FreeB/
use super::{RightEdges, Tess, predicates};
struct IntersectData {
    org_up: usize,
    dst_up: usize,
    org_lo: usize,
    dst_lo: usize,
}
impl Tess {
    fn vertex_weights(&mut self, isect: usize, org: usize, dst: usize) {
        let i = self.v(isect);
        let o = self.v(org);
        let d = self.v(dst);
        let t1 = (o.s - i.s).abs() + (o.t - i.t).abs();
        let t2 = (d.s - i.s).abs() + (d.t - i.t).abs();
        // The source stores GLfloat weights before its GLdouble accumulation.
        let w0 = (0.5 * t2 / (t1 + t2)) as f32;
        let w1 = (0.5 * t1 / (t1 + t2)) as f32;
        for axis in 0..3 {
            self.mesh.vertices[isect].coords[axis] +=
                (w0 as f64) * o.coords[axis] + (w1 as f64) * d.coords[axis];
        }
    }
    fn intersect_data(&mut self, isect: usize, data: IntersectData) {
        let IntersectData {
            org_up,
            dst_up,
            org_lo,
            dst_lo,
        } = data;
        self.mesh.vertices[isect].coords = [0.0; 3];
        self.vertex_weights(isect, org_up, dst_up);
        self.vertex_weights(isect, org_lo, dst_lo);
        self.mesh.vertices[isect].data = Some(self.v(isect).coords);
    }
    pub(super) fn check_intersection(&mut self, mut up: usize) -> bool {
        let mut lo = self.below(up);
        let mut eu = self.r(up).edge;
        let mut el = self.r(lo).edge;
        let ou = self.e(eu).org;
        let ol = self.e(el).org;
        let du = self.dst(eu);
        let dl = self.dst(el);
        if ou == ol {
            return false;
        }
        let min_up = if self.v(ou).t <= self.v(du).t {
            self.v(ou).t
        } else {
            self.v(du).t
        };
        let max_lo = if self.v(ol).t >= self.v(dl).t {
            self.v(ol).t
        } else {
            self.v(dl).t
        };
        if min_up > max_lo {
            return false;
        }
        if self.leq(ou, ol) {
            if self.sign(dl, ou, ol) > 0.0 {
                return false;
            }
        } else if self.sign(du, ol, ou) < 0.0 {
            return false;
        }
        let mut isect = predicates::intersect(self.v(du), self.v(ou), self.v(dl), self.v(ol));
        if predicates::leq(isect, self.v(self.event)) {
            isect.s = self.v(self.event).s;
            isect.t = self.v(self.event).t;
        }
        let org_min = if self.leq(ou, ol) { ou } else { ol };
        if predicates::leq(self.v(org_min), isect) {
            isect.s = self.v(org_min).s;
            isect.t = self.v(org_min).t;
        }
        if predicates::eq(isect, self.v(ou)) || predicates::eq(isect, self.v(ol)) {
            self.check_right_splice(up);
            return false;
        }
        if (!self.eq(du, self.event)
            && predicates::sign(self.v(du), self.v(self.event), isect) >= 0.0)
            || (!self.eq(dl, self.event)
                && predicates::sign(self.v(dl), self.v(self.event), isect) <= 0.0)
        {
            if dl == self.event {
                self.mesh.split_edge(self.e(eu).sym);
                self.mesh.splice(self.e(el).sym, eu);
                up = self.top_left(up);
                eu = self.r(self.below(up)).edge;
                self.finish_left(self.below(up), lo);
                self.add_right(
                    up,
                    RightEdges {
                        first: self.oprev(eu),
                        last: eu,
                        top_left: eu,
                    },
                    true,
                );
                return true;
            }
            if du == self.event {
                self.mesh.split_edge(self.e(el).sym);
                self.mesh.splice(self.e(eu).lnext, self.oprev(el));
                lo = up;
                up = self.top_right(up);
                let e = self.rprev(self.r(self.below(up)).edge);
                self.regions[lo].edge = self.oprev(el);
                el = self.finish_left(lo, 0);
                self.add_right(
                    up,
                    RightEdges {
                        first: self.e(el).onext,
                        last: self.rprev(eu),
                        top_left: e,
                    },
                    true,
                );
                return true;
            }
            if predicates::sign(self.v(du), self.v(self.event), isect) >= 0.0 {
                let above = self.above(up);
                self.regions[above].dirty = true;
                self.regions[up].dirty = true;
                self.mesh.split_edge(self.e(eu).sym);
                let v = self.e(eu).org;
                self.mesh.vertices[v].s = self.v(self.event).s;
                self.mesh.vertices[v].t = self.v(self.event).t;
            }
            if predicates::sign(self.v(dl), self.v(self.event), isect) <= 0.0 {
                self.regions[up].dirty = true;
                self.regions[lo].dirty = true;
                self.mesh.split_edge(self.e(el).sym);
                let v = self.e(el).org;
                self.mesh.vertices[v].s = self.v(self.event).s;
                self.mesh.vertices[v].t = self.v(self.event).t;
            }
            return false;
        }
        self.mesh.split_edge(self.e(eu).sym);
        self.mesh.split_edge(self.e(el).sym);
        self.mesh.splice(self.oprev(el), eu);
        let v = self.e(eu).org;
        self.mesh.vertices[v].s = isect.s;
        self.mesh.vertices[v].t = isect.t;
        self.mesh.vertices[v].pq_handle = self.queue.insert(&self.mesh, v);
        self.intersect_data(
            v,
            IntersectData {
                org_up: ou,
                dst_up: du,
                org_lo: ol,
                dst_lo: dl,
            },
        );
        let above = self.above(up);
        self.regions[above].dirty = true;
        self.regions[up].dirty = true;
        self.regions[lo].dirty = true;
        false
    }
}
