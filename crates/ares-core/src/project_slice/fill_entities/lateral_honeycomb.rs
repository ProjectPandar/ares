//! Lateral-honeycomb sparse infill — `FillLateralHoneycomb::fill_surface`
//! (`FillRectilinear.cpp:3550-3610`): an inverted-Y honeycomb. Each layer is
//! either a single vertical-line family (`single_line`) or a double family,
//! chosen by which third of the vertical period the layer height lands in;
//! the double family splits the density and offsets the two lines
//! symmetrically around the period knot.
use crate::project_slice::perimeters::classic::materialize::Polyline3;

use crate::{
    SliceError,
    fill::multiline::{MultilineFillParams, fill_surface_lateral},
    geometry::CoordinateScale,
    project_slice::group_fills::SurfaceFill,
};

use super::{FillExtrusionCollection, FillExtrusionEntity, FillExtrusionPath, LayerFillEntities};

pub(super) fn polylines(
    fill: &SurfaceFill,
    layer_id: usize,
    print_z: f64,
    scale: CoordinateScale,
    object_reference: crate::geometry::Point,
) -> Result<Vec<crate::geometry::Polyline>, SliceError> {
    let density = (0.01_f64 * f64::from(fill.params.density)) as f32;
    let (sweeps, sweep_density) = crate::fill::multiline::lateral_honeycomb_sweeps(
        fill.params.spacing,
        fill.params.multiline,
        density,
        fill.params.infill_overhang_angle,
        print_z,
    );

    let params = MultilineFillParams {
        spacing: fill.params.spacing,
        overlap: fill.params.overlap,
        angle: fill.params.angle + std::f32::consts::FRAC_PI_2,
        reference: object_reference,
        density: sweep_density,
        multiline: fill.params.multiline,
        anchor_length: fill.params.anchor_length,
        anchor_length_max: fill.params.anchor_length_max,
        dont_sort: false,
    };
    let mut polylines = Vec::new();
    for expolygon in &fill.expolygons {
        polylines.extend(
            fill_surface_lateral(expolygon, params, &sweeps, scale).map_err(honeycomb_error)?,
        );
    }
    if layer_id % 2 == 1 {
        for polyline in &mut polylines {
            polyline.reverse();
        }
    }
    Ok(polylines)
}

pub(super) fn append(
    output: &mut LayerFillEntities,
    mut fill: SurfaceFill,
    layer: &crate::project_slice::layers::PlannedLayer,
    scale: CoordinateScale,
    object_reference: crate::geometry::Point,
) -> Result<(), SliceError> {
    let expolygons = std::mem::take(&mut fill.expolygons);
    for expolygon in expolygons {
        fill.expolygons = vec![expolygon];
        let polylines = polylines(&fill, layer.id, layer.print_z, scale, object_reference)?;
        if polylines.is_empty() {
            continue;
        }
        let polylines = super::join_close_ends::join(
            polylines,
            fill.params.extrusion_role,
            fill.params.spacing,
            (false, true),
            scale,
        );
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
    }
    Ok(())
}

fn honeycomb_error(error: crate::geometry::ClipperError) -> SliceError {
    SliceError::InvalidInput(format!(
        "lateral-honeycomb infill generation failed: {error:?}"
    ))
}
