use crate::{
    ExtrusionRole, ProcessSurfaceFillOrder, SliceError,
    fill::{FillParams, spiral_inset},
    geometry::{CoordinateScale, JoinType, offset_expolygon, to_thick_polylines},
    project_slice::{
        group_fills::SurfaceFill,
        perimeters::{
            classic::{
                gap_extrusion::{GapFillEntity, variable_width},
                materialize::ExtrusionRole as MaterializedRole,
            },
            flow::with_spacing,
        },
    },
};

use super::{
    FillExtrusionCollection, FillExtrusionEntity, LayerFillEntities,
    gap_residual::{self, ResidualInput},
    geometry_error,
};

pub(super) fn append(
    output: &mut LayerFillEntities,
    fill: SurfaceFill,
    layer_height: f64,
    min_nozzle: f64,
    scale: CoordinateScale,
) -> Result<(), SliceError> {
    let params = fill.params;
    let density = (0.01 * f64::from(params.density)) as f32;
    let mut spacing = params.spacing;
    let role = match params.extrusion_role {
        ExtrusionRole::TopSolidInfill => MaterializedRole::TopSolidInfill,
        ExtrusionRole::BottomSurface => MaterializedRole::BottomSurface,
        _ => MaterializedRole::SolidInfill,
    };
    for expolygon in fill.expolygons {
        let domains = offset_expolygon(
            &expolygon,
            scale.scaled_delta(params.overlap - 0.5 * spacing) as f32,
            JoinType::Miter,
            3.0,
        )
        .map_err(geometry_error)?;
        let mut paths = Vec::new();
        for domain in domains {
            let scaled_spacing =
                scale
                    .checked_scale(spacing)
                    .filter(|s| *s > 0)
                    .ok_or_else(|| {
                        SliceError::InvalidInput("Spiral Inset spacing is out of range".into())
                    })?;
            if density <= 0.9999 {
                let lines = spiral_inset::classic(
                    domain,
                    &FillParams {
                        density,
                        multiline: params.multiline,
                        dont_adjust: false,
                    },
                    &mut spacing,
                    params.fill_order,
                    scale,
                )
                .map_err(geometry_error)?;
                paths.extend(to_thick_polylines(lines, scaled_spacing as f64));
            } else {
                let walls = super::concentric::generate::generate_standard_thick_polylines(
                    domain.clone(),
                    scaled_spacing,
                    layer_height,
                    min_nozzle,
                    scale,
                )?;
                paths.extend(
                    spiral_inset::spiral_walls(walls, scaled_spacing, params.fill_order, &domain)
                        .map_err(geometry_error)?,
                );
            }
        }
        if paths.is_empty() {
            continue;
        }
        let mut generated = variable_width::convert_with_role(
            &paths,
            with_spacing(params.flow, spacing as f32)?,
            scale,
            role,
        )?;
        if params.fill_order != ProcessSurfaceFillOrder::Default {
            keep_order(&mut generated.entities);
        }
        let mut entities = generated
            .entities
            .into_iter()
            .map(FillExtrusionEntity::VariableWidth)
            .collect();
        gap_residual::append_residual(ResidualInput {
            output_entities: &mut entities,
            no_sort: true,
            no_overlap_expolygons: &fill.no_overlap_expolygons,
            params,
            kind: fill.representative.kind,
            expolygon: &expolygon,
            scale,
        })?;
        output.collections.push(FillExtrusionCollection {
            entities,
            no_sort: true,
        });
    }
    Ok(())
}

fn keep_order(entities: &mut [GapFillEntity]) {
    for entity in entities {
        if let GapFillEntity::Path(path) = entity {
            path.can_reverse = false;
        }
    }
}
