//! MultiMaterialSegmentation.cpp1825–1870: default-state masks and side/slab merge.
use crate::geometry::{
    ClipperError, CoordinateScale, ExPolygon, JoinType, difference_ex, offset2_ex, union_expolygons,
};
fn merge_layer(
    sides: Vec<Vec<ExPolygon>>,
    caps: &[Vec<Vec<ExPolygon>>],
    layer: usize,
    epsilon: f32,
) -> Result<Vec<Vec<ExPolygon>>, ClipperError> {
    let mut colors = Vec::with_capacity(sides.len() - 1);
    for (state, mut polygons) in sides.into_iter().enumerate().skip(1) {
        for cap in caps {
            if !cap[layer].is_empty() && !polygons.is_empty() {
                polygons = difference_ex(&polygons, &cap[layer])?;
            }
        }
        if !caps.is_empty() && !caps[state][layer].is_empty() {
            let empty = polygons.is_empty();
            polygons.extend(caps[state][layer].iter().cloned());
            if !empty {
                polygons = offset2_ex(
                    &union_expolygons(&polygons, &[])?,
                    epsilon,
                    -epsilon,
                    JoinType::Miter,
                    3.0,
                )?;
            }
        }
        colors.push(polygons);
    }
    Ok(colors)
}
pub(super) fn merge(
    side: Vec<Vec<Vec<ExPolygon>>>,
    caps: Vec<Vec<Vec<ExPolygon>>>,
    scale: CoordinateScale,
) -> Result<Vec<Vec<Vec<ExPolygon>>>, ClipperError> {
    let epsilon = scale.scaled_delta(1e-4) as f32;
    let output = side
        .into_iter()
        .enumerate()
        .map(|(layer, sides)| merge_layer(sides, &caps, layer, epsilon))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(output)
}
