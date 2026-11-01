use crate::{
    ProjectObject, SliceError,
    mesh_slicer::{IntersectionLine, RawIntersectionBudget, slice_mesh_on_planes},
    options::FilamentRegionSourceOptions,
    project::{effective_config::types::ResolvedProjectObject, region_graph::object_region_graph},
};

// PrintObjectSlice.cpp:213-219: use the existing PrintApply region graph's
// has_volume membership; modifier parent records may repeat a volume per range.
pub(super) fn active_volume_ranges(
    source: &ProjectObject,
    resolved: &ResolvedProjectObject,
    filament: &FilamentRegionSourceOptions,
    logical_count: usize,
    process_variant_indices: &[usize],
) -> Vec<Vec<(f64, f64)>> {
    let graph = object_region_graph(
        source,
        resolved,
        filament,
        logical_count,
        process_variant_indices,
    );
    (0..source.volumes().len())
        .map(|index| {
            graph
                .ranges
                .iter()
                .zip(&resolved.layer_candidates)
                .filter(|(records, _)| {
                    records
                        .iter()
                        .any(|record| record.source_volume_index == index)
                })
                .map(|(_, candidate)| (candidate.min_z, candidate.max_z))
                .collect()
        })
        .collect()
}

// PrintObjectSlice.cpp:100-136: float Z filtering, single-interval shortcut,
// then scatter into the original aligned layer slots without resampling.
pub(super) fn slice_volume(
    vertices: &[[f32; 3]],
    triangles: &[[u32; 3]],
    planes: &[f32],
    ranges: &[(f64, f64)],
    budget: &mut RawIntersectionBudget,
) -> Result<Vec<Vec<IntersectionLine>>, SliceError> {
    if planes.is_empty() || ranges.is_empty() {
        return Ok(Vec::new());
    }
    if let [(low, high)] = ranges
        && f64::from(planes[0]) >= *low
        && f64::from(planes[planes.len() - 1]) < *high
    {
        return slice_mesh_on_planes(vertices, triangles, planes, budget);
    }
    let mut filtered = Vec::with_capacity(planes.len());
    let mut spans = Vec::with_capacity(2 * ranges.len());
    let mut index = 0;
    for &(low, high) in ranges {
        while index < planes.len() && f64::from(planes[index]) < low {
            index += 1;
        }
        let first = index;
        while index < planes.len() && f64::from(planes[index]) < high {
            filtered.push(planes[index]);
            index += 1;
        }
        if index > first {
            spans.push(first..index);
        }
    }
    if spans.is_empty() {
        return Ok(Vec::new());
    }
    let layers = slice_mesh_on_planes(vertices, triangles, &filtered, budget)?;
    let mut output = vec![Vec::new(); planes.len()];
    let mut slices = layers.into_iter();
    for span in spans {
        for (slot, layer) in output[span].iter_mut().zip(&mut slices) {
            *slot = layer;
        }
    }
    Ok(output)
}
