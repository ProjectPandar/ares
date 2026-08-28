use crate::GenerationMetadata;

use super::super::value::{self, Value};

pub(super) fn insert(config: &mut value::Config, metadata: GenerationMetadata) {
    let (year, month, day, hour, minute, second) = metadata.timestamp();
    config.insert(
        "timestamp",
        Value::String(format!(
            "{year:04}{month:02}{day:02}-{hour:02}{minute:02}{second:02}"
        )),
    );
    for (name, number) in [
        ("year", year as f64),
        ("month", f64::from(month)),
        ("day", f64::from(day)),
        ("hour", f64::from(hour)),
        ("minute", f64::from(minute)),
        ("second", f64::from(second)),
    ] {
        config.insert(name, Value::number(number));
    }
}
