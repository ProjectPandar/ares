use super::{
    FloatOrPercent, OrcaBool, OrcaFloat, OrcaFloatOrPercentsNullable, OrcaFloatsNullable, OrcaInt,
    OrcaString, Percent, ProcessBrimType, ProcessExtraBridgeLayer, ProcessGapFillTarget,
    ProcessInfillPattern, ProcessInternalBridgeFilter, ProcessObjectSourceOptions,
    ProcessPerimeterGenerator, ProcessSeamPosition, ProcessSlicingMode, ProcessSupportBasePattern,
    ProcessSupportInterfacePattern, ProcessSupportStyle, ProcessSupportType,
    object_fields::object_option_fields,
};

mod overrides;

macro_rules! overlay_field {
    ($result:ident, $source:ident, $indices:ident, support_speed) => {
        overlay_variant!($result, $source, $indices, support_speed);
    };
    ($result:ident, $source:ident, $indices:ident, support_interface_speed) => {
        overlay_variant!($result, $source, $indices, support_interface_speed);
    };
    ($result:ident, $source:ident, $indices:ident, $field:ident) => {
        if let Some(value) = &$source.$field {
            $result.$field = value.clone();
        }
    };
}

macro_rules! overlay_variant {
    ($result:ident, $source:ident, $indices:ident, $field:ident) => {
        if let Some(value) = &$source.$field
            && !$result.$field.source_equals(value)
        {
            // PrintObject.cpp:4289–4295 retains source-column indices for overrides.
            super::project_variants::apply_nullable_vector(
                &mut $result.$field.0,
                &value.0,
                $indices,
            );
        }
    };
}

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
                process_variant_indices: &[usize],
            ) -> Self {
                let mut result = Self::from_base(base);
                $(overlay_field!(result, overrides, process_variant_indices, $field);)*
                result
            }

            #[cfg_attr(not(test), allow(dead_code))]
            pub(crate) fn resolve(
                base: &ProcessObjectSourceOptions,
                overrides: &ObjectOptionOverrides,
                num_extruders: usize,
                process_variant_indices: &[usize],
            ) -> Self {
                let mut result = Self::overlay(base, overrides, process_variant_indices);
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

impl ObjectOptions {}
