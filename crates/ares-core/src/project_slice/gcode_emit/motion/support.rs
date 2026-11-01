//! `GCode.cpp::extrude_support` and the joined wipe of `extrude_multi_path`.

use super::{EmitState, LayerGeometry, features::PathProperties, local_cursor, path};
use crate::{
    ExtrusionRole,
    project_slice::support_material::{SupportExtrusionEntity, SupportExtrusionPath},
};

#[derive(Clone, Copy)]
pub(in crate::project_slice::gcode_emit) struct LayerAreas<'a> {
    /// Regionless SupportLayer spacing, in the route's scaled units
    /// (AvoidCrossingPerimeters.cpp:480-507), without an f32 mm round-trip.
    pub(in crate::project_slice::gcode_emit) scaled_default_perimeter_spacing: Option<f32>,
    pub(in crate::project_slice::gcode_emit) islands: &'a [crate::geometry::ExPolygon],
    pub(in crate::project_slice::gcode_emit) tree_base: Option<&'a [crate::geometry::ExPolygon]>,
}

pub(in crate::project_slice::gcode_emit) fn emit(
    output: &mut Vec<u8>,
    entities: &mut Vec<SupportExtrusionEntity>,
    selected_role: ExtrusionRole,
    phase: super::shared_reversals::EmissionPhase<'_>,
    state: &mut EmitState,
) -> Result<(), crate::SliceError> {
    let geometry = phase.geometry;
    let (selected, mut remaining): (Vec<_>, Vec<_>) =
        std::mem::take(entities).into_iter().partition(|entity| {
            let role = entity.role();
            role == selected_role
                || (selected_role == ExtrusionRole::Mixed && role != ExtrusionRole::Ironing)
                || (matches!(
                    selected_role,
                    ExtrusionRole::Mixed | ExtrusionRole::SupportMaterialInterface
                ) && role == ExtrusionRole::Ironing)
        });
    let (mut ordinary, mut ironing): (Vec<_>, Vec<_>) = selected
        .into_iter()
        .partition(|entity| entity.role() != ExtrusionRole::Ironing);
    // Native Mixed excludes ironing; the caller dispatches ironing last.
    for (group, selected_role) in [
        (&mut ordinary, selected_role),
        (&mut ironing, ExtrusionRole::Ironing),
    ] {
        // GCode.cpp:6199-6201 — extrude_support chains the SHARED
        // support_fills entities in place (no_sort false), so reversals
        // persist to later islands/instances of the same source object.
        state.shared_reversals.chain_support(
            phase.reversal,
            group,
            local_cursor(state, geometry),
            geometry.scale,
        );
        for entity in &*group {
            emit_entity(output, entity, selected_role, geometry, state)?;
        }
    }
    ordinary.append(&mut ironing);
    ordinary.append(&mut remaining);
    *entities = ordinary;
    Ok(())
}

fn emit_entity(
    output: &mut Vec<u8>,
    entity: &SupportExtrusionEntity,
    selected_role: ExtrusionRole,
    geometry: LayerGeometry<'_>,
    state: &mut EmitState,
) -> Result<(), crate::SliceError> {
    match entity {
        SupportExtrusionEntity::Path(path) => emit_path(output, path, geometry, state, true)?,
        SupportExtrusionEntity::Loop(loop_) => emit_loop(output, loop_, geometry, state)?,
        SupportExtrusionEntity::NoSort(entities) => {
            for entity in entities.iter().filter(|entity| {
                let role = entity.role();
                role == selected_role
                    || (selected_role == ExtrusionRole::Mixed && role != ExtrusionRole::Ironing)
            }) {
                emit_entity(output, entity, selected_role, geometry, state)?;
            }
        }
        SupportExtrusionEntity::MultiPath(paths) => {
            for path in paths {
                emit_path(output, path, geometry, state, false)?;
            }
            if state.options.wipe {
                let mut wipe = Vec::new();
                for path in paths {
                    let points = &path.polyline.points;
                    let first = (points[0].x, points[0].y);
                    let skip = usize::from(wipe.last().copied() == Some(first));
                    wipe.extend(points[skip..].iter().map(|point| (point.x, point.y)));
                }
                state.wipe_path = wipe
                    .into_iter()
                    .rev()
                    .map(|(x, y)| super::arc::Point {
                        x: geometry.scale.unscale(x),
                        y: geometry.scale.unscale(y),
                    })
                    .collect();
            }
        }
    }
    Ok(())
}

fn emit_path(
    output: &mut Vec<u8>,
    support: &SupportExtrusionPath,
    geometry: LayerGeometry<'_>,
    state: &mut EmitState,
    single_path: bool,
) -> Result<(), crate::SliceError> {
    path::emit(
        output,
        support
            .polyline
            .points
            .iter()
            .map(|point| (point.x, point.y)),
        PathProperties {
            contour: None,
            mm3_per_mm: support.mm3_per_mm,
            width: support.width,
            height: support.height,
            feature: feature(support.role),
            is_perimeter: false,
            end_clip: 0.0,
            fitting: &support.polyline.fitting,
            slope: None,
        },
        geometry,
        state,
    )?;
    if !single_path
        || !state.options.wipe
        || !matches!(
            state.options.support_type,
            crate::ProcessSupportType::TreeAuto | crate::ProcessSupportType::TreeManual
        )
        || !matches!(
            support.role,
            ExtrusionRole::SupportMaterial
                | ExtrusionRole::SupportMaterialInterface
                | ExtrusionRole::SupportTransition
        )
    {
        return Ok(());
    }
    // GCode.cpp:6112-6123: Tree single paths keep a forward closest-end tail.
    state.wipe_path.reverse();
    let points = &support.polyline.points;
    let last = points.last().unwrap();
    let threshold = geometry.scale.scaled_delta(0.2);
    let dx = (points[0].x - last.x) as f64;
    let dy = (points[0].y - last.y) as f64;
    if (dx * dx + dy * dy).sqrt() <= threshold {
        return Ok(());
    }
    let mut minimum = threshold;
    let mut index = points.len();
    for (i, point) in points.iter().enumerate() {
        let dx = (point.x - last.x) as f64;
        let dy = (point.y - last.y) as f64;
        let dz = (point.z - last.z) as f64;
        let dist = (dx * dx + dy * dy + dz * dz).sqrt();
        if dist < minimum {
            minimum = dist;
        }
        if minimum < threshold && dist > minimum {
            index = i;
            break;
        }
    }
    state.wipe_path.drain(..index - 1);
    Ok(())
}

fn emit_loop(
    output: &mut Vec<u8>,
    support: &SupportExtrusionPath,
    geometry: LayerGeometry<'_>,
    state: &mut EmitState,
) -> Result<(), crate::SliceError> {
    let polyline = support.loop_polyline(
        local_cursor(state, geometry),
        geometry.scale,
        state.spiral_vase.then_some(state.options.wall_direction),
    );
    let end_clip = if state.spiral_vase_layer {
        0.0
    } else {
        state.options.seam_gap
    };
    let length = polyline
        .points
        .windows(2)
        .map(|pair| {
            let dx = (pair[1].x - pair[0].x) as f64;
            let dy = (pair[1].y - pair[0].y) as f64;
            (dx * dx + dy * dy).sqrt()
        })
        .sum::<f64>();
    if end_clip > 0.0 && length <= end_clip / geometry.scale.factor() {
        return Ok(());
    }
    let previous_wipe = if state.options.wipe {
        Vec::new()
    } else {
        std::mem::take(&mut state.wipe_path)
    };
    path::emit(
        output,
        polyline.points.iter().map(|point| (point.x, point.y)),
        PathProperties {
            contour: None,
            mm3_per_mm: support.mm3_per_mm,
            width: support.width,
            height: support.height,
            feature: feature(support.role),
            is_perimeter: false,
            end_clip,
            fitting: &polyline.fitting,
            slope: None,
        },
        geometry,
        state,
    )?;
    if state.options.wipe {
        state.wipe_path.reverse();
    } else {
        state.wipe_path = previous_wipe;
    }
    Ok(())
}

fn feature(role: ExtrusionRole) -> &'static str {
    match role {
        ExtrusionRole::SupportMaterial => "Support",
        ExtrusionRole::SupportMaterialInterface => "Support interface",
        ExtrusionRole::SupportTransition => "Support transition",
        ExtrusionRole::Ironing => "Ironing",
        _ => unreachable!("support generator emits only support and ironing roles"),
    }
}
