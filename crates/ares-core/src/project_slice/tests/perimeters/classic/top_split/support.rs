use crate::project_slice::perimeters::{
    classic::top_split::{TopSplitOutcome, TopSplitUpperSource},
    prepare_post_classic_top_split,
};

use super::super::super::super::support::{KsrArchive, ksr_project};

pub(super) use super::super::super::super::support::metadata;

pub(super) fn archive() -> KsrArchive {
    KsrArchive::new()
}

pub(super) fn project() -> &'static [u8] {
    ksr_project()
}

pub(super) fn outcomes(input: impl AsRef<[u8]>) -> Vec<TopSplitOutcome> {
    prepare_post_classic_top_split(input)
        .unwrap()
        .objects
        .into_iter()
        .flat_map(|object| object.into_parts().1)
        .flatten()
        .flat_map(|record| record.surfaces)
        .map(|surface| surface.outcome)
        .collect()
}

pub(super) fn upper_sources(input: impl AsRef<[u8]>) -> Vec<TopSplitUpperSource> {
    prepare_post_classic_top_split(input)
        .unwrap()
        .objects
        .into_iter()
        .flat_map(|object| object.into_parts().1)
        .flatten()
        .flat_map(|record| record.surfaces)
        .map(|surface| surface.upper_source)
        .collect()
}

pub(super) fn geometry_summary(
    input: impl AsRef<[u8]>,
) -> Vec<(usize, usize, usize, usize, i128, i128)> {
    prepare_post_classic_top_split(input)
        .unwrap()
        .objects
        .into_iter()
        .flat_map(|object| object.into_parts().1)
        .flatten()
        .flat_map(|record| record.surfaces)
        .map(|surface| {
            let remaining_sum = surface
                .remaining
                .iter()
                .flat_map(|value| value.contour().points())
                .map(|point| i128::from(point.x()) + 31 * i128::from(point.y()))
                .sum();
            let fill_clip_sum = surface
                .fill_clip
                .iter()
                .flat_map(|value| value.contour().points())
                .map(|point| i128::from(point.x()) + 31 * i128::from(point.y()))
                .sum();
            (
                surface.normal_first_offset.len(),
                surface.smaller_first_offset.len(),
                surface.remaining.len(),
                surface.fill_clip.len(),
                remaining_sum,
                fill_clip_sum,
            )
        })
        .collect()
}
