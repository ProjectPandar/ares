use crate::SliceError;

use super::super::{top_split::PreparedPostClassicTopSplit, types::ClassicPreludeRecord};

const INSET_OVERLAP_TOLERANCE: f64 = 0.4;

#[derive(Clone, Copy, Debug)]
pub(super) struct ValidatedOnionConfig {
    pub(super) sparse_infill_density: i32,
    pub(super) has_gap_fill: bool,
    pub(super) minimum_spacing: i64,
    pub(super) external_to_internal_spacing: i64,
    pub(super) perimeter_spacing: i64,
    pub(super) gap_fill_room: f64,
    pub(super) inner_wall_room: Option<f64>,
}

pub(super) fn validate_project(
    prepared: &PreparedPostClassicTopSplit,
) -> Result<Vec<Vec<Option<ValidatedOnionConfig>>>, SliceError> {
    prepared
        .objects
        .iter()
        .map(|object| {
            let inputs = object.predecessor.object.as_parts().1;
            let records = inputs
                .iter()
                .zip(&object.predecessor.records)
                .zip(&object.records)
                .map(
                    |((input, prelude), top_split)| match (input, prelude, top_split) {
                        (Some(input), Some(prelude), Some(_)) => {
                            Some((object.predecessor.object.region_options(input), prelude))
                        }
                        (None, None, None) => None,
                        _ => unreachable!("Task 22O.2 record slots must remain aligned"),
                    },
                )
                .collect::<Vec<_>>();
            let densities = validate_densities(records.iter().map(|record| {
                record
                    .as_ref()
                    .map(|(region, _)| region.sparse_infill_density.0)
            }))?;
            Ok(records
                .into_iter()
                .zip(densities)
                .map(|(record, density)| match (record, density) {
                    (Some((region, prelude)), Some(density)) => {
                        Some(from_prelude(density, prelude, region.wall_sequence))
                    }
                    (None, None) => None,
                    _ => unreachable!("validated density slots must remain aligned"),
                })
                .collect())
        })
        .collect()
}

fn from_prelude(
    density: i32,
    prelude: &ClassicPreludeRecord,
    wall_sequence: crate::ProcessWallSequence,
) -> ValidatedOnionConfig {
    // PerimeterGenerator.cpp:1507,1714: nominal widths, before top-wall reduction.
    let gap_fill_minimum = 0.2
        * prelude.perimeter_width.min(prelude.external_width) as f64
        * (1.0 - INSET_OVERLAP_TOLERANCE);
    ValidatedOnionConfig {
        sparse_infill_density: density,
        has_gap_fill: prelude.has_gap_fill,
        minimum_spacing: prelude.minimum_spacing,
        external_to_internal_spacing: prelude.external_to_internal_spacing,
        perimeter_spacing: prelude.perimeter_spacing,
        gap_fill_room: 0.5 * (prelude.external_to_internal_spacing as f64 + gap_fill_minimum),
        // PerimeterGenerator.cpp:1713-1715: this reach exists only for sandwich order.
        inner_wall_room: (wall_sequence == crate::ProcessWallSequence::InnerOuterInner).then_some(
            prelude.external_to_internal_spacing as f64 + prelude.minimum_spacing as f64 / 2.0,
        ),
    }
}

fn validate_densities(
    values: impl IntoIterator<Item = Option<f64>>,
) -> Result<Vec<Option<i32>>, SliceError> {
    values
        .into_iter()
        .map(|value| value.map(validate_density).transpose())
        .collect()
}

fn validate_density(value: f64) -> Result<i32, SliceError> {
    if value.is_finite() && (0.0..=100.0).contains(&value) {
        Ok(value as i32)
    } else {
        Err(SliceError::InvalidInput(
            "invalid Orca option sparse_infill_density".to_owned(),
        ))
    }
}
