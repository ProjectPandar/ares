//! glu-libtess sweep.c:438–452/499–603/760–844, splice/dirty repair.
//! Copyright (C) 1991-2000 Silicon Graphics, Inc. All Rights Reserved.
//! SGI FREE SOFTWARE LICENSE B, Version 2.0:
//! http://oss.sgi.com/projects/FreeB/
use super::Tess;
impl Tess {
    pub(super) fn merge_vertices(&mut self, one: usize, two: usize) {
        let v = self.e(one).org;
        self.mesh.vertices[v].data = Some(self.v(v).coords);
        self.mesh.splice(one, two);
    }
    pub(super) fn check_right_splice(&mut self, up: usize) -> bool {
        let lo = self.below(up);
        let eu = self.r(up).edge;
        let el = self.r(lo).edge;
        if self.leq(self.e(eu).org, self.e(el).org) {
            if self.sign(self.dst(el), self.e(eu).org, self.e(el).org) > 0.0 {
                return false;
            }
            if !self.eq(self.e(eu).org, self.e(el).org) {
                self.mesh.split_edge(self.e(el).sym);
                self.mesh.splice(eu, self.oprev(el));
                self.regions[up].dirty = true;
                self.regions[lo].dirty = true;
            } else if self.e(eu).org != self.e(el).org {
                self.queue
                    .delete(&self.mesh, self.v(self.e(eu).org).pq_handle);
                self.merge_vertices(self.oprev(el), eu);
            }
        } else {
            if self.sign(self.dst(eu), self.e(el).org, self.e(eu).org) < 0.0 {
                return false;
            }
            let above = self.above(up);
            self.regions[above].dirty = true;
            self.regions[up].dirty = true;
            self.mesh.split_edge(self.e(eu).sym);
            self.mesh.splice(self.oprev(el), eu);
        }
        true
    }
    fn check_left_splice(&mut self, up: usize) -> bool {
        let lo = self.below(up);
        let eu = self.r(up).edge;
        let el = self.r(lo).edge;
        if self.leq(self.dst(eu), self.dst(el)) {
            if self.sign(self.dst(eu), self.dst(el), self.e(eu).org) < 0.0 {
                return false;
            }
            let above = self.above(up);
            self.regions[above].dirty = true;
            self.regions[up].dirty = true;
            let e = self.mesh.split_edge(eu);
            self.mesh.splice(self.e(el).sym, e);
            let f = self.e(e).face;
            self.mesh.faces[f].inside = self.r(up).inside;
        } else {
            if self.sign(self.dst(el), self.dst(eu), self.e(el).org) > 0.0 {
                return false;
            }
            self.regions[up].dirty = true;
            self.regions[lo].dirty = true;
            let e = self.mesh.split_edge(el);
            self.mesh.splice(self.e(eu).lnext, self.e(el).sym);
            let f = self.rface(e);
            self.mesh.faces[f].inside = self.r(up).inside;
        }
        true
    }
    pub(super) fn walk_dirty(&mut self, mut up: usize) {
        let mut lo = self.below(up);
        while let Some((next_up, next_lo)) = self.walk_dirty_region(up, lo) {
            up = next_up;
            lo = next_lo;
        }
    }
    fn walk_dirty_region(&mut self, mut up: usize, mut lo: usize) -> Option<(usize, usize)> {
        while self.r(lo).dirty {
            up = lo;
            lo = self.below(lo);
        }
        if !self.r(up).dirty {
            lo = up;
            up = self.above(up);
            if up == 0 || !self.r(up).dirty {
                return None;
            }
        }
        self.regions[up].dirty = false;
        let mut eu = self.r(up).edge;
        let mut el = self.r(lo).edge;
        if self.dst(eu) != self.dst(el) && self.check_left_splice(up) {
            if self.r(lo).fix_upper {
                self.delete_region(lo);
                self.mesh.delete(el);
                lo = self.below(up);
                el = self.r(lo).edge;
            } else if self.r(up).fix_upper {
                self.delete_region(up);
                self.mesh.delete(eu);
                up = self.above(lo);
                eu = self.r(up).edge;
            }
        }
        if self.e(eu).org != self.e(el).org && self.walk_dirty_intersection(up, lo, eu, el) {
            return None;
        }
        if self.e(eu).org == self.e(el).org && self.dst(eu) == self.dst(el) {
            self.mesh.add_winding(el, eu);
            self.delete_region(up);
            self.mesh.delete(eu);
            up = self.above(lo);
        }
        Some((up, lo))
    }
    fn walk_dirty_intersection(&mut self, up: usize, lo: usize, eu: usize, el: usize) -> bool {
        if self.dst(eu) != self.dst(el)
            && !self.r(up).fix_upper
            && !self.r(lo).fix_upper
            && (self.dst(eu) == self.event || self.dst(el) == self.event)
        {
            self.check_intersection(up)
        } else {
            self.check_right_splice(up);
            false
        }
    }
}
