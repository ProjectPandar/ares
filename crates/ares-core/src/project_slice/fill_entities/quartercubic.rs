//! Quarter-cubic sparse infill — `FillQuarterCubic::fill_surface`
//! (`FillRectilinear.cpp:3518-3548`): four scanline families — two pairs at
//! 0° and 90° — each pair mirrored by a z-dependent `dx` shift so the pattern
//! forms tetrahedral cells.

use crate::{
    SliceError,
    fill::multiline::{MultilineFillParams, Sweep, fill_surface},
    geometry::CoordinateScale,
    project_slice::group_fills::SurfaceFill,
};

use super::{FillExtrusionCollection, FillExtrusionEntity, FillExtrusionPath, LayerFillEntities};

const ONE_DIV_ROOT_TWO: f64 = std::f64::consts::FRAC_1_SQRT_2;

pub(super) fn append(
    output: &mut LayerFillEntities,
    fill: SurfaceFill,
    print_z: f64,
    scale: CoordinateScale,
    object_reference: crate::geometry::Point,
) -> Result<(), SliceError> {
    let density = (0.01_f64 * f64::from(fill.params.density)) as f32;
    let z_scaled = scale
        .checked_scale(print_z)
        .ok_or(crate::geometry::ClipperError::CoordinateOutOfRange)
        .map_err(quartercubic_error)? as f64;
    let spacing_scaled = scale
        .checked_scale(fill.params.spacing)
        .ok_or(crate::geometry::ClipperError::CoordinateOutOfRange)
        .map_err(quartercubic_error)? as f64;
    let line_width = spacing_scaled as i64 * i64::from(fill.params.multiline);
    let period =
        (spacing_scaled * f64::from(fill.params.multiline) / f64::from(density)) as i64 * 4;

    // `dx1`/`dx2`: the first/second half of the tetrahedral period
    // (`FillRectilinear.cpp:3528-3540`). The `coord_t` shift is wrapped into
    // `[-period/2, period/2]` then clamped clear of the line width.
    let shift_for = |pattern_z_shift: f64| -> f64 {
        let mut shift =
            (ONE_DIV_ROOT_TWO * (z_scaled + pattern_z_shift * period as f64 * 2.0)) as i64 % period;
        shift = shift.min(period - shift);
        shift = shift.min(period / 2 - line_width / 2);
        shift = shift.max(line_width / 2);
        scale.unscale(shift)
    };
    let dx1 = shift_for(0.0) as f32;
    let dx2 = shift_for(0.5) as f32;

    let sweeps = [
        Sweep {
            angle: 0.0,
            shift: dx1,
        },
        Sweep {
            angle: 0.0,
            shift: -dx1,
        },
        Sweep {
            angle: std::f32::consts::FRAC_PI_2,
            shift: dx2,
        },
        Sweep {
            angle: std::f32::consts::FRAC_PI_2,
            shift: -dx2,
        },
    ];
    let params = MultilineFillParams {
        spacing: fill.params.spacing,
        overlap: fill.params.overlap,
        angle: fill.params.angle + std::f32::consts::FRAC_PI_2,
        reference: object_reference,
        density,
        multiline: fill.params.multiline,
        anchor_length: fill.params.anchor_length,
        anchor_length_max: fill.params.anchor_length_max,
        dont_sort: false,
    };
    let mut polylines = Vec::new();
    for expolygon in &fill.expolygons {
        polylines
            .extend(fill_surface(expolygon, params, &sweeps, scale).map_err(quartercubic_error)?);
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

fn quartercubic_error(error: crate::geometry::ClipperError) -> SliceError {
    SliceError::InvalidInput(format!("quarter-cubic infill generation failed: {error:?}"))
}
