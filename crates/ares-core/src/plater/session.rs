//! An editable plater: the loaded project, its plate list and the GUI
//! edits on them. Slicing reads the edits (`PlaterSession::edits`).

use std::collections::{BTreeMap, BTreeSet};

use super::edits::{EditedPlate, PlaterEdits};
use super::plate_list::{BoundingBox, InstanceKey, PlateList, PlateModel};
use super::{eigen, selection};
use crate::project_slice::gcode_emit::wipe_tower_estimate::config::DynamicPrintConfig;
use crate::{Project, SliceError, Transform3d};

pub struct PlaterSession {
    input: Vec<u8>,
    pub(super) project: Project,
    pub(super) plates: PlateList,
    /// `ModelInstance::arrange_order` by `(object_id, instance_id)`; 0 until an arrange.
    pub(super) arrange_orders: BTreeMap<(u32, u32), i32>,
    /// The project config's tower positions (`wipe_tower_x`, `wipe_tower_y`) once an edit set them.
    pub(super) wipe_tower: Option<super::arrange_job::TowerPositions>,
    /// `get_current_vendor_type` over the installed vendors (`set_installed_vendors`).
    pub(super) vendor_type: crate::profiles::VendorType,
    /// `PartPlateList::m_current_plate`.
    pub(super) current_plate: usize,
}

/// One plate of the session: its grid origin and instances.
#[derive(Clone, Debug, PartialEq)]
pub struct PlaterPlate {
    pub origin: [f64; 2],
    /// `(object index, instance index)` pairs.
    pub instances: Vec<(usize, usize)>,
    /// The loaded plate (0-based) this plate is, `None` for an added plate.
    pub source: Option<usize>,
}

pub(super) struct Model<'a>(pub(super) &'a mut Project);

impl PlateModel for Model<'_> {
    fn keys(&self) -> Vec<InstanceKey> {
        keys(self.0)
    }
    fn convex_hull_bounding_box(&self, key: InstanceKey) -> BoundingBox {
        selection::instance_bounding_box(self.0, key)
    }
    fn translate(&mut self, key: InstanceKey, offset: [f64; 2]) {
        let mut rows = *self.0.objects()[key.0].instances()[key.1]
            .transform()
            .as_matrix();
        rows[0][3] += offset[0];
        rows[1][3] += offset[1];
        self.0
            .set_instance_transform(key.0, key.1, Transform3d::from_rows(rows));
    }
}

pub(super) fn keys(project: &Project) -> Vec<InstanceKey> {
    project
        .objects()
        .iter()
        .enumerate()
        .flat_map(|(object, item)| {
            (0..item.instances().len()).map(move |instance| (object, instance))
        })
        .collect()
}

impl PlaterSession {
    /// Loads a 3MF project with its plates (`PartPlateList::load_from_3mf_structure`).
    pub fn load(input: Vec<u8>) -> Result<Self, SliceError> {
        let project = crate::load_project(&input)?;
        Ok(Self::with_project(input, project))
    }

    /// Loads a model with selected presets onto an empty plate, centered on the bed and dropped
    /// when `request.placement` asks (`Plater::priv::load_model_objects`); the composed
    /// `printable_area` sizes the plates.
    pub async fn load_with_profiles(
        input: Vec<u8>,
        request: crate::ProfileSliceRequest<'_>,
    ) -> Result<Self, SliceError> {
        let project = crate::profile_placed_project(&input, request).await?;
        Ok(Self::with_project(input, project))
    }

    fn with_project(input: Vec<u8>, project: Project) -> Self {
        let plates = project
            .plates()
            .iter()
            .map(|plate| {
                plate
                    .instances()
                    .iter()
                    .filter_map(|identity| find_instance(&project, identity[0], identity[1]))
                    .collect::<BTreeSet<_>>()
            })
            .collect::<Vec<_>>();
        let settings = project.settings();
        let area = &settings.printer.remaining.printable_area.0;
        let mut shape = ([f64::MAX; 2], [f64::MIN; 2]);
        for point in area {
            shape.0 = [shape.0[0].min(point.x), shape.0[1].min(point.y)];
            shape.1 = [shape.1[0].max(point.x), shape.1[1].max(point.y)];
        }
        let size = crate::project::plate_dimensions(settings);
        let height = settings.printer.remaining.printable_height.0 as i32;
        let mut plates = PlateList::new(plates, size, height, shape);
        // `load_from_3mf_structure` (`PartPlate.cpp:8559`).
        for (index, plate) in plates.plates.iter_mut().enumerate() {
            plate.locked = project.plate_locked(index);
        }
        Self {
            input,
            project,
            plates,
            arrange_orders: BTreeMap::new(),
            wipe_tower: None,
            vendor_type: crate::profiles::VendorType::Unknown,
            // The plater selects plate 0 after loading (`Plater.cpp:10222, 10229`).
            current_plate: 0,
        }
    }

    pub fn input(&self) -> &[u8] {
        &self.input
    }

    pub fn project(&self) -> &Project {
        &self.project
    }

    pub fn instance_transform(
        &self,
        object: usize,
        instance: usize,
    ) -> Result<Transform3d, SliceError> {
        self.check(object, instance)?;
        Ok(self.project.objects()[object].instances()[instance].transform())
    }

    pub fn plates(&self) -> Vec<PlaterPlate> {
        self.plates
            .plates
            .iter()
            .enumerate()
            .map(|(index, plate)| PlaterPlate {
                origin: self.plates.plate_origin(index),
                instances: plate.instances.iter().copied().collect(),
                source: plate.source,
            })
            .collect()
    }

    /// Instances outside every plate (`PartPlateList::unprintable_plate`).
    pub fn unprintable_instances(&self) -> Vec<(usize, usize)> {
        self.plates.unprintable.instances.iter().copied().collect()
    }

    /// A move by `displacement` in world coordinates: `Selection::translate`
    /// then `GLCanvas3D::do_move` (`GLCanvas3D.cpp:4910-5001`).
    pub fn translate_instance(
        &mut self,
        object: usize,
        instance: usize,
        displacement: [f64; 3],
    ) -> Result<(), SliceError> {
        self.check(object, instance)?;
        let key = (object, instance);
        selection::translate(&mut self.project, key, displacement);
        self.plates
            .notify_instance_update(&Model(&mut self.project), key);
        if selection::drop_after_move(&mut self.project, key) {
            self.plates
                .notify_instance_update(&Model(&mut self.project), key);
        }
        Ok(())
    }

    /// The "Position" field (`GizmoObjectManipulation::change_position_value`,
    /// `GizmoObjectManipulation.cpp:309-341`): a move by the difference to the
    /// instance offset on `axis`.
    pub fn set_instance_position(
        &mut self,
        object: usize,
        instance: usize,
        axis: usize,
        value: f64,
    ) -> Result<(), SliceError> {
        let axis = check_axis(axis)?;
        let offset = self.instance_transform(object, instance)?.as_matrix()[axis][3];
        let mut displacement = [0.0; 3];
        displacement[axis] = value - offset;
        self.translate_instance(object, instance, displacement)
    }

    /// A rotation by `rotation` (radians, `Geometry::rotation_transform` order)
    /// about the world axes: `Selection::rotate` then `GLCanvas3D::do_rotate`
    /// (`GLCanvas3D.cpp:5026-5110`). The gizmo rings and the relative rotation
    /// fields both rotate this way.
    pub fn rotate_instance(
        &mut self,
        object: usize,
        instance: usize,
        rotation: [f64; 3],
    ) -> Result<(), SliceError> {
        self.check(object, instance)?;
        let key = (object, instance);
        let min_z_before = selection::instance_bounding_box(&self.project, key).0[2];
        selection::rotate(&mut self.project, key, rotation);
        self.plates
            .notify_instance_update(&Model(&mut self.project), key);
        if selection::drop_after_transform(&mut self.project, key, min_z_before) {
            self.plates
                .notify_instance_update(&Model(&mut self.project), key);
        }
        Ok(())
    }

    /// The absolute rotation fields' values in degrees
    /// (`update_settings_value`, `GizmoObjectManipulation.cpp:120-122`).
    pub fn absolute_rotation(
        &self,
        object: usize,
        instance: usize,
    ) -> Result<[f64; 3], SliceError> {
        let transform = self.instance_transform(object, instance)?;
        Ok(eigen::rotation_by_quaternion(transform).map(|radians| {
            let degrees = radians * (180.0 / std::f64::consts::PI);
            if degrees.abs() < 0.001 { 0.0 } else { degrees }
        }))
    }

    /// An absolute rotation field edit (`change_absolute_rotation_value`,
    /// `GizmoObjectManipulation.cpp:370-398`): a world rotation by the change.
    pub fn set_absolute_rotation(
        &mut self,
        object: usize,
        instance: usize,
        axis: usize,
        degrees: f64,
    ) -> Result<(), SliceError> {
        let current = self.absolute_rotation(object, instance)?;
        let mut change = [0.0; 3];
        let axis = check_axis(axis)?;
        change[axis] = degrees - current[axis];
        self.rotate_instance(
            object,
            instance,
            change.map(|value| (std::f64::consts::PI / 180.0) * value),
        )
    }

    /// "Add plate" (`PartPlateList::create_plate`). In GUI mode the new plate takes its default
    /// tower position (`set_default_wipe_tower_pos_for_plate(new_index, true)`,
    /// `PartPlate.cpp:6644-6646`).
    pub fn add_plate(&mut self) -> Result<(), SliceError> {
        let new_index = self.plates.plates.len();
        self.plates.create_plate(&mut Model(&mut self.project));
        let full = DynamicPrintConfig::new(self.project.settings())?;
        let mut positions = self.tower_positions();
        self.place_new_towers(&full, &mut positions, new_index)?;
        self.wipe_tower = Some(positions);
        Ok(())
    }

    /// "Delete plate" (`PartPlateList::delete_plate`); a sole plate stays. In GUI mode the plate's
    /// tower position is erased where the vectors reach it (`PartPlate.cpp:6766-6781`).
    pub fn delete_plate(&mut self, index: usize) -> bool {
        if !self
            .plates
            .delete_plate(index, &mut Model(&mut self.project))
        {
            return false;
        }
        let mut positions = self.tower_positions();
        for values in [&mut positions.0, &mut positions.1] {
            if index < values.len() {
                values.remove(index);
            }
        }
        self.wipe_tower = Some(positions);
        // `PartPlate.cpp:6809-6815`: the current plate moves back past a deleted one at or
        // before it.
        if !(self.current_plate == index && index == 0) && self.current_plate >= index {
            self.current_plate -= 1;
        }
        true
    }

    /// The project config's `wipe_tower_x`/`wipe_tower_y`, one entry per plate as the GUI's
    /// project export writes them: the loaded ones until an edit sets them.
    pub fn tower_positions(&self) -> (Vec<f64>, Vec<f64>) {
        self.wipe_tower.clone().unwrap_or_else(|| {
            let print = &self.project.settings().project.print;
            let values =
                |floats: &crate::OrcaFloats| floats.0.iter().map(|value| value.0).collect();
            (values(&print.wipe_tower_x), values(&print.wipe_tower_y))
        })
    }

    /// The edits slicing applies to the loaded input.
    pub fn edits(&self) -> PlaterEdits {
        let mut transforms = Vec::new();
        for object in self.project.objects() {
            for instance in object.instances() {
                transforms.push(((object.id(), instance.instance_id()), instance.transform()));
            }
        }
        let plates = self
            .plates
            .plates
            .iter()
            .map(|plate| EditedPlate {
                source: plate.source,
                instances: plate
                    .instances
                    .iter()
                    .filter_map(|(object, instance)| {
                        let item = &self.project.objects()[*object].instances()[*instance];
                        item.loaded_label_id()
                            .map(|label| [item.object_id(), item.instance_id(), label])
                    })
                    .collect(),
            })
            .collect();
        PlaterEdits {
            transforms,
            plates: Some(plates),
            object_order: Some(
                self.project
                    .objects()
                    .iter()
                    .map(|object| object.id())
                    .collect(),
            ),
            wipe_tower: self.wipe_tower.clone(),
        }
    }

    pub(super) fn check(&self, object: usize, instance: usize) -> Result<(), SliceError> {
        self.project
            .objects()
            .get(object)
            .and_then(|item| item.instances().get(instance))
            .map(|_| ())
            .ok_or_else(|| {
                SliceError::InvalidInput(format!("no instance {instance} of object {object}"))
            })
    }
}

pub(super) fn check_axis(axis: usize) -> Result<usize, SliceError> {
    if axis < 3 {
        Ok(axis)
    } else {
        Err(SliceError::InvalidInput(format!("no axis {axis}")))
    }
}

fn find_instance(project: &Project, object_id: u32, instance_id: u32) -> Option<InstanceKey> {
    project
        .objects()
        .iter()
        .enumerate()
        .find_map(|(object, item)| {
            (item.id() == object_id)
                .then(|| {
                    item.instances()
                        .iter()
                        .position(|instance| instance.instance_id() == instance_id)
                })
                .flatten()
                .map(|instance| (object, instance))
        })
}
