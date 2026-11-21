//! "Add instance" / "Remove instance" (`Plater::increase_instances` / `decrease_instances`,
//! `Plater.cpp:19289-19364` at v2.5.0-beta `345efec64`). See
//! `docs/architecture/web-plater-instance-copies.md`.

use super::edits::InstanceChange;
use super::session::Model;
use super::{PlaterSession, transformation};
use crate::{SliceError, Transform3d};

fn refused(what: &str) -> SliceError {
    SliceError::InvalidInput(what.to_owned())
}

/// `GLCanvas3D::get_size_proportional_to_max_bed_size(0.05)` (`GLCanvas3D.cpp:6195-6199`): 0.05 ×
/// the larger side of the build volume's 2D bounding box.
fn offset_base(session: &PlaterSession) -> f64 {
    let area = &session
        .project
        .settings()
        .printer
        .remaining
        .printable_area
        .0;
    let (mut min, mut max) = ([f64::MAX; 2], [f64::MIN; 2]);
    for point in area {
        min = [min[0].min(point.x), min[1].min(point.y)];
        max = [max[0].max(point.x), max[1].max(point.y)];
    }
    0.05 * (max[0] - min[0]).max(max[1] - min[1])
}

impl PlaterSession {
    /// `increase_instances(num)` on the object at `object`: `num` copies of its last instance's
    /// offset (moved by `(i·o, i·o, 0)`), scaling factor, rotation and mirror
    /// (`ModelObject::add_instance(offset, scaling, rotation, mirror)`, `Model.cpp:1573-1581`), each
    /// registered with the plate it lands on. `InvalidInput` where `can_increase_instances` is
    /// false (`Plater.cpp:15097-15107`): no such object, or a non-printable instance.
    pub fn increase_instances(&mut self, object: usize, num: usize) -> Result<(), SliceError> {
        let item = self
            .project
            .objects()
            .get(object)
            .ok_or_else(|| refused("there is no such object"))?;
        if !Self::can_increase_instances(item) {
            return Err(refused(
                "an object with a non-printable instance cannot get more instances",
            ));
        }
        let last = item.instances().last().expect("an instance").transform();
        let (last_offset, scaling, rotation, mirror) = (
            transformation::get_offset(last),
            transformation::scaling_factor(last),
            transformation::rotation(last),
            transformation::mirror(last),
        );
        let base = offset_base(self);
        let mut offset = base;
        for _ in 0..num {
            let position = [
                last_offset[0] + offset,
                last_offset[1] + offset,
                last_offset[2],
            ];
            // `add_instance()` then `set_offset`, `set_scaling_factor`, `set_rotation`, `set_mirror`.
            let mut transform = transformation::with_offset(Transform3d::IDENTITY, position);
            transform = transformation::with_scaling_factor(transform, scaling);
            transform = transformation::with_rotation(transform, rotation);
            transform = transformation::with_mirror(transform, mirror);
            let ids = [self.next_instance_id(object), self.next_identify_id()];
            self.project.add_instance(object, transform, ids);
            let key = (object, self.project.objects()[object].instances().len() - 1);
            self.plates
                .notify_instance_update(&Model(&mut self.project), key);
            offset += base;
        }
        Ok(())
    }

    /// `decrease_instances(num)` on the object at `object`: its last `num` instances removed, or,
    /// when `num` reaches the count, the object itself (`Plater::priv::remove`,
    /// `Plater.cpp:10727-10747, 19351-19353`, the "Delete" changes). `InvalidInput` where
    /// `can_decrease_instances` is false (`:15109-15118`): no such object or a single instance.
    pub fn decrease_instances(&mut self, object: usize, num: usize) -> Result<(), SliceError> {
        let item = self
            .project
            .objects()
            .get(object)
            .ok_or_else(|| refused("there is no such object"))?;
        let count = item.instances().len();
        if count <= 1 {
            return Err(refused("an object with one instance cannot lose it"));
        }
        if count <= num {
            // Upstream then reads the freed object to select an instance (`:19354-19355`), undefined
            // behaviour in UI selection code, not ported.
            return self.delete_object(object);
        }
        for _ in 0..num {
            let last = self.project.objects()[object].instances().len() - 1;
            self.plates.notify_instance_removed((object, last));
            self.project.remove_last_instance(object);
        }
        Ok(())
    }

    /// `Plater::set_number_of_copies` (`Plater.cpp:19386-19406`) answered with `num`: copies added
    /// or removed to reach it, a disabled `increase_instances` / `decrease_instances` returning
    /// silently as upstream's do. `InvalidInput` for no such object or a count outside the
    /// dialog's 0..1000 (`:19394-19395`).
    pub fn set_number_of_copies(&mut self, object: usize, num: usize) -> Result<(), SliceError> {
        let item = self
            .project
            .objects()
            .get(object)
            .ok_or_else(|| refused("there is no such object"))?;
        if num > 1000 {
            return Err(refused("the number of copies is 0 to 1000"));
        }
        let count = item.instances().len();
        match num.cmp(&count) {
            std::cmp::Ordering::Greater if Self::can_increase_instances(item) => {
                self.increase_instances(object, num - count)
            }
            // `can_decrease_instances`: more than one instance.
            std::cmp::Ordering::Less if count > 1 => self.decrease_instances(object, count - num),
            _ => Ok(()),
        }
    }

    /// `can_increase_instances` (`Plater.cpp:15097-15107`): every instance printable.
    fn can_increase_instances(item: &crate::ProjectObject) -> bool {
        item.instances().iter().all(|instance| instance.printable())
    }

    /// `Transformation::get_rotation`, `get_scaling_factor` and `get_mirror` of an instance
    /// (`Geometry.cpp:471-474, 513-517, 550-554`).
    pub fn transformation_components(
        &self,
        object: usize,
        instance: usize,
    ) -> Result<[[f64; 3]; 3], SliceError> {
        self.check(object, instance)?;
        let transform = self.project.objects()[object].instances()[instance].transform();
        Ok([
            transformation::rotation(transform),
            transformation::scaling_factor(transform),
            transformation::mirror(transform),
        ])
    }

    /// A copy's export identity, standing in for upstream's run-time `ObjectBase` id
    /// (`bbs_3mf.cpp:8582-8605`): one past the largest identify id of the project.
    fn next_identify_id(&self) -> u32 {
        self.project
            .objects()
            .iter()
            .flat_map(|object| object.instances())
            .filter_map(|instance| instance.saved_identify_id())
            .max()
            .map_or(1, |id| id + 1)
    }

    /// A copy's instance id: one past every instance id the object had at load or has now, so a
    /// copy never takes a loaded instance's identity.
    fn next_instance_id(&self, object: usize) -> u32 {
        let item = &self.project.objects()[object];
        self.loaded_instances
            .get(&item.id())
            .into_iter()
            .flatten()
            .copied()
            .chain(
                item.instances()
                    .iter()
                    .map(|instance| instance.instance_id()),
            )
            .max()
            .map_or(0, |id| id + 1)
    }

    /// The objects whose instances differ from the loaded ones.
    pub(super) fn instance_changes(&self) -> Vec<InstanceChange> {
        self.project
            .objects()
            .iter()
            .filter_map(|object| {
                let loaded = self.loaded_instances.get(&object.id())?;
                let (kept, copies): (Vec<_>, Vec<_>) = object
                    .instances()
                    .iter()
                    .partition(|instance| loaded.contains(&instance.instance_id()));
                let kept: Vec<u32> = kept.iter().map(|instance| instance.instance_id()).collect();
                let copies: Vec<([u32; 2], Transform3d)> = copies
                    .iter()
                    .map(|copy| {
                        (
                            [
                                copy.instance_id(),
                                copy.loaded_label_id().expect("a copy's identify id"),
                            ],
                            copy.transform(),
                        )
                    })
                    .collect();
                (kept != *loaded || !copies.is_empty()).then_some(InstanceChange {
                    object_id: object.id(),
                    kept,
                    copies,
                })
            })
            .collect()
    }
}
