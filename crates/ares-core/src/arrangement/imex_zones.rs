//! The IMEX zone layout (`libslic3r/IMEXZones.cpp:17-377`, `IMEXZones.hpp:25-71` at v2.5.0-beta
//! `345efec64`): where an IDEX/IQEX plate's primary zone, Copy/Mirror zones, carriage collision
//! strips and advisory margin bands sit, in plate-local mm. See
//! `docs/architecture/web-plater-arrange-imex.md`, "A2c1".

use std::collections::BTreeMap;

use super::imex::BoundingBoxf;
use crate::DynamicPrintConfig;

/// `kImexPrimaryMode` (`IMEXHelpers.hpp:27`): the mode with no parallel tools.
pub const IMEX_PRIMARY_MODE: &str = "primary";

/// `BoundingBoxf3`: a box in mm.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoundingBoxf3 {
    pub min: [f64; 3],
    pub max: [f64; 3],
}

/// `ImexZoneLayout` (`IMEXZones.hpp:25-71`). Every list is empty and `primary_zone_box` unset
/// when no parallel mode is in effect.
#[derive(Clone, Debug, PartialEq)]
pub struct ImexZoneLayout {
    /// The active mode's Primary tool; -1 when the layout is empty.
    pub primary_head: i32,
    /// Physical head -> the XY centre of its zone, Primary included; a Span tool has none.
    pub head_zone_centers: BTreeMap<i32, [f64; 2]>,
    /// The primary tool's zone, where objects go.
    pub primary_zone_box: Option<BoundingBoxf>,
    pub copy_zones: Vec<BoundingBoxf>,
    pub mirror_zones: Vec<BoundingBoxf>,
    /// `copy_zones` then `mirror_zones` as full-height boxes, for the placement check.
    pub secondary_zone_boxes: Vec<BoundingBoxf3>,
    /// Carriage danger strips inside the primary zone, right / left / top / bottom.
    pub collision_zones: Vec<BoundingBoxf3>,
    /// Advisory bands inside each strip, when `imex_carriage_margin` > 0.
    pub margin_bands: Vec<BoundingBoxf>,
}

/// `compute_imex_zone_layout` (`IMEXZones.cpp:17-377`): the layout of one plate from the printer
/// config, the plate's own mode (which wins unless it is `IMEX_PRIMARY_MODE`), the process
/// preset's `imex_parallel_mode` and the plate's XY extents.
pub fn compute_imex_zone_layout(
    _printer_cfg: &DynamicPrintConfig,
    _plate_mode: &str,
    _process_mode: &str,
    _bed_extents: BoundingBoxf,
) -> ImexZoneLayout {
    todo!("arrangement::imex_zones::compute_imex_zone_layout")
}

/// `compute_imex_slice_offset` (`IMEXHelpers.cpp:634-643`): the primary zone's centre when the
/// firmware places the secondaries and a parallel mode is active, else zero.
pub fn compute_imex_slice_offset(
    _firmware_managed: bool,
    _parallel_mode: &str,
    _primary_zone_box: Option<BoundingBoxf>,
) -> [f64; 2] {
    todo!("arrangement::imex_zones::compute_imex_slice_offset")
}
