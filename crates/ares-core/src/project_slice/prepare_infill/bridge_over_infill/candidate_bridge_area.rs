use crate::geometry::{
    BoundingBox, ClipperError, Coord, CoordinateScale, ExPolygon, JoinType, Polygon,
    clip_clipper_polygons_with_subject_bbox, intersection_ex_polygons,
    intersection_polygons_polygons_ex, offset_paths, union_polygons_paths,
};

const MITER_LIMIT: f64 = 3.0;

pub(in crate::project_slice) struct CandidateBridgeAreaInput<'a> {
    pub(in crate::project_slice) candidate_polygons: &'a [Polygon],
    pub(in crate::project_slice) deep_infill_area: &'a [Polygon],
    pub(in crate::project_slice) internal_unsupported_area: &'a [Polygon],
    pub(in crate::project_slice) expansion_area: &'a [Polygon],
    pub(in crate::project_slice) scaled_spacing: Coord,
    pub(in crate::project_slice) scale: CoordinateScale,
}

pub(in crate::project_slice) struct CandidateBridgeArea {
    pub(in crate::project_slice) area_to_be_bridge: Vec<Polygon>,
    pub(in crate::project_slice) limiting_area: Vec<Polygon>,
}

pub(in crate::project_slice) fn prepare_candidate_bridge_area(
    input: CandidateBridgeAreaInput<'_>,
) -> Result<CandidateBridgeArea, ClipperError> {
    prepare_candidate_bridge_area_using(
        input,
        |paths, delta| offset_paths(paths, delta, JoinType::Miter, MITER_LIMIT),
        intersection_polygons_polygons_ex,
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
    Intersect: FnMut(&[Polygon], &[Polygon]) -> Result<Vec<ExPolygon>, ClipperError>,
    Union: FnOnce(&[Polygon]) -> Result<Vec<Polygon>, ClipperError>,
{
    let CandidateBridgeAreaInput {
        candidate_polygons,
        deep_infill_area,
        internal_unsupported_area,
        expansion_area,
        scaled_spacing,
        scale,
    } = input;
    debug_assert!(scaled_spacing > 0);
    let expanded = offset(candidate_polygons, scaled_spacing as f32)?;
    let scaled_epsilon = scale.scaled_delta(1e-4) as Coord;
    let components = if expanded.is_empty() {
        Vec::new()
    } else {
        let bounds = BoundingBox::from_polygons(&expanded)
            .expect("expanded bridge polygons have nonempty contours")
            .inflated(scaled_epsilon);
        let clipped_deep = clip_clipper_polygons_with_subject_bbox(deep_infill_area, bounds);
        intersect(&expanded, &clipped_deep)?
    };
    let mut area_to_be_bridge = Vec::with_capacity(components.len());
    for component in components {
        let bounds = BoundingBox::from_expolygon(&component)
            .expect("bridge intersection components have nonempty contours")
            .inflated(scaled_epsilon);
        let clipped_unsupported =
            clip_clipper_polygons_with_subject_bbox(internal_unsupported_area, bounds);
        if !intersection_ex_polygons(std::slice::from_ref(&component), &clipped_unsupported)?
            .is_empty()
        {
            let (contour, holes) = component.into_parts();
            area_to_be_bridge.push(contour);
            area_to_be_bridge.extend(holes);
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
