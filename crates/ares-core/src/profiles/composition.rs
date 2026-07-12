use std::collections::BTreeSet;

use serde_json::{Map, Value, json};

use super::fragment::{ProfileFragment, merge_profile_fragments};
use crate::{ProfileKind, SliceError, SliceOptions};

const PROFILE_LOCAL_KEYS: &[&str] = &[
    "type",
    "name",
    "inherits",
    "from",
    "setting_id",
    "instantiation",
    "compatible_prints",
    "compatible_prints_condition",
    "compatible_printers",
    "compatible_printers_condition",
    "different_settings_to_system",
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProfileSelection {
    process: String,
    machine: String,
    filaments: Vec<String>,
}

impl ProfileSelection {
    pub fn new(
        process: impl Into<String>,
        machine: impl Into<String>,
        filaments: impl IntoIterator<Item = impl Into<String>>,
    ) -> Result<Self, SliceError> {
        let process = process.into();
        let machine = machine.into();
        let filaments = filaments.into_iter().map(Into::into).collect::<Vec<_>>();
        if process.is_empty()
            || machine.is_empty()
            || filaments.is_empty()
            || filaments.iter().any(String::is_empty)
        {
            return Err(SliceError::InvalidInput(
                "profile selection must include process, machine, and filaments".to_owned(),
            ));
        }

        Ok(Self {
            process,
            machine,
            filaments,
        })
    }

    pub fn process(&self) -> &str {
        &self.process
    }

    pub fn machine(&self) -> &str {
        &self.machine
    }

    pub fn filaments(&self) -> &[String] {
        &self.filaments
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ComposedProfile {
    process: String,
    machine: String,
    filaments: Vec<String>,
    options: SliceOptions,
}

impl ComposedProfile {
    pub fn options(&self) -> &SliceOptions {
        &self.options
    }

    pub fn into_options(self) -> SliceOptions {
        self.options
    }

    pub fn process_name(&self) -> &str {
        &self.process
    }

    pub fn machine_name(&self) -> &str {
        &self.machine
    }

    pub fn filament_names(&self) -> &[String] {
        &self.filaments
    }
}

pub fn compose_profile_fragments(
    fragments: &[ProfileFragment],
    selection: &ProfileSelection,
) -> Result<ComposedProfile, SliceError> {
    let machine = merge_profile_fragments(fragments, ProfileKind::Machine, selection.machine())?;
    let process = merge_profile_fragments(fragments, ProfileKind::Process, selection.process())?;
    let filaments = selection
        .filaments()
        .iter()
        .map(|name| merge_profile_fragments(fragments, ProfileKind::Filament, name))
        .collect::<Result<Vec<_>, _>>()?;

    let mut values = Map::new();
    apply_options(&mut values, &machine);
    apply_options(&mut values, &process);
    apply_filaments(&mut values, &filaments);
    for key in PROFILE_LOCAL_KEYS {
        values.remove(*key);
    }
    add_metadata(&mut values, selection, &machine, &process, &filaments);

    let options = serde_json::from_value(Value::Object(values)).map_err(|error| {
        SliceError::InvalidInput(format!("composed profile options are invalid: {error}"))
    })?;

    Ok(ComposedProfile {
        process: selection.process().to_owned(),
        machine: selection.machine().to_owned(),
        filaments: selection.filaments().to_vec(),
        options,
    })
}

fn apply_options(values: &mut Map<String, Value>, options: &SliceOptions) {
    for (key, value) in options.values() {
        values.insert(key.clone(), value.clone());
    }
}

fn apply_filaments(values: &mut Map<String, Value>, filaments: &[SliceOptions]) {
    if filaments.len() == 1 {
        apply_options(values, &filaments[0]);
        return;
    }

    let keys = filaments
        .iter()
        .flat_map(|options| options.values().keys().cloned())
        .collect::<BTreeSet<_>>();
    for key in keys {
        let collected = filaments
            .iter()
            .filter_map(|options| options.values().get(&key))
            .cloned()
            .collect::<Vec<_>>();
        values.insert(key, merge_filament_values(collected));
    }
}

fn merge_filament_values(values: Vec<Value>) -> Value {
    Value::Array(
        values
            .into_iter()
            .flat_map(|value| match value {
                Value::Array(values) => values,
                value => vec![value],
            })
            .collect(),
    )
}

fn add_metadata(
    values: &mut Map<String, Value>,
    selection: &ProfileSelection,
    machine: &SliceOptions,
    process: &SliceOptions,
    filaments: &[SliceOptions],
) {
    values.insert("print_settings_id".to_owned(), json!(selection.process()));
    values.insert("printer_settings_id".to_owned(), json!(selection.machine()));
    values.insert(
        "filament_settings_id".to_owned(),
        json!(selection.filaments()),
    );
    values.insert(
        "filament_map".to_owned(),
        json!(vec![1; selection.filaments().len()]),
    );

    insert_non_empty_group(
        values,
        "inherits_group",
        std::iter::once(process)
            .chain(filaments.iter())
            .chain(std::iter::once(machine))
            .filter_map(|options| string_value(options, "inherits"))
            .collect(),
    );
    insert_non_empty_group(
        values,
        "compatible_machine_expression_group",
        std::iter::once(process)
            .chain(filaments.iter())
            .filter_map(|options| string_value(options, "compatible_printers_condition"))
            .collect(),
    );
    insert_non_empty_group(
        values,
        "compatible_process_expression_group",
        filaments
            .iter()
            .filter_map(|options| string_value(options, "compatible_prints_condition"))
            .collect(),
    );
    if let Some(value) = process.values().get("compatible_printers") {
        values.insert("print_compatible_printers".to_owned(), value.clone());
    }
    insert_non_empty_group(
        values,
        "filament_ids",
        filaments
            .iter()
            .filter_map(|options| string_value(options, "filament_id"))
            .collect(),
    );
}

fn insert_non_empty_group(values: &mut Map<String, Value>, key: &str, group: Vec<String>) {
    let group = group
        .into_iter()
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    if !group.is_empty() {
        values.insert(key.to_owned(), json!(group));
    }
}

fn string_value(options: &SliceOptions, key: &str) -> Option<String> {
    options
        .values()
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ProfileFragment, SliceError};
    use serde_json::json;

    #[test]
    fn selection_validates_and_exposes_names() {
        let selection = ProfileSelection::new("0.20", "printer", ["pla", "petg"]).unwrap();

        assert_eq!(selection.process(), "0.20");
        assert_eq!(selection.machine(), "printer");
        assert_eq!(selection.filaments(), &["pla", "petg"]);

        assert_invalid_selection(ProfileSelection::new("", "printer", ["pla"]));
        assert_invalid_selection(ProfileSelection::new("0.20", "", ["pla"]));
        assert_invalid_selection(ProfileSelection::new("0.20", "printer", Vec::<&str>::new()));
        assert_invalid_selection(ProfileSelection::new("0.20", "printer", [""]));
    }

    #[test]
    fn composes_single_filament_profile_with_metadata_and_child_overrides() {
        let fragments = fragments([
            br#"{"type":"machine","name":"base-printer"}"#.as_slice(),
            br#"{"type":"machine","name":"printer","nozzle_diameter":["0.4"],"inherits":"base-printer","machine_unknown":7,"compatible_printers":["local-only"]}"#,
            br#"{"type":"process","name":"base-process"}"#,
            br#"{"type":"process","name":"0.20","inherits":"base-process","layer_height":0.2,"compatible_printers":["printer"],"compatible_printers_condition":"printer_model=~/A1/","process_unknown":"kept"}"#,
            br#"{"type":"filament","name":"base-filament"}"#,
            br#"{"type":"filament","name":"pla","inherits":"base-filament","filament_id":"PLA-ID","filament_diameter":["1.75"],"compatible_prints_condition":"layer_height==0.2","compatible_printers_condition":"nozzle_diameter[0]==0.4","filament_unknown":true}"#,
        ]);
        let selection = ProfileSelection::new("0.20", "printer", ["pla"]).unwrap();

        let composed = compose_profile_fragments(&fragments, &selection).unwrap();
        let options = composed.options();

        assert_eq!(composed.process_name(), "0.20");
        assert_eq!(composed.machine_name(), "printer");
        assert_eq!(composed.filament_names(), &["pla"]);
        assert_eq!(options.layer_height().unwrap(), 0.2);
        assert_eq!(options.nozzle_diameters().unwrap(), vec![0.4]);
        assert_eq!(options.filament_diameters().unwrap(), vec![1.75]);
        assert_eq!(options.values()["machine_unknown"], json!(7));
        assert_eq!(options.values()["process_unknown"], json!("kept"));
        assert_eq!(options.values()["filament_unknown"], json!(true));
        assert_eq!(options.values()["print_settings_id"], json!("0.20"));
        assert_eq!(options.values()["printer_settings_id"], json!("printer"));
        assert_eq!(options.values()["filament_settings_id"], json!(["pla"]));
        assert_eq!(options.values()["filament_map"], json!([1]));
        assert_eq!(options.values()["filament_ids"], json!(["PLA-ID"]));
        assert_eq!(
            options.values()["inherits_group"],
            json!(["base-process", "base-filament", "base-printer"])
        );
        assert_eq!(
            options.values()["compatible_machine_expression_group"],
            json!(["printer_model=~/A1/", "nozzle_diameter[0]==0.4"])
        );
        assert_eq!(
            options.values()["compatible_process_expression_group"],
            json!(["layer_height==0.2"])
        );
        assert_eq!(
            options.values()["print_compatible_printers"],
            json!(["printer"])
        );
        for removed in [
            "type",
            "name",
            "inherits",
            "compatible_printers",
            "compatible_printers_condition",
            "compatible_prints_condition",
        ] {
            assert!(
                !options.values().contains_key(removed),
                "{removed} should be profile-local"
            );
        }
    }

    #[test]
    fn composes_multiple_filaments_deterministically() {
        let fragments = fragments([
            br#"{"type":"machine","name":"printer","nozzle_diameter":["0.4"]}"#.as_slice(),
            br#"{"type":"process","name":"0.20","layer_height":0.2}"#,
            br##"{"type":"filament","name":"pla","filament_diameter":"1.75","filament_colour":["#fff"],"filament_type":"PLA"}"##,
            br##"{"type":"filament","name":"petg","filament_id":"PETG-ID","filament_diameter":["2.85"],"filament_colour":["#000"],"filament_type":"PETG","petg_only_unknown":9}"##,
        ]);
        let selection = ProfileSelection::new("0.20", "printer", ["pla", "petg"]).unwrap();

        let options = compose_profile_fragments(&fragments, &selection)
            .unwrap()
            .into_options();

        assert_eq!(
            options.values()["filament_settings_id"],
            json!(["pla", "petg"])
        );
        assert_eq!(options.values()["filament_map"], json!([1, 1]));
        assert_eq!(options.values()["filament_ids"], json!(["PETG-ID"]));
        assert_eq!(
            options.values()["filament_diameter"],
            json!(["1.75", "2.85"])
        );
        assert_eq!(options.values()["filament_colour"], json!(["#fff", "#000"]));
        assert_eq!(options.values()["filament_type"], json!(["PLA", "PETG"]));
        assert_eq!(options.values()["petg_only_unknown"], json!([9]));
        assert_eq!(options.filament_diameters().unwrap(), vec![1.75, 2.85]);
    }

    #[test]
    fn composition_reports_missing_profiles() {
        let fragments = fragments([
            br#"{"type":"machine","name":"printer"}"#.as_slice(),
            br#"{"type":"process","name":"0.20"}"#,
        ]);
        let selection = ProfileSelection::new("0.20", "printer", ["missing-filament"]).unwrap();

        assert!(matches!(
            compose_profile_fragments(&fragments, &selection),
            Err(SliceError::InvalidInput(_))
        ));
    }

    fn fragments<const N: usize>(inputs: [&[u8]; N]) -> Vec<ProfileFragment> {
        inputs
            .into_iter()
            .map(|input| ProfileFragment::from_json_bytes(input).unwrap())
            .collect()
    }

    fn assert_invalid_selection(result: Result<ProfileSelection, SliceError>) {
        assert!(matches!(result, Err(SliceError::InvalidInput(_))));
    }
}
