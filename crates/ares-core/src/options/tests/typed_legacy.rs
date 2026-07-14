use std::collections::BTreeSet;

use super::super::typed_legacy::{EXPLICIT_RULES, LegacyRule};

mod direct_feature;
mod convert;
mod fixture_oracle;
mod inventory;
mod project;
mod thumbnails;

fn rule(source: &str) -> &'static LegacyRule {
    EXPLICIT_RULES
        .iter()
        .find(|rule| rule.source == source)
        .unwrap_or_else(|| panic!("missing typed legacy rule for {source}"))
}

fn source_names() -> BTreeSet<&'static str> {
    EXPLICIT_RULES.iter().map(|rule| rule.source).collect()
}
