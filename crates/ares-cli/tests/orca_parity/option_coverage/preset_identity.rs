use super::domains::{OptionCase, OptionPlan};
use serde_json::{Map, Value};

pub(super) fn activate(
    plan: &OptionPlan,
    case: &OptionCase,
    machine: &mut Map<String, Value>,
    process: &mut Map<String, Value>,
    filaments: &mut [Map<String, Value>],
) {
    // PresetBundle::construct_full_config writes identities from preset metadata.
    match plan.key.as_str() {
        "printer_settings_id" => {
            let requested = case.value.as_ref().unwrap();
            let previous = machine.insert("name".into(), requested.clone()).unwrap();
            // CLI process compatibility compares these names before export/slice.
            for printer in process
                .get_mut("compatible_printers")
                .unwrap()
                .as_array_mut()
                .unwrap()
            {
                if *printer == previous {
                    *printer = requested.clone();
                }
            }
        }
        "print_settings_id" => {
            process.insert("name".into(), case.value.clone().unwrap());
        }
        "filament_settings_id" => {
            filaments[0].insert("name".into(), case.value.as_ref().unwrap()[0].clone());
        }
        "filament_ids" => {
            filaments[0].insert(
                "filament_id".into(),
                case.value.as_ref().unwrap()[0].clone(),
            );
        }
        _ => {}
    }
}
