use serde::{Deserialize, Serialize};

/// Selection and project flush data updated by `PresetBundle::update_multi_material_filament_presets`
/// (40ecc66f7, PresetBundle.cpp:8598-8671). Matrices are contiguous per nozzle.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct FilamentSlotState {
    pub filaments: Vec<String>,
    pub flush_multiplier: Vec<f64>,
    pub flush_volumes_vector: Vec<f64>,
    pub flush_volumes_matrix: Vec<f64>,
    pub filament_colour: Vec<String>,
    pub filament_multi_colour: Vec<String>,
    pub filament_colour_type: Vec<String>,
    pub filament_map: Vec<i32>,
    pub filament_nozzle_map: Vec<i32>,
    pub filament_volume_map: Vec<i32>,
}

impl Default for FilamentSlotState {
    fn default() -> Self {
        Self {
            filaments: Vec::new(),
            // PrintConfig.cpp:8291: the initial nozzle starts with the project default.
            flush_multiplier: vec![0.3],
            flush_volumes_vector: vec![140.0, 140.0],
            flush_volumes_matrix: vec![0.0],
            filament_colour: vec!["#F2754E".to_owned()],
            filament_multi_colour: vec![String::new()],
            filament_colour_type: vec!["1".to_owned()],
            filament_map: vec![1],
            filament_nozzle_map: vec![0],
            filament_volume_map: vec![0],
        }
    }
}

impl FilamentSlotState {
    pub(super) fn normalize(
        &mut self,
        minimum: usize,
        nozzles: usize,
        first_visible: &str,
    ) -> Result<(), crate::SliceError> {
        if minimum == 0 {
            return Ok(());
        }
        let old_nozzles = self.flush_multiplier.len();
        if old_nozzles == 0 || !self.flush_volumes_vector.len().is_multiple_of(2) {
            return Err(crate::SliceError::InvalidInput(
                "flush multipliers must be nonempty and flush volumes must be load/unload pairs"
                    .to_owned(),
            ));
        }
        let old_count = ((self.flush_volumes_matrix.len() / old_nozzles) as f64).sqrt() as usize;
        if old_count * old_count * old_nozzles != self.flush_volumes_matrix.len() {
            return Err(crate::SliceError::InvalidInput(
                "flush matrix must contain one square matrix per nozzle".to_owned(),
            ));
        }
        if self.filaments.len() < minimum {
            let last = self
                .filaments
                .last()
                .map(String::as_str)
                .unwrap_or(first_visible)
                .to_owned();
            self.filaments.resize(minimum, last);
        }
        let count = self.filaments.len();
        // v2.5.0 load_config_file_config, PresetBundle.cpp:6455-6459,
        // calls set_num_filaments (:4328-4372) after padding a short palette.
        if self.filament_colour.len() < count {
            self.filament_colour.resize(
                count,
                self.filament_colour.first().cloned().unwrap_or_default(),
            );
            self.filament_multi_colour.clone_from(&self.filament_colour);
            self.filament_colour_type.resize(
                count,
                self.filament_colour_type
                    .first()
                    .cloned()
                    .unwrap_or_default(),
            );
            self.filament_map.resize(count, 1);
            self.filament_nozzle_map.resize(count, 0);
            self.filament_volume_map.resize(count, 0);
        }
        self.flush_multiplier.resize(nozzles, 1.0);
        if self.flush_volumes_matrix.len() == count * count * nozzles {
            return Ok(());
        }
        while self.flush_volumes_vector.len() < 2 * count {
            let load = self.flush_volumes_vector.first().copied().unwrap_or(140.0);
            let unload = self.flush_volumes_vector.get(1).copied().unwrap_or(140.0);
            self.flush_volumes_vector.extend([load, unload]);
        }
        self.flush_volumes_vector.truncate(2 * count);
        let mut matrix = vec![0.0; count * count * nozzles];
        for (from, to, nozzle) in (0..count).flat_map(|from| {
            (0..count).flat_map(move |to| (0..nozzles).map(move |nozzle| (from, to, nozzle)))
        }) {
            matrix[nozzle * count * count + from * count + to] =
                if from < old_count && to < old_count && nozzle < old_nozzles {
                    self.flush_volumes_matrix
                        [nozzle * old_count * old_count + from * old_count + to]
                } else if from == to {
                    0.0
                } else {
                    self.flush_volumes_vector[2 * from] + self.flush_volumes_vector[2 * to + 1]
                };
        }
        self.flush_volumes_matrix = matrix;
        Ok(())
    }
}
