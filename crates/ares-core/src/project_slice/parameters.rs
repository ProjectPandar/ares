use crate::{ObjectOptions, OrcaFloats, ProjectSettings, SliceError};

const MIN_LAYER_HEIGHT: f64 = 0.01;
const DEFAULT_MIN_LAYER_HEIGHT: f64 = 0.07;

#[derive(Clone, Debug, PartialEq)]
pub(super) struct SlicingParameters {
    pub(super) base_raft_layers: usize,
    pub(super) interface_raft_layers: usize,
    pub(super) base_raft_layer_height: f64,
    pub(super) interface_raft_layer_height: f64,
    pub(super) contact_raft_layer_height: f64,
    pub(super) layer_height: f64,
    pub(super) min_layer_height: f64,
    pub(super) max_layer_height: f64,
    pub(super) first_print_layer_height: f64,
    pub(super) first_object_layer_height: f64,
    pub(super) first_object_layer_bridging: bool,
    pub(super) gap_raft_object: f64,
    pub(super) gap_object_support: f64,
    pub(super) gap_support_object: f64,
    pub(super) raft_base_top_z: f64,
    pub(super) raft_interface_top_z: f64,
    pub(super) raft_contact_top_z: f64,
    pub(super) object_print_z_min: f64,
    pub(super) object_print_z_max: f64,
    pub(super) object_print_z_uncompensated_max: f64,
    pub(super) object_shrinkage_compensation_z: f64,
}

pub(super) fn slicing_parameters(
    settings: &ProjectSettings,
    object: &ObjectOptions,
    object_height: f64,
    object_extruders: &[usize],
) -> Result<SlicingParameters, SliceError> {
    let layer_height = object.layer_height.0;
    require_positive("layer_height", layer_height)?;

    let configured_first_height = settings.process.print.initial_layer_print_height.0;
    let first_height = if configured_first_height > 0.0 {
        configured_first_height
    } else {
        layer_height
    };
    require_positive("initial_layer_print_height", first_height)?;
    if !object_height.is_finite() || object_height <= 0.0 {
        return Err(SliceError::InvalidInput(
            "project-object Z bounds require finite model-part vertices and a positive maximum Z"
                .to_owned(),
        ));
    }

    let print = &settings.project.print;
    let mut min_layer_height = MIN_LAYER_HEIGHT;
    let mut max_layer_height = f64::MAX;
    let mut accumulate = |extruder| {
        let (nozzle_minimum, nozzle_maximum) = nozzle_limits(
            extruder,
            &print.nozzle_diameter,
            &print.min_layer_height,
            &print.max_layer_height,
        );
        min_layer_height = min_layer_height.max(nozzle_minimum);
        max_layer_height = max_layer_height.min(nozzle_maximum);
    };
    if object_extruders.is_empty() {
        accumulate(0);
    } else {
        for &extruder in object_extruders {
            accumulate(extruder);
        }
    }
    min_layer_height = min_layer_height.min(layer_height);
    max_layer_height = max_layer_height.max(layer_height);

    Ok(SlicingParameters {
        base_raft_layers: 0,
        interface_raft_layers: 0,
        base_raft_layer_height: 0.0,
        interface_raft_layer_height: 0.0,
        contact_raft_layer_height: 0.0,
        layer_height,
        min_layer_height,
        max_layer_height,
        first_print_layer_height: first_height,
        first_object_layer_height: first_height,
        first_object_layer_bridging: false,
        gap_raft_object: 0.0,
        gap_object_support: 0.0,
        gap_support_object: 0.0,
        raft_base_top_z: 0.0,
        raft_interface_top_z: 0.0,
        raft_contact_top_z: 0.0,
        object_print_z_min: 0.0,
        object_print_z_max: object_height,
        object_print_z_uncompensated_max: object_height,
        object_shrinkage_compensation_z: 1.0,
    })
}

fn nozzle_limits(
    extruder: usize,
    nozzle_diameters: &OrcaFloats,
    configured_minimums: &OrcaFloats,
    configured_maximums: &OrcaFloats,
) -> (f64, f64) {
    let configured_minimum = get_at(configured_minimums, extruder);
    let nozzle_minimum = if configured_minimum == 0.0 {
        DEFAULT_MIN_LAYER_HEIGHT
    } else {
        MIN_LAYER_HEIGHT.max(configured_minimum)
    };
    let configured_maximum = get_at(configured_maximums, extruder);
    let nozzle_maximum = nozzle_minimum.max(if configured_maximum == 0.0 {
        0.75 * get_at(nozzle_diameters, extruder)
    } else {
        configured_maximum
    });
    (nozzle_minimum, nozzle_maximum)
}

fn get_at(values: &OrcaFloats, object_extruder_id: usize) -> f64 {
    let lookup = object_extruder_id.checked_sub(1).unwrap_or(usize::MAX);
    values.0.get(lookup).or_else(|| values.0.first()).unwrap().0
}

fn require_positive(key: &str, value: f64) -> Result<(), SliceError> {
    if value.is_finite() && value > 0.0 {
        Ok(())
    } else {
        Err(SliceError::InvalidInput(format!(
            "invalid Orca option {key}"
        )))
    }
}
