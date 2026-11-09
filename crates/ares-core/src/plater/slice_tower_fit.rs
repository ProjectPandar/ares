//! The prime tower fit of a plain slice (`OrcaSlicer.cpp:6480-6509` at v2.5.0-beta `345efec64`):
//! the plate's stored position passes through `estimate_wipe_tower_polygon` before anything is
//! sliced. See `docs/architecture/slice-wipe-tower-fit-spec.md`.

use super::arrange_job::{TowerPositions, plate_sequence, set_at};
use super::plate_list::{Relocation, Towers};
use super::{PlaterEdits, PlaterSession};
use crate::options::ProcessPrintSequence;
use crate::project_slice::gcode_emit::wipe_tower_estimate::config::DynamicPrintConfig;
use crate::{Project, SliceError};

/// `EPSILON` (`libslic3r.h`).
const EPSILON: f64 = 1e-4;

/// `project` with the tower of the plate a slice of `plate` prints fitted to that plate. Ares
/// slices one plate per call, so only that plate's entry is fitted.
pub(crate) fn fit_slice_wipe_tower(
    project: Project,
    plate: Option<u32>,
) -> Result<Project, SliceError> {
    let config = DynamicPrintConfig::new(project.settings())?;
    // Belt printers are deferred (no `belt_printer` key).
    if !config.option("enable_prime_tower").bool() {
        return Ok(project);
    }
    let plate = plate.unwrap_or_else(|| project.first_plate_id());
    // `select_plate` reports a plate the project does not have.
    let Some(index) = project.plates().iter().position(|p| p.id() == plate) else {
        return Ok(project);
    };
    let mut session = PlaterSession::with_project(Vec::new(), project);
    if let Some((x, y)) = session.fitted_positions(index, &config)? {
        session.project.apply_plater_edits(&PlaterEdits {
            wipe_tower: Some((x, y)),
            ..PlaterEdits::NONE
        });
    }
    Ok(session.project)
}

impl PlaterSession {
    /// The `wipe_tower_x`/`wipe_tower_y` vectors with the plate's entry fitted, or `None` for a
    /// plate the fit leaves alone.
    fn fitted_positions(
        &self,
        index: usize,
        config: &DynamicPrintConfig,
    ) -> Result<Option<TowerPositions>, SliceError> {
        let zones = self.plates.imex_zones(&self.project, config);
        let by_object = plate_sequence(&self.project, &self.plates, index).map_or_else(
            || config.option("print_sequence").key() == "by object",
            |sequence| sequence == ProcessPrintSequence::ByObject,
        );
        if by_object && self.printable_instance_size(index, (config, &zones))? > 1 {
            return Ok(None);
        }
        let (mut x, mut y) = self.tower_positions();
        let towers = Towers {
            plates: &self.plates,
            project: &self.project,
            config,
            positions: (&x, &y),
            zones: &zones,
        };
        // A plate that prints no tower: one filament and neither smooth timelapse, wrapping
        // detection nor a raft.
        let footprint = towers.footprint(index, 0, false)?;
        if footprint.width < EPSILON || footprint.depth < EPSILON {
            return Ok(None);
        }
        let (_, fitted, _) = towers.polygon((index, index), 0, Some(&mut Relocation::default()))?;
        set_at(&mut x, index, fitted[0]);
        set_at(&mut y, index, fitted[1]);
        Ok(Some((x, y)))
    }
}
