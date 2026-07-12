use serde_json::Value;

use crate::SliceError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum InternalBridgeFilter {
    Disabled,
    Limited,
    NoFilter,
}

impl InternalBridgeFilter {
    pub(crate) fn parse(value: Option<&Value>) -> Result<Self, SliceError> {
        let Some(value) = value else {
            return Ok(Self::Disabled);
        };
        let Some(text) = value.as_str() else {
            return Err(invalid());
        };
        match text {
            "disabled" => Ok(Self::Disabled),
            "limited" => Ok(Self::Limited),
            "nofilter" => Ok(Self::NoFilter),
            _ => Err(invalid()),
        }
    }

    #[cfg(test)]
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::Limited => "limited",
            Self::NoFilter => "nofilter",
        }
    }
}

fn invalid() -> SliceError {
    SliceError::InvalidInput("dont_filter_internal_bridges has unknown enum value".to_owned())
}
