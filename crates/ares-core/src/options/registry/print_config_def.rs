use std::sync::LazyLock;

use serde::Serialize;

use super::key_lists;
use super::{OptionMetadata, option_metadata_table};

/// One entry of the `printConfigDef` export. Field names and value vocabulary
/// match the schema negotiated with the web consumer; `label`, `full_label`,
/// `category`, `tooltip` and `sidetext` carry the upstream English msgid
/// (untranslated).
#[derive(Serialize)]
struct OptionDefEntry {
    key: &'static str,
    /// OptionValueKind snake_case. Vector variants end in `s` (floats, bools,
    /// ...); `*_nullable` vectors accept `nil` elements.
    #[serde(rename = "type")]
    value_kind: &'static str,
    /// Serialized default in the crate's deserialize-compatible encoding
    /// (the same strings OPTION_DEFINITIONS has always carried; vectors are
    /// `,`-joined except `strings`/`points_groups` which are `;`-joined).
    default_value: &'static str,
    nullable: bool,
    label: Option<&'static str>,
    full_label: Option<&'static str>,
    category: Option<&'static str>,
    tooltip: Option<&'static str>,
    sidetext: Option<&'static str>,
    enum_values: &'static [&'static str],
    /// Upstream leaves this empty for options whose labels equal the values;
    /// consumers fall back to `enum_values` (Field.cpp behaviour).
    enum_labels: &'static [&'static str],
    /// Integer codes from the upstream `enum_keys_map`; parallel to
    /// `enum_values` when the option defines a keys map.
    enum_keys: &'static [i32],
    /// Element-wise bound; for vector options it applies to each element.
    min: Option<f64>,
    max: Option<f64>,
    /// `simple|advanced|expert|develop` (ConfigOptionMode).
    mode: &'static str,
    /// `undefined|i_enum_open|f_enum_open|color|select_open|slider|legend|one_string`.
    gui_type: &'static str,
    gui_flags: &'static str,
    multiline: bool,
    full_width: bool,
    is_code: bool,
    readonly: bool,
    height: Option<f64>,
    width: Option<f64>,
    ratio_over: Option<&'static str>,
    max_literal: Option<f64>,
    aliases: &'static [&'static str],
    shortcut: &'static [&'static str],
    /// `any` (shared), `fff` or `sla` (PrinterTechnology).
    printer_technology: &'static str,
}

#[derive(Serialize)]
struct KeyGroups<'a> {
    extruder_option_keys: &'a [&'a str],
    filament_option_keys: &'a [&'a str],
    extruder_retract_keys: &'a [&'a str],
    filament_retract_keys: &'a [&'a str],
}

#[derive(Serialize)]
struct PrintConfigDefExport<'a> {
    /// Sorted by `key`.
    options: Vec<OptionDefEntry>,
    groups: KeyGroups<'a>,
}

fn entry(meta: &'static OptionMetadata) -> OptionDefEntry {
    let definition = super::option_definition(meta.key);
    OptionDefEntry {
        key: meta.key,
        value_kind: definition.map(|d| d.kind.json_name()).unwrap_or("unknown"),
        default_value: definition.map(|d| d.default_value).unwrap_or(""),
        nullable: meta.nullable,
        label: meta.label,
        full_label: meta.full_label,
        category: meta.category,
        tooltip: meta.tooltip,
        sidetext: meta.sidetext,
        enum_values: meta.enum_values,
        enum_labels: meta.enum_labels,
        enum_keys: meta.enum_keys,
        min: meta.min,
        max: meta.max,
        mode: meta.mode.json_name(),
        gui_type: meta.gui_type.json_name(),
        gui_flags: meta.gui_flags,
        multiline: meta.multiline,
        full_width: meta.full_width,
        is_code: meta.is_code,
        readonly: meta.readonly,
        height: meta.height,
        width: meta.width,
        ratio_over: meta.ratio_over,
        max_literal: meta.max_literal,
        aliases: meta.aliases,
        shortcut: meta.shortcut,
        printer_technology: meta.printer_technology,
    }
}

static JSON: LazyLock<Vec<u8>> = LazyLock::new(|| {
    let export = PrintConfigDefExport {
        options: option_metadata_table().iter().map(entry).collect(),
        groups: KeyGroups {
            extruder_option_keys: key_lists::extruder_option_keys(),
            filament_option_keys: key_lists::filament_option_keys(),
            extruder_retract_keys: key_lists::extruder_retract_keys(),
            filament_retract_keys: key_lists::filament_retract_keys(),
        },
    };
    serde_json::to_vec(&export).expect("PrintConfigDef metadata serialization is infallible")
});

/// Serialized `PrintConfigDef` metadata (UTF-8 JSON), computed once.
pub fn print_config_def_json() -> &'static [u8] {
    &JSON
}
