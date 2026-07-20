use crate::{FloatOrPercent, OrcaFloats, SliceError, geometry::CoordinateScale};

use super::super::region_slices::PostRegionPrintObject;
use super::{ValidatedTask22mConfig, flow::resolve_external_perimeter_flow, geometry_error};

#[derive(Clone, Copy)]
pub(super) struct PreparedLayerCompensation {
    pub(super) compensation_mm: f64,
    pub(super) minimum_width_mm: f64,
}

pub(super) struct PreparedObjectCompensation {
    pub(super) backup_len: usize,
    pub(super) layers: Vec<Option<PreparedLayerCompensation>>,
}

pub(super) fn prepare_object_compensation(
    object: &PostRegionPrintObject,
    config: ValidatedTask22mConfig,
    initial_layer_width: FloatOrPercent,
    nozzle_diameters: &OrcaFloats,
    scale: CoordinateScale,
) -> Result<PreparedObjectCompensation, SliceError> {
    let scaled = if config.raft_layers != 0 {
        0.0_f32
    } else {
        (config.compensation_mm / scale.factor()) as f32
    };
    if !scaled.is_finite() {
        return Err(geometry_error());
    }

    let backup_len = if scaled > 0.0 {
        config.compensation_layers.min(object.plan.layers.len())
    } else {
        0
    };
    let mut layers = vec![None; object.plan.layers.len()];
    let [region] = object.regions.as_slice() else {
        return Ok(PreparedObjectCompensation { backup_len, layers });
    };

    for (layer_index, prepared_layer) in layers.iter_mut().enumerate().take(backup_len) {
        let ramp = scaled - (scaled / config.compensation_layers as f32) * layer_index as f32;
        if ramp <= 0.0 {
            continue;
        }
        let flow = resolve_external_perimeter_flow(
            &object.plan.layers[layer_index],
            initial_layer_width,
            region.options.outer_wall_line_width,
            config.object_line_width,
            region.options.outer_wall_filament_id,
            nozzle_diameters,
        )?;
        let compensation_mm = f64::from(ramp) * scale.factor();
        let minimum_width_mm = f64::from(flow.minimum_width);
        if !compensation_mm.is_finite() || !minimum_width_mm.is_finite() {
            return Err(geometry_error());
        }
        *prepared_layer = Some(PreparedLayerCompensation {
            compensation_mm,
            minimum_width_mm,
        });
    }

    Ok(PreparedObjectCompensation { backup_len, layers })
}
