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
        ("year", i64::from(year)),
        ("month", i64::from(month)),
        ("day", i64::from(day)),
        ("hour", i64::from(hour)),
        ("minute", i64::from(minute)),
        ("second", i64::from(second)),
    ] {
        config.insert(name, Value::Integer(number));
    }
}
