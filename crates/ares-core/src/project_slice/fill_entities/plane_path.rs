use crate::project_slice::perimeters::classic::materialize::Polyline3;
use crate::{
    ExtrusionRole, ProcessInfillPattern, SliceError,
    fill::plane_path::{PlanePathFillParams, PlanePathPattern, fill_surface},
    geometry::{BoundingBox, CoordinateScale, ExPolygon, Polyline},
    project_slice::group_fills::{SurfaceFill, SurfaceFillParams},
};

use super::{
    FillExtrusionCollection, FillExtrusionEntity, FillExtrusionPath, LayerFillEntities,
    geometry_error,
};

/// Surface entry retains the real extrusion role for beta
/// `FillPlanePath.cpp:96`; non-sparse fills use their snug surface bounds.
pub(super) fn expolygon_polylines(
    expolygon: &ExPolygon,
    pattern: PlanePathPattern,
    params: SurfaceFillParams,
    scale: CoordinateScale,
) -> Result<Vec<Polyline>, SliceError> {
    let fill_params = PlanePathFillParams {
        sparse: params.extrusion_role == crate::ExtrusionRole::InternalInfill,
        spacing: params.spacing,
        overlap: params.overlap,
        density: (0.01_f64 * f64::from(params.density)) as f32,
        angle: params.angle,
        multiline: params.multiline,
        resolution: 0.0125,
        anchor_length: params.anchor_length,
        anchor_length_max: params.anchor_length_max,
        object_bounding_box: BoundingBox::from_expolygon(expolygon)
            .expect("expolygon contour is non-empty by invariant"),
        calibration_order: false,
    };
    fill_surface(expolygon, pattern, fill_params, scale)
        .map(|components| components.into_iter().flatten().collect())
        .map_err(geometry_error)
}

#[expect(
    clippy::too_many_arguments,
    reason = "fill materialization keeps the source pattern, object bounds, calibration, and scale explicit"
)]
pub(super) fn append(
    output: &mut LayerFillEntities,
    fill: SurfaceFill,
    pattern: ProcessInfillPattern,
    object_bounding_box: BoundingBox,
    resolution: f64,
    calibration_order: bool,
    scale: CoordinateScale,
) -> Result<(), SliceError> {
    let pattern = match pattern {
        ProcessInfillPattern::HilbertCurve => PlanePathPattern::HilbertCurve,
        ProcessInfillPattern::ArchimedeanChords => PlanePathPattern::ArchimedeanChords,
        ProcessInfillPattern::OctagramSpiral => PlanePathPattern::OctagramSpiral,
        _ => unreachable!("plane-path dispatch only receives its three source patterns"),
    };
    let params = fill.params;
    let density = (0.01 * f64::from(params.density)) as f32;
    let fill_params = PlanePathFillParams {
        sparse: params.extrusion_role == crate::ExtrusionRole::InternalInfill,
        spacing: params.spacing,
        overlap: params.overlap,
        density,
        angle: params.angle,
        multiline: params.multiline,
        resolution,
        anchor_length: params.anchor_length,
        anchor_length_max: params.anchor_length_max,
        object_bounding_box,
        calibration_order: calibration_order
            && params.extrusion_role == ExtrusionRole::TopSolidInfill,
    };
    let no_overlap_expolygons = fill.no_overlap_expolygons;
    let fill_kind = fill.representative.kind;

    for expolygon in fill.expolygons {
        let components =
            fill_surface(&expolygon, pattern, fill_params, scale).map_err(geometry_error)?;
        let polylines = components.into_iter().flatten().collect::<Vec<_>>();
        if polylines.is_empty() {
            continue;
        }

        let polylines = super::join_close_ends::join(
            polylines,
            params.extrusion_role,
            params.spacing,
            (fill_params.calibration_order, true),
            scale,
        );
        let flow = super::materialized_flow(params, params.spacing as f32)?;
        let mut entities = polylines
            .into_iter()
            .map(|polyline| {
                FillExtrusionEntity::Path(FillExtrusionPath {
                    polyline: Polyline3::from_xy(polyline, Vec::new()),
                    z_contoured: false,
                    role: params.extrusion_role,
                    mm3_per_mm: flow.mm3_per_mm,
                    width: flow.width,
                    height: flow.height,
                })
            })
            .collect::<Vec<_>>();
        super::gap_residual::append_residual(super::gap_residual::ResidualInput {
            output_entities: &mut entities,
            no_overlap_expolygons: &no_overlap_expolygons,
            params,
            kind: fill_kind,
            expolygon: &expolygon,
            scale,
        })?;
        output.collections.push(FillExtrusionCollection {
            entities,
            no_sort: fill_params.calibration_order,
        });
    }
    Ok(())
}
