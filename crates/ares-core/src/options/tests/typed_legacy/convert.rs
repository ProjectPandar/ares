use super::rule;
use crate::options::typed_legacy::{
    LegacyOutcome, LegacyTransformError, transform_json_array, transform_lexical,
};

mod arrays;
mod conditional;
mod rewrites;

fn scalar(source: &str, value: &str) -> LegacyOutcome {
    transform_lexical(rule(source), value)
}

fn array(source: &str, json: &str) -> LegacyOutcome {
    let mut deserializer = serde_json::Deserializer::from_str(json);
    transform_json_array(rule(source), &mut deserializer)
}

fn assert_assign(outcome: LegacyOutcome, target: &'static str, value: &str) {
    assert_eq!(
        outcome,
        LegacyOutcome::Assign {
            target,
            value: value.to_owned(),
        }
    );
}

fn assert_invalid_array(outcome: LegacyOutcome, source: &'static str) {
    assert_eq!(
        outcome,
        LegacyOutcome::Error(LegacyTransformError::InvalidArrayValue { source })
    );
}
