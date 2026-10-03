//! `SupportCommon.cpp::modulate_extrusion_by_overlapping_layers:1084-1328`.

mod chaining;
mod lookup;

use super::{
    extruded_layer::SourceLayer, extrusion_path::SupportExtrusionEntity,
    grid_generation::GeneratedSupportGeometry,
};
use crate::{
    SliceError,
    geometry::{
        CoordinateScale, FillRule, JoinType, Point, Polyline, difference_open_polylines,
        intersection_open_polylines, offset_expolygons_paths, union_ex,
    },
    project_slice::perimeters::flow::ordinary_volume,
};

struct PathFragment {
    polylines: Vec<Polyline>,
    mm3_per_mm: f64,
    width: f32,
    height: f32,
}

pub(super) fn modulate_extrusion_by_overlapping_layers(
    extrusions: &mut Vec<SupportExtrusionEntity>,
    source: SourceLayer,
    overlapping: &[SourceLayer],
    geometry: &GeneratedSupportGeometry,
    scale: CoordinateScale,
) -> Result<(), SliceError> {
    if overlapping.is_empty() || extrusions.is_empty() {
        return Ok(());
    }
    let SupportExtrusionEntity::Path(template) = &extrusions[0] else {
        unreachable!("native overlapping support contains straight paths");
    };
    let role = template.role;
    let width = template.width;
    let mut fragments = (0..=overlapping.len())
        .map(|_| PathFragment {
            polylines: Vec::new(),
            mm3_per_mm: template.mm3_per_mm,
            width,
            height: template.height,
        })
        .collect::<Vec<_>>();
    let mut ends = Vec::with_capacity(extrusions.len());
    let remaining = &mut fragments.last_mut().unwrap().polylines;
    remaining.reserve(extrusions.len());
    for entity in extrusions.drain(..) {
        let SupportExtrusionEntity::Path(path) = entity else {
            unreachable!("native overlapping support contains straight paths");
        };
        let polyline = Polyline::new(
            path.polyline
                .points
                .into_iter()
                .map(|point| Point::new(point.x, point.y))
                .collect(),
        );
        ends.push((polyline.front().unwrap(), polyline.back().unwrap()));
        remaining.push(polyline);
    }
    let print_z = source.print_z(geometry);
    let offset = (0.5 * f64::from(width) / scale.factor()) as f32;
    for (index, layer) in overlapping.iter().enumerate().rev() {
        let regions = union_ex(layer.polygons(geometry), FillRule::NonZero).map_err(|error| {
            SliceError::InvalidInput(format!("support overlap union: {error:?}"))
        })?;
        let trimming =
            offset_expolygons_paths(&regions, offset, JoinType::Miter, 3.0).map_err(|error| {
                SliceError::InvalidInput(format!("support overlap offset: {error:?}"))
            })?;
        fragments[index].polylines =
            intersection_open_polylines(&fragments.last().unwrap().polylines, &trimming).map_err(
                |error| {
                    SliceError::InvalidInput(format!("support overlap intersection: {error:?}"))
                },
            )?;
        fragments.last_mut().unwrap().polylines =
            difference_open_polylines(&fragments.last().unwrap().polylines, &trimming).map_err(
                |error| SliceError::InvalidInput(format!("support overlap difference: {error:?}")),
            )?;
        let fragment = &mut fragments[index];
        debug_assert!(print_z > layer.print_z(geometry));
        fragment.height = (print_z - layer.print_z(geometry)) as f32;
        let spacing = fragment.width - fragment.height * (1.0 - 0.25 * std::f64::consts::PI) as f32;
        if spacing <= 0.0 {
            return Err(SliceError::InvalidInput(
                "negative support overlap spacing".into(),
            ));
        }
        fragment.mm3_per_mm = ordinary_volume(fragment.width, fragment.height);
        if fragment.mm3_per_mm <= 0.0 {
            return Err(SliceError::InvalidInput(
                "negative support overlap flow".into(),
            ));
        }
    }
    chaining::reconnect(extrusions, &mut fragments, &ends, role);
    Ok(())
}
