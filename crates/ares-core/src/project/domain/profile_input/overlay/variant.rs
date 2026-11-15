use crate::project::raw_settings::{ProjectSettingsRaw, RawConfigValue};

/// Compose selected per-filament variant segments for the CLI refresh layout.
/// The corresponding Native path derives variant indices and applies each
/// source segment with restore semantics (`OrcaSlicer.cpp:3639–3701,
/// 3753–3759`).
pub(super) fn variant_values(
    embedded_raw: &ProjectSettingsRaw,
    filament_raws: &[ProjectSettingsRaw],
    key: &str,
    selected: usize,
) -> Vec<RawConfigValue> {
    let mut values = filament_raws
        .iter()
        .flat_map(|raw| match raw.get(key) {
            Some(RawConfigValue::List(values)) => values.clone(),
            Some(value) => vec![value.clone()],
            None => Vec::new(),
        })
        .collect::<Vec<_>>();
    let Some(embedded) = embedded_raw.get(key) else {
        return values;
    };
    let Some(ids) = embedded_raw.get("filament_self_index") else {
        return values;
    };
    let tail_start = tail_start(ids, selected);
    if values.len() < embedded.len() {
        values.extend(
            (tail_start..embedded.len()).filter_map(|index| embedded.element(index).cloned()),
        );
    }
    values
}

pub(super) fn filament_self_index(
    embedded_raw: &ProjectSettingsRaw,
    filament_raws: &[ProjectSettingsRaw],
    embedded_count: usize,
    selected: usize,
) -> Vec<RawConfigValue> {
    let mut self_index = filament_raws
        .iter()
        .enumerate()
        .flat_map(|(index, raw)| {
            let count = raw
                .get("filament_extruder_variant")
                .map_or(0, RawConfigValue::len);
            std::iter::repeat_n((index + 1).to_string(), count)
        })
        .map(RawConfigValue::Text)
        .collect::<Vec<_>>();
    let Some(value) = embedded_raw.get("filament_self_index") else {
        return self_index;
    };
    if self_index.len() < embedded_count {
        self_index.extend(
            (tail_start(value, selected)..value.len())
                .filter_map(|index| value.element(index).cloned()),
        );
    }
    self_index
}

fn tail_start(ids: &RawConfigValue, selected: usize) -> usize {
    (0..ids.len())
        .find(|&index| {
            ids.element(index)
                .and_then(RawConfigValue::token)
                .and_then(|token| token.parse::<usize>().ok())
                .is_some_and(|id| id > selected)
        })
        .unwrap_or(ids.len())
}
