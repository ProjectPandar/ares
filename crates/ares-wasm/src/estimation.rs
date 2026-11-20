use ares_core::ProcessedGCode;

/// Inputs of the legend estimation footer (`GCodeViewer.cpp:4547-4644`):
/// `Print::print_statistics()` (`Print.hpp:789-816`) and the Normal-mode
/// `PrintEstimatedStatistics::Mode` times. Preparation time widens from
/// f32 to f64; normal time already retains its upstream double precision.
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Estimation {
    total_used_filament: f64,
    total_weight: f64,
    total_cost: f64,
    time: f64,
    prepare_time: f64,
}

impl From<&ProcessedGCode> for Estimation {
    fn from(output: &ProcessedGCode) -> Self {
        let stats = &output.result.print_statistics;
        Self {
            total_used_filament: output.print_statistics.total_used_filament,
            total_weight: output.print_statistics.total_weight,
            total_cost: output.print_statistics.total_cost,
            time: stats.normal_time,
            prepare_time: f64::from(stats.normal_prepare_time),
        }
    }
}
