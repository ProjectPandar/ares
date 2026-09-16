//! Raft layer z-grid, ported from the layer-push loops of upstream
//! `generate_raft_base` (`OrcaSlicer/src/libslic3r/Support/SupportCommon.cpp:244-390`).
//!
//! Upstream pushes, in order: the 1st layer at
//! `first_print_layer_height`, then `base_raft_layers - 1` base layers
//! stepping `base_raft_layer_height`, then `interface_raft_layers - 1`
//! interface layers stepping `interface_raft_layer_height`, then the
//! single contact layer stepping `contact_raft_layer_height`. The
//! object's first layer sits at `raft_contact_top_z + gap_raft_object`
//! (`Slicing.cpp:232-236`), which `SlicingParameters` already carries
//! as `object_print_z_min`.

use crate::SliceError;

use super::super::parameters::SlicingParameters;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RaftLayerKind {
    Base,
    Interface,
    Contact,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct RaftLayerZ {
    pub(crate) bottom_z: f64,
    pub(crate) print_z: f64,
    pub(crate) height: f64,
    pub(crate) kind: RaftLayerKind,
}

/// The full raft layer stack below the object. Empty when the object
/// has no raft (`raft_layers == 0`).
pub(crate) fn raft_layer_grid(
    parameters: &SlicingParameters,
) -> Result<Vec<RaftLayerZ>, SliceError> {
    let mut layers = Vec::new();
    let total = parameters.base_raft_layers + parameters.interface_raft_layers;
    if total == 0 {
        return Ok(layers);
    }

    // Insert the 1st layer: a base layer when any base layers exist,
    // otherwise the raft is interface-only (`SupportCommon.cpp:366`).
    layers.push(RaftLayerZ {
        bottom_z: 0.0,
        print_z: parameters.first_print_layer_height,
        height: parameters.first_print_layer_height,
        kind: if parameters.base_raft_layers > 0 {
            RaftLayerKind::Base
        } else {
            RaftLayerKind::Interface
        },
    });

    if total == 1 {
        // `raft_layers() == 1`: generate_raft_base's `> 1` branch is
        // not taken; the single layer IS the contact layer at
        // `initial_layer_print_height` (`Slicing.cpp:208-211`),
        // arriving from top_contacts rather than a pushed raft layer.
        layers[0].kind = RaftLayerKind::Contact;
        return Ok(layers);
    }

    let push = |layers: &mut Vec<RaftLayerZ>, height: f64, kind: RaftLayerKind| {
        let bottom_z = layers.last().map(|layer| layer.print_z).unwrap_or(0.0);
        layers.push(RaftLayerZ {
            bottom_z,
            print_z: bottom_z + height,
            height,
            kind,
        });
    };

    // Base layers (`SupportCommon.cpp:373-381`).
    for _ in 1..parameters.base_raft_layers {
        push(
            &mut layers,
            parameters.base_raft_layer_height,
            RaftLayerKind::Base,
        );
    }
    // Interface layers (`SupportCommon.cpp:382-390`).
    for _ in 1..parameters.interface_raft_layers {
        push(
            &mut layers,
            parameters.interface_raft_layer_height,
            RaftLayerKind::Interface,
        );
    }
    // Contact layer: not pushed by generate_raft_base — it arrives
    // from top_contacts (the 1st object layer silhouette without
    // holes) at `raft_contact_top_z`
    // (`Slicing.cpp:232` / `SupportMaterial.cpp:487` comment).
    push(
        &mut layers,
        parameters.contact_raft_layer_height,
        RaftLayerKind::Contact,
    );
    Ok(layers)
}

#[cfg(test)]
mod tests;
