//! Brim.cpp:529-570,678-711: bounded inner-only area and contact retention.
use crate::{
    SliceError,
    geometry::{ExPolygon, JoinType, difference_ex, intersection_ex, offset_expolygons},
};

pub(crate) struct InnerArea {
    pub(crate) area: Vec<ExPolygon>,
    pub(crate) forbidden: Vec<ExPolygon>,
    pub(crate) object_islands: Vec<ExPolygon>,
}

pub(crate) fn area(
    source: &[ExPolygon],
    gap: f32,
    width: f32,
    spacing: f32,
    resolution: f64,
) -> Result<InnerArea, SliceError> {
    let mut area = Vec::new();
    let mut forbidden = Vec::new();
    for polygon in source {
        let holes = reversed_holes(polygon);
        area.extend(hole_area(&holes, gap, width)?);
        let contour = ExPolygon::new(polygon.contour().clone(), Vec::new());
        let expanded =
            offset_expolygons(&[contour], spacing, JoinType::Miter, 3.0).map_err(geometry_error)?;
        forbidden.extend(difference_ex(&expanded, &holes).map_err(geometry_error)?);
    }
    let object_islands =
        offset_expolygons(source, gap, JoinType::Round, resolution).map_err(geometry_error)?;
    forbidden.extend(object_islands.iter().cloned());
    Ok(InnerArea {
        area,
        forbidden,
        object_islands,
    })
}

pub(crate) fn reversed_holes(polygon: &ExPolygon) -> Vec<ExPolygon> {
    polygon
        .holes()
        .iter()
        .map(|hole| {
            let mut reversed = hole.clone();
            reversed.reverse();
            ExPolygon::new(reversed, Vec::new())
        })
        .collect()
}

pub(crate) fn hole_area(
    holes: &[ExPolygon],
    gap: f32,
    width: f32,
) -> Result<Vec<ExPolygon>, SliceError> {
    // Brim.cpp:547-558 uses the original paired width and default
    // jtMiter / limit3, independently of the outer brim_width_mod.
    let inset =
        offset_expolygons(holes, -width - gap, JoinType::Miter, 3.0).map_err(geometry_error)?;
    let outer = offset_expolygons(holes, -gap, JoinType::Miter, 3.0).map_err(geometry_error)?;
    let ring = difference_ex(&outer, &inset).map_err(geometry_error)?;
    intersection_ex(&ring, holes).map_err(geometry_error)
}

pub(crate) fn retain_contact(
    area: Vec<ExPolygon>,
    object_islands: &[ExPolygon],
    spacing: f32,
    resolution: f64,
) -> Result<Vec<ExPolygon>, SliceError> {
    let mut retained = Vec::new();
    for (index, island) in area.iter().enumerate() {
        let expanded = offset_expolygons(
            std::slice::from_ref(island),
            spacing * 2.0,
            JoinType::Round,
            resolution,
        )
        .map_err(geometry_error)?;
        let other_area: Vec<_> = area
            .iter()
            .enumerate()
            .filter(|(other, _)| *other != index)
            .map(|(_, polygon)| polygon.clone())
            .collect();
        // This slice has one object, so the upstream other-object set is empty.
        if !intersection_ex(&expanded, object_islands)
            .map_err(geometry_error)?
            .is_empty()
            || !intersection_ex(&expanded, &other_area)
                .map_err(geometry_error)?
                .is_empty()
        {
            retained.push(island.clone());
        }
    }
    Ok(retained)
}

fn geometry_error(_: crate::geometry::ClipperError) -> SliceError {
    SliceError::InvalidInput("brim generation failed".into())
}
