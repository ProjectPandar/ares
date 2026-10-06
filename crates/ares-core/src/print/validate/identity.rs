//! Printer identity for the `Print::validate` gate, not G-code export.
use crate::{
    ProjectSettings, SliceError,
    options::{model_config_deserialize::wire, registry::OptionValueKind},
    project::raw_settings::{ProjectSettingsRaw, RawConfigValue},
};

/// `OrcaSlicer.cpp:1730-1765,6043-6059`: preserve selected/system identity.
pub(crate) fn project_is_bbl(
    settings: &ProjectSettings,
    raw: &ProjectSettingsRaw,
    selected: Option<&str>,
) -> Result<bool, SliceError> {
    let model = &settings.printer.remaining.printer_model.0;
    let selected = selected.unwrap_or_default();
    if !model.is_empty() || !selected.is_empty() {
        return Ok(super::is_bbl_printer(model, selected, ""));
    }
    let current = &settings.project.preset.printer_settings_id.0;
    let inherited = match raw.get("inherits_group") {
        None => None,
        Some(RawConfigValue::List(values)) => match values.last() {
            None => None,
            Some(RawConfigValue::Text(name)) => Some(name.as_str()),
            Some(_) => {
                return Err(SliceError::InvalidInput(
                    "inherits_group must contain strings".to_owned(),
                ));
            }
        },
        Some(RawConfigValue::Text(token)) => {
            let wire::WireValue::Sequence(values) =
                wire::decode(OptionValueKind::Strings, token).map_err(SliceError::InvalidInput)?
            else {
                unreachable!("Strings decoder returns a sequence");
            };
            let system = values
                .last()
                .filter(|name| !name.is_empty())
                .map_or(current.as_str(), String::as_str);
            return Ok(super::is_bbl_printer("", "", system));
        }
        Some(_) => {
            return Err(SliceError::InvalidInput(
                "inherits_group must be a string vector".to_owned(),
            ));
        }
    };
    let system = inherited.filter(|name| !name.is_empty()).unwrap_or(current);
    Ok(super::is_bbl_printer("", "", system))
}
