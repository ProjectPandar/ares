use std::collections::BTreeMap;

use crate::project::raw_settings::{ProjectSettingsRaw, RawConfigValue};
use crate::{ExtruderType, NozzleVolumeType, ProjectSettings, SliceError};

use super::index::{canonical_variant, get_at};

#[derive(Default)]
pub(crate) struct NozzleInventory(Vec<BTreeMap<i32, i32>>);

impl NozzleInventory {
    // PrintConfig.cpp:11581–11583 reads the per-extruder string vector.
    pub(crate) fn read(raw: &ProjectSettingsRaw) -> Self {
        let Some(option) = raw.get("extruder_nozzle_stats") else {
            return Self::default();
        };
        Self(
            (0..option.len())
                .map(|index| match option.element(index) {
                    Some(RawConfigValue::Text(wire)) => parse_nozzle_counts(wire),
                    _ => BTreeMap::new(),
                })
                .collect(),
        )
    }

    pub(crate) fn counts(&self) -> &[BTreeMap<i32, i32>] {
        &self.0
    }

    // PrintConfig.cpp:11575–11598: only a full inventory defines the count
    // of configured volume types; counts of individual nozzles are irrelevant.
    pub(super) fn has_extra_volumes(&self, extruders: usize) -> bool {
        self.0.len() == extruders && self.0.iter().map(BTreeMap::len).sum::<usize>() > extruders
    }

    // PrintConfig.cpp:11700–11723: expand concrete volumes in inventory order.
    pub(super) fn slots(
        &self,
        settings: &ProjectSettings,
    ) -> Result<Vec<(i32, String)>, SliceError> {
        let physical_count = settings.project.print.nozzle_diameter.0.len();
        let counts = self.0.as_slice();
        let all_heads = self.has_extra_volumes(physical_count);
        let mut slots = Vec::with_capacity(physical_count);
        for physical in 0..physical_count {
            let (extruder, nominal) = nominal_variant(settings, physical)?;
            let id = physical as i32 + 1;
            if !all_heads && nominal != NozzleVolumeType::Hybrid {
                slots.push((id, canonical_variant(extruder, nominal)));
                continue;
            }
            if counts.len() != physical_count {
                continue;
            }
            slots.extend(
                counts[physical]
                    .keys()
                    .map(|&v| (id, volume_variant(extruder, v))),
            );
        }
        Ok(slots)
    }
}

// PrintConfig.cpp:11879–11914: the selected filament volume replaces the
// nominal head volume only for per-type columns or a hybrid head.
pub(super) fn filament_variant(
    settings: &ProjectSettings,
    (physical, filament): (usize, usize),
    sized_volume_map: bool,
    all_heads: bool,
) -> Result<String, SliceError> {
    let (extruder, nominal) = nominal_variant(settings, physical)?;
    let volume = if sized_volume_map && (all_heads || nominal == NozzleVolumeType::Hybrid) {
        settings.project.gcode.filament_volume_map.0[filament].0
    } else {
        nominal as i32
    };
    Ok(volume_variant(extruder, volume))
}

// PrintConfig.cpp:11704–11705 reads the nominal per-head variant.
fn nominal_variant(
    settings: &ProjectSettings,
    physical: usize,
) -> Result<(ExtruderType, NozzleVolumeType), SliceError> {
    let types = &settings.printer.gcode.extruder_type.0;
    let volumes = &settings.project.gcode.nozzle_volume_type.0;
    Ok((
        *get_at(types, physical, "extruder_type")?,
        *get_at(volumes, physical, "nozzle_volume_type")?,
    ))
}

// PrintConfig.cpp:777–798: a hybrid or unknown volume has no variant string.
fn volume_variant(extruder: ExtruderType, volume: i32) -> String {
    let volume = match volume {
        0 => NozzleVolumeType::Standard,
        1 => NozzleVolumeType::HighFlow,
        3 => NozzleVolumeType::TpuHighFlow,
        5 => NozzleVolumeType::E3dHighFlow,
        6 => NozzleVolumeType::ExtraHighFlow,
        _ => return String::new(),
    };
    canonical_variant(extruder, volume)
}

// PrintConfig.cpp:940–972: drop malformed tokens; the last count per type wins.
pub(crate) fn parse_nozzle_counts(wire: &str) -> BTreeMap<i32, i32> {
    wire.split('|')
        .filter_map(|token| {
            let mut parts = token.split('#');
            let volume = match parts.next()? {
                "Standard" => NozzleVolumeType::Standard,
                "High Flow" => NozzleVolumeType::HighFlow,
                "Hybrid" => NozzleVolumeType::Hybrid,
                "TPU High Flow" => NozzleVolumeType::TpuHighFlow,
                "E3D High Flow" => NozzleVolumeType::E3dHighFlow,
                "Extra High Flow" => NozzleVolumeType::ExtraHighFlow,
                _ => return None,
            };
            Some((volume as i32, atoi(parts.next()?)))
        })
        .collect()
}

// std::atoi consumes a signed C-locale decimal prefix (PrintConfig.cpp:967).
fn atoi(text: &str) -> i32 {
    let text = text.trim_start_matches([' ', '\t', '\n', '\u{b}', '\u{c}', '\r']);
    let sign = usize::from(text.starts_with(['+', '-']));
    let end = sign
        + text
            .bytes()
            .skip(sign)
            .take_while(u8::is_ascii_digit)
            .count();
    text[..end].parse().unwrap_or(0)
}
