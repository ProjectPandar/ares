use super::{
    super::{
        catalog::Catalog,
        project_refresh::preset::{dirty_keys, strings},
    },
    PresetConfig, collection,
};
use crate::{
    ProfileKind, SliceError,
    project::raw_settings::{ProjectSettingsRaw, RawConfigValue},
};

pub(super) struct ExternalFilamentArgs<'a> {
    pub(super) catalog: &'a Catalog,
    pub(super) source: &'a ProjectSettingsRaw,
    pub(super) filament_ids: &'a [String],
    pub(super) inherited: &'a [String],
    pub(super) machine_conditions: &'a [String],
    pub(super) process_conditions: &'a [String],
    pub(super) differences: &'a [String],
    pub(super) filament_count: usize,
    pub(super) filename: &'a str,
    pub(super) version: &'a str,
}

pub(super) fn load_external_filaments(
    ExternalFilamentArgs {
        catalog,
        source,
        filament_ids,
        inherited,
        machine_conditions,
        process_conditions,
        differences,
        filament_count,
        filename,
        version,
    }: ExternalFilamentArgs<'_>,
) -> Result<Vec<collection::PresetCollectionState>, SliceError> {
    let mut variant_values = strings(source, "filament_extruder_variant");
    let mut self_indices = strings(source, "filament_self_index")
        .into_iter()
        .map(|value| value.parse::<usize>().unwrap_or_default())
        .collect::<Vec<_>>();
    if self_indices.len() < filament_count {
        self_indices.resize(filament_count, 0);
        for (index, value) in self_indices.iter_mut().enumerate() {
            *value = index + 1;
        }
    }
    if variant_values.len() < filament_count {
        variant_values.resize(filament_count, "Direct Drive Standard".to_owned());
    }
    let extruder_variant_count = variant_values.len();
    if source.contains_key("extruder_variant_list")
        && (variant_values.len() != self_indices.len() || variant_values.len() < filament_count)
    {
        return Err(SliceError::InvalidInput(
            "invalid configuration file: filament variant and self-index counts differ".to_owned(),
        ));
    }
    let process_multi_extruder =
        source.contains_key("extruder_variant_list") && filament_count != extruder_variant_count;
    let mut variant_indices = vec![0; filament_count];
    if process_multi_extruder {
        let mut cur_filament_id = 1;
        for (index, self_index) in self_indices.iter().enumerate() {
            if cur_filament_id > filament_count {
                break;
            }
            if *self_index != cur_filament_id {
                continue;
            }
            variant_indices[cur_filament_id - 1] = index;
            cur_filament_id += 1;
        }
    }
    let variant_keys = crate::options::registry::filament_options_with_variant();
    let defaults = PresetConfig::defaults(ProfileKind::Filament)?.0;
    let mut configs = Vec::with_capacity(filament_count);
    for filament in 0..filament_count {
        let mut config = PresetConfig(defaults.clone());
        for (key, _default) in defaults.entries() {
            let Some(value) = source.get(key) else {
                continue;
            };
            let RawConfigValue::List(values) = value else {
                config.0.insert(key.clone(), value.clone());
                continue;
            };
            if matches!(key.as_str(), "compatible_printers" | "compatible_prints") {
                continue;
            }
            let values = if process_multi_extruder
                && (key == "filament_extruder_variant" || variant_keys.contains(&key.as_str()))
            {
                let mut values = values.clone();
                let fallback = first_variant_value(&values)?;
                values.resize(extruder_variant_count, fallback);
                let start = variant_indices[filament].min(values.len());
                let end = variant_indices
                    .get(filament + 1)
                    .copied()
                    .unwrap_or(extruder_variant_count)
                    .min(values.len());
                RawConfigValue::List(values[start..end].to_vec())
            } else {
                values.get(filament).cloned().map_or_else(
                    || RawConfigValue::List(Vec::new()),
                    |value| RawConfigValue::List(vec![value]),
                )
            };
            config.0.insert(key.clone(), values);
        }
        config.set_string(
            "compatible_printers_condition",
            machine_conditions
                .get(filament + 1)
                .cloned()
                .unwrap_or_default(),
        );
        config.set_string(
            "compatible_prints_condition",
            process_conditions
                .get(filament)
                .cloned()
                .unwrap_or_default(),
        );
        let mut parent = inherited.get(filament + 1).cloned().unwrap_or_default();
        if !parent.is_empty() {
            parent = convert_filament_preset_name(source, parent);
        }
        config.set_string("inherits", parent.clone());
        config.set_string(
            "different_settings_to_system",
            differences.get(filament + 1).cloned().unwrap_or_default(),
        );
        let mut name = strings(source, "filament_settings_id")
            .get(filament)
            .cloned()
            .unwrap_or_default();
        if parent.is_empty() {
            name = convert_filament_preset_name(source, name);
        }
        config.set_string("filament_settings_id", name);
        config.set_string(
            "filament_self_index",
            self_indices
                .get(variant_indices[filament])
                .copied()
                .unwrap_or(filament + 1)
                .to_string(),
        );
        configs.push(config);
    }

    let mut collection = collection::Collection::load(catalog, ProfileKind::Filament)?;
    let mut states = Vec::with_capacity(filament_count);
    for filament in (0..filament_count).rev() {
        let mut protected = dirty_keys(differences, filament + 1)?;
        protected.extend(
            [
                "inherits",
                "print_settings_id",
                "filament_settings_id",
                "printer_settings_id",
            ]
            .map(str::to_owned),
        );
        let config = collection.materialize(&configs[filament].0, &protected)?;
        let select = if filament == 0 {
            collection::LoadAndSelect::Always
        } else {
            collection::LoadAndSelect::OnlyIfModified
        };
        let state = collection.select(collection::SelectionArgs {
            config,
            filename,
            project_version: version,
            filament_id: filament_ids.get(filament).map(String::as_str).unwrap_or(""),
            select,
        })?;
        states.push(state);
    }
    states.reverse();
    Ok(states)
}

fn first_variant_value(values: &[RawConfigValue]) -> Result<RawConfigValue, SliceError> {
    values.first().cloned().ok_or_else(|| {
        SliceError::InvalidInput("invalid configuration file: empty variant vector".to_owned())
    })
}

fn convert_filament_preset_name(source: &ProjectSettingsRaw, mut name: String) -> String {
    let machine = strings(source, "printer_settings_id")
        .into_iter()
        .next()
        .unwrap_or_default();
    let replacement = match machine.as_str() {
        "Bambu Lab H2D 0.6 nozzle" => match name.as_str() {
            "Bambu PLA Basic @BBL H2D" => Some("Bambu PLA Basic @BBL H2D 0.6 nozzle"),
            "Bambu PLA Matte @BBL H2D" => Some("Bambu PLA Matte @BBL H2D 0.6 nozzle"),
            "Bambu ABS @BBL H2D" => Some("Bambu ABS @BBL H2D 0.6 nozzle"),
            _ => None,
        },
        "Bambu Lab H2D 0.8 nozzle" => match name.as_str() {
            "Bambu PETG HF @BBL H2D 0.6 nozzle" => Some("Bambu PETG HF @BBL H2D 0.8 nozzle"),
            "Bambu ASA @BBL H2D 0.6 nozzle" => Some("Bambu ASA @BBL H2D 0.8 nozzle"),
            _ => None,
        },
        _ => None,
    };
    if let Some(replacement) = replacement {
        name = replacement.to_owned();
    }
    name
}
