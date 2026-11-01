//! The bed-exclusion regions of a print config at v2.5.0-beta (`345efec64`): `BedExcludeRegion`
//! (`PrintConfig.hpp:2545-2558`) and their resolution (`PrintConfig.cpp:13887-14369`). See
//! `docs/architecture/web-plater-arrange-imex.md`, "A2b".

use crate::geometry::Polygon;
use crate::{BedExcludeVolumeMode, DynamicPrintConfig};

/// `BedExcludeRegion::Purpose`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Purpose {
    #[default]
    MaterialKeepout,
    CollisionVolume,
}

/// `BedExcludeRegion` (`PrintConfig.hpp:2545-2558`): a bed polygon kept clear between `z_min` and
/// `z_max` mm.
#[derive(Clone, Debug, PartialEq)]
pub struct BedExcludeRegion {
    pub polygon: Polygon,
    pub z_min: f64,
    pub z_max: f64,
    pub purpose: Purpose,
    pub has_z_range: bool,
}

impl BedExcludeRegion {
    pub fn is_collision_volume(&self) -> bool {
        self.purpose == Purpose::CollisionVolume
    }
}

/// `get_bed_excluded_regions_by_extruder(const DynamicPrintConfig&)` (`PrintConfig.cpp:14211-14243`).
pub fn get_bed_excluded_regions_by_extruder(
    _config: &DynamicPrintConfig,
) -> Vec<Vec<BedExcludeRegion>> {
    todo!("get_bed_excluded_regions_by_extruder")
}

/// `get_bed_excluded_regions(const DynamicPrintConfig&)` (`PrintConfig.cpp:14283-14286`).
pub fn get_bed_excluded_regions(_config: &DynamicPrintConfig) -> Vec<BedExcludeRegion> {
    todo!("get_bed_excluded_regions")
}

/// `is_valid_bed_exclude_volumes_string` (`PrintConfig.cpp:14298-14317`).
pub fn is_valid_bed_exclude_volumes_string(_value: &str, _printable_height: f64) -> bool {
    todo!("is_valid_bed_exclude_volumes_string")
}

/// `has_bed_exclude_volumes(const DynamicPrintConfig&)` (`PrintConfig.cpp:14346-14355`).
pub fn has_bed_exclude_volumes(_config: &DynamicPrintConfig) -> bool {
    todo!("has_bed_exclude_volumes")
}

/// `active_bed_exclude_volume_mode(const DynamicPrintConfig&)` (`PrintConfig.cpp:14364-14369`).
pub fn active_bed_exclude_volume_mode(_config: &DynamicPrintConfig) -> BedExcludeVolumeMode {
    todo!("active_bed_exclude_volume_mode")
}
