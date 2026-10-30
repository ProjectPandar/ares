//! A loaded project with the plater edits applied: the instance
//! transformations and plate map the GUI holds after a move, rotate or plate
//! change, as its project export writes them (`bbs_3mf.cpp:7918, 8050-8073`).

use super::{PlateMetadata, Project};
use crate::plater::PlaterEdits;
use crate::project::filament_sequence::{FilamentSequences, PlateId};
use crate::project::model_settings::PlateSettings;

impl Project {
    pub(crate) fn set_instance_transform(
        &mut self,
        object: usize,
        instance: usize,
        transform: crate::Transform3d,
    ) {
        let object = &mut self.objects[object];
        let identity = object.instances[instance].instance_id;
        object.instances[instance].transform = transform;
        for loaded in object
            .import_instances
            .iter_mut()
            .filter(|loaded| loaded.instance_id == identity)
        {
            loaded.transform = transform;
        }
    }

    /// Sets each listed instance's transformation and, with a plate list,
    /// replaces the plates; per-plate documents follow their source plate,
    /// while `Model::plates_custom_gcodes` stays keyed by plate index
    /// (`PartPlateList::delete_plate`, `PartPlate.cpp:4554-4660`, leaves it).
    pub(crate) fn apply_plater_edits(&mut self, edits: &PlaterEdits) {
        let updates =
            self.objects
                .iter()
                .enumerate()
                .flat_map(|(object_index, object)| {
                    object.instances.iter().enumerate().filter_map(
                        move |(instance_index, instance)| {
                            edits
                                .transform(object.id(), instance.instance_id)
                                .map(|transform| (object_index, instance_index, transform))
                        },
                    )
                })
                .collect::<Vec<_>>();
        for (object, instance, transform) in updates {
            self.set_instance_transform(object, instance, transform);
        }
        let Some(plates) = &edits.plates else {
            return;
        };
        let documents = &mut self.documents;
        let old_settings = std::mem::take(&mut documents.model_settings.plates);
        let old_sequences = std::mem::replace(
            &mut documents.filament_sequences,
            FilamentSequences(Default::default()),
        );
        let old_documents = std::mem::take(&mut documents.plate_documents);
        let mut new_plates = Vec::with_capacity(plates.len());
        for (index, plate) in plates.iter().enumerate() {
            let id = index as u32 + 1;
            new_plates.push(PlateMetadata::new(id, plate.instances.clone()));
            let source = plate.source;
            documents.model_settings.plates.push(
                source
                    .and_then(|source| old_settings.get(source).cloned())
                    .unwrap_or(PlateSettings {
                        metadata: Vec::new(),
                        model_instances: Vec::new(),
                    }),
            );
            let Some(source) = source else {
                continue;
            };
            if let Some(sequence) = old_sequences.0.get(&PlateId::new(source as u32 + 1)) {
                documents
                    .filament_sequences
                    .0
                    .insert(PlateId::new(id), sequence.clone());
            }
            documents
                .plate_documents
                .extend(old_documents.get(source).cloned());
        }
        self.plates = new_plates;
    }
}
