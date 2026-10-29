//! glu-libtess normal.c:76–169/198–257, automatic normal/projection.
//! Copyright (C) 1991-2000 Silicon Graphics, Inc. All Rights Reserved.
//! SGI FREE SOFTWARE LICENSE B, Version 2.0:
//! http://oss.sgi.com/projects/FreeB/
use super::mesh::{F_HEAD, Mesh, V_HEAD};
fn long_axis(v: [f64; 3]) -> usize {
    let mut i = 0;
    if v[1].abs() > v[0].abs() {
        i = 1;
    }
    if v[2].abs() > v[i].abs() {
        i = 2;
    }
    i
}
fn normal(mesh: &Mesh) -> [f64; 3] {
    let mut max = [-2e150; 3];
    let mut min = [2e150; 3];
    let mut max_vertex = [0; 3];
    let mut min_vertex = [0; 3];
    let mut v = mesh.v(V_HEAD).next;
    while v != V_HEAD {
        for axis in 0..3 {
            let c = mesh.v(v).coords[axis];
            if c < min[axis] {
                min[axis] = c;
                min_vertex[axis] = v;
            }
            if c > max[axis] {
                max[axis] = c;
                max_vertex[axis] = v;
            }
        }
        v = mesh.v(v).next;
    }
    let mut i = 0;
    if max[1] - min[1] > max[0] - min[0] {
        i = 1;
    }
    if max[2] - min[2] > max[i] - min[i] {
        i = 2;
    }
    if min[i] >= max[i] {
        return [0.0, 0.0, 1.0];
    }
    let a = mesh.v(min_vertex[i]).coords;
    let b = mesh.v(max_vertex[i]).coords;
    let d1 = std::array::from_fn::<_, 3, _>(|axis| a[axis] - b[axis]);
    let mut result = [0.0; 3];
    let mut max_length = 0.0;
    let mut v = mesh.v(V_HEAD).next;
    while v != V_HEAD {
        let c = mesh.v(v).coords;
        let d2 = std::array::from_fn::<_, 3, _>(|axis| c[axis] - b[axis]);
        let n = [
            d1[1] * d2[2] - d1[2] * d2[1],
            d1[2] * d2[0] - d1[0] * d2[2],
            d1[0] * d2[1] - d1[1] * d2[0],
        ];
        let length = n[0] * n[0] + n[1] * n[1] + n[2] * n[2];
        if length > max_length {
            max_length = length;
            result = n;
        }
        v = mesh.v(v).next;
    }
    if max_length <= 0.0 {
        result = [0.0; 3];
        result[long_axis(d1)] = 1.0;
    }
    result
}
pub(super) fn project(mesh: &mut Mesh) {
    let norm = normal(mesh);
    let i = long_axis(norm);
    let mut s_unit = [0.0; 3];
    let mut t_unit = [0.0; 3];
    s_unit[(i + 1) % 3] = 1.0;
    s_unit[(i + 2) % 3] = 0.0;
    t_unit[(i + 1) % 3] = if norm[i] > 0.0 { -0.0 } else { 0.0 };
    t_unit[(i + 2) % 3] = if norm[i] > 0.0 { 1.0 } else { -1.0 };
    let mut v = mesh.v(V_HEAD).next;
    while v != V_HEAD {
        let c = mesh.v(v).coords;
        mesh.vertices[v].s = c[0] * s_unit[0] + c[1] * s_unit[1] + c[2] * s_unit[2];
        mesh.vertices[v].t = c[0] * t_unit[0] + c[1] * t_unit[1] + c[2] * t_unit[2];
        v = mesh.v(v).next;
    }
    check_orientation(mesh);
}
fn check_orientation(mesh: &mut Mesh) {
    let mut area = 0.0;
    let mut f = mesh.f(F_HEAD).next;
    while f != F_HEAD {
        let start = mesh.f(f).edge;
        if mesh.e(start).winding > 0 {
            orientation_face_area(mesh, start, &mut area);
        }
        f = mesh.f(f).next;
    }
    if area < 0.0 {
        let mut v = mesh.v(V_HEAD).next;
        while v != V_HEAD {
            mesh.vertices[v].t = -mesh.v(v).t;
            v = mesh.v(v).next;
        }
    }
}

fn orientation_face_area(mesh: &Mesh, start: usize, area: &mut f64) {
    let mut e = start;
    loop {
        let o = mesh.v(mesh.e(e).org);
        let d = mesh.v(mesh.dst(e));
        *area += (o.s - d.s) * (o.t + d.t);
        e = mesh.e(e).lnext;
        if e == start {
            break;
        }
    }
}
