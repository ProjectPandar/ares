//! `GCodeProcessor::post_process:993–1010,1147–1174`: material export
//! derived only from the processor's authoritative total-volume map.
use super::ProcessorConfig;

pub(super) struct MaterialStatistics {
    values: [(&'static str, Vec<f64>); 6],
    pub(super) length_mm: f64,
    pub(super) weight_g: f64,
    pub(super) cost: f64,
}

impl MaterialStatistics {
    pub(super) fn new(
        statistics: &crate::PrintEstimatedStatistics,
        config: &ProcessorConfig<'_>,
    ) -> Self {
        let filament = &config.settings.filament.gcode;
        let mut mm = Vec::new();
        let mut cm3 = Vec::new();
        let mut grams = Vec::new();
        let mut cost = Vec::new();
        for (id, &volume) in statistics.total_volumes_per_extruder.iter().enumerate() {
            let diameter = f64::from(filament.filament_diameter.0[id].0 as f32);
            // PrintConfig apply_config uses ConfigOptionFloats::get_at(i):
            // a scalar-length option applies its first value to other tools.
            let density = f64::from(
                filament
                    .filament_density
                    .0
                    .get(id)
                    .unwrap_or(&filament.filament_density.0[0])
                    .0 as f32,
            );
            let price = f64::from(
                filament
                    .filament_cost
                    .0
                    .get(id)
                    .unwrap_or(&filament.filament_cost.0[0])
                    .0 as f32,
            );
            mm.push(volume / (std::f64::consts::PI * (0.5 * diameter).powi(2)));
            cm3.push(volume * 0.001);
            grams.push(cm3[id] * density);
            cost.push(grams[id] * price * 0.001);
        }
        let total_grams = grams.iter().sum();
        let total_cost = cost.iter().sum();
        Self {
            length_mm: mm.iter().sum(),
            weight_g: total_grams,
            cost: total_cost,
            values: [
                ("; filament used [mm] =", mm),
                ("; filament used [g] =", grams),
                ("; total filament used [g] =", vec![total_grams]),
                ("; filament used [cm3] =", cm3),
                ("; filament cost =", cost),
                ("; total filament cost =", vec![total_cost]),
            ],
        }
    }

    pub(super) fn process(&self, line: &str) -> Option<String> {
        let (tag, values) = self.values.iter().find(|(tag, _)| line.starts_with(tag))?;
        let mut result = tag.to_string();
        for (index, value) in values.iter().enumerate() {
            use std::fmt::Write;
            write!(result, " {value:.2}").unwrap();
            if index + 1 != values.len() {
                result.push(',');
            }
        }
        Some(result)
    }
}
