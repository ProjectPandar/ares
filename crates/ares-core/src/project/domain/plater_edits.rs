//! A loaded project with the plater edits applied: the instance
//! transformations and plate map the GUI holds after a move, rotate or plate
//! change, as its project export writes them (`bbs_3mf.cpp:7918, 8050-8073`).

use super::{PlateMetadata, Project};
use crate::plater::PlaterEdits;
use crate::project::filament_sequence::{FilamentSequences, PlateId};
use crate::project::model_settings::PlateSettings;

impl Project {
    /// Sets an instance's transformation. Its imported copy keeps the frame it was loaded in:
    /// `normalize_volume_offsets` moved a single volume's offset `v` into the loaded instance
    /// (`loaded = imported · T(v)`), so the import becomes `transform · T(-v)`.
    pub(crate) fn set_instance_transform(
        &mut self,
        object: usize,
        instance: usize,
        transform: crate::Transform3d,
    ) {
        let object = &mut self.objects[object];
        let loaded = &mut object.instances[instance];
        let (identity, previous) = (loaded.instance_id, loaded.transform);
        if previous == transform {
            return;
        }
        loaded.transform = transform;
        for imported in object
            .import_instances
            .iter_mut()
            .filter(|imported| imported.instance_id == identity)
        {
            imported.transform = transform.translated(object_offset(previous, imported.transform));
        }
    }

    /// Sets each listed instance's transformation and, with a plate list,
    /// replaces the plates; per-plate documents follow their source plate,
    /// while `Model::plates_custom_gcodes` stays keyed by plate index
    /// (`PartPlateList::delete_plate`, `PartPlate.cpp:4554-4660`, leaves it).
    pub(crate) fn apply_plater_edits(&mut self, edits: &PlaterEdits) {
        if let Some(order) = &edits.object_order {
            self.order_objects(order);
        }
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

impl Project {
    /// Puts the objects in the order of their ids in `order`.
    pub(crate) fn order_objects(&mut self, order: &[u32]) {
        self.objects
            .sort_by_key(|object| order.iter().position(|id| *id == object.id()));
    }
}

/// `c` with `loaded · T(c) = imported`: `A·c = t_imported - t_loaded` for the shared linear part
/// `A`, solved by Cramer's rule.
fn object_offset(loaded: crate::Transform3d, imported: crate::Transform3d) -> crate::Point3d {
    let (m, t) = (loaded.as_matrix(), imported.as_matrix());
    let b = [0, 1, 2].map(|row| t[row][3] - m[row][3]);
    let determinant = loaded.linear_determinant();
    let solve = |column: usize| {
        let mut replaced = *m;
        for row in 0..3 {
            replaced[row][column] = b[row];
        }
        crate::Transform3d::from_rows(replaced).linear_determinant() / determinant
    };
    crate::Point3d::new(solve(0), solve(1), solve(2))
}
