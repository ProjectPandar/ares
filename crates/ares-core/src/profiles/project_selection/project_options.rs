use super::{PresetConfig, strings};
use crate::{
    FilamentSlotState, option_definition,
    project::raw_settings::{ProjectSettingsRaw, RawConfigValue},
};

pub(super) fn normalize_filament_options(
    raw: &ProjectSettingsRaw,
    result: &mut ProjectSettingsRaw,
    printer: &PresetConfig,
    count: usize,
) {
    // PresetBundle.cpp:5402-5424,6455-6459: use the edited printer. A short
    // palette requires adding actual presets and is not a project-option edit.
    let nozzles = printer
        .0
        .get("nozzle_diameter")
        .map_or(1, RawConfigValue::len)
        .max(1);
    if count < nozzles
        && !matches!(
            printer.string("single_extruder_multi_material").as_str(),
            "1" | "true"
        )
    {
        return;
    }

    if let (Some(multiplier), Some(vector), Some(matrix)) = (
        floats(raw, "flush_multiplier"),
        floats(raw, "flush_volumes_vector"),
        floats(raw, "flush_volumes_matrix"),
    ) {
        let mut slots = FilamentSlotState {
            flush_multiplier: multiplier,
            flush_volumes_vector: vector,
            flush_volumes_matrix: matrix,
            ..Default::default()
        };
        // The archive loader does not add rejection rules for unsupported old layouts.
        if slots.resize_flush(count, nozzles, Some).is_ok() {
            for (key, values) in [
                ("flush_multiplier", slots.flush_multiplier),
                ("flush_volumes_vector", slots.flush_volumes_vector),
                ("flush_volumes_matrix", slots.flush_volumes_matrix),
            ] {
                let values = values
                    .into_iter()
                    .map(|value| value.to_string())
                    .collect::<Vec<_>>();
                result.insert_strings(key, &values);
            }
        }
    }

    // Plater.cpp:8939-8949: fill empty entries only if the colour type vector needs resizing.
    let key = "filament_colour_type";
    let default = option_definition(key)
        .expect("registered colour type")
        .default_wire_value;
    let mut types = if raw.contains_key(key) {
        strings(raw, key)
    } else {
        vec![default.to_owned()]
    };
    if types.len() != count {
        types.resize(count, String::new());
        for value in types.iter_mut().filter(|value| value.is_empty()) {
            value.push_str(default);
        }
    }
    result.insert_strings(key, &types);
}

fn floats(raw: &ProjectSettingsRaw, key: &str) -> Option<Vec<f64>> {
    let parse = |value: &str| {
        value
            .trim()
            .parse::<f64>()
            .ok()
            .filter(|value| value.is_finite())
    };
    if raw.contains_key(key) {
        strings(raw, key).iter().map(|value| parse(value)).collect()
    } else {
        option_definition(key)?
            .default_wire_value
            .split(',')
            .map(parse)
            .collect()
    }
}
