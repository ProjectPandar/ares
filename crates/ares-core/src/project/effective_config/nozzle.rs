use super::types::BoundedResolvedProjectConfig;

impl BoundedResolvedProjectConfig {
    // Print.cpp:5155–5184: absence of a publication is not a missing filament.
    pub(crate) fn nozzle_config_index(&self, filament: usize) -> usize {
        let Some(assignments) = &self.process_nozzle_assignments else {
            return crate::options::mapped_extruder_id(&self.views.full, filament);
        };
        assignments
            .get(filament)
            .copied()
            .flatten()
            .map_or(0, |nozzle| {
                crate::options::process_nozzle_config_index(&self.views.full, nozzle)
            })
    }
}
