//! The GUI arrange job (`src/slic3r/GUI/Jobs/ArrangeJob.cpp` at v2.5.0-beta `345efec64`) on the
//! plater session: `prepare_all`, `check_unprintable`, `process` and `finalize`
//! (`:342-414, 632-965`) over the plate list, running `arrangement::arrange`. The wipe tower,
//! locked plates, per-object skirts, the BBL calibration strip and several nozzles' shared bed
//! are not ported yet; projects needing them are rejected.

mod params;
mod polygon;

use std::collections::{BTreeMap, BTreeSet};

use super::PlaterSession;
use super::plate_list::{InstanceKey, PlateList};
use super::session::Model;
use super::transformation;
use crate::arrangement::{
    ArrangeParams, ArrangePolygon, Bed, SCALING_FACTOR, arrange, get_shrink_bedpts,
    update_arrange_params, update_selected_items_axis_align, update_selected_items_inflation,
    update_unselected_items_inflation,
};
use crate::geometry::fixed_gcc_sort_by;
use crate::options::{ProcessPrintSequence, ProcessTimelapseType, ProjectSettings};
use crate::project::raw_settings::RawConfigValue;
use crate::{Project, SliceError};

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

/// `MAX_NUM_PLATES` the job reserves exclusion areas on (`ArrangeJob.cpp:410`).
const MAX_NUM_PLATES: i32 = 36;

fn unsupported(what: &str) -> SliceError {
    SliceError::InvalidInput(format!("arranging {what} is not supported yet"))
}

/// `prepare_all` locks plates whose print sequence differs from the global one
/// (`ArrangeJob.cpp:345-354`); plate locks are not ported.
fn check_plate_sequences(project: &Project, plates: &PlateList) -> Result<(), SliceError> {
    let global = project.settings().process.print.print_sequence;
    let documents = &project.documents().model_settings.plates;
    let differs = plates
        .plates
        .iter()
        .filter_map(|plate| plate.source)
        .any(|source| {
            documents.get(source).is_some_and(|plate| {
                plate.metadata.iter().any(|item| {
                    item.key == "print_sequence"
                        && match item.value.as_str() {
                            "by layer" => global != ProcessPrintSequence::ByLayer,
                            "by object" => global != ProcessPrintSequence::ByObject,
                            _ => false,
                        }
                })
            })
        });
    if differs {
        Err(unsupported("plates with their own print sequence"))
    } else {
        Ok(())
    }
}

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

/// `process`'s BBL first-layer-scan calibration strip (`ArrangeJob.cpp:731-734`).
fn needs_calibration_strip(project: &Project, params: &ArrangeParams) -> bool {
    let settings = project.settings();
    let scan = project
        .documents()
        .project_settings_raw
        .get("scan_first_layer")
        .is_some_and(|value| match value {
            RawConfigValue::Text(text) => text == "1",
            RawConfigValue::Flag(flag) => *flag,
            _ => false,
        });
    settings
        .printer
        .remaining
        .printer_model
        .0
        .starts_with("Bambu Lab")
        && params.avoid_extrusion_cali_region
        && scan
}

/// The exclusion areas of `preprocess_exclude_areas` as fixed items on `num_plates` beds.
fn exclusion_items(
    settings: &ProjectSettings,
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
    PlateList::exclude_areas(wrapping.as_deref(), &excluded, num_plates, inflation)
}

/// `apply`: the instance takes the item's Z rotation and position and its arrange order
/// (`ModelInstance::apply_arrange_result`, `Model.hpp:1531-1538`; `ModelArrange.hpp:87-91`).
fn apply(session: &mut PlaterSession, (object, instance): InstanceKey, ap: &ArrangePolygon) {
    if !ap.is_arranged() {
        return;
    }
    let item = &session.project.objects()[object];
    let id = (item.id(), item.instances()[instance].instance_id());
    let transform = item.instances()[instance].transform();
    let rotated = transformation::with_axis_rotation(transform, 2, ap.rotation);
    let z = rotated.as_matrix()[2][3];
    let x = ap.translation.x() as f64 * SCALING_FACTOR;
    let y = ap.translation.y() as f64 * SCALING_FACTOR;
    let moved = transformation::with_offset(rotated, [x, y, z]);
    session
        .project
        .set_instance_transform(object, instance, moved);
    session.arrange_orders.insert(id, ap.itemid);
}

/// Appends `ap` numbered by its place in `list` (`ap.itemid = cont.size()`).
fn push_numbered(
    list: &mut Vec<(InstanceKey, ArrangePolygon)>,
    key: InstanceKey,
    mut ap: ArrangePolygon,
) {
    ap.itemid = list.len() as i32;
    list.push((key, ap));
}

impl PlaterSession {
    /// Arranges the `scope` instances (`ArrangeJob::prepare`, `process` and `finalize`,
    /// `ArrangeJob.cpp:632-965`).
    pub fn arrange(
        &mut self,
        scope: ArrangeScope,
        settings: ArrangeSettings,
    ) -> Result<(), SliceError> {
        if let ArrangeScope::CurrentPlate(_) = scope {
            return Err(unsupported("the current plate on its own"));
        }
        check_plate_sequences(&self.project, &self.plates)?;
        let project_settings = self.project.settings();
        let mut params = params::arrange_params(
            project_settings,
            settings,
            params::object_skirt_offset(project_settings)?,
        );
        let config = params::arrange_config(project_settings)?;
        // prepare_all
        let (mut selected, mut unprintable) = (Vec::new(), Vec::new());
        let keys: Vec<(InstanceKey, bool)> = self
            .project
            .objects()
            .iter()
            .enumerate()
            .flat_map(|(object, item)| {
                item.instances()
                    .iter()
                    .enumerate()
                    .map(move |(instance, placed)| ((object, instance), placed.printable()))
            })
            .collect();
        for (key, printable) in keys {
            let ap = polygon::instance_arrange_polygon(&self.project, key);
            push_numbered(
                if printable {
                    &mut selected
                } else {
                    &mut unprintable
                },
                key,
                ap,
            );
        }
        let aps: Vec<ArrangePolygon> = selected.iter().map(|(_, ap)| ap.clone()).collect();
        if needs_wipe_tower(project_settings, &params, &aps) {
            return Err(unsupported("with a prime tower"));
        }
        if needs_calibration_strip(&self.project, &params) {
            return Err(unsupported("around the first-layer scan region"));
        }
        let mut unselected = exclusion_items(project_settings, MAX_NUM_PLATES, 0);
        // check_unprintable (`ArrangeJob.cpp:697-720`)
        let printable_height = f64::from(params.printable_height);
        let (kept, too_small_or_tall): (Vec<_>, Vec<_>) = selected
            .into_iter()
            .partition(|(_, ap)| !(ap.poly.area() < 0.001 || ap.height > printable_height));
        selected = kept;
        unprintable.extend(too_small_or_tall);
        // process (`ArrangeJob.cpp:722-805`)
        let (keys, mut items): (Vec<_>, Vec<_>) = selected.into_iter().unzip();
        update_arrange_params(&mut params, &config, &items);
        update_selected_items_inflation(&mut items, &config, &mut params);
        update_unselected_items_inflation(&mut unselected, &config, &params);
        update_selected_items_axis_align(&mut items, &config, &params);
        let bed = Bed::from_points(&get_shrink_bedpts(&config, &params));
        params.excluded_regions =
            exclusion_items(project_settings, 1, (1.0 / SCALING_FACTOR) as i64);
        arrange(&mut items, &unselected, &bed, &params);
        let mut selected: Vec<_> = keys.into_iter().zip(items).collect();
        fixed_gcc_sort_by(&mut selected, |a, b| a.1.itemid < b.1.itemid);
        self.finalize(selected, unprintable);
        Ok(())
    }

    /// `finalize` (`ArrangeJob.cpp:819-965`) for "Arrange all".
    fn finalize(
        &mut self,
        mut selected: Vec<(InstanceKey, ArrangePolygon)>,
        mut unprintable: Vec<(InstanceKey, ArrangePolygon)>,
    ) {
        if !selected.is_empty() && selected.iter().all(|(_, ap)| !ap.is_arranged()) {
            return;
        }
        self.plates.clear_instances();
        let mut beds = 0;
        for (_, ap) in &selected {
            self.plates.create_plates_for(ap);
            beds = beds.max(ap.bed_idx);
        }
        for (key, ap) in &mut selected {
            self.plates.move_to_plate(ap, true);
            apply(self, *key, ap);
        }
        for (key, ap) in &mut unprintable {
            ap.bed_idx = beds + 1;
            self.plates.move_to_plate(ap, true);
            apply(self, *key, ap);
        }
        // rebuild_plates_after_arrangement(false, true) (`PartPlate.cpp:8382-8422`)
        let order = |object: &crate::ProjectObject| {
            let first = object.instances()[0].instance_id();
            self.arrange_orders
                .get(&(object.id(), first))
                .copied()
                .unwrap_or(0)
        };
        let mut objects: Vec<(i32, u32)> = self
            .project
            .objects()
            .iter()
            .map(|object| (order(object), object.id()))
            .collect();
        fixed_gcc_sort_by(&mut objects, |a, b| a.0 < b.0);
        let ids: Vec<u32> = objects.into_iter().map(|(_, id)| id).collect();
        self.project.order_objects(&ids);
        self.plates.reload_all_objects(&Model(&mut self.project));
    }
}
