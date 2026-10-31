//! `ArrangeJob::finalize` (`ArrangeJob.cpp:819-965` at v2.5.0-beta `345efec64`): the arranged,
//! locked and unprintable items applied to their instances, and the plates rebuilt.

use super::prepare::Prepared;
use crate::arrangement::{ArrangePolygon, SCALING_FACTOR};
use crate::geometry::fixed_gcc_sort_by;
use crate::plater::PlaterSession;
use crate::plater::plate_list::InstanceKey;
use crate::plater::session::Model;
use crate::plater::transformation;

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

impl PlaterSession {
    /// `finalize` for "Arrange all" (`current` `None`) or the plate at `current`
    /// (`only_on_partplate`). Ares has no non-virtual unselected items: no plate is locked.
    pub(super) fn finalize(&mut self, prepared: Prepared, current: Option<usize>) {
        let Prepared {
            mut selected,
            mut unprintable,
            mut locked,
        } = prepared;
        if !selected.is_empty() && selected.iter().all(|(_, ap)| !ap.is_arranged()) {
            return;
        }
        self.plates.clear_instances();
        let mut beds = 0;
        for (_, ap) in &mut selected {
            match current {
                Some(plate) => self.plates.bed_index_for_current_plate(ap, plate),
                None => self.plates.create_plates_for(ap),
            }
            beds = beds.max(ap.bed_idx);
        }
        for (key, ap) in &mut locked {
            beds = beds.max(ap.bed_idx);
            self.plates.move_to_plate(ap, false);
            apply(self, *key, ap);
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
        self.rebuild_plates_after_arrangement();
    }

    /// `rebuild_plates_after_arrangement(false, true, plate_index)` (`PartPlate.cpp:8382-8422`):
    /// the objects sorted by their first instance's arrange order, then every plate reloaded.
    fn rebuild_plates_after_arrangement(&mut self) {
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
