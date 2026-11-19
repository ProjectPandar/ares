//! The single-shell, exact-only branch of `MeshRepair.cpp::repair_its`.
use std::collections::{HashMap, HashSet, hash_map::Entry};

use super::{Face, SliceError, Vertex, coordinate_key, cross, dot, minus, unsupported};

pub(super) fn exact_cleanup(vertices: &[Vertex], faces: &mut Vec<Face>) -> Result<(), SliceError> {
    if !one_soup(vertices.len(), faces) {
        return Err(unsupported(
            "STL repair requires one vertex-connected input shell",
        ));
    }
    // clean_soup uses the lowest original index for exact coincident vertices.
    let mut index = HashMap::new();
    let merged: Vec<_> = vertices
        .iter()
        .enumerate()
        .map(|(i, &p)| *index.entry(coordinate_key(p)).or_insert(i))
        .collect();
    let mut cleaned = Vec::with_capacity(faces.len());
    let mut seen = HashMap::new();
    for face in faces.iter().map(|face| face.map(|i| merged[i])) {
        let [a, b, c] = face;
        if a == b || b == c || a == c {
            continue;
        }
        let mut key = face;
        key.sort_unstable();
        let odd = (a > b) ^ (b > c) ^ (a > c);
        match seen.entry(key) {
            Entry::Vacant(entry) => {
                entry.insert(odd);
                cleaned.push(face);
            }
            Entry::Occupied(entry) if *entry.get() != odd => {
                return Err(unsupported("STL repair of opposite coincident faces"));
            }
            Entry::Occupied(_) => {}
        }
    }
    let Some(used) = closed_shell(&cleaned) else {
        return Err(unsupported(
            "STL repair requires one closed consistently wound shell after exact cleanup",
        ));
    };
    if !outward_convex(vertices, &cleaned, used) {
        return Err(unsupported(
            "STL repair requires an outward convex shell enclosing volume",
        ));
    }
    if cleaned != *faces {
        if closed_shell(faces).is_some() {
            return Err(unsupported(
                "STL repair cannot weld an already closed indexed shell",
            ));
        }
        // CGAL's add_face records the closing halfedge. MeshRepair.cpp:1175–1177
        // exports its source, its target, then the next halfedge's target.
        for face in &mut cleaned {
            face.rotate_right(1);
        }
        *faces = cleaned;
    }
    Ok(())
}

fn find(parents: &mut [usize], mut i: usize) -> usize {
    while parents[i] != i {
        parents[i] = parents[parents[i]];
        i = parents[i];
    }
    i
}

fn union(parents: &mut [usize], a: usize, b: usize) {
    let a = find(parents, a);
    let b = find(parents, b);
    parents[a.max(b)] = a.min(b);
}

fn one_soup(vertices: usize, faces: &[Face]) -> bool {
    let mut parents: Vec<_> = (0..vertices).collect();
    for &[a, b, c] in faces {
        union(&mut parents, a, b);
        union(&mut parents, a, c);
    }
    let root = find(&mut parents, faces[0][0]);
    faces.iter().all(|f| find(&mut parents, f[0]) == root)
}

fn closed_shell(faces: &[Face]) -> Option<HashSet<usize>> {
    let mut parents: Vec<_> = (0..faces.len()).collect();
    let mut edges = HashMap::new();
    let mut used = HashSet::new();
    for (i, &[a, b, c]) in faces.iter().enumerate() {
        used.extend([a, b, c]);
        for (from, to) in [(a, b), (b, c), (c, a)] {
            if from == to {
                return None;
            }
            let forward = from < to;
            match edges.entry((from.min(to), from.max(to))) {
                Entry::Vacant(entry) => {
                    entry.insert(Some((i, forward)));
                }
                Entry::Occupied(mut entry) => {
                    let (other, _) = entry.get_mut().take().filter(|&(_, dir)| dir != forward)?;
                    union(&mut parents, i, other);
                }
            }
        }
    }
    // A convex shell has Euler characteristic two. With one edge-connected
    // oriented component this also excludes pinched vertex fans: splitting any
    // such fan would raise the characteristic above two.
    (faces.len() >= 4
        && used.len() + faces.len() == edges.len() + 2
        && edges.values().all(Option::is_none)
        && (0..faces.len()).all(|i| find(&mut parents, i) == 0))
    .then_some(used)
}

fn outward_convex(vertices: &[Vertex], faces: &[Face], used: HashSet<usize>) -> bool {
    let used: Vec<_> = used
        .into_iter()
        .map(|i| (i, vertices[i].map(f64::from)))
        .collect();
    let origin = vertices[faces[0][0]].map(f64::from);
    let mut area = 0.0;
    let mut volume = 0.0;
    for face in faces {
        let [a, b, c] = face.map(|i| vertices[i].map(f64::from));
        let normal = cross(minus(b, a), minus(c, a));
        if normal == [0.0; 3]
            || used
                .iter()
                .any(|&(i, p)| !face.contains(&i) && dot(normal, minus(p, a)) > 0.0)
        {
            return false;
        }
        area += dot(normal, normal).sqrt() * 0.5;
        volume += dot(normal, minus(a, origin)) / 6.0;
    }
    let diagonal = std::array::from_fn(|i| {
        let (min, max) = vertices
            .iter()
            .fold((f32::INFINITY, f32::NEG_INFINITY), |(min, max), p| {
                (min.min(p[i]), max.max(p[i]))
            });
        f64::from(max) - f64::from(min)
    });
    // Same dimension-scaled flat-shell cutoff as repair_soup.
    volume > 0.1 * (1e-4 * dot(diagonal, diagonal).sqrt()) * area
}
