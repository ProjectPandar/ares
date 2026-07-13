#[cfg(test)]
use super::object_fields::OBJECT_OPTION_DECLARATION_ORDER;
use super::{
    FloatOrPercent, OrcaBool, OrcaFloat, OrcaInt, Percent, ProcessBrimType,
    ProcessExtraBridgeLayer, ProcessGapFillTarget, ProcessInfillPattern,
    ProcessInternalBridgeFilter, ProcessObjectSourceOptions, ProcessPerimeterGenerator,
    ProcessSeamPosition, ProcessSlicingMode, ProcessSupportBasePattern,
    ProcessSupportInterfacePattern, ProcessSupportStyle, ProcessSupportType,
    object_fields::object_option_fields,
};

mod overrides;

macro_rules! declare_object_options {
    ($($field:ident => $key:literal: $ty:ty = $default:expr),* $(,)?) => {
        #[derive(Clone, Debug, PartialEq)]
        pub struct ObjectOptions {
            $(pub $field: $ty),*
        }

        #[derive(Clone, Debug, Default, PartialEq)]
        pub(crate) struct ObjectOptionOverrides {
            $(pub(crate) $field: Option<$ty>),*
        }

        impl ObjectOptions {
            pub(crate) fn from_base(base: &ProcessObjectSourceOptions) -> Self {
                Self {
                    $($field: base.$field.clone()),*
                }
            }

            pub(crate) fn overlay(
                base: &ProcessObjectSourceOptions,
                overrides: &ObjectOptionOverrides,
            ) -> Self {
                let mut result = Self::from_base(base);
                $(
                    if let Some(value) = &overrides.$field {
                        result.$field = value.clone();
                    }
                )*
                result
            }

            #[cfg_attr(not(test), allow(dead_code))]
            pub(crate) fn resolve(
                base: &ProcessObjectSourceOptions,
                overrides: &ObjectOptionOverrides,
                num_extruders: usize,
            ) -> Self {
                let mut result = Self::overlay(base, overrides);
                let num_extruders = num_extruders as i32;
                if result.support_filament.0 > num_extruders {
                    result.support_filament = OrcaInt(1);
                }
                if result.support_interface_filament.0 > num_extruders {
                    result.support_interface_filament = OrcaInt(1);
                }
                result
            }
        }
    };
}

object_option_fields!(declare_object_options);

impl ObjectOptions {
    #[cfg(test)]
    pub(crate) const DECLARATION_ORDER: [&'static str; 126] = OBJECT_OPTION_DECLARATION_ORDER;
}
