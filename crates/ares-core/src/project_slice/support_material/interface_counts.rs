//! Interface counts from `SupportParameters.hpp:25-59`.

use crate::project_slice::parameters::SlicingParameters;
use crate::{ObjectOptions, OrcaBools};

pub(super) struct InterfaceCounts {
    pub(super) top: usize,
    pub(super) bottom: usize,
    pub(super) top_base: usize,
    pub(super) bottom_base: usize,
}

impl InterfaceCounts {
    pub(super) fn new(
        object: &ObjectOptions,
        slicing: &SlicingParameters,
        soluble: &OrcaBools,
    ) -> Self {
        let get_at = |filament: i32| {
            let index = (filament - 1) as usize;
            soluble.0.get(index).unwrap_or(&soluble.0[0]).0
        };
        let soluble_interface_non_soluble_base = object.support_interface_filament.0 > 0
            && get_at(object.support_interface_filament.0)
            && (object.support_filament.0 == 0 || !get_at(object.support_filament.0));
        let different_filament = object.support_interface_filament.0 != 0
            && object.support_interface_filament.0 != object.support_filament.0;
        let top = object.support_interface_top_layers.0.max(0) as usize;
        let bottom = if object.support_interface_bottom_layers.0 < 0 {
            object.support_interface_top_layers.0
        } else {
            object.support_interface_bottom_layers.0
        } as usize;
        let base_count = |count: usize, zero_gap: bool| {
            if zero_gap && soluble_interface_non_soluble_base {
                (count / 2).min(2)
            } else {
                usize::from(different_filament && count > 1)
            }
        };
        Self {
            top,
            bottom,
            top_base: base_count(top, slicing.zero_gap_interface_top),
            bottom_base: base_count(bottom, slicing.zero_gap_interface_bottom),
        }
    }

    /// SupportCommon.cpp:215–225: a separately printed contact consumes one requested
    /// interface layer; Organic bottom contacts are projection seeds and consume none.
    pub(super) fn projected(&self, organic: bool) -> Self {
        let top = if self.top > 0 { self.top - 1 } else { 0 };
        let bottom = if self.bottom > 0 {
            self.bottom - if organic { 0 } else { 1 }
        } else {
            0
        };
        Self {
            top,
            bottom,
            top_base: self.top_base.min(top),
            bottom_base: self.bottom_base.min(bottom),
        }
    }
}
