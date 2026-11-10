//! MultiMaterialSegmentation.cpp:1450-1498 and ClipperUtils.cpp:904-928.
use super::super::{math::paths, tiles::tiles};
use super::{Layer, Stat};
use crate::geometry::{
    ClipperError, CoordinateScale, ExPolygon, JoinType, clip_clipper_expolygons_with_subject_bbox,
    intersection_ex, intersection_polygons_polygons_ex, offset_expolygons, opening_ex,
};

pub(super) fn project_to_shells(
    patch: &[ExPolygon],
    (layers, scale): (&[Layer], CoordinateScale),
    source: usize,
    targets: &[usize],
    stat: &Stat,
) -> Result<Vec<Vec<ExPolygon>>, ClipperError> {
    let mut offset = 0.0_f32;
    let offsets = targets
        .iter()
        .map(|_| {
            offset -= stat.spacing + stat.width;
            offset
        })
        .collect::<Vec<_>>();
    let mut output = vec![Vec::new(); targets.len()];
    let Some(last) = offsets.last() else {
        return Ok(output);
    };
    let reach =
        (3.0_f64 * f64::from(last.abs())).ceil() as i64 + 10 * scale.scaled_delta(1e-4) as i64;
    for tile in tiles(patch, 16) {
        let bbox = tile.bounds.unwrap().inflated(reach);
        let tile_patch = tile
            .members
            .iter()
            .map(|&i| patch[i].clone())
            .collect::<Vec<_>>();
        let mut slices = clip_clipper_expolygons_with_subject_bbox(&layers[source].input, bbox);
        for (index, (&target, &offset)) in targets.iter().zip(&offsets).enumerate() {
            if slices.is_empty() {
                break;
            }
            let clip = clip_clipper_expolygons_with_subject_bbox(&layers[target].input, bbox);
            let trimmed = intersection_polygons_polygons_ex(&slices, &clip)?;
            output[index].extend(opening_ex(
                &intersection_ex(
                    &tile_patch,
                    &offset_expolygons(&trimmed, offset, JoinType::Miter, 3.0)?,
                )?,
                stat.small,
                JoinType::Miter,
                3.0,
            )?);
            slices = paths(trimmed);
        }
    }
    if let Some(index) = output.iter().position(Vec::is_empty) {
        output.truncate(index);
    }
    Ok(output)
}
