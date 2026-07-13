use super::{PrinterOptions, ProcessOptions};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProjectSettings {
    pub printer: PrinterOptions,
    pub process: ProcessOptions,
}
