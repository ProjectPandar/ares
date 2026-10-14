//! Original glu-libtess mesh.h:110–190; mesh.c:69–543/603–649.
//! Copyright (C) 1991-2000 Silicon Graphics, Inc. All Rights Reserved.
//! SGI FREE SOFTWARE LICENSE B, Version 2.0:
//! http://oss.sgi.com/projects/FreeB/
#[derive(Clone, Copy, Default)]
pub(super) struct Vertex {
    pub next: usize,
    pub prev: usize,
    pub edge: usize,
    pub coords: [f64; 3],
    pub s: f64,
    pub t: f64,
    pub data: Option<[f64; 3]>,
    pub pq_handle: isize,
}
#[derive(Clone, Copy, Default)]
pub(super) struct Face {
    pub next: usize,
    pub prev: usize,
    pub edge: usize,
    pub trail: usize,
    pub marked: bool,
    pub inside: bool,
}
#[derive(Clone, Copy, Default)]
pub(super) struct Edge {
    pub next: usize,
    pub sym: usize,
    pub onext: usize,
    pub lnext: usize,
    pub org: usize,
    pub face: usize,
    pub region: usize,
    pub winding: i32,
}
pub(super) struct Mesh {
    pub vertices: Vec<Vertex>,
    pub faces: Vec<Face>,
    pub edges: Vec<Edge>,
}
pub(super) const V_HEAD: usize = 1;
pub(super) const F_HEAD: usize = 1;
pub(super) const E_HEAD: usize = 2;
impl Mesh {
    pub fn new() -> Self {
        let mut mesh = Self {
            vertices: vec![Vertex::default(); 2],
            faces: vec![Face::default(); 2],
            edges: vec![Edge::default(); 4],
        };
        mesh.vertices[V_HEAD].next = V_HEAD;
        mesh.vertices[V_HEAD].prev = V_HEAD;
        mesh.faces[F_HEAD].next = F_HEAD;
        mesh.faces[F_HEAD].prev = F_HEAD;
        mesh.edges[E_HEAD].next = E_HEAD;
        mesh.edges[E_HEAD].sym = E_HEAD + 1;
        mesh.edges[E_HEAD + 1].next = E_HEAD + 1;
        mesh.edges[E_HEAD + 1].sym = E_HEAD;
        mesh
    }
    pub fn e(&self, e: usize) -> Edge {
        self.edges[e]
    }
    pub fn v(&self, v: usize) -> Vertex {
        self.vertices[v]
    }
    pub fn f(&self, f: usize) -> Face {
        self.faces[f]
    }
    pub fn dst(&self, e: usize) -> usize {
        self.e(self.e(e).sym).org
    }
    pub fn rface(&self, e: usize) -> usize {
        self.e(self.e(e).sym).face
    }
    pub fn oprev(&self, e: usize) -> usize {
        self.e(self.e(e).sym).lnext
    }
    pub fn lprev(&self, e: usize) -> usize {
        self.e(self.e(e).onext).sym
    }
    pub fn dprev(&self, e: usize) -> usize {
        self.e(self.e(e).lnext).sym
    }
    pub fn rprev(&self, e: usize) -> usize {
        self.e(self.e(e).sym).onext
    }
    pub fn dnext(&self, e: usize) -> usize {
        self.e(self.rprev(e)).sym
    }
    fn make_pair(&mut self, mut next: usize) -> usize {
        let e = self.edges.len();
        let sym = e + 1;
        // Both halves occupy consecutive indices, as EdgePair::e/eSym do.
        if self.e(next).sym < next {
            next = self.e(next).sym;
        }
        let prev = self.e(self.e(next).sym).next;
        self.edges.push(Edge {
            next,
            sym,
            onext: e,
            lnext: sym,
            ..Edge::default()
        });
        self.edges.push(Edge {
            next: prev,
            sym: e,
            onext: sym,
            lnext: e,
            ..Edge::default()
        });
        let ps = self.e(prev).sym;
        self.edges[ps].next = e;
        let ns = self.e(next).sym;
        self.edges[ns].next = sym;
        e
    }
    fn splice_links(&mut self, a: usize, b: usize) {
        let an = self.e(a).onext;
        let bn = self.e(b).onext;
        let ans = self.e(an).sym;
        let bns = self.e(bn).sym;
        self.edges[ans].lnext = b;
        self.edges[bns].lnext = a;
        self.edges[a].onext = bn;
        self.edges[b].onext = an;
    }
    fn make_vertex(&mut self, e: usize, next: usize) -> usize {
        let v = self.vertices.len();
        let prev = self.v(next).prev;
        self.vertices.push(Vertex {
            next,
            prev,
            edge: e,
            ..Vertex::default()
        });
        self.vertices[prev].next = v;
        self.vertices[next].prev = v;
        let mut cursor = e;
        loop {
            self.edges[cursor].org = v;
            cursor = self.e(cursor).onext;
            if cursor == e {
                break;
            }
        }
        v
    }
    fn make_face(&mut self, e: usize, next: usize) -> usize {
        let f = self.faces.len();
        let prev = self.f(next).prev;
        self.faces.push(Face {
            next,
            prev,
            edge: e,
            inside: self.f(next).inside,
            ..Face::default()
        });
        self.faces[prev].next = f;
        self.faces[next].prev = f;
        let mut cursor = e;
        loop {
            self.edges[cursor].face = f;
            cursor = self.e(cursor).lnext;
            if cursor == e {
                break;
            }
        }
        f
    }
    fn kill_edge(&mut self, mut e: usize) {
        if self.e(e).sym < e {
            e = self.e(e).sym;
        }
        let next = self.e(e).next;
        let prev = self.e(self.e(e).sym).next;
        let ns = self.e(next).sym;
        let ps = self.e(prev).sym;
        self.edges[ns].next = prev;
        self.edges[ps].next = next;
    }
    fn kill_vertex(&mut self, v: usize, org: usize) {
        let start = self.v(v).edge;
        let mut e = start;
        loop {
            self.edges[e].org = org;
            e = self.e(e).onext;
            if e == start {
                break;
            }
        }
        let prev = self.v(v).prev;
        let next = self.v(v).next;
        self.vertices[next].prev = prev;
        self.vertices[prev].next = next;
    }
    fn kill_face(&mut self, f: usize, face: usize) {
        let start = self.f(f).edge;
        let mut e = start;
        loop {
            self.edges[e].face = face;
            e = self.e(e).lnext;
            if e == start {
                break;
            }
        }
        let prev = self.f(f).prev;
        let next = self.f(f).next;
        self.faces[next].prev = prev;
        self.faces[prev].next = next;
    }
    pub fn make_edge(&mut self) -> usize {
        let e = self.make_pair(E_HEAD);
        self.make_vertex(e, V_HEAD);
        self.make_vertex(self.e(e).sym, V_HEAD);
        self.make_face(e, F_HEAD);
        e
    }
    pub fn splice(&mut self, org: usize, dst: usize) {
        if org == dst {
            return;
        }
        let joining_vertices = self.e(dst).org != self.e(org).org;
        let joining_loops = self.e(dst).face != self.e(org).face;
        if joining_vertices {
            self.kill_vertex(self.e(dst).org, self.e(org).org);
        }
        if joining_loops {
            self.kill_face(self.e(dst).face, self.e(org).face);
        }
        self.splice_links(dst, org);
        if !joining_vertices {
            self.make_vertex(dst, self.e(org).org);
            let v = self.e(org).org;
            self.vertices[v].edge = org;
        }
        if !joining_loops {
            self.make_face(dst, self.e(org).face);
            let f = self.e(org).face;
            self.faces[f].edge = org;
        }
    }
    pub fn delete(&mut self, e: usize) {
        let sym = self.e(e).sym;
        let joining = self.e(e).face != self.rface(e);
        if joining {
            self.kill_face(self.e(e).face, self.rface(e));
        }
        if self.e(e).onext == e {
            self.kill_vertex(self.e(e).org, 0);
        } else {
            let f = self.rface(e);
            self.faces[f].edge = self.oprev(e);
            let v = self.e(e).org;
            self.vertices[v].edge = self.e(e).onext;
            self.splice_links(e, self.oprev(e));
            if !joining {
                self.make_face(e, self.e(e).face);
            }
        }
        if self.e(sym).onext == sym {
            self.kill_vertex(self.e(sym).org, 0);
            self.kill_face(self.e(sym).face, 0);
        } else {
            let f = self.e(e).face;
            self.faces[f].edge = self.oprev(sym);
            let v = self.e(sym).org;
            self.vertices[v].edge = self.e(sym).onext;
            self.splice_links(sym, self.oprev(sym));
        }
        self.kill_edge(e);
    }
    fn add_edge_vertex(&mut self, org: usize) -> usize {
        let e = self.make_pair(org);
        let sym = self.e(e).sym;
        self.splice_links(e, self.e(org).lnext);
        self.edges[e].org = self.dst(org);
        self.make_vertex(sym, self.e(e).org);
        let face = self.e(org).face;
        self.edges[e].face = face;
        self.edges[sym].face = face;
        e
    }
    pub fn split_edge(&mut self, org: usize) -> usize {
        let temp = self.add_edge_vertex(org);
        let e = self.e(temp).sym;
        let sym = self.e(org).sym;
        self.splice_links(sym, self.oprev(sym));
        self.splice_links(sym, e);
        self.edges[sym].org = self.e(e).org;
        let dst = self.dst(e);
        self.vertices[dst].edge = self.e(e).sym;
        let es = self.e(e).sym;
        self.edges[es].face = self.rface(org);
        self.edges[e].winding = self.e(org).winding;
        self.edges[es].winding = self.e(sym).winding;
        e
    }
    pub fn connect(&mut self, org: usize, dst: usize) -> usize {
        let e = self.make_pair(org);
        let sym = self.e(e).sym;
        let joining = self.e(dst).face != self.e(org).face;
        if joining {
            self.kill_face(self.e(dst).face, self.e(org).face);
        }
        self.splice_links(e, self.e(org).lnext);
        self.splice_links(sym, dst);
        self.edges[e].org = self.dst(org);
        self.edges[sym].org = self.e(dst).org;
        let f = self.e(org).face;
        self.edges[e].face = f;
        self.edges[sym].face = f;
        self.faces[f].edge = sym;
        if !joining {
            self.make_face(e, f);
        }
        e
    }
    pub fn add_winding(&mut self, dst: usize, src: usize) {
        self.edges[dst].winding += self.e(src).winding;
        let sym = self.e(dst).sym;
        self.edges[sym].winding += self.e(self.e(src).sym).winding;
    }
}
