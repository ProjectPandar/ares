//! `PerimeterGenerator::process_no_bridge`, Filled branch (:1777–1855).
use super::{RegionSurface, SurfaceContext, SurfaceReference};
use crate::geometry::simplification::polygons_simplify;
use crate::geometry::{
    ClipperError, ExPolygon, JoinType, Polygon, difference_ex_with_safety_offset, intersection_ex,
    intersection_ex_with_safety_offset, offset_expolygons, offset_paths, offset2_ex,
    union_expolygons,
};

pub(super) fn mask(
    traversal: (&mut Vec<RegionSurface>, &mut usize, &mut SurfaceReference),
    last: &[ExPolygon],
    unsupported: &[ExPolygon],
    bridgeable: &[ExPolygon],
    context: &SurfaceContext<'_>,
) -> Result<Vec<ExPolygon>, ClipperError> {
    let (surfaces, current, reference) = traversal;
    let geometry = &context.geometry;
    let spacing = geometry.spacing;
    let margin = geometry.margin;
    let candidates = offset_expolygons(unsupported, (-spacing) as f32, JoinType::Miter, 3.0)?;
    let mut filtered = Vec::new();
    for candidate in candidates {
        let simplified = polygons_simplify(vec![candidate.contour().clone()], spacing as f64)?;
        let expanded = offset_paths(
            std::slice::from_ref(candidate.contour()),
            margin as f32,
            JoinType::Miter,
            3.0,
        )?;
        if simplified.len() != 1 || expanded.len() != 1 || has_concave_vertex(&simplified[0]) {
            continue;
        }
        if intersection_ex(bridgeable, std::slice::from_ref(&candidate))?.is_empty() {
            continue;
        }
        let bigger = ExPolygon::new(expanded[0].clone(), candidate.holes().to_vec());
        let supported = union_expolygons(
            last,
            &offset_expolygons(
                bridgeable,
                (margin + (spacing / 2) as f64) as f32,
                JoinType::Miter,
                3.0,
            )?,
        )?;
        if difference_ex_with_safety_offset(&[bigger], &supported)?.is_empty() {
            filtered.push(candidate);
        }
    }
    let restored = offset_expolygons(
        &filtered,
        (spacing as f64 + geometry.anchor) as f32,
        JoinType::Miter,
        3.0,
    )?;
    let mut mask = intersection_ex(last, &restored)?;
    for polygon in &mut mask {
        remove_unsupported_holes(polygon, context)?;
        recover_nested_islands(polygon, (surfaces, current, reference), spacing, margin)?;
    }
    Ok(mask)
}

// `Polygon.cpp:235–255,315–317`, concave_points() with its default zero threshold.
fn has_concave_vertex(polygon: &Polygon) -> bool {
    let points = polygon.points();
    (0..points.len()).any(|i| {
        let previous = points[(i + points.len() - 1) % points.len()];
        let point = points[i];
        let next = points[(i + 1) % points.len()];
        let incoming = (
            (point.x() - previous.x()) as f64,
            (point.y() - previous.y()) as f64,
        );
        let outgoing = ((next.x() - point.x()) as f64, (next.y() - point.y()) as f64);
        incoming.0 * outgoing.1 - incoming.1 * outgoing.0 < 0.0
    })
}

fn remove_unsupported_holes(
    polygon: &mut ExPolygon,
    context: &SurfaceContext<'_>,
) -> Result<(), ClipperError> {
    let width = (context.geometry.width / 2) as f32;
    let mut index = 0;
    while index < polygon.holes().len() {
        let mut contour = polygon.holes()[index].clone();
        if !contour.is_counter_clockwise() {
            contour.reverse();
        }
        let area = [ExPolygon::new(contour, Vec::new())];
        let band = difference_ex_with_safety_offset(
            &offset_expolygons(&area, width, JoinType::Miter, 3.0)?,
            &area,
        )?;
        let band = intersection_ex_with_safety_offset(&band, std::slice::from_ref(polygon))?;
        if !band.is_empty() && intersection_ex_with_safety_offset(&band, context.lower)?.is_empty()
        {
            polygon.holes_mut().remove(index);
        } else {
            index += 1;
        }
    }
    Ok(())
}

fn recover_nested_islands(
    polygon: &mut ExPolygon,
    traversal: (&mut Vec<RegionSurface>, &mut usize, &mut SurfaceReference),
    spacing: i64,
    margin: f64,
) -> Result<(), ClipperError> {
    let (surfaces, current, reference) = traversal;
    let mut index = 0;
    while index < surfaces.len() {
        if index == *current {
            index += 1;
            continue;
        }
        let other = surfaces[index].clone();
        if intersection_ex(
            std::slice::from_ref(polygon),
            std::slice::from_ref(other.as_parts().1),
        )?
        .is_empty()
        {
            index += 1;
            continue;
        }
        let replacement = offset2_ex(
            std::slice::from_ref(other.as_parts().1),
            (-margin - spacing as f64) as f32,
            spacing as f32,
            JoinType::Miter,
            3.0,
        )?;
        if replacement.len() == 1 {
            surfaces[index] = other.clone_with_expolygon(replacement[0].clone());
            append_hole(polygon, replacement[0].contour());
            index += 1;
        } else {
            for island in replacement {
                append_hole(polygon, island.contour());
                reference.append(surfaces.len());
                surfaces.push(other.clone_with_expolygon(island));
            }
            surfaces.remove(index);
            if index < *current {
                *current -= 1;
                // :1846–1848 refreshes the captured reference after an earlier erase.
                reference.valid = true;
            }
        }
    }
    Ok(())
}

fn append_hole(polygon: &mut ExPolygon, contour: &Polygon) {
    let mut hole = contour.clone();
    if hole.is_counter_clockwise() {
        hole.reverse();
    }
    polygon.holes_mut().push(hole);
}
