use serde::{Serialize, de::DeserializeOwned};

use super::super::super::{
    ProcessDraftShield, ProcessPrintOrder, ProcessPrintSequence, ProcessSkirtType,
    ProcessTimelapseType, ProcessWipeTowerWallType,
};

#[test]
fn process_remaining_enum_domains_accept_only_canonical_machine_tokens() {
    assert_domain::<ProcessDraftShield>(&["disabled", "enabled"], &["limited", "Disabled"]);
    assert_domain::<ProcessPrintOrder>(&["default", "as_obj_list"], &["object list", "as-object-list"]);
    assert_domain::<ProcessPrintSequence>(&["by layer", "by object"], &["By layer", "by_layer"]);
    assert_domain::<ProcessSkirtType>(&["combined", "perobject"], &["per object", "per_object"]);
    assert_domain::<ProcessTimelapseType>(&["0", "1"], &["2", "traditional", "smooth"]);
    assert_domain::<ProcessWipeTowerWallType>(&["rectangle", "cone", "rib"], &["Rectangular", "ribs"]);
}

fn assert_domain<T>(tokens: &[&str], aliases: &[&str])
where
    T: DeserializeOwned + Serialize,
{
    for token in tokens {
        let value: T = serde_json::from_value((*token).into()).unwrap();
        assert_eq!(serde_json::to_value(value).unwrap(), *token);
    }
    for invalid in ["", "__invalid__"].into_iter().chain(aliases.iter().copied()) {
        assert!(serde_json::from_value::<T>(invalid.into()).is_err(), "{invalid}");
    }
}
