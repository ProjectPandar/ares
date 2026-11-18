use crate::{
    PrinterSelection,
    presets::VendorProfiles,
    runner::{CaseInputs, OrcaRunner},
};
use serde_json::Map;

#[path = "stock_presets/elegoo_m6211.rs"]
mod elegoo_m6211;
#[path = "stock_presets/sweep.rs"]
mod sweep;
#[path = "stock_presets/tests.rs"]
mod tests;
#[path = "stock_presets/timelapse.rs"]
mod timelapse;

fn export(
    runner: &OrcaRunner,
    profiles: &VendorProfiles,
    selection: &PrinterSelection,
    model: &std::path::Path,
) -> Result<crate::runner::ExportedCase, String> {
    let machine = profiles.machine(&selection.printer)?;
    let process = profiles.process(&selection.process)?;
    let filaments = selection
        .filaments
        .iter()
        .map(|name| profiles.filament(name))
        .collect::<Result<Vec<_>, _>>()?;
    let label = format!("stock/{}/{}", selection.vendor, selection.printer);
    runner
        .export_case(
            &CaseInputs {
                label: &label,
                machine: &machine,
                process: &process,
                filaments: &filaments,
            },
            &Map::new(),
            model,
        )
        .map_err(|error| error.to_string())
}
