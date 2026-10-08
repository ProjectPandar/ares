use crate::project_slice::volume_regions::VolumeRegionRange;

const EPSILON: f64 = 1e-4;

// PrintObjectSlice.cpp::layer_range_first.
pub(super) fn first(ranges: &[VolumeRegionRange], z: f64) -> usize {
    let mut index =
        ranges.partition_point(|range| range.max_z < z && (range.max_z - z).abs() > EPSILON);
    if ranges[index].max_z == z && index + 1 < ranges.len() && ranges[index + 1].min_z == z {
        index += 1;
    }
    index
}

// PrintObjectSlice.cpp::layer_range_next, including the first complex layer.
pub(super) fn next(ranges: &[VolumeRegionRange], mut index: usize, z: f64) -> usize {
    while ranges[index].max_z <= z + EPSILON {
        index += 1;
    }
    index
}
