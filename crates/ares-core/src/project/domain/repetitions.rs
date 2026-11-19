use super::{PlateMetadata, Project, ProjectObject, instance::InstanceLabel};
use crate::SliceError;

impl Project {
    /// `PartPlate::duplicate_all_instance` (`PartPlate.cpp:4785–4839`): copy the whole
    /// object once per member of the selected plate and repetition.
    pub(crate) fn repeated_model(
        &self,
        plate: usize,
        additional: usize,
    ) -> Result<Self, SliceError> {
        let mut copy = Self {
            models: self.models.clone(),
            objects: self.objects.clone(),
            plates: self.plates.clone(),
            settings: self.settings.clone(),
            documents: self.documents.clone_shallow(),
            import_scope: self.import_scope,
            plate_view: self.plate_view,
            plate_origin: self.plate_origin,
            plates_custom_gcodes: self.plates_custom_gcodes.clone(),
            curr_plate_index: self.curr_plate_index,
        };
        let mut identities = self.plates[plate].instances().to_vec();
        let mut members: Vec<_> = identities
            .iter()
            .filter_map(|&[object_id, instance_id, _]| {
                let index = self.objects.iter().position(|o| o.id == object_id)?;
                self.objects[index]
                    .instances
                    .iter()
                    .any(|instance| instance.instance_id == instance_id)
                    .then_some((index, instance_id))
            })
            .collect();
        members.sort_unstable();
        let mut object_id = self.objects.iter().map(|o| o.id).max().unwrap_or(0);
        let mut label_id = self
            .objects
            .iter()
            .flat_map(ProjectObject::instances)
            .map(|i| i.labeled_id())
            .max()
            .unwrap_or(0);
        let capacity = additional
            .checked_mul(members.len())
            .ok_or_else(invalid_count)?;
        copy.objects
            .try_reserve(capacity)
            .map_err(|_| invalid_count())?;
        for (source, _) in members {
            let original = &self.objects[source];
            for number in 1..=additional {
                object_id = object_id.checked_add(1).ok_or_else(invalid_count)?;
                let mut object = original.clone();
                object.id = object_id;
                object.name = format!("{}_{number}", original.name);
                relabel_copy(&mut object, &mut label_id, &mut identities)?;
                copy.objects.push(object);
            }
        }
        copy.plates[plate] = PlateMetadata::new(self.plates[plate].id(), identities);
        Ok(copy)
    }
}

fn relabel_copy(
    object: &mut ProjectObject,
    label_id: &mut u32,
    identities: &mut Vec<[u32; 3]>,
) -> Result<(), SliceError> {
    for instance in &mut object.instances {
        *label_id = label_id.checked_add(1).ok_or_else(invalid_count)?;
        instance.object_id = object.id;
        instance.label = InstanceLabel::Loaded(*label_id);
        identities.push([object.id, instance.instance_id, *label_id]);
        if let Some(imported) = object
            .import_instances
            .iter_mut()
            .find(|i| i.instance_id == instance.instance_id)
        {
            imported.object_id = object.id;
            imported.label = instance.label;
        }
    }
    Ok(())
}

fn invalid_count() -> SliceError {
    SliceError::InvalidInput("repetition count is too large for the loaded plate".to_owned())
}
