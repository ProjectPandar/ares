//! `SupportMaterial.cpp::new_contact_layer` and its top-contact gap synchronizer.

use super::contact_layer::TopContactLayer;
use crate::ObjectOptions;
use crate::project_slice::{layers::PlannedLayer, parameters::SlicingParameters};

const EPSILON: f64 = 1e-4;

#[derive(Clone, Copy)]
pub(super) struct ContactPlacement<'a> {
    pub(super) parameters: &'a SlicingParameters,
    pub(super) object: &'a ObjectOptions,
    pub(super) independent_support_layer_height: bool,
    pub(super) support_layer_height_min: f64,
    pub(super) layers: &'a [PlannedLayer],
}

impl ContactPlacement<'_> {
    /// `bridging_height` is the region-average native bridging height, only
    /// present when the object layer has bridging extrusions.
    pub(super) fn new_contact_layer(
        &self,
        layer_index: usize,
        bridging_height: Option<f64>,
    ) -> Option<(TopContactLayer, Option<TopContactLayer>)> {
        let parameters = self.parameters;
        let layer = &self.layers[layer_index];
        let raft_layers = parameters.base_raft_layers + parameters.interface_raft_layers;
        let zero_gap_interface_top = parameters.zero_gap_interface_top;
        if layer_index == 0 {
            return Some((
                TopContactLayer {
                    print_z: parameters.raft_contact_top_z,
                    bottom_z: parameters.raft_interface_top_z,
                    height: parameters.contact_raft_layer_height,
                    ..Default::default()
                },
                None,
            ));
        }
        if zero_gap_interface_top {
            let lower = &self.layers[layer_index - 1];
            return Some((
                TopContactLayer {
                    print_z: layer.print_z - layer.height,
                    bottom_z: if layer_index == 1 {
                        parameters.object_print_z_min
                    } else {
                        self.layers[layer_index - 2].print_z
                    },
                    height: lower.height,
                    ..Default::default()
                },
                None,
            ));
        }
        let mut bridging_layer = None;
        let (mut print_z, mut height) = if self.independent_support_layer_height {
            (
                layer.print_z - layer.height - parameters.gap_support_object,
                0.0,
            )
        } else {
            let synced = sync_top_gap(self.layers, layer_index, parameters.gap_support_object);
            (synced.print_z, synced.height)
        };
        let mut bottom_z = print_z - height;
        if print_z < parameters.first_print_layer_height - EPSILON {
            return None;
        }
        let has_raft = raft_layers > 1;
        let min_print_z = if has_raft {
            parameters.raft_contact_top_z
        } else {
            parameters.first_print_layer_height
        };
        if print_z < min_print_z + self.support_layer_height_min {
            print_z = min_print_z;
            bottom_z = if has_raft {
                parameters.raft_interface_top_z
            } else {
                0.0
            };
            height = if has_raft {
                parameters.contact_raft_layer_height
            } else {
                min_print_z
            };
        }
        if self.object.thick_bridges.0
            && self.independent_support_layer_height
            && let Some(bridging_height) = bridging_height
        {
            let mut bridging_print_z =
                layer.print_z - bridging_height - parameters.gap_support_object;
            if bridging_print_z >= min_print_z
                && print_z < min_print_z + self.support_layer_height_min
            {
                bridging_print_z = min_print_z;
            }
            let height = if bridging_print_z == parameters.first_print_layer_height {
                parameters.first_print_layer_height
            } else {
                0.0
            };
            if bridging_print_z >= min_print_z && bridging_print_z < print_z - EPSILON {
                bridging_layer = Some(TopContactLayer {
                    print_z: bridging_print_z,
                    bottom_z: bridging_print_z - height,
                    height,
                    ..Default::default()
                });
            }
        }
        Some((
            TopContactLayer {
                print_z,
                bottom_z,
                height,
                ..Default::default()
            },
            bridging_layer,
        ))
    }
}

fn sync_top_gap(layers: &[PlannedLayer], layer_index: usize, gap: f64) -> &PlannedLayer {
    // Upstream accumulates the double layer heights into a float gap.
    let mut gap_synced = 0.0_f32;
    let mut lower = Some(layer_index - 1);
    let mut last_valid = layer_index - 1;
    while let Some(index) = lower
        && f64::from(gap_synced) < gap
    {
        last_valid = index;
        gap_synced = (f64::from(gap_synced) + layers[index].height) as f32;
        lower = index.checked_sub(1);
    }
    if (f64::from(gap_synced) - layers[last_valid].height - gap).abs()
        < (f64::from(gap_synced) - gap).abs()
    {
        last_valid += 1;
    }
    &layers[last_valid.checked_sub(1).unwrap_or(last_valid)]
}
