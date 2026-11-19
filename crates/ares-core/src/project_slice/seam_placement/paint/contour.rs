use std::collections::HashMap;

use super::Vec3;

pub(super) struct PaintEdge {
    from: [f64; 3],
    to: [f64; 3],
    inward: [f64; 2],
}

impl PaintEdge {
    pub(super) fn crossing(&self, z: f64) -> Option<([f64; 2], [f64; 2])> {
        if (self.from[2] < z) == (self.to[2] < z) {
            return None;
        }
        let t = (z - self.from[2]) / (self.to[2] - self.from[2]);
        Some((
            std::array::from_fn(|i| self.from[i] + t * (self.to[i] - self.from[i])),
            self.inward,
        ))
    }
}

// SeamPlacer.cpp:1077–1099: count edges by shared selector vertex identity.
pub(super) fn edges(vertices: &[Vec3], faces: &[[u32; 3]]) -> Vec<PaintEdge> {
    let indices = |[a, b, c]: [u32; 3]| [(a, b), (b, c), (c, a)];
    let key = |a: u32, b: u32| (a.min(b), a.max(b));
    let mut uses = HashMap::<_, usize>::new();
    for &face in faces {
        for (a, b) in indices(face) {
            *uses.entry(key(a, b)).or_default() += 1;
        }
    }
    let vertex = |index: u32| {
        let p = vertices[index as usize];
        [f64::from(p.x), f64::from(p.y), f64::from(p.z)]
    };
    let mut edges = Vec::new();
    for &face in faces {
        let [a, b, c] = face.map(vertex);
        let normal = cross(sub(b, a), sub(c, a));
        for (a, b) in indices(face) {
            if uses[&key(a, b)] == 1 {
                let (from, to) = (vertex(a), vertex(b));
                let inward = cross(normal, sub(to, from));
                let length = inward.iter().map(|v| v * v).sum::<f64>().sqrt();
                if length > 0.0 {
                    edges.push(PaintEdge {
                        from,
                        to,
                        inward: [inward[0] / length, inward[1] / length],
                    });
                }
            }
        }
    }
    edges
}

fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    std::array::from_fn(|i| a[i] - b[i])
}

fn cross([ax, ay, az]: [f64; 3], [bx, by, bz]: [f64; 3]) -> [f64; 3] {
    [ay * bz - az * by, az * bx - ax * bz, ax * by - ay * bx]
}
