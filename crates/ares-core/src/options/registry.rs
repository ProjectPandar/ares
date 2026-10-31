#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OptionValueKind {
    Float,
    FloatOrPercent,
    Percent,
    Percents,
    PercentsNullable,
    Int,
    Bool,
    Bools,
    BoolsNullable,
    Enum,
    Enums,
    EnumsNullable,
    Floats,
    FloatsNullable,
    IntsNullable,
    Ints,
    Strings,
    String,
    Point,
    Points,
    PointsGroups,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OptionDefinition {
    pub key: &'static str,
    pub kind: OptionValueKind,
    pub default_value: &'static str,
    /// `ConfigOptionEnum<T>` type-map keys when wider than the option's own
    /// `enum_values` whitelist (`ConfigOptionEnum::from_string`,
    /// `Config.hpp:2032-2040`); `None` when the whitelist is the type map.
    pub enum_type_keys: Option<&'static [&'static str]>,
}

impl OptionValueKind {
    /// Stable snake_case name used by the `printConfigDef` JSON export.
    pub const fn json_name(self) -> &'static str {
        match self {
            Self::Float => "float",
            Self::FloatOrPercent => "float_or_percent",
            Self::Percent => "percent",
            Self::Percents => "percents",
            Self::PercentsNullable => "percents_nullable",
            Self::Int => "int",
            Self::Bool => "bool",
            Self::Bools => "bools",
            Self::BoolsNullable => "bools_nullable",
            Self::Enum => "enum",
            Self::Enums => "enums",
            Self::EnumsNullable => "enums_nullable",
            Self::Floats => "floats",
            Self::FloatsNullable => "floats_nullable",
            Self::IntsNullable => "ints_nullable",
            Self::Ints => "ints",
            Self::Strings => "strings",
            Self::String => "string",
            Self::Point => "point",
            Self::Points => "points",
            Self::PointsGroups => "points_groups",
        }
    }
}

/// Upstream `ConfigOptionMode` (Config.hpp). Ordering matches comSimple..
/// comDevelop so `>=` comparisons keep upstream semantics.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum OptionMode {
    Simple,
    Advanced,
    Expert,
    Develop,
}

impl OptionMode {
    pub const fn json_name(self) -> &'static str {
        match self {
            Self::Simple => "simple",
            Self::Advanced => "advanced",
            Self::Expert => "expert",
            Self::Develop => "develop",
        }
    }
}

/// Upstream `ConfigOptionDef::GUIType` (Config.hpp).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OptionGuiType {
    Undefined,
    IEnumOpen,
    FEnumOpen,
    Color,
    SelectOpen,
    Slider,
    Legend,
    OneString,
}

impl OptionGuiType {
    pub const fn json_name(self) -> &'static str {
        match self {
            Self::Undefined => "undefined",
            Self::IEnumOpen => "i_enum_open",
            Self::FEnumOpen => "f_enum_open",
            Self::Color => "color",
            Self::SelectOpen => "select_open",
            Self::Slider => "slider",
            Self::Legend => "legend",
            Self::OneString => "one_string",
        }
    }
}

/// Presentation metadata for one option, generated from upstream
/// `PrintConfigDef::init_fff_params`/`init_common_params` (`ConfigOptionDef`).
/// All data is `&'static`; the table is sorted by `key` for binary search.
#[derive(Clone, Copy, Debug)]
pub struct OptionMetadata {
    pub key: &'static str,
    pub label: Option<&'static str>,
    pub full_label: Option<&'static str>,
    pub category: Option<&'static str>,
    /// Upstream msgid (untranslated English source string).
    pub tooltip: Option<&'static str>,
    pub sidetext: Option<&'static str>,
    pub enum_values: &'static [&'static str],
    pub enum_labels: &'static [&'static str],
    /// Integer codes from the upstream `enum_keys_map`.
    pub enum_keys: &'static [i32],
    /// Element-wise bound for vector options (upstream semantics).
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub max_literal: Option<f64>,
    pub mode: OptionMode,
    pub gui_type: OptionGuiType,
    pub gui_flags: &'static str,
    pub multiline: bool,
    pub full_width: bool,
    pub is_code: bool,
    pub readonly: bool,
    pub nullable: bool,
    pub height: Option<f64>,
    pub width: Option<f64>,
    /// Key this option's value is a ratio of (e.g. line width vs nozzle diameter).
    pub ratio_over: Option<&'static str>,
    pub aliases: &'static [&'static str],
    pub shortcut: &'static [&'static str],
    /// `any` (common), `fff`, or `sla` (unused in this slice).
    pub printer_technology: &'static str,
}

mod definitions;
mod key_lists;
mod print_config_def;

pub use key_lists::{
    extruder_option_keys, extruder_retract_keys, filament_option_keys,
    filament_options_with_variant, filament_retract_keys, print_options_with_variant,
    printer_extruder_options, printer_options_with_variant_1, printer_options_with_variant_2,
};

use definitions::METADATA;
use definitions::OPTION_DEFINITIONS;

pub use print_config_def::print_config_def_json;

pub const fn option_definitions() -> &'static [OptionDefinition] {
    OPTION_DEFINITIONS
}

pub fn option_definition(key: &str) -> Option<&'static OptionDefinition> {
    OPTION_DEFINITIONS
        .binary_search_by_key(&key, |definition| definition.key)
        .ok()
        .map(|index| &OPTION_DEFINITIONS[index])
}

/// Metadata rows sorted by `key`, in the same keyspace as OPTION_DEFINITIONS
/// (minus SLA-only options).
pub const fn option_metadata_table() -> &'static [OptionMetadata] {
    METADATA
}

pub fn option_metadata(key: &str) -> Option<&'static OptionMetadata> {
    METADATA
        .binary_search_by_key(&key, |m| m.key)
        .ok()
        .map(|index| &METADATA[index])
}
