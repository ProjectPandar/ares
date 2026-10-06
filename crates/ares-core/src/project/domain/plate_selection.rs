use super::Project;
use crate::{ProjectSettings, SliceError};

impl Project {
    /// Projects slice one plate at a time (OrcaSlicer exports `plate_<n>.gcode`
    /// per plate). Returns a view containing only the instances of `plate_id`.
    pub(crate) fn select_plate(&self, plate_id: u32) -> Result<Self, SliceError> {
        let plate_count = self.plates.len();
        let index = self
            .plates
            .iter()
            .position(|plate| plate.id() == plate_id)
            .ok_or_else(|| SliceError::InvalidInput(format!("project has no plate {plate_id}")))?;
        let identity = self.plates[index]
            .instances()
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>();
        let objects = self
            .objects
            .iter()
            .filter_map(|object| {
                let mut instances = object.instances().to_vec();
                instances.retain(|instance| {
                    // Unassigned instances (no plate metadata) stay for the
                    // geometric print-volume gate; assigned instances match
                    // the plate's identity set.
                    match instance.loaded_label_id() {
                        Some(label) => identity.contains(&[
                            instance.object_id(),
                            instance.instance_id(),
                            label,
                        ]),
                        None => true,
                    }
                });
                (!instances.is_empty()).then(|| {
                    let mut object = object.clone();
                    object.instances = instances;
                    object
                })
            })
            .collect::<Vec<_>>();
        if objects.is_empty() {
            // Model.cpp:354-355 rejects an empty selected importer model.
            return Err(SliceError::InvalidInput(
                "The supplied file couldn't be read because it's empty".to_owned(),
            ));
        }
        let mut documents = self.documents.clone_shallow();
        documents.plate_documents = vec![self.documents.plate_documents.get(index).cloned()]
            .into_iter()
            .flatten()
            .collect();
        // The plate's own config entries (stored as `model_settings.config`
        // plate metadata) are applied over the project settings
        // (`OrcaSlicer.cpp:5979` `new_print_config.apply(*part_plate->config())`;
        // keys listed in `BBS_3MF_Importer`, `bbs_3mf.cpp:4418-4565`).
        let mut settings = self.settings.clone();
        if let Some(plate_settings) = self.documents.model_settings.plates.get(index) {
            super::super::plate_config::apply_plate_config_overrides(
                &mut settings,
                &plate_settings.metadata,
            );
        }
        Ok(Self {
            models: self.models.clone(),
            objects,
            plates: vec![self.plates[index].clone()],
            settings,
            documents,
            plate_origin: plate_grid_origin(index, plate_count, &self.settings),
        })
    }
}

/// `compute_colum_count` (`PartPlate.hpp:38-48`), ported verbatim: the
/// float `sqrt`/`round` comparison decides the column count, not an f64
/// `ceil`, so boundary counts match upstream exactly.
fn compute_colum_count(count: usize) -> usize {
    let value = (count as f32).sqrt();
    let round_value = value.round();
    if value > round_value {
        (round_value + 1.0) as usize
    } else {
        round_value as usize
    }
}

/// The plate grid origin (`PartPlate.cpp:3914-3921` `compute_origin` +
/// `:3952-3962` `compute_shape_position` with `LOGICAL_PART_PLATE_GAP =
/// 1/5` from `:55`): stride = bed bounding box × 1.2, row-major columns
/// with negative-Y rows. The functions live in `slic3r/GUI/PartPlate`
/// upstream but the CLI path uses this same grid geometry; the port stays
/// in the project/plate layer without GUI dependencies.
fn plate_grid_origin(index: usize, plate_count: usize, settings: &ProjectSettings) -> (f64, f64) {
    let area = &settings.printer.remaining.printable_area.0;
    let width = area.iter().map(|p| p.x).fold(f64::NEG_INFINITY, f64::max)
        - area.iter().map(|p| p.x).fold(f64::INFINITY, f64::min);
    let depth = area.iter().map(|p| p.y).fold(f64::NEG_INFINITY, f64::max)
        - area.iter().map(|p| p.y).fold(f64::INFINITY, f64::min);
    let cols = compute_colum_count(plate_count);
    let row = index / cols;
    let col = index % cols;
    (col as f64 * width * 1.2, -(row as f64) * depth * 1.2)
}
