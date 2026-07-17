use crate::SliceError;

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct MeshTopology {
    face_edge_ids: Vec<[u32; 3]>,
    edge_count: u64,
}

impl MeshTopology {
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
    let uses = sorted_edge_uses(triangles);
    reject_non_manifold_groups(&uses)?;

    let mut face_edge_ids = vec![[0; 3]; triangles.len()];
    let mut group_start = 0;
    let mut edge_count = 0_u64;
    while group_start < uses.len() {
        let group_end = equal_key_group_end(&uses, group_start);
        let group = &uses[group_start..group_end];
        let edge_id = checked_edge_id(edge_count)?;
        assign_edge_id(&mut face_edge_ids, group[0], edge_id);

        if group.len() == 2 {
            let first = group[0];
            let second = group[1..]
                .iter()
                .copied()
                .find(|candidate| candidate.reversed != first.reversed)
                .unwrap_or(group[1]);
            assign_edge_id(&mut face_edge_ids, second, edge_id);
        }

        edge_count = edge_count.checked_add(1).ok_or_else(edge_range_error)?;
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
    uses.sort_unstable_by_key(|edge_use| {
        (
            edge_use.low,
            edge_use.high,
            edge_use.face,
            edge_use.local_edge,
        )
    });
    uses
}

pub(super) fn checked_edge_id(edge_index: u64) -> Result<u32, SliceError> {
    u32::try_from(edge_index).map_err(|_| edge_range_error())
}

fn reject_non_manifold_groups(uses: &[EdgeUse]) -> Result<(), SliceError> {
    let mut group_start = 0;
    while group_start < uses.len() {
        let group_end = equal_key_group_end(uses, group_start);
        if group_end - group_start > 2 {
            return Err(SliceError::UnsupportedProjectFeature(
                "mesh_topology".to_owned(),
            ));
        }
        group_start = group_end;
    }
    Ok(())
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
