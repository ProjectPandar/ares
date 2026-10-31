//! `FillLightning::_fill_surface_single` emit (`FillLightning.cpp:10-37`): the
//! per-object `Generator` (built once in `prepare`) converts the layer's
//! lightning trees to lines clipped to the fill surface; `multiline_fill`
//! expands them; `chain_or_connect_infill` merges them into entities.
use crate::project_slice::perimeters::classic::materialize::Polyline3;

use crate::{
    SliceError,
    fill::connect::{FillConnectionParams, connect_infill},
    fill::lightning::generator::Generator,
    geometry::{CoordinateScale, ExPolygon, Polygon},
    project_slice::group_fills::SurfaceFill,
};

use super::{FillExtrusionCollection, FillExtrusionEntity, FillExtrusionPath, LayerFillEntities};

/// `to_polygons(ExPolygon)` — flatten contour+holes.
fn to_polygons(expolygon: &ExPolygon) -> Vec<Polygon> {
    let mut v = vec![expolygon.contour().clone()];
    v.extend(expolygon.holes().iter().cloned());
    v
}

pub(super) fn append(
    output: &mut LayerFillEntities,
    generator: &Generator,
    fill: &SurfaceFill,
    layer_id: usize,
    scale: CoordinateScale,
) -> Result<(), SliceError> {
    // `scaled<coord_t>(0.5 * spacing - overlap)`.
    let line_overlap = scale
        .checked_scale(0.5 * fill.params.spacing - fill.params.overlap)
        .unwrap_or(0);
    let mut polylines = Vec::new();
    for surface in &fill.expolygons {
        // Fill::fill_surface (FillBase.cpp:107) offsets before pattern dispatch.
        let domains = crate::geometry::offset_expolygon(
            surface,
            ((fill.params.overlap - 0.5 * fill.params.spacing) / scale.factor()) as f32,
            crate::geometry::JoinType::Miter,
            3.0,
        )
        .map_err(lightning_error)?;
        for expolygon in &domains {
            let outline = to_polygons(expolygon);
            let lines = generator
                .convert_to_lines(layer_id, &outline, line_overlap)
                .map_err(lightning_error)?;
            // `multiline_fill(fill_lines, params, spacing)` (FillLightning.cpp:20)
            // — offset the lightning lines BEFORE the second `intersection_pl`.
            let lines = crate::fill::multiline_offset::apply(
                lines,
                fill.params.multiline,
                fill.params.spacing,
                scale,
            )
            .map_err(lightning_error)?;
            let lines = crate::geometry::intersection_open_polylines(&lines, &outline)
                .map_err(lightning_error)?;
            // `chain_or_connect_infill` (`FillBase.cpp:1822`) guards
            // `!infill_ordered.empty()` before `connect_infill` — an empty
            // clipped set skips connection entirely.
            if lines.is_empty() {
                continue;
            }
            let connected = connect_infill(
                lines,
                expolygon,
                fill.params.spacing,
                FillConnectionParams {
                    anchor_length: fill.params.anchor_length,
                    anchor_length_max: fill.params.anchor_length_max,
                    multiline: fill.params.multiline,
                    dont_sort: false,
                },
                scale,
            )
            .map_err(lightning_error)?;
            polylines.extend(connected);
        }
    }
    if polylines.is_empty() {
        return Ok(());
    }
    let flow = super::materialized_flow(fill.params, fill.params.spacing as f32)?;
    output.collections.push(FillExtrusionCollection {
        entities: polylines
            .into_iter()
            .map(|polyline| {
                FillExtrusionEntity::Path(FillExtrusionPath {
                    polyline: Polyline3::from_xy(polyline, Vec::new()),
                    z_contoured: false,
                    role: fill.params.extrusion_role,
                    mm3_per_mm: flow.mm3_per_mm,
                    width: flow.width,
                    height: flow.height,
                })
            })
            .collect(),
        no_sort: false,
    });
    Ok(())
}

fn lightning_error(error: crate::geometry::ClipperError) -> SliceError {
    SliceError::InvalidInput(format!("lightning infill generation failed: {error:?}"))
}
