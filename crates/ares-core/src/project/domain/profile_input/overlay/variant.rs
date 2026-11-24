use std::ops::Range;

use crate::project::raw_settings::{ProjectSettingsRaw, RawConfigValue};

use super::FilamentSlots;

/// Embedded variant segments in filament slot order (OrcaSlicer.cpp:3512-3561).
pub(super) struct SparseVariants<'a>(Vec<(Range<usize>, Option<&'a ProjectSettingsRaw>)>);

impl<'a> SparseVariants<'a> {
    pub(super) fn new(
        embedded: &ProjectSettingsRaw,
        loaded: &'a [ProjectSettingsRaw],
        count: usize,
        slots: FilamentSlots<'_>,
    ) -> Self {
        let mut counts = vec![0; count];
        let ids = embedded
            .get("filament_self_index")
            .into_iter()
            .flat_map(|ids| (0..ids.len()).filter_map(|index| ids.element(index)));
        for id in ids {
            if let Some(slot_count) = id
                .token()
                .and_then(|token| token.parse::<usize>().ok())
                .and_then(|id| id.checked_sub(1))
                .and_then(|slot| counts.get_mut(slot))
            {
                *slot_count += 1;
            }
        }
        let mut start = 0;
        Self(
            counts
                .into_iter()
                .enumerate()
                .map(|(slot, len)| {
                    let range = start..start + len;
                    start = range.end;
                    (range, slots.source(slot).map(|source| &loaded[source]))
                })
                .collect(),
        )
    }

    pub(super) fn values(&self, embedded: &ProjectSettingsRaw, key: &str) -> Vec<RawConfigValue> {
        let embedded = embedded.get(key);
        let mut values = Vec::new();
        for (segment, loaded) in &self.0 {
            if let Some(value) = loaded.and_then(|raw| raw.get(key)) {
                values.extend((0..value.len()).filter_map(|index| value.element(index).cloned()));
            } else if let Some(value) = embedded {
                values.extend(
                    segment
                        .clone()
                        .filter_map(|index| value.element(index).cloned()),
                );
            }
        }
        values
    }

    pub(super) fn self_index(&self) -> Vec<RawConfigValue> {
        self.0
            .iter()
            .enumerate()
            .flat_map(|(slot, (segment, loaded))| {
                let count = loaded
                    .and_then(|raw| raw.get("filament_extruder_variant"))
                    .map_or_else(|| segment.len(), RawConfigValue::len);
                std::iter::repeat_n(RawConfigValue::Text((slot + 1).to_string()), count)
            })
            .collect()
    }
}

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
