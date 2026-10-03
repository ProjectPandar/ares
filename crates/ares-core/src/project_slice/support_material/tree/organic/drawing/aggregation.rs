//! Ordered per-branch, per-tree and cross-tree polygon accumulation.

use super::BranchSlices;
use crate::geometry::{ClipperError, Polygon, union_polygons_paths};

#[derive(Default)]
pub(in super::super) struct Slice {
    pub(in super::super) polygons: Vec<Polygon>,
    pub(in super::super) bottom_contacts: Vec<Polygon>,
    pub(in super::super) num_branches: usize,
}

#[derive(Default)]
pub(super) struct TreeSlices {
    first: Option<usize>,
    slices: Vec<Slice>,
}

impl TreeSlices {
    pub(super) fn insert(&mut self, branch: BranchSlices) {
        if branch.polygons.is_empty() {
            return;
        }
        let begin = self
            .first
            .map_or(branch.begin, |first| first.min(branch.begin));
        let end = self
            .first
            .map_or(branch.begin + branch.polygons.len(), |first| {
                (first + self.slices.len()).max(branch.begin + branch.polygons.len())
            });
        if let Some(first) = self.first
            && first > begin
        {
            self.slices
                .splice(..0, (0..first - begin).map(|_| Slice::default()));
        }
        self.slices.resize_with(end - begin, Slice::default);
        let mut contacts = branch.contacts.into_iter();
        for (index, polygons) in branch.polygons.into_iter().enumerate() {
            self.slices[branch.begin - begin + index]
                .append(polygons, contacts.next().unwrap_or_default());
        }
        self.first = Some(begin);
    }

    pub(super) fn union_branches(&mut self) -> Result<(), ClipperError> {
        for slice in &mut self.slices {
            if slice.num_branches <= 1 {
                continue;
            }
            if !slice.polygons.is_empty() {
                slice.polygons = union_polygons_paths(&slice.polygons)?;
            }
            if !slice.bottom_contacts.is_empty() {
                slice.bottom_contacts = union_polygons_paths(&slice.bottom_contacts)?;
            }
            slice.num_branches = 1;
        }
        Ok(())
    }
}

impl Slice {
    fn append(&mut self, mut polygons: Vec<Polygon>, mut contacts: Vec<Polygon>) {
        if polygons.is_empty() && contacts.is_empty() {
            return;
        }
        self.num_branches += 1;
        if self.num_branches > 1 {
            self.polygons.append(&mut polygons);
            self.bottom_contacts.append(&mut contacts);
        } else {
            self.polygons = polygons;
            self.bottom_contacts = contacts;
        }
    }
}

pub(super) fn combine_trees(trees: Vec<TreeSlices>) -> Vec<Slice> {
    let count = trees
        .iter()
        .filter_map(|tree| tree.first.map(|first| first + tree.slices.len()))
        .max()
        .unwrap_or(0);
    let mut output: Vec<Slice> = (0..count).map(|_| Slice::default()).collect();
    for tree in trees {
        if let Some(first) = tree.first {
            for (index, slice) in tree.slices.into_iter().enumerate() {
                output[first + index].append(slice.polygons, slice.bottom_contacts);
            }
        }
    }
    output
}
