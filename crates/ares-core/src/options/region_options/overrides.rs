#![cfg_attr(not(test), allow(dead_code))]

use crate::SliceError;

use super::super::{
    FloatOrPercent, OrcaBool, OrcaFloat, OrcaInt, OrcaInts, OrcaString, OrcaStrings, Percent,
    ProcessCounterboreHoleBridging, ProcessEnsureVerticalShellThickness, ProcessFuzzySkinMode,
    ProcessFuzzySkinType, ProcessInfillPattern, ProcessIroningType, ProcessNoiseType,
    ProcessSeamScarfType, ProcessWallDirection, ProcessWallSequence,
    region_fields::region_option_fields,
};
use super::metadata::RegionMetadataCodec;

fn deserialize_value<T>(key: &str, value: &str) -> Result<T, SliceError>
where
    T: RegionMetadataCodec,
{
    T::deserialize_metadata(value).map_err(|error| {
        SliceError::InvalidInput(format!("invalid Orca region option {key}: {error}"))
    })
}

macro_rules! declare_region_option_overrides {
    ($($field:ident => $key:literal: $ty:ty = $default:expr),* $(,)?) => {
        #[derive(Clone, Debug, Default, PartialEq)]
        pub(crate) struct RegionOptionOverrides {
            $(pub(crate) $field: Option<$ty>),*,
            pub(crate) extruder: Option<OrcaInt>,
        }

        impl RegionOptionOverrides {
            pub(crate) fn deserialize_known_field(
                &mut self,
                key: &str,
                value: &str,
            ) -> Result<bool, SliceError> {
                match key {
                    $($key => {
                        self.$field = Some(deserialize_value(key, value)?);
                        Ok(true)
                    }),*,
                    "extruder" => {
                        self.extruder = Some(deserialize_value(key, value)?);
                        Ok(true)
                    }
                    _ => Ok(false),
                }
            }

            #[cfg(test)]
            pub(crate) fn present_keys(&self) -> Vec<&'static str> {
                let mut keys = Vec::new();
                $(if self.$field.is_some() { keys.push($key); })*
                if self.extruder.is_some() {
                    keys.push("extruder");
                }
                keys
            }
        }
    };
}

region_option_fields!(declare_region_option_overrides);
