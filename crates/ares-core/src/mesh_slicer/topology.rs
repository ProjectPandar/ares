use crate::{SliceError, geometry::fixed_gcc_sort_by};

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct MeshTopology {
    face_edge_ids: Vec<[u32; 3]>,
    edge_count: u64,
}

impl MeshTopology {
    pub(crate) fn build_face_neighbors(&self) -> Vec<[Option<usize>; 3]> {
        let mut neighbors = vec![[None; 3]; self.face_edge_ids.len()];
        let mut first_use: Vec<Option<(usize, usize)>> = vec![None; self.edge_count as usize];
        for (slot, &edge) in self.face_edge_ids.iter().flatten().enumerate() {
            let (face, local) = (slot / 3, slot % 3);
            if let Some((other_face, other_local)) = first_use[edge as usize] {
                neighbors[face][local] = Some(other_face);
                neighbors[other_face][other_local] = Some(face);
            } else {
                first_use[edge as usize] = Some((face, local));
            }
        }
        neighbors
    }
    pub(crate) fn face_edge_ids(&self) -> &[[u32; 3]] {
        &self.face_edge_ids
    }

    pub(crate) const fn edge_count(&self) -> u64 {
        self.edge_count
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct EdgeUse {
    pub(super) low: u32,
    pub(super) high: u32,
    pub(super) face: usize,
    pub(super) local_edge: usize,
    pub(super) reversed: bool,
}

impl EdgeUse {
    const fn key(self) -> (u32, u32) {
        (self.low, self.high)
    }
}

pub(crate) fn index_mesh_edges(triangles: &[[u32; 3]]) -> Result<MeshTopology, SliceError> {
    let mut uses = sorted_edge_uses(triangles);

    let mut face_edge_ids = vec![[0; 3]; triangles.len()];
    let mut group_start = 0;
    let mut edge_count = 0_u64;
    while group_start < uses.len() {
        let group_end = equal_key_group_end(&uses, group_start);
        let group = &mut uses[group_start..group_end];
        // TriangleMesh.cpp:565-613: pair opposite first, then same direction.
        for i in 0..group.len() {
            let first = group[i];
            if first.face == usize::MAX {
                continue;
            }
            let remaining = &group[i + 1..];
            let partner = remaining
                .iter()
                .position(|candidate| {
                    candidate.face != usize::MAX && candidate.reversed != first.reversed
                })
                .or_else(|| {
                    remaining
                        .iter()
                        .position(|candidate| candidate.face != usize::MAX)
                });
            let edge_id = checked_edge_id(edge_count)?;
            assign_edge_id(&mut face_edge_ids, first, edge_id);
            if let Some(partner) = partner {
                let partner = i + 1 + partner;
                assign_edge_id(&mut face_edge_ids, group[partner], edge_id);
                // The upstream consumed-face sentinel is -1.
                group[partner].face = usize::MAX;
            }
            edge_count = edge_count.checked_add(1).ok_or_else(edge_range_error)?;
        }
        group_start = group_end;
    }

    Ok(MeshTopology {
        face_edge_ids,
        edge_count,
    })
}

pub(super) fn sorted_edge_uses(triangles: &[[u32; 3]]) -> Vec<EdgeUse> {
    let mut uses = Vec::with_capacity(triangles.len() * 3);
    for (face, triangle) in triangles.iter().enumerate() {
        for local_edge in 0..3 {
            let start = triangle[local_edge];
            let end = triangle[(local_edge + 1) % 3];
            let (low, high, reversed) = if start <= end {
                (start, end, false)
            } else {
                (end, start, true)
            };
            uses.push(EdgeUse {
                low,
                high,
                face,
                local_edge,
                reversed,
            });
        }
    }
    // TriangleMesh.cpp:531/558: compare endpoints only with the reference sort.
    fixed_gcc_sort_by(&mut uses, |left, right| left.key() < right.key());
    uses
}

pub(super) fn checked_edge_id(edge_index: u64) -> Result<u32, SliceError> {
    u32::try_from(edge_index).map_err(|_| edge_range_error())
}

fn equal_key_group_end(uses: &[EdgeUse], group_start: usize) -> usize {
    let key = uses[group_start].key();
    let mut group_end = group_start + 1;
    while group_end < uses.len() && uses[group_end].key() == key {
        group_end += 1;
    }
    group_end
}

fn assign_edge_id(face_edge_ids: &mut [[u32; 3]], edge_use: EdgeUse, edge_id: u32) {
    face_edge_ids[edge_use.face][edge_use.local_edge] = edge_id;
}

fn edge_range_error() -> SliceError {
    SliceError::InvalidInput("project mesh edge count exceeds supported range".to_owned())
}
