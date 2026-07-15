use crate::LayerConfigRange;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct LayerCandidateRange {
    pub(crate) min_z: f64,
    pub(crate) max_z: f64,
    pub(crate) source_range_index: Option<usize>,
}

const EPSILON: f64 = 1e-4;

pub(crate) fn layer_candidate_ranges(source: &[LayerConfigRange]) -> Vec<LayerCandidateRange> {
    let mut ranges = Vec::with_capacity(source.len().saturating_add(1));
    let mut last_z = 0.0;

    for (source_range_index, source_range) in source.iter().enumerate() {
        let max_z = source_range.max_z();
        if max_z <= last_z {
            continue;
        }

        let min_z = source_range.min_z().max(0.0);
        if min_z > last_z + EPSILON {
            ranges.push(LayerCandidateRange {
                min_z: last_z,
                max_z: min_z,
                source_range_index: None,
            });
            last_z = min_z;
        }
        if max_z > last_z + EPSILON {
            ranges.push(LayerCandidateRange {
                min_z: last_z,
                max_z,
                source_range_index: Some(source_range_index),
            });
            last_z = max_z;
        }
    }

    if ranges.is_empty() {
        ranges.push(LayerCandidateRange {
            min_z: 0.0,
            max_z: f64::MAX,
            source_range_index: None,
        });
    } else if ranges
        .last()
        .is_some_and(|range| range.source_range_index.is_none())
    {
        ranges.last_mut().expect("range exists").max_z = f64::MAX;
    } else {
        ranges.push(LayerCandidateRange {
            min_z: last_z,
            max_z: f64::MAX,
            source_range_index: None,
        });
    }

    ranges
}

pub(crate) fn layer_range_source_index(
    ranges: &[LayerCandidateRange],
    requested: (f64, f64),
) -> Option<Option<usize>> {
    let key = (requested.0 - EPSILON, requested.1 - EPSILON);
    let index = ranges.partition_point(|range| (range.min_z, range.max_z) < key);
    let found = ranges.get(index)?;
    if (found.min_z - requested.0).abs() > EPSILON || (found.max_z - requested.1).abs() > EPSILON {
        return None;
    }
    Some(found.source_range_index)
}
