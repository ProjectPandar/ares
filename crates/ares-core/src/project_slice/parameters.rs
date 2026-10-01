use crate::{ObjectOptions, OrcaFloats, ProjectSettings, SliceError};

const MIN_LAYER_HEIGHT: f64 = 0.01;
const DEFAULT_MIN_LAYER_HEIGHT: f64 = 0.07;
const EPSILON: f64 = 1e-4;

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
    if object.enable_support.0 || object.raft_layers.0 > 0 || object.enforce_support_layers.0 > 0 {
        accumulate(object.support_filament.0.max(0) as usize);
        accumulate(object.support_interface_filament.0.max(0) as usize);
    }
    if object_extruders.is_empty() {
        accumulate(0);
    } else {
        for &extruder in object_extruders {
            accumulate(extruder);
        }
    }
    min_layer_height = min_layer_height.min(layer_height);
    max_layer_height = max_layer_height.max(layer_height);

    // Raft z-math (`Slicing.cpp:194-236`).
    let nozzles = &settings.project.print.nozzle_diameter.0;
    let nozzle_at = |filament: i32| -> f64 {
        let index = filament.saturating_sub(1).max(0) as usize;
        nozzles
            .get(index)
            .or_else(|| nozzles.first())
            .map_or(0.4, |value| value.0)
    };
    let support_nozzle = nozzle_at(object.support_filament.0);
    let support_interface_nozzle = nozzle_at(object.support_interface_filament.0);
    let mut base_raft_layers = object.raft_layers.0.max(0) as usize;
    let mut interface_raft_layers = 0usize;
    let (base_raft_layer_height, interface_raft_layer_height, contact_raft_layer_height) =
        if base_raft_layers > 0 {
            interface_raft_layers = base_raft_layers.div_ceil(2);
            base_raft_layers -= interface_raft_layers;
            (
                layer_height.max(0.75 * support_nozzle),
                layer_height.max(0.75 * support_interface_nozzle),
                layer_height.max(0.75 * support_interface_nozzle),
            )
        } else {
            (0.0, 0.0, 0.0)
        };
    let total_raft_layers = base_raft_layers + interface_raft_layers;
    // Asymmetric zero-gap detection and assignment (`Slicing.cpp:86-190`).
    let zero_top_contact = object.support_top_z_distance.0 == 0.0;
    let zero_gap_top = object.support_interface_top_layers.0 > 0 && zero_top_contact;
    let bottom_interface_layers = if object.support_interface_bottom_layers.0 < 0 {
        object.support_interface_top_layers.0
    } else {
        object.support_interface_bottom_layers.0
    };
    let zero_gap_bottom = bottom_interface_layers > 0
        && (object.support_bottom_z_distance.0 == 0.0 || zero_top_contact);
    let gap = |distance: f64, zero_gap: bool| {
        if zero_gap {
            0.0
        } else if settings.process.print.independent_support_layer_height.0 {
            distance
        } else {
            (distance / layer_height + EPSILON).round() * layer_height
        }
    };
    let gap_raft_object = if total_raft_layers == 0 {
        0.0
    } else {
        gap(object.raft_contact_distance.0, zero_top_contact)
    };
    let gap_object_support = gap(object.support_bottom_z_distance.0, zero_gap_bottom);
    let gap_support_object = gap(object.support_top_z_distance.0, zero_gap_top);
    let (first_object_layer_height, first_object_layer_bridging) = if total_raft_layers > 0 {
        (layer_height, false)
    } else {
        (first_height, false)
    };
    let (raft_base_top_z, raft_interface_top_z, raft_contact_top_z, object_print_z_min) =
        if total_raft_layers == 1 {
            (0.0, 0.0, first_height, first_height + gap_raft_object)
        } else if total_raft_layers > 1 {
            let base_top = first_height + (base_raft_layers - 1) as f64 * base_raft_layer_height;
            let interface_top =
                base_top + (interface_raft_layers - 1) as f64 * interface_raft_layer_height;
            let contact_top = interface_top + contact_raft_layer_height;
            (
                base_top,
                interface_top,
                contact_top,
                contact_top + gap_raft_object,
            )
        } else {
            (0.0, 0.0, 0.0, 0.0)
        };

    Ok(SlicingParameters {
        base_raft_layers,
        interface_raft_layers,
        base_raft_layer_height,
        interface_raft_layer_height,
        contact_raft_layer_height,
        layer_height,
        min_layer_height,
        max_layer_height,
        first_print_layer_height: first_height,
        first_object_layer_height,
        first_object_layer_bridging,
        gap_raft_object,
        gap_object_support,
        gap_support_object,
        raft_base_top_z,
        raft_interface_top_z,
        raft_contact_top_z,
        object_print_z_min,
        object_print_z_max: object_print_z_min + object_height,
        object_print_z_uncompensated_max: object_print_z_min + object_height,
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
