//! The GUI arrange job (`src/slic3r/GUI/Jobs/ArrangeJob.cpp` at v2.5.0-beta `345efec64`) on the
//! plater session: `prepare`, `check_unprintable`, `process` and `finalize`
//! (`:342-414, 532-599, 632-965`) over the plate list, running `arrangement::arrange`, with the
//! prime tower, the IDEX/IQEX zones, locked plates, the nozzles' shared bed and the first-layer-scan
//! calibration region. Per-object skirts are not ported yet; projects needing them are rejected.

mod finalize;
mod imex;
mod nozzle_ids;
mod object_skirt;
mod params;
pub(super) use params::shared_poly;
mod polygon;
mod prepare;
mod wipe_tower;

pub(super) use wipe_tower::{TowerPositions, check_positions_for_new_plate};

pub(super) use polygon::convex_hull_2d;

use std::collections::{BTreeMap, BTreeSet};

use super::PlaterSession;
use super::plate_list::PlateList;
use crate::SliceError;
use crate::arrangement::imex::ImexArranger;
use crate::arrangement::{
    ArrangeParams, ArrangePolygon, Bed, SCALING_FACTOR, arrange, get_shrink_bedpts,
    update_arrange_params, update_selected_items_axis_align, update_selected_items_inflation,
    update_unselected_items_inflation,
};
use crate::geometry::fixed_gcc_sort_by;
use crate::options::{ProcessTimelapseType, ProjectSettings};
use crate::profiles::VendorType;
use crate::project_slice::gcode_emit::wipe_tower_estimate::config::DynamicPrintConfig;

/// Which instances an arrange moves (`Job::JobPrepareState`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArrangeScope {
    /// `PREPARE_STATE_DEFAULT`: every instance on an unlocked plate (`prepare_all`).
    All,
    /// `PREPARE_STATE_MENU`: the instances on the plate at this 0-based index, the others
    /// left where they are (`prepare_partplate`).
    CurrentPlate(usize),
}

/// `GLCanvas3D::ArrangeSettings` (`GLCanvas3D.hpp:603-614`) of the FFF settings set in use;
/// `is_seq_print` follows the global `print_sequence` (`GLCanvas3D.cpp:1563-1580`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ArrangeSettings {
    /// mm; 0 spaces the items automatically.
    pub distance: f32,
    pub enable_rotation: bool,
    pub allow_multi_materials_on_same_plate: bool,
    pub avoid_extrusion_cali_region: bool,
    pub align_to_y_axis: bool,
}

/// `prepare_partplate`'s warning for a locked plate (`ArrangeJob.cpp:548-552`).
const LOCKED_PLATE: &str = "This plate is locked.\nCannot auto-arrange on this plate.";

/// `MAX_NUM_PLATES` the job reserves exclusion areas on (`ArrangeJob.cpp:410`).
const MAX_NUM_PLATES: i32 = 36;

/// `prepare_wipe_tower`'s `need_wipe_tower` (`ArrangeJob.cpp:442-489`).
fn needs_wipe_tower(
    settings: &ProjectSettings,
    params: &ArrangeParams,
    selected: &[ArrangePolygon],
) -> bool {
    let print = &settings.process.print;
    if !print.enable_prime_tower.0 || params.is_seq_print {
        return false;
    }
    if print.timelapse_type == ProcessTimelapseType::Smooth {
        return true;
    }
    if selected
        .iter()
        .any(|item| item.extrude_ids.iter().collect::<BTreeSet<_>>().len() > 1)
    {
        return true;
    }
    if !params.allow_multi_materials_on_same_plate {
        return false;
    }
    let mut by_bed_temperature = BTreeMap::<i32, BTreeSet<i32>>::new();
    for item in selected {
        by_bed_temperature
            .entry(item.bed_temp)
            .or_default()
            .extend(&item.extrude_ids);
    }
    by_bed_temperature.values().any(|ids| ids.len() > 1)
}

/// The exclusion areas of `preprocess_exclude_areas` as fixed items on `num_plates` beds.
fn exclusion_items(
    settings: &ProjectSettings,
    full: &DynamicPrintConfig,
    num_plates: i32,
    inflation: i64,
) -> Vec<ArrangePolygon> {
    let to_pairs =
        |points: &[crate::options::Point2d]| points.iter().map(|p| [p.x, p.y]).collect::<Vec<_>>();
    let wrapping = settings
        .process
        .gcode
        .enable_wrapping_detection
        .0
        .then(|| to_pairs(&settings.printer.gcode.wrapping_exclude_area.0));
    let excluded = to_pairs(&settings.printer.remaining.bed_exclude_area.0);
    PlateList::exclude_areas(wrapping.as_deref(), &excluded, full, num_plates, inflation)
}

impl PlaterSession {
    /// Arranges the `scope` instances (`ArrangeJob::prepare`, `process` and `finalize`,
    /// `ArrangeJob.cpp:632-965`).
    pub fn arrange(
        &mut self,
        scope: ArrangeScope,
        settings: ArrangeSettings,
    ) -> Result<(), SliceError> {
        let current = match scope {
            ArrangeScope::All => None,
            ArrangeScope::CurrentPlate(plate) if plate < self.plates.plates.len() => {
                // The plate's arrange icon selects its plate first.
                self.current_plate = plate;
                // `prepare_partplate` (`ArrangeJob.cpp:540-552`): an empty plate arranges
                // nothing, a locked one warns and arranges nothing. The warning notification is
                // returned as the error.
                let target = &self.plates.plates[plate];
                if !target.instances.is_empty() && target.locked {
                    return Err(SliceError::InvalidInput(LOCKED_PLATE.to_owned()));
                }
                Some(plate)
            }
            ArrangeScope::CurrentPlate(plate) => {
                return Err(SliceError::InvalidInput(format!(
                    "there is no plate {plate}"
                )));
            }
        };
        let project_settings = self.project.settings();
        let plate_sequence =
            current.map(|plate| prepare::real_print_sequence(&self.project, &self.plates, plate));
        let mut params = params::arrange_params(
            project_settings,
            settings,
            self.current_print_skirt_offset()?,
            plate_sequence,
        );
        let config = params::arrange_config(project_settings)?;
        let full = DynamicPrintConfig::new(project_settings)?;
        let zones = self.plates.imex_zones(&self.project, &full);
        let (mut prepared, uncompatible) = match current {
            None => {
                let uncompatible =
                    prepare::lock_uncompatible_plates(&self.project, &mut self.plates);
                (self.prepare_all(&full), uncompatible)
            }
            Some(plate) => (self.prepare_partplate(plate, &full), Vec::new()),
        };
        let aps: Vec<ArrangePolygon> = prepared.selected.iter().map(|(_, ap)| ap.clone()).collect();
        let mut positions = self.tower_positions();
        let shown = self.shown_towers((&full, &zones), &mut positions)?;
        let mut unselected = match current {
            None => {
                self.prepare_wipe_tower((&full, &zones), (&positions, &shown), &params, &aps)?
            }
            Some(plate) => shown[plate]
                .iter()
                .map(|tower| tower.arrange_polygon())
                .collect(),
        };
        let beds = current.map_or(MAX_NUM_PLATES, |plate| plate as i32 + 1);
        unselected.extend(exclusion_items(project_settings, &full, beds, 0));
        // check_unprintable (`ArrangeJob.cpp:697-720`)
        let printable_height = f64::from(params.printable_height);
        let (kept, too_small_or_tall): (Vec<_>, Vec<_>) = prepared
            .selected
            .into_iter()
            .partition(|(_, ap)| !(ap.poly.area() < 0.001 || ap.height > printable_height));
        prepared.unprintable.extend(too_small_or_tall);
        // process (`ArrangeJob.cpp:722-805`)
        let (keys, mut items): (Vec<_>, Vec<_>) = kept.into_iter().unzip();
        // The extrusion and hand-eye calibration region of a first-layer-scanning Bambu Lab
        // printer (`ArrangeJob.cpp:732-734`, `is_bbl_vendor`, `PresetBundle.hpp:374`).
        if self.vendor_type == VendorType::MarlinBbl
            && params.avoid_extrusion_cali_region
            && full.option("scan_first_layer").bool()
        {
            unselected.extend(PlateList::nonprefered_areas(MAX_NUM_PLATES));
        }
        update_arrange_params(&mut params, &config, &items);
        update_selected_items_inflation(&mut items, &config, &mut params);
        update_unselected_items_inflation(&mut unselected, &config, &params);
        update_selected_items_axis_align(&mut items, &config, &params);
        // Keep the parts on IDEX/IQEX plates in their primary zones (`ArrangeJob.cpp:743-746`).
        let imex = ImexArranger::new(
            &imex::prepare_imex_zones(&self.plates, &zones, current, &full),
            &params,
            &get_shrink_bedpts(&config, &params),
        );
        let bed = Bed::from_points(&imex.bed_shape());
        imex.add_keep_outs(&mut unselected);
        params.excluded_regions =
            exclusion_items(project_settings, &full, 1, (1.0 / SCALING_FACTOR) as i64);
        let before = if imex.active() {
            items.clone()
        } else {
            Vec::new()
        };
        arrange(&mut items, &unselected, &bed, &params);
        imex.finish(&mut items, &before, &unselected, &params);
        prepared.selected = keys.into_iter().zip(items).collect();
        fixed_gcc_sort_by(&mut prepared.selected, |a, b| a.1.itemid < b.1.itemid);
        // `finalize` would create a plate on an emptied position list: refused before it changes
        // anything, with the print-sequence locks released.
        if current.is_none()
            && prepared
                .selected
                .iter()
                .any(|(_, ap)| self.plates.needs_new_plate(ap))
            && let Err(error) = check_positions_for_new_plate(&positions)
        {
            for &plate in &uncompatible {
                self.plates.plates[plate].locked = false;
            }
            return Err(error);
        }
        let plates_before = self.plates.plates.len();
        self.finalize(prepared, current, &uncompatible);
        self.place_new_towers(&full, &mut positions, plates_before)?;
        self.wipe_tower = Some(positions);
        Ok(())
    }
}
