//! glu-libtess tessmono.c:72–147, monotone-face triangulation.
//! Copyright (C) 1991-2000 Silicon Graphics, Inc. All Rights Reserved.
//! SGI FREE SOFTWARE LICENSE B, Version 2.0:
//! http://oss.sgi.com/projects/FreeB/
use super::{Tess, mesh::F_HEAD};
impl Tess {
    fn triangulate_face(&mut self, face: usize) {
        let mut up = self.f(face).edge;
        while self.leq(self.dst(up), self.e(up).org) {
            up = self.lprev(up);
        }
        while self.leq(self.e(up).org, self.dst(up)) {
            up = self.e(up).lnext;
        }
        let mut lo = self.lprev(up);
        while self.e(up).lnext != lo {
            if self.leq(self.dst(up), self.e(lo).org) {
                lo = self.tessellate_mono_lower(up, lo);
                lo = self.lprev(lo);
            } else {
                up = self.tessellate_mono_upper(up, lo);
                up = self.e(up).lnext;
            }
        }
        while self.e(self.e(lo).lnext).lnext != up {
            let temp = self.mesh.connect(self.e(lo).lnext, lo);
            lo = self.e(temp).sym;
        }
    }
    fn tessellate_mono_lower(&mut self, up: usize, mut lo: usize) -> usize {
        while self.e(lo).lnext != up
            && (self.leq(self.dst(self.e(lo).lnext), self.e(self.e(lo).lnext).org)
                || self.sign(self.e(lo).org, self.dst(lo), self.dst(self.e(lo).lnext)) <= 0.0)
        {
            let temp = self.mesh.connect(self.e(lo).lnext, lo);
            lo = self.e(temp).sym;
        }
        lo
    }
    fn tessellate_mono_upper(&mut self, mut up: usize, lo: usize) -> usize {
        while self.e(lo).lnext != up
            && (self.leq(self.e(self.lprev(up)).org, self.dst(self.lprev(up)))
                || self.sign(self.dst(up), self.e(up).org, self.e(self.lprev(up)).org) >= 0.0)
        {
            let temp = self.mesh.connect(up, self.lprev(up));
            up = self.e(temp).sym;
        }
        up
    }
    pub(super) fn triangulate_interior(&mut self) {
        let mut f = self.f(F_HEAD).next;
        while f != F_HEAD {
            let next = self.f(f).next;
            if self.f(f).inside {
                self.triangulate_face(f);
            }
            f = next;
        }
    }
}
