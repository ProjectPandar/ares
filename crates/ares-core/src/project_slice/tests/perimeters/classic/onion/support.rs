use crate::project_slice::perimeters::prepare_post_classic_onion;

use super::super::super::super::support::{KsrArchive, ksr_project};

pub(super) fn archive() -> KsrArchive {
    KsrArchive::new()
}

pub(super) fn project() -> &'static [u8] {
    ksr_project()
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct SurfaceSummary {
    pub(super) source_index: usize,
    pub(super) initial: i32,
    pub(super) effective: i32,
    pub(super) depths: Vec<i32>,
    pub(super) depth_zero_normal: usize,
    pub(super) depth_zero_smaller: usize,
    pub(super) last: usize,
    pub(super) gaps: usize,
    pub(super) geometry_checksum: i128,
}

pub(super) fn summaries(input: impl AsRef<[u8]>) -> Vec<SurfaceSummary> {
    prepare_post_classic_onion(input.as_ref())
        .unwrap()
        .objects
        .iter()
        .flat_map(|object| &object.records)
        .flatten()
        .flat_map(|record| &record.surfaces)
        .map(|surface| {
            let geometry_checksum = surface
                .shells
                .iter()
                .flat_map(|shell| shell.normal.iter().chain(&shell.smaller_width))
                .chain(&surface.last)
                .chain(&surface.gaps)
                .flat_map(|polygon| polygon.contour().points())
                .map(|point| i128::from(point.x()) + 31 * i128::from(point.y()))
                .sum();
            SurfaceSummary {
                source_index: surface.source_index,
                initial: surface.initial_loop_number,
                effective: surface.effective_loop_number,
                depths: surface.shells.iter().map(|shell| shell.depth).collect(),
                depth_zero_normal: surface.shells.first().map_or(0, |shell| shell.normal.len()),
                depth_zero_smaller: surface
                    .shells
                    .first()
                    .map_or(0, |shell| shell.smaller_width.len()),
                last: surface.last.len(),
                gaps: surface.gaps.len(),
                geometry_checksum,
            }
        })
        .collect()
}
