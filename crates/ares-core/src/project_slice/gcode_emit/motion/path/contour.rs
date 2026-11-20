//! GCode.cpp:6374-6400,6992-7030: source XYZ paths and height-scaled extrusion.
use std::borrow::Cow;

use super::{EmitState, LayerGeometry, PathProperties, SOURCE_EPSILON_MM, arc, extrusion, fan};
use crate::project_slice::perimeters::classic::materialize::Point3;

pub(in crate::project_slice::gcode_emit::motion) fn clipped(
    points: &[Point3],
    mut distance: f64,
) -> Cow<'_, [Point3]> {
    if distance <= 0.0 {
        return Cow::Borrowed(points);
    }
    let mut points = points.to_vec();
    // Polyline.cpp:683-705: scaled Line3 length and truncating XYZ endpoint.
    while distance > 0.0 {
        let last = points.pop().unwrap();
        let Some(previous) = points.last() else { break };
        let delta = [
            (previous.x - last.x) as f64,
            (previous.y - last.y) as f64,
            (previous.z - last.z) as f64,
        ];
        let squared = delta.iter().map(|v| v * v).sum::<f64>();
        if squared > distance * distance {
            let ratio = distance / squared.sqrt();
            points.push(Point3 {
                x: (last.x as f64 + delta[0] * ratio) as i64,
                y: (last.y as f64 + delta[1] * ratio) as i64,
                z: (last.z as f64 + delta[2] * ratio) as i64,
            });
            break;
        }
        distance -= squared.sqrt();
    }
    Cow::Owned(points)
}

pub(super) fn emit(
    output: &mut Vec<u8>,
    xy: &[(f64, f64)],
    (properties, path_overhang_fan_active): (PathProperties<'_>, bool),
    geometry: LayerGeometry<'_>,
    state: &mut EmitState,
) -> Result<(), crate::SliceError> {
    let points = properties.contour.unwrap();
    fan::update_for_constant_path(output, properties, path_overhang_fan_active, state);
    for (pair, end) in points.windows(2).zip(&xy[1..]) {
        let dx = (pair[1].x - pair[0].x) as f64;
        let dy = (pair[1].y - pair[0].y) as f64;
        let dz = (pair[1].z - pair[0].z) as f64;
        let length = (dx * dx + dy * dy + dz * dz).sqrt() * geometry.scale.factor();
        if length < SOURCE_EPSILON_MM {
            continue;
        }
        segment(
            output,
            crate::Point3d::new(end.0, end.1, geometry.scale.unscale(pair[1].z)),
            length,
            properties,
            state,
        );
        // GCode.cpp:10118: the farthest-point check after each z-contoured line.
        crate::project_slice::gcode_emit::layers::farthest::check_and_insert(
            output,
            state,
            (
                geometry.scale.unscale(pair[1].x) + state.origin.0,
                geometry.scale.unscale(pair[1].y) + state.origin.1,
            ),
            geometry.timelapse_context,
        )?;
    }
    output.extend_from_slice(b";_EXTRUDE_END\n");
    Ok(())
}

pub(super) fn segment(
    output: &mut Vec<u8>,
    end: crate::Point3d,
    length: f64,
    properties: PathProperties<'_>,
    state: &mut EmitState,
) {
    let z_diff = end.z;
    // GCode.cpp:7012-7030/7220-7237: ironing retains its original volume.
    let ratio = if properties.feature == "Ironing" {
        1.0
    } else {
        (f64::from(properties.height) + z_diff) / f64::from(properties.height)
    };
    let z = state.layer_z + z_diff;
    // GCode.cpp:7025-7026/7233-7234: the export exception reaches the slicing API.
    if z < 0.1 {
        state.contour_z_too_low = true;
        return;
    }
    let delta = extrusion::for_length(
        length,
        extrusion::effective_flow(properties, &state.options, state.on_first_layer),
        state.options.filament_flow_ratio,
        state.options.filament_area,
    ) * extrusion::small_area_multiplier(state, properties.feature, length)
        * ratio;
    extrusion::extrude_to_xyz(
        output,
        extrusion::XyzEmission {
            end: arc::Point { x: end.x, y: end.y },
            z,
            delta,
            properties,
        },
        state,
    );
}
