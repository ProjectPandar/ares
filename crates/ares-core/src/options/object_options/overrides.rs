use serde::{
    Deserialize,
    de::value::{BorrowedStrDeserializer, Error as ValueError},
};

use crate::SliceError;

use super::{
    FloatOrPercent, ObjectOptionOverrides, OrcaBool, OrcaFloat, OrcaInt, Percent, ProcessBrimType,
    ProcessExtraBridgeLayer, ProcessGapFillTarget, ProcessInfillPattern,
    ProcessInternalBridgeFilter, ProcessPerimeterGenerator, ProcessSeamPosition,
    ProcessSlicingMode, ProcessSupportBasePattern, ProcessSupportInterfacePattern,
    ProcessSupportStyle, ProcessSupportType, object_option_fields,
};

fn deserialize_value<'de, T>(key: &str, value: &'de str) -> Result<T, SliceError>
where
    T: Deserialize<'de>,
{
    match T::deserialize(BorrowedStrDeserializer::<ValueError>::new(value)) {
        Ok(parsed) => Ok(parsed),
        Err(error) => Err(SliceError::InvalidInput(format!(
            "invalid Orca object option {key}: {error}"
        ))),
    }
}

macro_rules! implement_object_option_overrides {
    ($($field:ident => $key:literal: $ty:ty = $default:expr),* $(,)?) => {
        impl ObjectOptionOverrides {
            pub(crate) fn deserialize_known_field(
                &mut self,
                key: &str,
                value: &str,
            ) -> Result<bool, SliceError> {
                match key {
                    $($key => {
                        let parsed: $ty = deserialize_value(key, value)?;
                        self.$field = Some(parsed);
                        Ok(true)
                    }),*
                    _ => Ok(false),
                }
            }
        }
    };
}

object_option_fields!(implement_object_option_overrides);
