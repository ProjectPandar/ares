use super::LegacyOutcome;
use crate::options::typed_legacy::is_obsolete_input;

pub(crate) fn transform_obsolete(source: &str) -> Option<LegacyOutcome> {
    is_obsolete_input(source).then_some(LegacyOutcome::Consume)
}
