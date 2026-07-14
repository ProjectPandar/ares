use super::LegacyOutcome;
use crate::options::typed_legacy::OBSOLETE_INPUTS;

pub(crate) fn transform_obsolete(source: &str) -> Option<LegacyOutcome> {
    OBSOLETE_INPUTS
        .contains(&source)
        .then_some(LegacyOutcome::Consume)
}
