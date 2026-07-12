use crate::{SliceError, SliceOptions};
use serde_json::Value;

const TIMELAPSE_TYPE: &str = "timelapse_type";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TimelapseType {
    Traditional,
    Smooth,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct TimelapseTypeOptions {
    mode: TimelapseType,
}

impl TimelapseTypeOptions {
    pub(crate) const fn mode(self) -> TimelapseType {
        self.mode
    }

    pub(crate) fn consume_runtime(self) {
        let _ = self.mode();
    }
}

impl SliceOptions {
    pub(crate) fn timelapse_type_options(&self) -> Result<TimelapseTypeOptions, SliceError> {
        Ok(TimelapseTypeOptions {
            mode: parse_timelapse_type(self.values().get(TIMELAPSE_TYPE))?,
        })
    }
}

fn parse_timelapse_type(value: Option<&Value>) -> Result<TimelapseType, SliceError> {
    let Some(value) = value else {
        return Ok(TimelapseType::Traditional);
    };
    let Some(value) = value.as_str() else {
        return Err(SliceError::InvalidInput(format!(
            "{TIMELAPSE_TYPE} must be an Orca enum string"
        )));
    };
    match value {
        "0" | "2" => Ok(TimelapseType::Traditional),
        "1" => Ok(TimelapseType::Smooth),
        _ => Err(SliceError::InvalidInput(format!(
            "{TIMELAPSE_TYPE} contains invalid value"
        ))),
    }
}
