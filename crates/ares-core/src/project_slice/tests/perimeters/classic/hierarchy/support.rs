use crate::project_slice::perimeters::{
    prepare_post_classic_hierarchy, prepare_post_classic_onion,
};

pub(super) use super::super::super::super::support::ksr_project as project;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct Summary {
    pub(super) source_index: usize,
    pub(super) roots: usize,
    pub(super) root_checksum: i128,
    pub(super) orphan_contours: usize,
    pub(super) orphan_holes: usize,
    pub(super) raw_checksum: i128,
}

pub(super) fn summaries(input: impl AsRef<[u8]>) -> Vec<Summary> {
    let prepared = prepare_post_classic_hierarchy(input).unwrap();
    prepared
        .objects
        .iter()
        .flat_map(|object| object.records.iter().zip(&object.predecessor.records))
        .filter_map(|(hierarchy, onion)| Some((hierarchy.as_ref()?, onion.as_ref()?)))
        .flat_map(|(hierarchy, onion)| hierarchy.surfaces.iter().zip(&onion.surfaces))
        .map(|(hierarchy, onion)| Summary {
            source_index: hierarchy.source_index,
            roots: hierarchy.roots.len(),
            root_checksum: loop_checksum(&hierarchy.roots),
            orphan_contours: hierarchy.remaining_contours.iter().map(Vec::len).sum(),
            orphan_holes: hierarchy.remaining_holes.iter().map(Vec::len).sum(),
            raw_checksum: onion
                .shells
                .iter()
                .flat_map(|shell| shell.normal.iter().chain(&shell.smaller_width))
                .flat_map(|ex| std::iter::once(ex.contour()).chain(ex.holes()))
                .flat_map(|polygon| polygon.points())
                .map(|point| i128::from(point.x()) + 31 * i128::from(point.y()))
                .sum(),
        })
        .collect()
}

pub(super) fn direct_raw_checksums(input: impl AsRef<[u8]>) -> Vec<(usize, i128)> {
    prepare_post_classic_onion(input)
        .unwrap()
        .objects
        .iter()
        .flat_map(|object| &object.records)
        .flatten()
        .flat_map(|record| &record.surfaces)
        .map(|surface| {
            let checksum = surface
                .shells
                .iter()
                .flat_map(|shell| shell.normal.iter().chain(&shell.smaller_width))
                .flat_map(|ex| std::iter::once(ex.contour()).chain(ex.holes()))
                .flat_map(|polygon| polygon.points())
                .map(|point| i128::from(point.x()) + 31 * i128::from(point.y()))
                .sum();
            (surface.source_index, checksum)
        })
        .collect()
}

fn loop_checksum(
    roots: &[crate::project_slice::perimeters::classic::hierarchy::PerimeterGeneratorLoop],
) -> i128 {
    let mut checksum = 0_i128;
    let mut pending = roots.iter().rev().collect::<Vec<_>>();
    while let Some(loop_) = pending.pop() {
        checksum = checksum
            .wrapping_mul(37)
            .wrapping_add(i128::from(loop_.depth))
            .wrapping_add(i128::from(u8::from(loop_.is_contour)))
            .wrapping_add(2 * i128::from(u8::from(loop_.is_smaller_width_perimeter)));
        for point in loop_.polygon.points() {
            checksum = checksum
                .wrapping_mul(31)
                .wrapping_add(i128::from(point.x()) + 7 * i128::from(point.y()));
        }
        pending.extend(loop_.children.iter().rev());
    }
    checksum
}
