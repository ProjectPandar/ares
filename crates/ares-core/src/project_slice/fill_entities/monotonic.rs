use crate::{
    ProcessInfillPattern, SliceError,
    fill::rectilinear::{MonotonicFillParams, fill_monotonic_surface},
    geometry::{CoordinateScale, Point},
    project_slice::group_fills::SurfaceFill,
};

use super::{
    FillExtrusionCollection, FillExtrusionEntity, FillExtrusionPath, LayerFillEntities,
    geometry_error,
};

pub(super) fn append(
    output: &mut LayerFillEntities,
    fill: SurfaceFill,
    // `FillRectilinear::fill_surface` and its monotonic subclasses share
    // `fill_surface_by_lines` (`FillRectilinear.cpp:3386-3419`).
    pattern: ProcessInfillPattern,
    layer_id: usize,
    scale: CoordinateScale,
) -> Result<(), SliceError> {
    let density = (0.01_f64 * f64::from(fill.params.density)) as f32;
    let anchor_length_max = if pattern == ProcessInfillPattern::MonotonicLine {
        0.0
    } else {
        fill.params.anchor_length_max
    };
    let params = MonotonicFillParams {
        spacing: fill.params.spacing,
        overlap: fill.params.overlap,
        density,
        angle: fill.params.angle,
        layer_index: layer_id,
        thickness_layers: fill.representative.thickness_layers.max(1),
        fixed_angle: fill.params.fixed_angle,
        bridge_angle: fill.params.bridge.then_some(fill.params.bridge_angle),
        // PrintObject::bounding_box() is symmetric about the centered object origin.
        reference_point: Point::new(0, 0),
        dont_adjust: fill.params.bridge,
        anchor_length_max,
        link_max_length: 0.0,
    };
    let no_overlap_expolygons = fill.no_overlap_expolygons.clone();
    let fill_params = fill.params;
    let fill_kind = fill.representative.kind;
    for expolygon in fill.expolygons {
        let generated =
            fill_monotonic_surface(&expolygon, params, scale).map_err(geometry_error)?;
        if generated.polylines.is_empty() {
            continue;
        }
        let flow = super::materialized_flow(fill.params, generated.spacing);
        let spacing = generated.spacing;
        let mut entities: Vec<FillExtrusionEntity> = generated
            .polylines
            .iter()
            .cloned()
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
            .collect();
        super::gap_residual::append_residual(super::gap_residual::ResidualInput {
            output_entities: &mut entities,
            no_overlap_expolygons: &no_overlap_expolygons,
            params: fill_params,
            kind: fill_kind,
            expolygon: &expolygon,
            filled: &generated.polylines,
            spacing,
            scale,
        })?;
        output.collections.push(FillExtrusionCollection {
            entities,
            no_sort: matches!(
                pattern,
                ProcessInfillPattern::Monotonic | ProcessInfillPattern::MonotonicLine
            ),
        });
    }
    Ok(())
}
