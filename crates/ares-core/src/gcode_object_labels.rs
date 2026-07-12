// Source: OrcaSlicer/src/libslic3r/GCode.cpp

use crate::gcode_format::format_decimal;
use crate::{LayerPrintPaths, SliceError, SliceOptions, options::GCodeFlavor};

const LABEL_START: &str = "; printing object ares-object-0 id:0 copy 0\n";
const LABEL_STOP: &str = "; stop printing object ares-object-0 id:0 copy 0\n";
const KLIPPER_START: &str = "EXCLUDE_OBJECT_START NAME=ares-object-0\n";
const KLIPPER_END: &str = "EXCLUDE_OBJECT_END NAME=ares-object-0\n";
const KLIPPER_LABEL_START: &str =
    "; printing object ares-object-0 id:0 copy 0\nEXCLUDE_OBJECT_START NAME=ares-object-0\n";
const KLIPPER_LABEL_END: &str =
    "; stop printing object ares-object-0 id:0 copy 0\nEXCLUDE_OBJECT_END NAME=ares-object-0\n";
const MARLIN_DEFINITION: &str = "M486 S0\nM486 Aares-object-0\nM486 S-1\n";
const MARLIN_RRF_START: &str = "M486 S0\n";
const MARLIN_RRF_END: &str = "M486 S-1\n";
const MARLIN_RRF_LABEL_START: &str = "; printing object ares-object-0 id:0 copy 0\nM486 S0\n";
const MARLIN_RRF_LABEL_END: &str = "; stop printing object ares-object-0 id:0 copy 0\nM486 S-1\n";
const RRF_DEFINITION: &str = "M486 S0 A\"ares-object-0\"\nM486 S-1\n";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ObjectLabelConfig {
    labels_enabled: bool,
    exclusion: Option<ObjectExclusionCommands>,
}

impl ObjectLabelConfig {
    pub(crate) fn from_options(
        options: &SliceOptions,
        gcode_flavor: GCodeFlavor,
    ) -> Result<Self, SliceError> {
        options
            .support_object_skip_flush_options()?
            .consume_runtime();
        let labels_enabled = enabled(options)?;
        let exclusion_enabled = match options.values().get("exclude_object") {
            Some(value) => value.as_bool().ok_or_else(|| {
                SliceError::InvalidInput("exclude_object must be a boolean".to_owned())
            })?,
            None => false,
        };
        let exclusion = exclusion_enabled
            .then(|| ObjectExclusionCommands::for_flavor(gcode_flavor))
            .flatten();

        Ok(Self {
            labels_enabled,
            exclusion,
        })
    }

    pub(crate) fn object_definition(self, layer_print_paths: &[LayerPrintPaths]) -> String {
        self.exclusion.map_or_else(String::new, |commands| {
            commands.definition(layer_print_paths)
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ObjectDefinition {
    Klipper,
    Static(&'static str),
}

impl ObjectDefinition {
    fn render(self, layer_print_paths: &[LayerPrintPaths]) -> String {
        match self {
            Self::Klipper => klipper_definition(layer_print_paths),
            Self::Static(definition) => definition.to_owned(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ObjectExclusionCommands {
    definition: ObjectDefinition,
    start: &'static str,
    end: &'static str,
    label_start: &'static str,
    label_end: &'static str,
}

impl ObjectExclusionCommands {
    fn definition(self, layer_print_paths: &[LayerPrintPaths]) -> String {
        self.definition.render(layer_print_paths)
    }

    fn for_flavor(gcode_flavor: GCodeFlavor) -> Option<Self> {
        match gcode_flavor {
            GCodeFlavor::Klipper => Some(Self {
                definition: ObjectDefinition::Klipper,
                start: KLIPPER_START,
                end: KLIPPER_END,
                label_start: KLIPPER_LABEL_START,
                label_end: KLIPPER_LABEL_END,
            }),
            GCodeFlavor::MarlinLegacy | GCodeFlavor::MarlinFirmware => Some(Self {
                definition: ObjectDefinition::Static(MARLIN_DEFINITION),
                start: MARLIN_RRF_START,
                end: MARLIN_RRF_END,
                label_start: MARLIN_RRF_LABEL_START,
                label_end: MARLIN_RRF_LABEL_END,
            }),
            GCodeFlavor::RepRapFirmware => Some(Self {
                definition: ObjectDefinition::Static(RRF_DEFINITION),
                start: MARLIN_RRF_START,
                end: MARLIN_RRF_END,
                label_start: MARLIN_RRF_LABEL_START,
                label_end: MARLIN_RRF_LABEL_END,
            }),
            _ => None,
        }
    }
}

fn klipper_definition(layer_print_paths: &[LayerPrintPaths]) -> String {
    let first_layer = crate::gcode_first_layer_print_placeholders::placeholders(layer_print_paths);
    let Some(bounds) = first_layer.bounds() else {
        return String::new();
    };
    let min = bounds.min();
    let max = bounds.max();

    format!(
        "EXCLUDE_OBJECT_DEFINE NAME=ares-object-0 CENTER={} POLYGON=[[{},{}],[{},{}],[{},{}],[{},{}],[{},{}]]\n",
        first_layer.center_list(),
        format_decimal(min.x()),
        format_decimal(min.y()),
        format_decimal(max.x()),
        format_decimal(min.y()),
        format_decimal(max.x()),
        format_decimal(max.y()),
        format_decimal(min.x()),
        format_decimal(max.y()),
        format_decimal(min.x()),
        format_decimal(min.y()),
    )
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ObjectLabelState {
    config: ObjectLabelConfig,
    started: bool,
    stopped: bool,
}

impl ObjectLabelState {
    pub(crate) const fn new(config: ObjectLabelConfig) -> Self {
        Self {
            config,
            started: false,
            stopped: false,
        }
    }

    pub(crate) fn before_first_object_move(&mut self) -> &'static str {
        if self.started {
            return "";
        }
        let output = match (self.config.labels_enabled, self.config.exclusion) {
            (true, Some(commands)) => commands.label_start,
            (true, None) => LABEL_START,
            (false, Some(commands)) => commands.start,
            (false, None) => "",
        };
        self.started = !output.is_empty();
        output
    }

    pub(crate) fn after_last_object_move(&mut self) -> &'static str {
        if !self.started || self.stopped {
            return "";
        }
        self.stopped = true;
        match (self.config.labels_enabled, self.config.exclusion) {
            (true, Some(commands)) => commands.label_end,
            (true, None) => LABEL_STOP,
            (false, Some(commands)) => commands.end,
            (false, None) => "",
        }
    }
}

pub(crate) fn enabled(options: &SliceOptions) -> Result<bool, SliceError> {
    let Some(value) = options.values().get("gcode_label_objects") else {
        return Ok(true);
    };
    value
        .as_bool()
        .ok_or_else(|| SliceError::InvalidInput("gcode_label_objects must be a boolean".to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{LayerPrintPaths, options::GCodeFlavor};

    #[test]
    fn disabled_state_never_emits_labels() {
        let mut state = ObjectLabelState::new(ObjectLabelConfig {
            labels_enabled: false,
            exclusion: None,
        });

        assert_eq!(state.before_first_object_move(), "");
        assert_eq!(state.after_last_object_move(), "");
    }

    #[test]
    fn enabled_state_emits_one_start_and_one_stop() {
        let mut state = ObjectLabelState::new(ObjectLabelConfig {
            labels_enabled: true,
            exclusion: None,
        });

        assert_eq!(
            state.before_first_object_move(),
            "; printing object ares-object-0 id:0 copy 0\n"
        );
        assert_eq!(state.before_first_object_move(), "");
        assert_eq!(
            state.after_last_object_move(),
            "; stop printing object ares-object-0 id:0 copy 0\n"
        );
        assert_eq!(state.after_last_object_move(), "");
    }

    #[test]
    fn enabled_state_does_not_stop_before_start() {
        let mut state = ObjectLabelState::new(ObjectLabelConfig {
            labels_enabled: true,
            exclusion: None,
        });

        assert_eq!(state.after_last_object_move(), "");
    }

    #[test]
    fn enabled_state_emits_labels_before_exclusion_commands() {
        let config = ObjectLabelConfig::from_options(
            &options_with_json(serde_json::json!({
                "exclude_object": true,
            })),
            GCodeFlavor::Klipper,
        )
        .unwrap();
        let mut state = ObjectLabelState::new(config);

        assert_eq!(
            state.before_first_object_move(),
            "; printing object ares-object-0 id:0 copy 0\nEXCLUDE_OBJECT_START NAME=ares-object-0\n"
        );
        assert_eq!(
            state.after_last_object_move(),
            "; stop printing object ares-object-0 id:0 copy 0\nEXCLUDE_OBJECT_END NAME=ares-object-0\n"
        );
    }

    #[test]
    fn exclude_object_defaults_to_disabled() {
        let config =
            ObjectLabelConfig::from_options(&SliceOptions::default(), GCodeFlavor::Klipper)
                .unwrap();

        assert!(config.labels_enabled);
        assert_eq!(config.object_definition(&[]), "");
        let mut state = ObjectLabelState::new(config);
        assert_eq!(
            state.before_first_object_move(),
            "; printing object ares-object-0 id:0 copy 0\n"
        );
        assert_eq!(
            state.after_last_object_move(),
            "; stop printing object ares-object-0 id:0 copy 0\n"
        );
    }

    #[test]
    fn exclude_object_reads_boolean_value_for_klipper() {
        let options = options_with_json(serde_json::json!({ "exclude_object": true }));
        let config = ObjectLabelConfig::from_options(&options, GCodeFlavor::Klipper).unwrap();

        let empty_layer = [LayerPrintPaths::new(0, 0.2, Vec::new())];
        assert_eq!(config.object_definition(&empty_layer), "");
        let mut state = ObjectLabelState::new(config);
        assert_eq!(
            state.before_first_object_move(),
            "; printing object ares-object-0 id:0 copy 0\nEXCLUDE_OBJECT_START NAME=ares-object-0\n"
        );
        assert_eq!(
            state.after_last_object_move(),
            "; stop printing object ares-object-0 id:0 copy 0\nEXCLUDE_OBJECT_END NAME=ares-object-0\n"
        );
    }

    #[test]
    fn exclude_object_false_suppresses_exclusion_commands() {
        let options = options_with_json(serde_json::json!({ "exclude_object": false }));
        let config =
            ObjectLabelConfig::from_options(&options, GCodeFlavor::MarlinFirmware).unwrap();

        assert_eq!(config.object_definition(&[]), "");
        let mut state = ObjectLabelState::new(config);
        assert_eq!(
            state.before_first_object_move(),
            "; printing object ares-object-0 id:0 copy 0\n"
        );
        assert_eq!(
            state.after_last_object_move(),
            "; stop printing object ares-object-0 id:0 copy 0\n"
        );
    }

    #[test]
    fn exclude_object_rejects_non_boolean_values() {
        let options = options_with_json(serde_json::json!({ "exclude_object": "true" }));

        let err = ObjectLabelConfig::from_options(&options, GCodeFlavor::Klipper).unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert_eq!(err.to_string(), "exclude_object must be a boolean");
    }

    #[test]
    fn unsupported_flavor_accepts_exclude_object_without_commands() {
        let options = options_with_json(serde_json::json!({ "exclude_object": true }));
        let config = ObjectLabelConfig::from_options(&options, GCodeFlavor::Repetier).unwrap();

        assert_eq!(config.object_definition(&[]), "");
        let mut state = ObjectLabelState::new(config);
        assert_eq!(
            state.before_first_object_move(),
            "; printing object ares-object-0 id:0 copy 0\n"
        );
        assert_eq!(
            state.after_last_object_move(),
            "; stop printing object ares-object-0 id:0 copy 0\n"
        );
    }

    #[test]
    fn option_defaults_to_enabled() {
        assert!(enabled(&SliceOptions::default()).unwrap());
    }

    #[test]
    fn option_reads_boolean_value() {
        let options: SliceOptions =
            serde_json::from_value(serde_json::json!({ "gcode_label_objects": false })).unwrap();

        assert!(!enabled(&options).unwrap());
    }

    #[test]
    fn option_rejects_non_boolean_values() {
        let options: SliceOptions =
            serde_json::from_value(serde_json::json!({ "gcode_label_objects": "true" })).unwrap();

        let err = enabled(&options).unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert_eq!(err.to_string(), "gcode_label_objects must be a boolean");
    }

    fn options_with_json(value: serde_json::Value) -> SliceOptions {
        serde_json::from_value(value).unwrap()
    }
}
