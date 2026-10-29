//! glu-libtess tess.c/normal.c/sweep.c/tessmono.c/render.c source port.
//! Copyright (C) 1991-2000 Silicon Graphics, Inc. All Rights Reserved.
//! SGI FREE SOFTWARE LICENSE B, Version 2.0:
//! http://oss.sgi.com/projects/FreeB/
#[path = "intersection.rs"]
mod intersection;
#[path = "mesh.rs"]
mod mesh;
#[path = "monotone.rs"]
mod monotone;
#[path = "predicates.rs"]
mod predicates;
#[path = "projection.rs"]
mod projection;
#[path = "queue.rs"]
mod queue;
#[path = "regions.rs"]
mod regions;
#[path = "render.rs"]
mod render;
#[path = "splice.rs"]
mod splice;
#[path = "sweep.rs"]
mod sweep;
use mesh::{Edge, Face, Mesh, Vertex};
use queue::Queue;
struct RightEdges {
    first: usize,
    last: usize,
    top_left: usize,
}
#[derive(Clone, Copy, Default)]
struct Region {
    edge: usize,
    node: usize,
    winding: i32,
    inside: bool,
    sentinel: bool,
    dirty: bool,
    fix_upper: bool,
}
#[derive(Clone, Copy, Default)]
struct Node {
    prev: usize,
    next: usize,
    key: usize,
}
struct Tess {
    mesh: Mesh,
    regions: Vec<Region>,
    nodes: Vec<Node>,
    queue: Queue,
    event: usize,
}
impl Tess {
    fn e(&self, e: usize) -> Edge {
        self.mesh.e(e)
    }
    fn v(&self, v: usize) -> Vertex {
        self.mesh.v(v)
    }
    fn f(&self, f: usize) -> Face {
        self.mesh.f(f)
    }
    fn r(&self, r: usize) -> Region {
        self.regions[r]
    }
    fn dst(&self, e: usize) -> usize {
        self.mesh.dst(e)
    }
    fn rface(&self, e: usize) -> usize {
        self.mesh.rface(e)
    }
    fn oprev(&self, e: usize) -> usize {
        self.mesh.oprev(e)
    }
    fn lprev(&self, e: usize) -> usize {
        self.mesh.lprev(e)
    }
    fn dprev(&self, e: usize) -> usize {
        self.mesh.dprev(e)
    }
    fn rprev(&self, e: usize) -> usize {
        self.mesh.rprev(e)
    }
    fn dnext(&self, e: usize) -> usize {
        self.mesh.dnext(e)
    }
    fn leq(&self, u: usize, v: usize) -> bool {
        predicates::leq(self.v(u), self.v(v))
    }
    fn eq(&self, u: usize, v: usize) -> bool {
        predicates::eq(self.v(u), self.v(v))
    }
    fn sign(&self, u: usize, v: usize, w: usize) -> f64 {
        predicates::sign(self.v(u), self.v(v), self.v(w))
    }
}
// tess.c:345–579. Cache flush preserves contour starts and GLU's 100-vertex cutoff.
pub(super) fn tessellate(contours: &[Vec<[f64; 3]>], z: f64, flip: bool) -> Vec<[f64; 3]> {
    let mut cached = Vec::new();
    let mut mesh = None;
    let mut last;
    for contour in contours {
        last = 0;
        let needs_empty_cache = !cached.is_empty();
        for &point in contour {
            if needs_empty_cache && mesh.is_none() {
                let mut new_mesh = Mesh::new();
                let mut cached_last = 0;
                empty_cache(&mut cached, &mut new_mesh, &mut cached_last);
                mesh = Some(new_mesh);
                last = 0;
            }
            if mesh.is_none() && cached.len() < 100 {
                cached.push(point);
                continue;
            }
            if mesh.is_none() {
                let mut new_mesh = Mesh::new();
                empty_cache(&mut cached, &mut new_mesh, &mut last);
                mesh = Some(new_mesh);
            }
            add_vertex(mesh.as_mut().unwrap(), &mut last, point);
        }
    }
    if mesh.is_none() {
        if let Some(result) = render::cache(&cached, z, flip) {
            return result;
        }
        let mut new_mesh = Mesh::new();
        last = 0;
        for point in cached {
            add_vertex(&mut new_mesh, &mut last, point);
        }
        mesh = Some(new_mesh);
    }
    let mut mesh = mesh.unwrap();
    projection::project(&mut mesh);
    let queue = Queue::default();
    let mut tess = Tess {
        mesh,
        queue,
        regions: vec![Region::default()],
        nodes: vec![Node::default()],
        event: 0,
    };
    tess.compute_interior();
    tess.triangulate_interior();
    tess.render(z, flip)
}
fn add_vertex(mesh: &mut Mesh, last: &mut usize, point: [f64; 3]) {
    let e = if *last == 0 {
        let e = mesh.make_edge();
        mesh.splice(e, mesh.e(e).sym);
        e
    } else {
        mesh.split_edge(*last);
        mesh.e(*last).lnext
    };
    let v = mesh.e(e).org;
    mesh.vertices[v].coords = point;
    mesh.vertices[v].data = Some(point);
    mesh.edges[e].winding = 1;
    let sym = mesh.e(e).sym;
    mesh.edges[sym].winding = -1;
    *last = e;
}

fn empty_cache(cached: &mut Vec<[f64; 3]>, mesh: &mut Mesh, last: &mut usize) {
    for point in cached.drain(..) {
        add_vertex(mesh, last, point);
    }
}
