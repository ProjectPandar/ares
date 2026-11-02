//! OrcaSlicer's arrangement kernel at v2.5.0-beta (`345efec64`): `src/libslic3r/Arrange.hpp`
//! and `Arrange.cpp` over the libnest2d first-fit / NFP placer and NLopt 2.5.0 subplex. See
//! `docs/architecture/web-plater-move-rotate-arrange.md`, "Auto-arrange on OrcaSlicer
//! v2.5.0-beta".

mod arrange;
mod arranger;
mod exclusion_volume;
pub mod imex;
pub mod imex_zones;
mod nest;
mod nlopt;
mod params;
mod rotcalipers;
mod types;

pub use crate::geometry::{BoundingBox, ExPolygon, Point, Polygon};
pub use exclusion_volume::bed_exclusion_arrange_polygon;
pub use params::{
    bed_exclusion_applies, get_shrink_bedpts, has_bed_exclusion_regions, update_arrange_params,
    update_selected_items_axis_align, update_selected_items_inflation,
    update_unselected_items_inflation,
};
pub use types::{ArrangeConfig, ArrangeParams, ArrangePolygon, Bed};

/// Scaled coordinate (`coord_t`).
pub type Coord = i64;

/// The logical bed of an item arrange has not placed (`Arrange.hpp:48`).
pub const UNARRANGED: i32 = -1;

/// `MAX_NUM_PLATES` (`libnest2d/common.hpp:14`): the beds an arrange may open.
pub(crate) const MAX_NUM_PLATES: usize = 36;

/// `MAX_OUTER_NOZZLE_DIAMETER` (`Print.hpp:72`), in mm.
pub const MAX_OUTER_NOZZLE_DIAMETER: f64 = 4.0;

/// `SCALING_FACTOR_INTERNAL` (`libslic3r.h:65`): the scale of a printer below 2147 mm.
pub const SCALING_FACTOR: f64 = 0.000_001;

/// `arrangement::arrange` (`Arrange.cpp:1279-1321`): places `items` on `bed` and the virtual
/// beds after it around the fixed `excludes`, setting each item's `translation`, `rotation` and
/// `bed_idx`; an item that does not fit an empty bed keeps its input and stays `UNARRANGED`.
pub fn arrange(
    items: &mut [ArrangePolygon],
    excludes: &[ArrangePolygon],
    bed: &Bed,
    params: &ArrangeParams,
) {
    arrange::arrange(items, excludes, bed, params);
}
