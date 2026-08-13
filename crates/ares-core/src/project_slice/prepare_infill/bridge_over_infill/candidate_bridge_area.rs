#[cfg(test)]
mod tests;

use crate::geometry::{
    ClipperError, Coord, JoinType, Polygon, intersection_polygons_paths, offset_paths,
    union_polygons_paths,
};

const MITER_LIMIT: f64 = 3.0;

struct CandidateBridgeAreaInput<'a> {
    candidate_polygons: &'a [Polygon],
    deep_infill_area: &'a [Polygon],
    internal_unsupported_area: &'a [Polygon],
    expansion_area: &'a [Polygon],
    scaled_spacing: Coord,
}

pub(in crate::project_slice) struct CandidateBridgeArea {
    pub(in crate::project_slice) area_to_be_bridge: Vec<Polygon>,
    pub(in crate::project_slice) limiting_area: Vec<Polygon>,
}

pub(in crate::project_slice) fn prepare_candidate_bridge_area(
    candidate_polygons: &[Polygon],
    deep_infill_area: &[Polygon],
    internal_unsupported_area: &[Polygon],
    expansion_area: &[Polygon],
    scaled_spacing: Coord,
) -> Result<CandidateBridgeArea, ClipperError> {
    prepare_candidate_bridge_area_using(
        CandidateBridgeAreaInput {
            candidate_polygons,
            deep_infill_area,
            internal_unsupported_area,
            expansion_area,
            scaled_spacing,
        },
        |paths, delta| offset_paths(paths, delta, JoinType::Miter, MITER_LIMIT),
        intersection_polygons_paths,
        union_polygons_paths,
    )
}

fn prepare_candidate_bridge_area_using<Offset, Intersect, Union>(
    input: CandidateBridgeAreaInput<'_>,
    mut offset: Offset,
    mut intersect: Intersect,
    union: Union,
) -> Result<CandidateBridgeArea, ClipperError>
where
    Offset: FnMut(&[Polygon], f32) -> Result<Vec<Polygon>, ClipperError>,
    Intersect: FnMut(&[Polygon], &[Polygon]) -> Result<Vec<Polygon>, ClipperError>,
    Union: FnOnce(&[Polygon]) -> Result<Vec<Polygon>, ClipperError>,
{
    let CandidateBridgeAreaInput {
        candidate_polygons,
        deep_infill_area,
        internal_unsupported_area,
        expansion_area,
        scaled_spacing,
    } = input;
    debug_assert!(scaled_spacing > 0);
    let expanded = offset(candidate_polygons, scaled_spacing as f32)?;
    let intersected = intersect(&expanded, deep_infill_area)?;
    let mut area_to_be_bridge = Vec::with_capacity(intersected.len());
    for polygon in intersected {
        if !intersect(std::slice::from_ref(&polygon), internal_unsupported_area)?.is_empty() {
            area_to_be_bridge.push(polygon);
        }
    }

    let mut union_input = area_to_be_bridge.clone();
    union_input.extend(expansion_area.iter().cloned());
    let limiting_area = union(&union_input)?;
    Ok(CandidateBridgeArea {
        area_to_be_bridge,
        limiting_area,
    })
}
