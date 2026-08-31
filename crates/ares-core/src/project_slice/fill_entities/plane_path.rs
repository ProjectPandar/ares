use crate::{
    ExtrusionRole, ProcessInfillPattern, SliceError,
    fill::plane_path::{PlanePathFillParams, PlanePathPattern, fill_surface},
    geometry::{BoundingBox, CoordinateScale},
    project_slice::group_fills::SurfaceFill,
};

use super::{
    FillExtrusionCollection, FillExtrusionEntity, FillExtrusionPath, LayerFillEntities,
    geometry_error,
};

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

        let flow = super::materialized_flow(params, params.spacing as f32);
        let mut entities = polylines
            .into_iter()
            .map(|polyline| {
                FillExtrusionEntity::Path(FillExtrusionPath {
                    polyline,
                    fitting: Vec::new(),
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
            simplify_reversed: pattern == PlanePathPattern::HilbertCurve
                && params.extrusion_role == crate::ExtrusionRole::BottomSurface,
        });
    }
    Ok(())
}
