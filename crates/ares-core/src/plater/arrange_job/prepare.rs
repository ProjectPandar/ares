//! `ArrangeJob::prepare_all` and `prepare_partplate` (`ArrangeJob.cpp:342-414, 532-599` at
//! v2.5.0-beta `345efec64`): which instances each scope arranges, locks or sets aside.

use super::{polygon, unsupported};
use crate::arrangement::ArrangePolygon;
use crate::options::ProcessPrintSequence;
use crate::plater::PlaterSession;
use crate::plater::plate_list::{InstanceKey, PlateList};
use crate::plater::selection::instance_bounding_box;
use crate::{Project, SliceError};

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

/// `prepare_all` locks plates whose print sequence differs from the global one
/// (`ArrangeJob.cpp:345-354`); plate locks are not ported.
pub(super) fn check_plate_sequences(
    project: &Project,
    plates: &PlateList,
) -> Result<(), SliceError> {
    let global = project.settings().process.print.print_sequence;
    if (0..plates.plates.len())
        .any(|index| plate_sequence(project, plates, index).is_some_and(|plate| plate != global))
    {
        Err(unsupported("plates with their own print sequence"))
    } else {
        Ok(())
    }
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

    /// `prepare_all` without locked plates: the printable instances are selected.
    pub(super) fn prepare_all(&self) -> Prepared {
        let mut prepared = Prepared::default();
        for (key, printable) in self.printable_keys() {
            let list = if printable {
                &mut prepared.selected
            } else {
                &mut prepared.unprintable
            };
            push_numbered(
                list,
                key,
                polygon::instance_arrange_polygon(&self.project, key),
            );
        }
        prepared
    }

    /// `prepare_partplate`: the instances on or reaching plate `current` are selected, the
    /// others locked; an empty plate arranges nothing.
    pub(super) fn prepare_partplate(&self, current: usize) -> Prepared {
        let mut prepared = Prepared::default();
        if self.plates.plates[current].instances.is_empty() {
            return prepared;
        }
        for (key, printable) in self.printable_keys() {
            let bbox = instance_bounding_box(&self.project, key);
            let in_plate = self.plates.holds_or_reaches(current, key, &bbox);
            let mut ap = polygon::instance_arrange_polygon(&self.project, key);
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
