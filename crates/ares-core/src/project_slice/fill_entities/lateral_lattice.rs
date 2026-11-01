//! Lateral-lattice sparse infill — `FillLateralLattice::fill_surface`
//! (`FillRectilinear.cpp:3449-3464`): two parallel scanline families at 90°,
//! shifted by `tan(lateral_lattice_angle_{1,2}) * z` so the lattice leans with
//! the layer height; the polylines are reversed on odd layers.
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
    let sweeps = crate::fill::multiline::lateral_lattice_sweeps(
        fill.params.lateral_lattice_angle_1,
        fill.params.lateral_lattice_angle_2,
        print_z,
    );
    let params = MultilineFillParams {
        spacing: fill.params.spacing,
        overlap: fill.params.overlap,
        angle: fill.params.angle + std::f32::consts::FRAC_PI_2,
        reference: object_reference,
        density: (0.01_f64 * f64::from(fill.params.density)) as f32,
        multiline: fill.params.multiline,
        anchor_length: fill.params.anchor_length,
        anchor_length_max: fill.params.anchor_length_max,
        dont_sort: false,
    };
    let mut polylines = Vec::new();
    for expolygon in &fill.expolygons {
        polylines.extend(
            fill_surface_lateral(expolygon, params, &sweeps, scale).map_err(lattice_error)?,
        );
    }
    // `if (this->layer_id % 2 == 1) std::reverse(...)` — odd layers reverse each
    // polyline so the lattice overhang leans the other way.
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

fn lattice_error(error: crate::geometry::ClipperError) -> SliceError {
    SliceError::InvalidInput(format!(
        "lateral-lattice infill generation failed: {error:?}"
    ))
}
