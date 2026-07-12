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
}

mod definitions;
mod key_lists;

pub use key_lists::{
    extruder_option_keys, extruder_retract_keys, filament_option_keys,
    filament_options_with_variant, filament_retract_keys, print_options_with_variant,
    printer_extruder_options, printer_options_with_variant_1, printer_options_with_variant_2,
};

use definitions::OPTION_DEFINITIONS;

pub const fn option_definitions() -> &'static [OptionDefinition] {
    OPTION_DEFINITIONS
}

pub fn option_definition(key: &str) -> Option<&'static OptionDefinition> {
    OPTION_DEFINITIONS
        .binary_search_by_key(&key, |definition| definition.key)
        .ok()
        .map(|index| &OPTION_DEFINITIONS[index])
}

#[cfg(test)]
mod tests;
