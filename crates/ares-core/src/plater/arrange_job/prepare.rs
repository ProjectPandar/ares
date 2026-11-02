//! `ArrangeJob::prepare_all` and `prepare_partplate` (`ArrangeJob.cpp:342-414, 532-599` at
//! v2.5.0-beta `345efec64`): which instances each scope arranges, locks or sets aside.

use super::polygon;
use crate::Project;
use crate::arrangement::ArrangePolygon;
use crate::options::ProcessPrintSequence;
use crate::plater::PlaterSession;
use crate::plater::plate_list::{InstanceKey, PlateList};
use crate::plater::selection::instance_bounding_box;
use crate::project_slice::gcode_emit::wipe_tower_estimate::config::DynamicPrintConfig;

type Items = Vec<(InstanceKey, ArrangePolygon)>;

/// The job's `m_selected`, `m_unprintable` and `m_locked`, each item with its instance.
#[derive(Default)]
pub(super) struct Prepared {
    pub(super) selected: Items,
    pub(super) unprintable: Items,
    pub(super) locked: Items,
}

/// `PartPlate::get_print_seq`: the print sequence the plate's document sets, if any.
pub(super) fn plate_sequence(
    project: &Project,
    plates: &PlateList,
    index: usize,
) -> Option<ProcessPrintSequence> {
    let source = plates.plates[index].source?;
    let plate = project.documents().model_settings.plates.get(source)?;
    plate
        .metadata
        .iter()
        .find(|item| item.key == "print_sequence")
        .and_then(|item| match item.value.as_str() {
            "by layer" => Some(ProcessPrintSequence::ByLayer),
            "by object" => Some(ProcessPrintSequence::ByObject),
            _ => None,
        })
}

/// `PartPlate::get_real_print_seq` (`PartPlate.cpp:370-382`): the plate's print sequence, by
/// default the global one.
pub(super) fn real_print_sequence(
    project: &Project,
    plates: &PlateList,
    index: usize,
) -> ProcessPrintSequence {
    plate_sequence(project, plates, index)
        .unwrap_or(project.settings().process.print.print_sequence)
}

/// `prepare_all`'s locks (`ArrangeJob.cpp:346-354`): each unlocked plate whose print sequence
/// is not the global one is locked for the arrange. `finalize` unlocks the returned plates.
pub(super) fn lock_uncompatible_plates(project: &Project, plates: &mut PlateList) -> Vec<usize> {
    let global = project.settings().process.print.print_sequence;
    let uncompatible: Vec<usize> = (0..plates.plates.len())
        .filter(|&index| {
            !plates.plates[index].locked
                && plate_sequence(project, plates, index).is_some_and(|plate| plate != global)
        })
        .collect();
    for &index in &uncompatible {
        plates.plates[index].locked = true;
    }
    uncompatible
}

/// Appends `ap` numbered by its place in `list` (`ap.itemid = cont.size()`).
fn push_numbered(list: &mut Items, key: InstanceKey, mut ap: ArrangePolygon) {
    ap.itemid = list.len() as i32;
    list.push((key, ap));
}

impl PlaterSession {
    /// Every instance with its printable flag, in `Model::objects` / `instances` order.
    fn printable_keys(&self) -> Vec<(InstanceKey, bool)> {
        self.project
            .objects()
            .iter()
            .enumerate()
            .flat_map(|(object, item)| {
                item.instances()
                    .iter()
                    .enumerate()
                    .map(move |(instance, placed)| ((object, instance), placed.printable()))
            })
            .collect()
    }

    /// `prepare_all`: the instances on locked plates are locked, the other printable ones
    /// selected.
    pub(super) fn prepare_all(&self, config: &DynamicPrintConfig) -> Prepared {
        let mut prepared = Prepared::default();
        for (key, printable) in self.printable_keys() {
            let mut ap = polygon::instance_arrange_polygon(&self.project, key);
            // `ArrangeJob.cpp:368-370`.
            self.set_bed_exclusion_extruders_for_all_plates(&mut ap, key, config);
            let list = if self.plates.lock_on_locked_plate(key, &mut ap) {
                &mut prepared.locked
            } else if printable {
                &mut prepared.selected
            } else {
                &mut prepared.unprintable
            };
            push_numbered(list, key, ap);
        }
        prepared
    }

    /// `prepare_partplate`: the instances on or reaching plate `current` are selected, the
    /// others locked; an empty plate arranges nothing.
    pub(super) fn prepare_partplate(
        &self,
        current: usize,
        config: &DynamicPrintConfig,
    ) -> Prepared {
        let mut prepared = Prepared::default();
        if self.plates.plates[current].instances.is_empty() {
            return prepared;
        }
        for (key, printable) in self.printable_keys() {
            let bbox = instance_bounding_box(&self.project, key);
            let in_plate = self.plates.holds_or_reaches(current, key, &bbox);
            let mut ap = polygon::instance_arrange_polygon(&self.project, key);
            // `prepare_arrange_polygon` with the mapping plate (`ArrangeJob.cpp:249-257, 565`).
            self.set_bed_exclusion_extruders(&mut ap, (key, current), config);
            let list = if self.plates.lock_off_current_plate(key, &mut ap, in_plate) {
                &mut prepared.locked
            } else if printable {
                &mut prepared.selected
            } else {
                &mut prepared.unprintable
            };
            push_numbered(list, key, ap);
        }
        prepared
    }
}
