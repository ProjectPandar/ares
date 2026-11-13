use super::{LoadAndSelect, PresetConfig, PresetOutcome, PresetRecord, connection};

pub(super) struct FinishArgs {
    pub(super) index: usize,
    pub(super) outcome: PresetOutcome,
    pub(super) edited_config: Option<PresetConfig>,
    pub(super) modified: bool,
    pub(super) select: LoadAndSelect,
}

pub(in super::super) struct SelectionArgs<'a> {
    pub(in super::super) config: PresetConfig,
    pub(in super::super) filename: &'a str,
    pub(in super::super) project_version: &'a str,
    pub(in super::super) filament_id: &'a str,
    pub(in super::super) select: LoadAndSelect,
}

pub(super) fn apply_non_connection(target: &mut PresetConfig, source: &PresetConfig) {
    for (key, value) in source.0.entries().filter(|(key, _)| !connection(key)) {
        target.0.insert(key.clone(), value.clone());
    }
}

pub(super) fn inherited_name(record: &PresetRecord) -> String {
    if record.system {
        record.name.clone()
    } else {
        record.config.string("inherits")
    }
}
