//! ToolOrdering.cpp:1477–1672: static single-nozzle-per-head grouping context.
use super::super::super::LayerTools;
use crate::project::raw_settings::ProjectSettingsRaw;
use crate::project_slice::gcode_emit::wipe_tower_estimate::config::DynamicPrintConfig;
use crate::{ProjectPrimeVolumeMode, ProjectSettings, SliceError};
use std::collections::{BTreeMap, BTreeSet};

pub(super) struct Context {
    pub total: usize,
    pub used: Vec<usize>,
    pub layers: Vec<Vec<usize>>,
    pub matrices: [Vec<Vec<f32>>; 2],
    pub limits: [BTreeSet<usize>; 2],
    pub volume_limits: Vec<BTreeSet<i32>>,
    pub volumes: [i32; 2],
    pub capacity: [usize; 2],
    pub master: usize,
    pub custom: BTreeMap<usize, Vec<usize>>,
    pub load: [f32; 2],
    pub unload: [f32; 2],
}

impl Context {
    pub(super) fn build(
        input: (&ProjectSettings, &ProjectSettings, &ProjectSettingsRaw),
        layers: &[LayerTools],
        geometric: Vec<BTreeSet<usize>>,
        mut config: DynamicPrintConfig,
    ) -> Result<Self, SliceError> {
        let (settings, checkpoint, raw) = input;
        let total = settings.filament.gcode.filament_colour.0.len();
        let print = &settings.project.print;
        // prepare_flush_matrices partitions by physical head, not selected columns.
        if print.flush_volumes_matrix.0.len() != 2 * total * total {
            return Err(SliceError::InvalidInput(
                "Flush volumes matrix do not match to the correct size!".to_owned(),
            ));
        }
        let raw_tokens = raw
            .iter()
            .collect::<Result<BTreeMap<_, _>, _>>()
            .map_err(SliceError::InvalidInput)?;
        for key in [
            "enable_filament_dynamic_map",
            "has_filament_switcher",
            "filament_is_mixed",
            "extruder_max_nozzle_count",
            "extruder_nozzle_stats",
        ] {
            let default = crate::option_definition(key)
                .expect("registered grouping option")
                .default_wire_value;
            config.set_deserialize_strict(
                key,
                raw_tokens
                    .get(key)
                    .and_then(Option::as_deref)
                    .unwrap_or(default),
            )?;
        }
        if settings.project.gcode.filament_map_mode != crate::ProjectFilamentMapMode::AutoForFlush
            || config.option("enable_filament_dynamic_map").bool()
            || config.option("has_filament_switcher").bool()
            || config
                .option("filament_is_mixed")
                .bools()
                .iter()
                .any(|v| *v)
            || config
                .option("extruder_max_nozzle_count")
                .ints()
                .iter()
                .any(|v| *v != 1)
            || print.nozzle_diameter.0.len() != 2
        {
            return Err(unsupported());
        }
        let volumes = std::array::from_fn(|head| {
            settings
                .project
                .gcode
                .nozzle_volume_type
                .0
                .get(head)
                .map_or(0, |v| *v as i32)
        });
        check_nozzle_stats(&config, &volumes)?;
        let mut used = layers
            .iter()
            .flat_map(|l| l.extruders.iter().copied())
            .collect::<Vec<_>>();
        used.sort_unstable();
        used.dedup();
        if used.len() >= 14 {
            return Err(unsupported());
        }
        let matrices = prepare_flush_matrices(settings, total);
        let mut physical: [BTreeSet<usize>; 2] = Default::default();
        for &id in &used {
            let mask = settings
                .filament
                .gcode
                .filament_printable
                .0
                .get(id)
                .unwrap_or(&settings.filament.gcode.filament_printable.0[0])
                .0;
            if let Some(head) = (0..2).find(|head| mask >> head & 1 == 0) {
                physical[head].insert(id);
            }
        }
        let mut geometric: [BTreeSet<usize>; 2] = geometric.try_into().unwrap();
        remove_intersection(&mut physical);
        remove_intersection(&mut geometric);
        // collect_unprintable_limits keeps the first owner on cross-source conflicts.
        let mut limits: [BTreeSet<usize>; 2] = std::array::from_fn(|head| {
            physical[head]
                .union(&geometric[head])
                .copied()
                .filter(|id| !physical[1 - head].contains(id))
                .collect()
        });
        let volume_limits = declaration_limits(settings, checkpoint, &used, total);
        for head in 0..2 {
            limits[head].extend(
                used.iter()
                    .copied()
                    .filter(|id| volume_limits[*id].contains(&volumes[head])),
            );
        }
        remove_intersection(&mut limits);
        let mut capacity = [0; 2];
        for (head, wire) in settings
            .project
            .gcode
            .extruder_ams_count
            .0
            .iter()
            .enumerate()
            .take(2)
        {
            for entry in wire.split('|') {
                let (size, count) = entry.split_once('#').expect("validated AMS count");
                capacity[head] += size.parse::<usize>().expect("validated AMS size")
                    * count.parse::<usize>().expect("validated AMS quantity");
            }
        }
        for value in &mut capacity {
            *value = (*value).max(1);
        }
        let custom = custom_sequences(settings, layers);
        Ok(Self {
            total,
            used,
            layers: layers.iter().map(|l| l.extruders.clone()).collect(),
            matrices,
            limits,
            volume_limits,
            volumes,
            capacity,
            master: settings.printer.gcode.master_extruder_id.0 as usize - 1,
            custom,
            load: [
                settings.printer.gcode.machine_load_filament_time.0 as f32,
                (settings.printer.gcode.machine_load_filament_time.0 / 2.0) as f32,
            ],
            unload: [
                settings.printer.gcode.machine_unload_filament_time.0 as f32,
                (settings.printer.gcode.machine_unload_filament_time.0 / 2.0) as f32,
            ],
        })
    }
}

fn check_nozzle_stats(config: &DynamicPrintConfig, volumes: &[i32; 2]) -> Result<(), SliceError> {
    // PrintConfig.cpp:940–972 / ToolOrdering.cpp:1508–1530. One physical
    // nozzle of one volume type is the admitted inventory boundary.
    for (head, wire) in config
        .option("extruder_nozzle_stats")
        .strings()
        .iter()
        .enumerate()
        .take(2)
    {
        let mut stats = BTreeMap::new();
        for token in wire.split('|') {
            let mut parts = token.split('#');
            let volume = match parts.next().unwrap() {
                "Standard" => 0,
                "High Flow" => 1,
                "Hybrid" => 2,
                "TPU High Flow" => 3,
                "E3D High Flow" => 5,
                "Extra High Flow" => 6,
                _ => continue,
            };
            let Some(count) = parts.next() else { continue };
            stats.insert(volume, atoi(count));
        }
        if !stats.is_empty() {
            if stats.len() != 1 || stats.values().any(|v| *v != 1) || stats.contains_key(&2) {
                return Err(unsupported());
            }
            // Non-hybrid build_nozzle_groups retains the configured volume.
            if !stats.contains_key(&volumes[head]) {
                return Err(unsupported());
            }
        }
    }
    Ok(())
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

fn prepare_flush_matrices(settings: &ProjectSettings, total: usize) -> [Vec<Vec<f32>>; 2] {
    let print = &settings.project.print;
    let multipliers = if print.prime_volume_mode == ProjectPrimeVolumeMode::Fast {
        &print.flush_multiplier_fast.0
    } else {
        &print.flush_multiplier.0
    };
    std::array::from_fn(|head| {
        let multiplier = multipliers.get(head).map_or(1.0, |v| v.0);
        (0..total)
            .map(|row| {
                (0..total)
                    .map(|col| {
                        let raw = print.flush_volumes_matrix.0
                            [head * total * total + row * total + col]
                            as f32;
                        (f64::from(raw) * multiplier) as f32
                    })
                    .collect()
            })
            .collect()
    })
}

fn declaration_limits(
    settings: &ProjectSettings,
    checkpoint: &ProjectSettings,
    used: &[usize],
    total: usize,
) -> Vec<BTreeSet<i32>> {
    let declarations = &checkpoint.filament.gcode;
    let mut declared = BTreeMap::<usize, BTreeSet<i32>>::new();
    for (index, variant) in declarations.filament_extruder_variant.0.iter().enumerate() {
        if let Some(volume) = volume_type(variant) {
            let id = checkpoint
                .project
                .preset
                .filament_self_index
                .0
                .get(index)
                .map_or(1, |v| v.0) as usize
                - 1;
            declared.entry(id).or_default().insert(volume);
        }
    }
    let mut volume_limits = vec![BTreeSet::new(); total];
    for (id, allowed) in declared {
        if !used.contains(&id) {
            continue;
        }
        for volume in &settings.printer.gcode.printer_extruder_variant.0 {
            if let Some(volume) = volume_type(volume)
                && !allowed.contains(&volume)
            {
                volume_limits[id].insert(volume);
            }
        }
    }
    volume_limits
}

fn custom_sequences(
    settings: &ProjectSettings,
    layers: &[LayerTools],
) -> BTreeMap<usize, Vec<usize>> {
    let print = &settings.project.print;
    let mut custom = BTreeMap::new();
    let configured = &print.other_layers_print_sequence.0;
    let ranges = print.other_layers_print_sequence_nums.0 as usize;
    if let Some(width) = configured.len().checked_div(ranges) {
        for (index, layer) in layers.iter().enumerate() {
            let layer_number = index as i32 + 1;
            if let Some(row) = configured
                .chunks_exact(width)
                .rev()
                .find(|row| layer_number >= row[0].0 && layer_number <= row[1].0)
            {
                let sequence = row[2..]
                    .iter()
                    .map(|v| (v.0 as u32).wrapping_sub(1) as usize)
                    .filter(|id| layer.extruders.contains(id))
                    .collect::<Vec<_>>();
                assert_eq!(sequence.len(), layer.extruders.len());
                custom.insert(index, sequence);
            }
        }
    }
    custom
}

fn remove_intersection(sets: &mut [BTreeSet<usize>; 2]) {
    let common = sets[0].intersection(&sets[1]).copied().collect::<Vec<_>>();
    for id in common {
        sets[0].remove(&id);
        sets[1].remove(&id);
    }
}

// PrintConfig.cpp:912–935: strip an extruder-type prefix and parse the remaining volume.
fn volume_type(variant: &str) -> Option<i32> {
    ["Direct Drive", "Bowden"].into_iter().find_map(|prefix| {
        variant.find(prefix).and_then(|position| {
            let mut volume = variant.to_owned();
            volume.replace_range(position..position + prefix.len(), "");
            match volume.trim() {
                "Standard" => Some(0),
                "High Flow" => Some(1),
                _ => None,
            }
        })
    })
}

pub(super) fn unsupported() -> SliceError {
    SliceError::InvalidInput("unsupported automatic filament grouping context".to_owned())
}
