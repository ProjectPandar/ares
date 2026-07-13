use super::{
    FilamentOptions, PresetMetadata, PrinterOptions, ProcessOptions, ProjectRuntimeOptions,
};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProjectSettings {
    pub filament: FilamentOptions,
    pub printer: PrinterOptions,
    pub process: ProcessOptions,
    pub project: ProjectRuntimeOptions,
    pub metadata: PresetMetadata,
}
