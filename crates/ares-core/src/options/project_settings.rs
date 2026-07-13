use super::{FilamentOptions, PrinterOptions, ProcessOptions};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProjectSettings {
    pub filament: FilamentOptions,
    pub printer: PrinterOptions,
    pub process: ProcessOptions,
}
