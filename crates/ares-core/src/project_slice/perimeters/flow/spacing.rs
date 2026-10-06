//! Flow.cpp:147-164,201-206: source spacing arithmetic and exceptions.
use super::super::types::Flow;
use super::{ROUNDED_RECTANGLE_FACTOR, bridge_volume, ordinary_volume};
use crate::SliceError;

pub(in crate::project_slice) fn rounded_spacing(
    width: f32,
    height: f32,
) -> Result<f32, SliceError> {
    let spacing = width - height * ROUNDED_RECTANGLE_FACTOR as f32;
    if spacing <= 0.0 {
        return Err(SliceError::InvalidInput(
            "Flow::spacing() produced negative spacing. Did you set some extrusion width too small?".to_owned(),
        ));
    }
    Ok(spacing)
}

pub(in crate::project_slice) fn with_spacing(flow: Flow, spacing: f32) -> Result<Flow, SliceError> {
    let (width, height, mm3_per_mm) = if flow.bridge {
        let diameter = spacing - (flow.spacing - flow.width);
        (diameter, diameter, bridge_volume(diameter))
    } else {
        debug_assert!(flow.width >= flow.height);
        let width = flow.width + (spacing - flow.spacing);
        if width < flow.height {
            return Err(SliceError::InvalidInput(
                "Invalid spacing supplied to Flow::with_spacing(), check your layer height and extrusion width".to_owned(),
            ));
        }
        (width, flow.height, ordinary_volume(width, flow.height))
    };
    Ok(Flow {
        width,
        height,
        spacing,
        nozzle_diameter: flow.nozzle_diameter,
        bridge: flow.bridge,
        mm3_per_mm,
    })
}
