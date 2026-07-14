use super::{
    FilamentOptions, PresetMetadata, PrinterOptions, ProcessOptions, ProjectRuntimeOptions,
};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProjectSettings {
    pub printer: PrinterOptions,
    pub process: ProcessOptions,
    pub filament: FilamentOptions,
    pub project: ProjectRuntimeOptions,
    pub metadata: PresetMetadata,
}
