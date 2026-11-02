//! `PartPlate::calc_imex_zones` (`PartPlate.cpp:670-738` at v2.5.0-beta `345efec64`) for every
//! plate, without the scene models. Upstream caches each plate's zones behind
//! `ensure_imex_zones`'s key (`:757-796, 828-839`); the session computes them once per arrange or
//! outside-check pass instead. See `docs/architecture/web-plater-arrange-imex.md`, "A2c2".

use super::PlateList;
use crate::Project;
use crate::arrangement::imex::BoundingBoxf;
use crate::arrangement::imex_zones::{IMEX_PRIMARY_MODE, ImexZoneLayout, compute_imex_zone_layout};
use crate::project_slice::gcode_emit::wipe_tower_estimate::config::DynamicPrintConfig;

impl PlateList {
    /// Each plate's zones, from the printer config, the plate's own mode, the process preset's mode
    /// and the plate shape where the plate sits (`get_extents(m_shape)`).
    pub(in crate::plater) fn imex_zones(
        &self,
        project: &Project,
        config: &DynamicPrintConfig,
    ) -> Vec<ImexZoneLayout> {
        let process_mode = config.option("imex_parallel_mode");
        let (min, max) = self.shape;
        self.plates
            .iter()
            .map(|plate| {
                // `get_imex_mode` (`:457-465`): the plate's own mode, else Primary. A plate the
                // session adds has no config of its own.
                let plate_mode = plate
                    .source
                    .and_then(|source| project.plate_imex_mode(source))
                    .filter(|mode| !mode.is_empty())
                    .unwrap_or(IMEX_PRIMARY_MODE);
                let [x, y] = plate.origin;
                let extents = BoundingBoxf {
                    min: [min[0] + x, min[1] + y],
                    max: [max[0] + x, max[1] + y],
                };
                compute_imex_zone_layout(config, plate_mode, process_mode.text(), extents)
            })
            .collect()
    }
}
