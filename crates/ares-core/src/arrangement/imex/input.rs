//! What the arrange job reads into `ImexArrangeInput` (`IMEXArrange.cpp:65-90` at v2.5.0-beta
//! `345efec64`).

use super::{BoundingBoxf, ImexArrangeInput, ImexArrangeZones};
use crate::DynamicPrintConfig;
use crate::arrangement::imex_zones::{IMEX_PRIMARY_MODE, ImexZoneLayout, compute_imex_zone_layout};

/// `imex_arrange_zones` (`IMEXArrange.cpp:65-73`): the zones of `layout`, none when it has no
/// primary zone, i.e. outside a parallel mode.
fn imex_arrange_zones(layout: &ImexZoneLayout) -> Option<ImexArrangeZones> {
    let primary_zone = layout.primary_zone_box?;
    Some(ImexArrangeZones {
        primary_zone,
        collision_zones: layout
            .collision_zones
            .iter()
            .map(|strip| BoundingBoxf {
                min: [strip.min[0], strip.min[1]],
                max: [strip.max[0], strip.max[1]],
            })
            .collect(),
    })
}

/// `get_extents(Points)` of points in mm.
fn get_extents(points: &[[f64; 2]]) -> BoundingBoxf {
    let (mut min, mut max) = ([f64::MAX; 2], [f64::MIN; 2]);
    for point in points {
        for axis in 0..2 {
            min[axis] = min[axis].min(point[axis]);
            max[axis] = max[axis].max(point[axis]);
        }
    }
    BoundingBoxf { min, max }
}

impl ImexArrangeInput {
    /// `ImexArrangeInput::read_config` (`IMEXArrange.cpp:75-90`): `added_plate` and `brim` from the
    /// full config. Call after setting `adds_plates`.
    pub(crate) fn read_config(&mut self, full_config: &DynamicPrintConfig) {
        // A plate the arrange adds has no mode of its own, so it takes the process preset's.
        self.added_plate = None;
        if self.adds_plates {
            let mode = full_config.option("imex_parallel_mode");
            let area = full_config.option("printable_area").points();
            self.added_plate = imex_arrange_zones(&compute_imex_zone_layout(
                full_config,
                IMEX_PRIMARY_MODE,
                mode.text(),
                get_extents(&area),
            ));
        }
        // An estimate: auto brim picks its own width, which brim_width stands in for.
        self.brim = match full_config.option("brim_type").key() {
            "no_brim" | "inner_only" => 0.0,
            _ => {
                full_config.option("brim_width").float()
                    + full_config.option("brim_object_gap").float()
            }
        };
    }
}
