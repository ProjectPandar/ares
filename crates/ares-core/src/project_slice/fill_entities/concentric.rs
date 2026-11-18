use crate::project_slice::perimeters::classic::materialize::Polyline3;
use crate::{
    SliceError,
    geometry::{
        CoordinateScale, ExPolygon, JoinType, ThickPolyline, intersection_ex_with_safety_offset,
        offset_expolygon,
    },
    project_slice::{
        group_fills::SurfaceFill,
        perimeters::{
            classic::{
                gap_extrusion::variable_width, materialize::ExtrusionRole as MaterializedRole,
                shortest_path::reorder_thick_polylines,
            },
            flow::with_spacing,
        },
    },
};

mod finalize;
pub(super) mod generate;

use finalize::finalize_standard_polylines;

use super::{FillExtrusionCollection, FillExtrusionEntity, LayerFillEntities, geometry_error};
use generate::{generate_standard_thick_polylines, generate_thick_polylines};

pub(super) fn append(
    output: &mut LayerFillEntities,
    fill: SurfaceFill,
    layer_height: f64,
    minimum_nozzle_diameter: f64,
    scale: CoordinateScale,
) -> Result<(), SliceError> {
    let spacing = scale
        .checked_scale(f64::from(fill.params.flow.spacing))
        .ok_or_else(|| SliceError::InvalidInput("concentric spacing is out of range".into()))?;
    if spacing <= 0 {
        return Err(SliceError::InvalidInput(
            "concentric spacing must be positive".into(),
        ));
    }

    for expolygon in fill.expolygons {
        let mut polylines = Vec::new();
        for domain in intersect_no_overlap_domains(&fill.no_overlap_expolygons, &expolygon)? {
            let generated = generate_thick_polylines(
                domain,
                spacing,
                layer_height,
                minimum_nozzle_diameter,
                scale,
            )?;
            finalize_polylines(&mut polylines, generated, fill.params.loop_clipping as f64);
        }
        let entities = variable_width::convert_with_role(
            &polylines,
            with_spacing(fill.params.flow, fill.params.spacing as f32)?,
            scale,
            MaterializedRole::SolidInfill,
        )?
        .entities
        .into_iter()
        .map(FillExtrusionEntity::VariableWidth)
        .collect::<Vec<_>>();
        if !entities.is_empty() {
            output.collections.push(FillExtrusionCollection {
                entities,
                no_sort: true,
            });
        }
    }
    Ok(())
}

pub(super) fn append_configured(
    output: &mut LayerFillEntities,
    fill: SurfaceFill,
    layer_height: f64,
    minimum_nozzle_diameter: f64,
    scale: CoordinateScale,
) -> Result<(), SliceError> {
    let spacing = scale
        .checked_scale(fill.params.spacing)
        .ok_or_else(|| SliceError::InvalidInput("concentric spacing is out of range".into()))?;
    let params = fill.params;
    let kind = fill.representative.kind;
    // `FillConcentric::_fill_surface_single` density dispatch
    // (`FillConcentric.cpp:22`): the Arachne variable-width generator runs only
    // at full density; below `density > 0.9999` upstream uses the plain
    // `offset2_ex` ring loop emitting fixed-width `Polyline` entities.
    // Fill.cpp:1283,1479: multiply in double, then narrow the fraction to float.
    let density_fraction = (0.01_f64 * f64::from(params.density)) as f32;
    let sparse = density_fraction <= 0.9999 || params.bridge;
    for expolygon in fill.expolygons {
        if sparse {
            // `Fill::fill_surface` contracts the surface by
            // `scale_(overlap - 0.5*spacing)` before
            // `_fill_surface_single` (`FillBase.cpp:107`) — the sparse
            // ring loop below receives the same shrunk domain as
            // upstream's classic path.
            let generation_domains = offset_expolygon(
                &expolygon,
                ((params.overlap - 0.5 * params.spacing) / scale.factor()) as f32,
                JoinType::Miter,
                3.0,
            )
            .map_err(geometry_error)?;
            let mut entities: Vec<FillExtrusionEntity> = Vec::new();
            for domain in generation_domains {
                let polylines = crate::fill::concentric_sparse::fill_concentric_sparse(
                    &domain,
                    params.spacing,
                    &crate::fill::FillParams {
                        density: density_fraction,
                        multiline: params.multiline,
                        dont_adjust: false,
                    },
                    params.loop_clipping,
                    scale,
                )
                .map_err(geometry_error)?;
                // The thick-line fallback stores integer min_spacing, then
                // variable_width reconstructs the constant-width run's flow
                // (FillConcentric.cpp:141-145, VariableWidth.cpp:205).
                let width = spacing as f32 * scale.factor() as f32
                    + params.flow.height * (1.0 - 0.25 * std::f64::consts::PI) as f32;
                let mut flow = params.flow.with_width(width)?;
                // Same bottom/internal-bridge pre-bake as materialized_flow.
                flow.mm3_per_mm *= params.flow_ratio;
                append_sparse_paths(&mut entities, polylines, params.extrusion_role, flow);
            }
            if entities.is_empty() {
                continue;
            }
            super::gap_residual::append_residual(super::gap_residual::ResidualInput {
                output_entities: &mut entities,
                no_sort: true,
                no_overlap_expolygons: &fill.no_overlap_expolygons,
                params,
                kind,
                expolygon: &expolygon,
                scale,
            })?;
            if !entities.is_empty() {
                output.collections.push(FillExtrusionCollection {
                    entities,
                    no_sort: true,
                });
            }
            continue;
        }
        let generation_domains = offset_expolygon(
            &expolygon,
            ((params.overlap - 0.5 * params.spacing) / scale.factor()) as f32,
            JoinType::Miter,
            3.0,
        )
        .map_err(geometry_error)?;
        let mut generated = Vec::new();
        for domain in generation_domains {
            generated.extend(generate_standard_thick_polylines(
                domain,
                spacing,
                layer_height,
                minimum_nozzle_diameter,
                scale,
            )?);
        }
        let polylines = finalize_standard_polylines(generated, params.loop_clipping as f64);
        let materialized_role = match params.extrusion_role {
            crate::ExtrusionRole::TopSolidInfill => MaterializedRole::TopSolidInfill,
            crate::ExtrusionRole::BottomSurface => MaterializedRole::BottomSurface,
            _ => MaterializedRole::SolidInfill,
        };
        let mut entities = variable_width::convert_with_role(
            &polylines,
            with_spacing(params.flow, params.spacing as f32)?,
            scale,
            materialized_role,
        )?
        .entities
        .into_iter()
        .map(FillExtrusionEntity::VariableWidth)
        .collect::<Vec<_>>();
        super::gap_residual::append_residual(super::gap_residual::ResidualInput {
            output_entities: &mut entities,
            no_sort: true,
            no_overlap_expolygons: &fill.no_overlap_expolygons,
            params,
            kind,
            expolygon: &expolygon,
            scale,
        })?;
        if !entities.is_empty() {
            output.collections.push(FillExtrusionCollection {
                entities,
                no_sort: true,
            });
        }
    }
    Ok(())
}

/// Fixed-width sparse runs (`FillConcentric.cpp:141-145`, `VariableWidth.cpp:205`).
fn append_sparse_paths(
    entities: &mut Vec<FillExtrusionEntity>,
    polylines: Vec<crate::geometry::Polyline>,
    role: crate::ExtrusionRole,
    flow: crate::project_slice::perimeters::types::Flow,
) {
    entities.extend(polylines.into_iter().map(|polyline| {
        FillExtrusionEntity::Path(super::FillExtrusionPath {
            polyline: Polyline3::from_xy(polyline, Vec::new()),
            z_contoured: false,
            role,
            mm3_per_mm: flow.mm3_per_mm,
            width: flow.width,
            height: flow.height,
        })
    }));
}

fn intersect_no_overlap_domains(
    no_overlap: &[ExPolygon],
    expolygon: &ExPolygon,
) -> Result<Vec<ExPolygon>, SliceError> {
    intersection_ex_with_safety_offset(no_overlap, std::slice::from_ref(expolygon))
        .map_err(geometry_error)
}

fn finalize_polylines(
    polylines: &mut Vec<ThickPolyline>,
    generated: Vec<(ThickPolyline, bool)>,
    loop_clipping: f64,
) {
    let first_polyline = polylines.len();
    for (mut polyline, is_closed) in generated {
        // `FillConcentricInternal.cpp:55`: gate on ExtrusionLine::is_closed
        // before the closed-shape checks.
        if is_closed
            && polyline.points.first() == polyline.points.last()
            && polyline.width.first() == polyline.width.last()
        {
            polyline.points.pop();
            let start = finalize::nearest_to_origin(&polyline.points);
            polyline.points.push(polyline.points[0]);
            polyline.start_at_index(start);
        }
        polylines.push(polyline);
    }
    let mut write_index = first_polyline;
    for read_index in first_polyline..polylines.len() {
        polylines[read_index].clip_end(loop_clipping);
        if polylines[read_index].points.len() >= 2 {
            polylines.swap(write_index, read_index);
            write_index += 1;
        }
    }
    polylines.truncate(write_index);
    reorder_thick_polylines(polylines);
}
