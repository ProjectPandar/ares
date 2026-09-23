//! Lateral-honeycomb sparse infill — `FillLateralHoneycomb::fill_surface`
//! (`FillRectilinear.cpp:3550-3610`): an inverted-Y honeycomb. Each layer is
//! either a single vertical-line family (`single_line`) or a double family,
//! chosen by which third of the vertical period the layer height lands in;
//! the double family splits the density and offsets the two lines
//! symmetrically around the period knot.

use crate::{
    SliceError,
    fill::multiline::{MultilineFillParams, Sweep, fill_surface},
    geometry::CoordinateScale,
    project_slice::group_fills::SurfaceFill,
};

use super::{FillExtrusionCollection, FillExtrusionEntity, FillExtrusionPath, LayerFillEntities};

const DEGREE: f32 = std::f32::consts::PI / 180.0;

pub(super) fn append(
    output: &mut LayerFillEntities,
    fill: SurfaceFill,
    print_z: f64,
    layer_id: usize,
    scale: CoordinateScale,
    object_reference: crate::geometry::Point,
) -> Result<(), SliceError> {
    let density = (0.01_f64 * f64::from(fill.params.density)) as f32;
    // `FillRectilinear.cpp:3571-3583` — period and per-layer phase.
    let half_horizontal_period = 0.5
        * (1.0 * (2.0 / 3.0) + 2.0 * (1.0 / 3.0))
        * fill.params.spacing as f32
        * fill.params.multiline as f32
        / density;
    let vertical_period =
        3.0 * half_horizontal_period / (DEGREE * fill.params.infill_overhang_angle).tan();
    let vertical_thirds_float = 3.0 * print_z as f32 / vertical_period;
    let vertical_thirds_int = vertical_thirds_float as i32;
    let single_line = (vertical_thirds_int + 1) % 3 != 0;
    let odd_layer = ((vertical_thirds_int + 1) / 3) % 2 != 0;
    let horizontal_offset = if odd_layer {
        half_horizontal_period
    } else {
        0.0
    };

    // The `(1*(2/3.) + 2*(1/3.))` normalizer = 4/3.
    let normalizer = 1.0 * (2.0 / 3.0) + 2.0 * (1.0 / 3.0);
    let (sweeps, sweep_density) = if single_line {
        (
            vec![Sweep {
                angle: std::f32::consts::FRAC_PI_2,
                shift: horizontal_offset,
            }],
            density * (1.0 / normalizer) as f32,
        )
    } else {
        let horizontal_position =
            (1.0 - (vertical_thirds_float - vertical_thirds_int as f32)) * half_horizontal_period;
        (
            vec![
                Sweep {
                    angle: std::f32::consts::FRAC_PI_2,
                    shift: -horizontal_position + horizontal_offset,
                },
                Sweep {
                    angle: std::f32::consts::FRAC_PI_2,
                    shift: horizontal_position + horizontal_offset,
                },
            ],
            density * (2.0 / normalizer) as f32,
        )
    };

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
        polylines.extend(fill_surface(expolygon, params, &sweeps, scale).map_err(honeycomb_error)?);
    }
    if layer_id % 2 == 1 {
        for polyline in &mut polylines {
            polyline.reverse();
        }
    }
    if polylines.is_empty() {
        return Ok(());
    }
    let flow = super::materialized_flow(fill.params, fill.params.spacing as f32);
    output.collections.push(FillExtrusionCollection {
        entities: polylines
            .into_iter()
            .map(|polyline| {
                FillExtrusionEntity::Path(FillExtrusionPath {
                    polyline,
                    fitting: Vec::new(),
                    role: fill.params.extrusion_role,
                    mm3_per_mm: flow.mm3_per_mm,
                    width: flow.width,
                    height: flow.height,
                })
            })
            .collect(),
        no_sort: false,
        simplify_reversed: false,
    });
    Ok(())
}

fn honeycomb_error(error: crate::geometry::ClipperError) -> SliceError {
    SliceError::InvalidInput(format!(
        "lateral-honeycomb infill generation failed: {error:?}"
    ))
}
