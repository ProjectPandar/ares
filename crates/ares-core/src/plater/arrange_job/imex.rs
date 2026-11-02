//! `ArrangeJob::prepare_imex_zones` (`ArrangeJob.cpp:605-629` at v2.5.0-beta `345efec64`). See
//! `docs/architecture/web-plater-arrange-imex.md`, "A2c2".

use crate::arrangement::imex::{BoundingBoxf, ImexArrangeInput, ImexArrangeZones};
use crate::arrangement::imex_zones::ImexZoneLayout;
use crate::plater::plate_list::PlateList;
use crate::project_slice::gcode_emit::wipe_tower_estimate::config::DynamicPrintConfig;

/// The arranger's beds: each arranged plate's primary zone and collision strips moved by the
/// plate's origin, for the current plate alone or else every unlocked plate. Arranging every plate
/// adds plates.
pub(super) fn prepare_imex_zones(
    plates: &PlateList,
    zones: &[ImexZoneLayout],
    current: Option<usize>,
    full: &DynamicPrintConfig,
) -> ImexArrangeInput {
    let arranged: Vec<usize> = match current {
        Some(plate) => vec![plate],
        None => (0..zones.len())
            .filter(|&plate| !plates.plates[plate].locked)
            .collect(),
    };
    let beds = arranged
        .into_iter()
        .map(|plate| {
            let layout = &zones[plate];
            let primary = layout.primary_zone_box?;
            let [x, y] = plates.plate_origin(plate);
            let local = |min: [f64; 2], max: [f64; 2]| BoundingBoxf {
                min: [min[0] - x, min[1] - y],
                max: [max[0] - x, max[1] - y],
            };
            Some(ImexArrangeZones {
                primary_zone: local(primary.min, primary.max),
                collision_zones: layout
                    .collision_zones
                    .iter()
                    .map(|strip| local([strip.min[0], strip.min[1]], [strip.max[0], strip.max[1]]))
                    .collect(),
            })
        })
        .collect();
    let mut input = ImexArrangeInput {
        beds,
        adds_plates: current.is_none(),
        ..ImexArrangeInput::default()
    };
    input.read_config(full);
    input
}
