use crate::{ObjectOptions, ProcessSupportType};

use crate::project_slice::region_slices::RegionSurfaceKind;

pub(super) fn bottom_kind(options: &ObjectOptions) -> RegionSurfaceKind {
    let has_support = options.enable_support.0 || options.enforce_support_layers.0 > 0;
    let automatic_supports_bottom = has_support
        && options.support_top_z_distance.0 == 0.0
        && match options.support_type {
            ProcessSupportType::NormalAuto => !options.bridge_no_support.0,
            ProcessSupportType::TreeAuto => {
                options.support_interface_top_layers.0 > 0
                    && options.max_bridge_length.0 == 0.0
                    && !options.support_critical_regions_only.0
            }
            ProcessSupportType::NormalManual | ProcessSupportType::TreeManual => false,
        };
    if automatic_supports_bottom {
        RegionSurfaceKind::Bottom
    } else {
        RegionSurfaceKind::BottomBridge
    }
}
