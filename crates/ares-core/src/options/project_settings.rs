use super::PrinterOptions;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProjectSettings {
    pub printer: PrinterOptions,
}
