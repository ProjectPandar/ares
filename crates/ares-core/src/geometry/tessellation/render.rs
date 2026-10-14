//! glu-libtess render.c:82–330/362–502; Tesselate.cpp:110–164.
//! Copyright (C) 1991-2000 Silicon Graphics, Inc. All Rights Reserved.
//! SGI FREE SOFTWARE LICENSE B, Version 2.0:
//! http://oss.sgi.com/projects/FreeB/
use super::{Tess, mesh::F_HEAD};
#[derive(Clone, Copy)]
enum Primitive {
    Triangles,
    Fan,
    Strip,
}
struct Emitter {
    output: Vec<[f64; 3]>,
    primitive: Primitive,
    count: usize,
    first: [f64; 3],
    second: [f64; 3],
    z: f64,
    flip: bool,
}
impl Emitter {
    fn new(z: f64, flip: bool) -> Self {
        Self {
            output: Vec::new(),
            primitive: Primitive::Triangles,
            count: 0,
            first: [0.0; 3],
            second: [0.0; 3],
            z,
            flip,
        }
    }
    fn begin(&mut self, primitive: Primitive) {
        self.primitive = primitive;
        self.count = 0;
    }
    fn vertex(&mut self, point: Option<[f64; 3]>) {
        let Some(mut point) = point else {
            return;
        };
        point[2] = self.z;
        self.count += 1;
        match self.count {
            1 => self.first = point,
            2 => self.second = point,
            _ => {
                let mut flip = self.flip;
                if matches!(self.primitive, Primitive::Strip) && self.count == 4 {
                    flip = !flip;
                    self.count = 2;
                }
                self.output.push(self.first);
                self.output.push(if flip { point } else { self.second });
                self.output.push(if flip { self.second } else { point });
                match self.primitive {
                    Primitive::Strip => {
                        self.first = self.second;
                        self.second = point;
                    }
                    Primitive::Fan => self.second = point,
                    Primitive::Triangles => self.count = 0,
                }
            }
        }
    }
}
#[derive(Clone, Copy)]
struct FaceCount {
    size: usize,
    edge: usize,
    primitive: Primitive,
}
impl Tess {
    fn marked(&self, f: usize) -> bool {
        !self.f(f).inside || self.f(f).marked
    }
    fn add_trail(&mut self, f: usize, trail: &mut usize) {
        self.mesh.faces[f].trail = *trail;
        *trail = f;
        self.mesh.faces[f].marked = true;
    }
    fn free_trail(&mut self, mut trail: usize) {
        while trail != 0 {
            self.mesh.faces[trail].marked = false;
            trail = self.f(trail).trail;
        }
    }
    fn maximum_fan(&mut self, orig: usize) -> FaceCount {
        let mut result = FaceCount {
            size: 0,
            edge: 0,
            primitive: Primitive::Fan,
        };
        let mut trail = 0;
        let mut e = orig;
        while !self.marked(self.e(e).face) {
            self.add_trail(self.e(e).face, &mut trail);
            result.size += 1;
            e = self.e(e).onext;
        }
        e = orig;
        while !self.marked(self.rface(e)) {
            self.add_trail(self.rface(e), &mut trail);
            result.size += 1;
            e = self.oprev(e);
        }
        result.edge = e;
        self.free_trail(trail);
        result
    }
    fn maximum_strip(&mut self, orig: usize) -> FaceCount {
        let mut trail = 0;
        let mut head_size = 0;
        let mut tail_size = 0;
        let mut e = orig;
        while !self.marked(self.e(e).face) {
            self.add_trail(self.e(e).face, &mut trail);
            tail_size += 1;
            e = self.dprev(e);
            if self.marked(self.e(e).face) {
                break;
            }
            self.add_trail(self.e(e).face, &mut trail);
            tail_size += 1;
            e = self.e(e).onext;
        }
        let tail = e;
        e = orig;
        while !self.marked(self.rface(e)) {
            self.add_trail(self.rface(e), &mut trail);
            head_size += 1;
            e = self.oprev(e);
            if self.marked(self.rface(e)) {
                break;
            }
            self.add_trail(self.rface(e), &mut trail);
            head_size += 1;
            e = self.dnext(e);
        }
        let head = e;
        let mut size = head_size + tail_size;
        let edge = if tail_size % 2 == 0 {
            self.e(tail).sym
        } else if head_size % 2 == 0 {
            head
        } else {
            size -= 1;
            self.e(head).onext
        };
        self.free_trail(trail);
        FaceCount {
            size,
            edge,
            primitive: Primitive::Strip,
        }
    }
    fn render_group(&mut self, f: usize, emitter: &mut Emitter, lonely: &mut usize) {
        let edge = self.f(f).edge;
        let mut largest = FaceCount {
            size: 1,
            edge,
            primitive: Primitive::Triangles,
        };
        for edge in [edge, self.e(edge).lnext, self.lprev(edge)] {
            let group = self.maximum_fan(edge);
            if group.size > largest.size {
                largest = group;
            }
        }
        for edge in [edge, self.e(edge).lnext, self.lprev(edge)] {
            let group = self.maximum_strip(edge);
            if group.size > largest.size {
                largest = group;
            }
        }
        let e = largest.edge;
        match largest.primitive {
            Primitive::Triangles => self.add_trail(self.e(e).face, lonely),
            Primitive::Fan => self.render_fan(e, emitter),
            Primitive::Strip => self.render_strip(e, emitter),
        }
    }
    fn render_fan(&mut self, mut e: usize, emitter: &mut Emitter) {
        emitter.begin(Primitive::Fan);
        emitter.vertex(self.v(self.e(e).org).data);
        emitter.vertex(self.v(self.dst(e)).data);
        while !self.marked(self.e(e).face) {
            let f = self.e(e).face;
            self.mesh.faces[f].marked = true;
            e = self.e(e).onext;
            emitter.vertex(self.v(self.dst(e)).data);
        }
    }
    fn render_strip(&mut self, mut e: usize, emitter: &mut Emitter) {
        emitter.begin(Primitive::Strip);
        emitter.vertex(self.v(self.e(e).org).data);
        emitter.vertex(self.v(self.dst(e)).data);
        while !self.marked(self.e(e).face) {
            let f = self.e(e).face;
            self.mesh.faces[f].marked = true;
            e = self.dprev(e);
            emitter.vertex(self.v(self.e(e).org).data);
            if self.marked(self.e(e).face) {
                break;
            }
            let f = self.e(e).face;
            self.mesh.faces[f].marked = true;
            e = self.e(e).onext;
            emitter.vertex(self.v(self.dst(e)).data);
        }
    }
    fn render_lonely_triangles(&self, mut f: usize, emitter: &mut Emitter) {
        while f != 0 {
            self.render_lonely_face(f, emitter);
            f = self.f(f).trail;
        }
    }
    fn render_lonely_face(&self, f: usize, emitter: &mut Emitter) {
        let start = self.f(f).edge;
        let mut e = start;
        loop {
            emitter.vertex(self.v(self.e(e).org).data);
            e = self.e(e).lnext;
            if e == start {
                break;
            }
        }
    }
    pub(super) fn render(&mut self, z: f64, flip: bool) -> Vec<[f64; 3]> {
        let mut emitter = Emitter::new(z, flip);
        let mut lonely = 0;
        let mut f = self.f(F_HEAD).next;
        while f != F_HEAD {
            self.mesh.faces[f].marked = false;
            f = self.f(f).next;
        }
        f = self.f(F_HEAD).next;
        while f != F_HEAD {
            if self.f(f).inside && !self.f(f).marked {
                self.render_group(f, &mut emitter, &mut lonely);
            }
            f = self.f(f).next;
        }
        if lonely != 0 {
            emitter.begin(Primitive::Triangles);
            self.render_lonely_triangles(lonely, &mut emitter);
        }
        emitter.output
    }
}
fn cache_normal(points: &[[f64; 3]], norm: &mut [f64; 3], check: bool) -> i32 {
    let origin = points[0];
    let mut current = std::array::from_fn::<_, 3, _>(|i| points[1][i] - origin[i]);
    let mut sign = 0;
    for point in &points[2..] {
        let previous = current;
        current = std::array::from_fn(|i| point[i] - origin[i]);
        let n = [
            previous[1] * current[2] - previous[2] * current[1],
            previous[2] * current[0] - previous[0] * current[2],
            previous[0] * current[1] - previous[1] * current[0],
        ];
        let dot = n[0] * norm[0] + n[1] * norm[1] + n[2] * norm[2];
        if !check {
            compute_normal_accumulate(norm, n, dot);
        } else if dot != 0.0 {
            sign = compute_normal_check(sign, dot);
            if sign == 2 {
                return 2;
            }
        }
    }
    sign
}
fn compute_normal_accumulate(norm: &mut [f64; 3], n: [f64; 3], dot: f64) {
    for axis in 0..3 {
        if dot >= 0.0 {
            norm[axis] += n[axis];
        } else {
            norm[axis] -= n[axis];
        }
    }
}
fn compute_normal_check(sign: i32, dot: f64) -> i32 {
    if dot > 0.0 {
        if sign < 0 {
            return 2;
        }
        1
    } else {
        if sign > 0 {
            return 2;
        }
        -1
    }
}
pub(super) fn cache(points: &[[f64; 3]], z: f64, flip: bool) -> Option<Vec<[f64; 3]>> {
    if points.len() < 3 {
        return Some(Vec::new());
    }
    let mut norm = [0.0; 3];
    cache_normal(points, &mut norm, false);
    let sign = cache_normal(points, &mut norm, true);
    if sign == 2 {
        return None;
    }
    if sign == 0 {
        return Some(Vec::new());
    }
    let mut emitter = Emitter::new(z, flip);
    emitter.begin(if points.len() > 3 {
        Primitive::Fan
    } else {
        Primitive::Triangles
    });
    emitter.vertex(Some(points[0]));
    if sign > 0 {
        for &point in &points[1..] {
            emitter.vertex(Some(point));
        }
    } else {
        for &point in points[1..].iter().rev() {
            emitter.vertex(Some(point));
        }
    }
    Some(emitter.output)
}
