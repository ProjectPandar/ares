use super::{ImportScope, Project};
use crate::{ProjectSettings, SliceError};

impl Project {
    /// Model::curr_plate_index, consumed by GCode.cpp:7276 wipe-tower options.
    pub(crate) fn current_plate_index(&self) -> usize {
        self.curr_plate_index as usize
    }

    /// Projects slice one plate at a time (OrcaSlicer exports `plate_<n>.gcode`
    /// per plate). Returns a view containing only the instances of `plate`
    /// (the first plate when `None`), selected as `Model::curr_plate_index`.
    pub fn select_plate(&self, plate: Option<u32>) -> Result<Self, SliceError> {
        let plate_id = plate.unwrap_or_else(|| self.first_plate_id());
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
        let mut objects = match plate {
            Some(_) => self.selected_import_objects(&self.plates[index]),
            None => self.objects.clone(),
        };
        for object in &mut objects {
            object
                .instances
                .retain(|instance| match instance.loaded_label_id() {
                    Some(label) => {
                        identity.contains(&[instance.object_id(), instance.instance_id(), label])
                    }
                    // All-import views retain unassigned instances for the volume gate.
                    // Explicit importer scopes already excluded absent object IDs.
                    None => true,
                });
        }
        objects.retain(|object| !object.instances.is_empty());
        if objects.is_empty() {
            // Model.cpp:354-355 rejects an empty selected importer model.
            // The CLI adapter maps this to `CLI_NO_SUITABLE_OBJECTS`
            // (-50, Utils.hpp:61) with an empty stderr channel — the
            // payload only feeds the legacy `Error:` detail.
            return Err(SliceError::NoSuitableObjects(
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
        let source_index = if self.plate_view {
            self.curr_plate_index as usize
        } else {
            index
        };
        if let Some(plate_settings) = self.documents.model_settings.plates.get(source_index) {
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
            import_scope: plate.map_or(self.import_scope, ImportScope::Plate),
            plate_view: true,
            plate_origin: if self.plate_view {
                self.plate_origin
            } else {
                plate_grid_origin(index, plate_count, &self.settings)
            },
            plates_custom_gcodes: self.plates_custom_gcodes.clone(),
            // OrcaSlicer.cpp:5690 `model.curr_plate_index = index`.
            curr_plate_index: source_index as i32,
        })
    }
}

impl Project {
    /// The own `filament_map_mode` and `filament_map` of loaded plate `source` (0-based), as
    /// `PartPlate::config` holds them (`plate_config::own_filament_map`).
    pub(crate) fn plate_filament_map(
        &self,
        source: usize,
    ) -> (Option<crate::options::ProjectFilamentMapMode>, Vec<i32>) {
        self.documents
            .model_settings
            .plates
            .get(source)
            .map_or((None, Vec::new()), |plate| {
                super::super::plate_config::own_filament_map(&plate.metadata)
            })
    }

    /// The own `imex_parallel_mode` of loaded plate `source` (0-based), as `PartPlate::config` holds
    /// it (`plate_config::own_imex_mode`).
    pub(crate) fn plate_imex_mode(&self, source: usize) -> Option<&str> {
        self.documents
            .model_settings
            .plates
            .get(source)
            .and_then(|plate| super::super::plate_config::own_imex_mode(&plate.metadata))
    }

    /// Whether loaded plate `source` (0-based) is locked (`plate_config::own_locked`).
    pub(crate) fn plate_locked(&self, source: usize) -> bool {
        self.documents
            .model_settings
            .plates
            .get(source)
            .is_some_and(|plate| super::super::plate_config::own_locked(&plate.metadata))
    }

    /// The lowest plate id; on a selected view, the selected plate's id.
    pub(crate) fn first_plate_id(&self) -> u32 {
        self.plates
            .iter()
            .map(|plate| plate.id())
            .min()
            .unwrap_or(1)
    }
}

/// `compute_colum_count` (`PartPlate.hpp:38-48`), ported verbatim: the
/// float `sqrt`/`round` comparison decides the column count, not an f64
/// `ceil`, so boundary counts match upstream exactly.
pub(crate) fn compute_colum_count(count: usize) -> usize {
    let value = (count as f32).sqrt();
    let round_value = value.round();
    if value > round_value {
        (round_value + 1.0) as usize
    } else {
        round_value as usize
    }
}

/// The plate dimensions: the printable area's bounding box size, truncated to
/// integers (`static_cast<int>(BoundingBoxf(area).size())`, `OrcaSlicer.cpp:1899-1903,
/// 4371-4374` at v2.5.0-beta `345efec64`; 2.4.2 took points [2]-[0]).
pub(crate) fn plate_dimensions(settings: &ProjectSettings) -> (i32, i32) {
    let area = &settings.printer.remaining.printable_area.0;
    let (mut min, mut max) = ([f64::MAX; 2], [f64::MIN; 2]);
    for p in area {
        min = [min[0].min(p.x), min[1].min(p.y)];
        max = [max[0].max(p.x), max[1].max(p.y)];
    }
    ((max[0] - min[0]) as i32, (max[1] - min[1]) as i32)
}

/// `PartPlateList::compute_origin` / `compute_shape_position`
/// (`PartPlate.cpp:3915-3962`) with the 1/5 gap of `plate_stride_x/y`
/// (`:4836-4849`).
pub(crate) fn plate_origin_at(index: usize, cols: usize, (width, depth): (i32, i32)) -> (f64, f64) {
    let row = index / cols;
    let col = index % cols;
    (
        col as f64 * f64::from(width) * 1.2,
        -(row as f64) * f64::from(depth) * 1.2,
    )
}

fn plate_grid_origin(index: usize, plate_count: usize, settings: &ProjectSettings) -> (f64, f64) {
    plate_origin_at(
        index,
        compute_colum_count(plate_count),
        plate_dimensions(settings),
    )
}
