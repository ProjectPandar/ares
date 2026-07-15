use crate::{ProjectVolume, project::transform::Transform3d};

use super::layers::LayerCandidateRange;

const EPSILON: f64 = 1e-4;

pub(crate) fn model_part_occupies_range(
    print_object_without_xy: Transform3d,
    volume: &ProjectVolume,
    normalized_range_count: usize,
    range: LayerCandidateRange,
) -> bool {
    let mesh = volume.mesh();
    if normalized_range_count == 1 {
        return !mesh.triangles().is_empty();
    }

    let combined = print_object_without_xy
        .then(volume.transform())
        .without_xy_translation();
    let expanded_min = range.min_z - EPSILON;
    let expanded_max = range.max_z + EPSILON;
    let vertices = mesh.vertices();

    mesh.triangles().iter().any(|triangle| {
        let z = triangle.map(|index| combined.transform_z_f32(vertices[index as usize]));
        [(z[2], z[0]), (z[0], z[1]), (z[1], z[2])]
            .into_iter()
            .any(|(first, second)| {
                let (lower, upper) = if first <= second {
                    (first, second)
                } else {
                    (second, first)
                };
                f64::from(upper) > expanded_min && f64::from(lower) < expanded_max
            })
    })
}
