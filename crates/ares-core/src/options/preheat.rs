use crate::{SliceError, SliceOptions};
use serde_json::Value;

const PREHEAT_TIME: &str = "preheat_time";
const PREHEAT_STEPS: &str = "preheat_steps";
const DEFAULT_PREHEAT_TIME_S: f64 = 30.0;
const MIN_PREHEAT_TIME_S: f64 = 0.0;
const MAX_PREHEAT_TIME_S: f64 = 120.0;
const DEFAULT_PREHEAT_STEPS: u32 = 1;
const MIN_PREHEAT_STEPS: u32 = 1;
const MAX_PREHEAT_STEPS: u32 = 10;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PreheatOptions {
    time_s: f64,
    steps: u32,
}

impl PreheatOptions {
    pub(crate) const fn time_s(self) -> f64 {
        self.time_s
    }

    pub(crate) const fn steps(self) -> u32 {
        self.steps
    }

    pub(crate) fn consume_runtime(self) {
        let _ = (self.time_s(), self.steps());
    }
}

impl SliceOptions {
    pub(crate) fn preheat_options(&self) -> Result<PreheatOptions, SliceError> {
        Ok(PreheatOptions {
            time_s: self.range_f64(
                PREHEAT_TIME,
                DEFAULT_PREHEAT_TIME_S,
                MIN_PREHEAT_TIME_S,
                MAX_PREHEAT_TIME_S,
            )?,
            steps: parse_preheat_steps(self.values().get(PREHEAT_STEPS))?,
        })
    }
}

fn parse_preheat_steps(value: Option<&Value>) -> Result<u32, SliceError> {
    let Some(value) = value else {
        return Ok(DEFAULT_PREHEAT_STEPS);
    };
    let parsed = match value {
        Value::Number(number) => number.as_u64().and_then(|value| u32::try_from(value).ok()),
        Value::String(text) => text.parse::<u32>().ok(),
        _ => None,
    }
    .ok_or_else(|| SliceError::InvalidInput(format!("{PREHEAT_STEPS} must be an integer")))?;
    if (MIN_PREHEAT_STEPS..=MAX_PREHEAT_STEPS).contains(&parsed) {
        Ok(parsed)
    } else {
        Err(SliceError::InvalidInput(format!(
            "{PREHEAT_STEPS} is out of range"
        )))
    }
}
