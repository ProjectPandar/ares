use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Map, Value};

use crate::{SliceError, SliceOptions};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ProfileKind {
    Process,
    Filament,
    Machine,
}

impl ProfileKind {
    fn parse(value: &str) -> Result<Self, SliceError> {
        match value {
            "process" => Ok(Self::Process),
            "filament" => Ok(Self::Filament),
            "machine" => Ok(Self::Machine),
            _ => Err(SliceError::InvalidInput(
                "profile type is unsupported".to_owned(),
            )),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProfileFragment {
    kind: ProfileKind,
    name: String,
    inherits: Option<String>,
    from: Option<String>,
    setting_id: Option<String>,
    instantiation: Option<String>,
    values: BTreeMap<String, Value>,
}

impl ProfileFragment {
    pub fn from_json_bytes(input: impl AsRef<[u8]>) -> Result<Self, SliceError> {
        let value: Value = serde_json::from_slice(input.as_ref()).map_err(|error| {
            SliceError::InvalidInput(format!("profile JSON is invalid: {error}"))
        })?;
        let Value::Object(values) = value else {
            return Err(SliceError::InvalidInput(
                "profile JSON must be an object".to_owned(),
            ));
        };
        let values = values.into_iter().collect::<BTreeMap<_, _>>();
        let kind = ProfileKind::parse(required_string(&values, "type")?)?;
        let name = required_string(&values, "name")?.to_owned();
        if name.is_empty() {
            return Err(SliceError::InvalidInput(
                "profile name must not be empty".to_owned(),
            ));
        }
        let inherits = optional_string(&values, "inherits")?;
        let from = optional_string(&values, "from")?;
        let setting_id = optional_string(&values, "setting_id")?;
        let instantiation = optional_string(&values, "instantiation")?;

        Ok(Self {
            kind,
            name,
            inherits,
            from,
            setting_id,
            instantiation,
            values,
        })
    }

    pub const fn kind(&self) -> ProfileKind {
        self.kind
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn inherits(&self) -> Option<&str> {
        self.inherits.as_deref()
    }

    pub fn from(&self) -> Option<&str> {
        self.from.as_deref()
    }

    pub fn setting_id(&self) -> Option<&str> {
        self.setting_id.as_deref()
    }

    pub fn instantiation(&self) -> Option<&str> {
        self.instantiation.as_deref()
    }

    pub fn values(&self) -> &BTreeMap<String, Value> {
        &self.values
    }
}

pub fn merge_profile_fragments(
    fragments: &[ProfileFragment],
    target_kind: ProfileKind,
    target_name: &str,
) -> Result<SliceOptions, SliceError> {
    let index = build_index(fragments)?;
    let mut chain = Vec::new();
    let mut visiting = BTreeSet::new();
    let resolver = ChainResolver { fragments, index };
    resolver.collect_chain(target_kind, target_name, &mut visiting, &mut chain)?;

    let mut merged = Map::new();
    for fragment in chain {
        for (key, value) in fragment.values() {
            merged.insert(key.clone(), value.clone());
        }
    }

    serde_json::from_value(Value::Object(merged)).map_err(|error| {
        SliceError::InvalidInput(format!("merged profile options are invalid: {error}"))
    })
}

fn build_index(
    fragments: &[ProfileFragment],
) -> Result<BTreeMap<(ProfileKind, String), usize>, SliceError> {
    let mut index = BTreeMap::new();
    for (position, fragment) in fragments.iter().enumerate() {
        let key = (fragment.kind(), fragment.name().to_owned());
        if index.insert(key, position).is_some() {
            return Err(SliceError::InvalidInput(
                "duplicate profile fragment".to_owned(),
            ));
        }
    }
    Ok(index)
}

struct ChainResolver<'a> {
    fragments: &'a [ProfileFragment],
    index: BTreeMap<(ProfileKind, String), usize>,
}

impl<'a> ChainResolver<'a> {
    fn collect_chain(
        &self,
        kind: ProfileKind,
        name: &str,
        visiting: &mut BTreeSet<(ProfileKind, String)>,
        chain: &mut Vec<&'a ProfileFragment>,
    ) -> Result<(), SliceError> {
        let key = (kind, name.to_owned());
        if !visiting.insert(key.clone()) {
            return Err(SliceError::InvalidInput(
                "profile inheritance cycle".to_owned(),
            ));
        }
        let Some(position) = self.index.get(&key).copied() else {
            return Err(SliceError::InvalidInput(format!(
                "profile '{name}' was not found"
            )));
        };
        let fragment = &self.fragments[position];
        if let Some(parent) = fragment.inherits() {
            self.collect_chain(kind, parent, visiting, chain)?;
        }
        visiting.remove(&key);
        chain.push(fragment);
        Ok(())
    }
}

fn required_string<'a>(
    values: &'a BTreeMap<String, Value>,
    key: &str,
) -> Result<&'a str, SliceError> {
    values
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| SliceError::InvalidInput(format!("profile {key} must be a string")))
}

fn optional_string(
    values: &BTreeMap<String, Value>,
    key: &str,
) -> Result<Option<String>, SliceError> {
    match values.get(key) {
        Some(Value::String(value)) if value.is_empty() => Ok(None),
        Some(Value::String(value)) => Ok(Some(value.clone())),
        Some(_) => Err(SliceError::InvalidInput(format!(
            "profile {key} must be a string"
        ))),
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SliceError;
    use serde_json::json;

    #[test]
    fn parses_profile_fragment_metadata_and_values() {
        let fragment = ProfileFragment::from_json_bytes(
            br#"{"type":"process","name":"child","inherits":"base","from":"system","setting_id":"GP001","instantiation":"false","layer_height":"0.16"}"#,
        )
        .unwrap();

        assert_eq!(fragment.kind(), ProfileKind::Process);
        assert_eq!(fragment.name(), "child");
        assert_eq!(fragment.inherits(), Some("base"));
        assert_eq!(fragment.from(), Some("system"));
        assert_eq!(fragment.setting_id(), Some("GP001"));
        assert_eq!(fragment.instantiation(), Some("false"));
        assert_eq!(fragment.values()["layer_height"], json!("0.16"));
    }

    #[test]
    fn parses_filament_and_machine_kinds() {
        let filament = ProfileFragment::from_json_bytes(
            br#"{"type":"filament","name":"pla","filament_diameter":["1.75"]}"#,
        )
        .unwrap();
        let machine = ProfileFragment::from_json_bytes(
            br#"{"type":"machine","name":"printer","nozzle_diameter":["0.4"]}"#,
        )
        .unwrap();

        assert_eq!(filament.kind(), ProfileKind::Filament);
        assert_eq!(machine.kind(), ProfileKind::Machine);
    }

    #[test]
    fn rejects_invalid_profile_json_and_required_fields() {
        for input in [
            br#"not json"#.as_slice(),
            br#"[]"#,
            br#"{"type":"process"}"#,
            br#"{"name":"missing-type"}"#,
            br#"{"type":"unknown","name":"bad"}"#,
            br#"{"type":"process","name":""}"#,
            br#"{"type":"process","name":"bad","inherits":1}"#,
        ] {
            let err = ProfileFragment::from_json_bytes(input).unwrap_err();

            assert!(matches!(err, SliceError::InvalidInput(_)));
        }
    }

    #[test]
    fn merges_grandparent_parent_child_with_child_overrides() {
        let fragments = [
            fragment(br#"{"type":"process","name":"child","inherits":"parent","layer_height":"0.12","wall_loops":3}"#),
            fragment(br##"{"type":"process","name":"grandparent","layer_height":"0.2","nozzle_diameter":["0.4"],"filament_colour":["#fff"]}"##),
            fragment(br#"{"type":"process","name":"parent","inherits":"grandparent","wall_loops":2,"filament_diameter":["1.75"]}"#),
        ];

        let options = merge_profile_fragments(&fragments, ProfileKind::Process, "child").unwrap();

        assert_eq!(options.values()["name"], json!("child"));
        assert_eq!(options.values()["inherits"], json!("parent"));
        assert_eq!(options.values()["layer_height"], json!("0.12"));
        assert_eq!(options.values()["wall_loops"], json!(3));
        assert_eq!(options.values()["filament_colour"], json!(["#fff"]));
        assert_eq!(options.nozzle_diameters().unwrap(), vec![0.4]);
        assert_eq!(options.filament_diameters().unwrap(), vec![1.75]);
    }

    #[test]
    fn merge_is_independent_of_input_order() {
        let ordered = [
            fragment(br#"{"type":"machine","name":"base","nozzle_diameter":["0.4"]}"#),
            fragment(
                br#"{"type":"machine","name":"child","inherits":"base","nozzle_diameter":["0.6"]}"#,
            ),
        ];
        let reversed = [ordered[1].clone(), ordered[0].clone()];

        let a = merge_profile_fragments(&ordered, ProfileKind::Machine, "child").unwrap();
        let b = merge_profile_fragments(&reversed, ProfileKind::Machine, "child").unwrap();

        assert_eq!(a, b);
        assert_eq!(a.nozzle_diameters().unwrap(), vec![0.6]);
    }

    #[test]
    fn rejects_missing_target_missing_parent_duplicate_and_cross_kind_parent() {
        let duplicate = [
            fragment(br#"{"type":"process","name":"same"}"#),
            fragment(br#"{"type":"process","name":"same"}"#),
        ];
        let missing_parent = [fragment(
            br#"{"type":"process","name":"child","inherits":"missing"}"#,
        )];
        let cross_kind = [
            fragment(br#"{"type":"process","name":"child","inherits":"base"}"#),
            fragment(br#"{"type":"machine","name":"base"}"#),
        ];

        assert_invalid(merge_profile_fragments(
            &[],
            ProfileKind::Process,
            "missing",
        ));
        assert_invalid(merge_profile_fragments(
            &duplicate,
            ProfileKind::Process,
            "same",
        ));
        assert_invalid(merge_profile_fragments(
            &missing_parent,
            ProfileKind::Process,
            "child",
        ));
        assert_invalid(merge_profile_fragments(
            &cross_kind,
            ProfileKind::Process,
            "child",
        ));
    }

    #[test]
    fn rejects_inheritance_cycles() {
        let fragments = [
            fragment(br#"{"type":"filament","name":"a","inherits":"b"}"#),
            fragment(br#"{"type":"filament","name":"b","inherits":"a"}"#),
        ];

        assert_invalid(merge_profile_fragments(
            &fragments,
            ProfileKind::Filament,
            "a",
        ));
    }

    fn fragment(bytes: &[u8]) -> ProfileFragment {
        ProfileFragment::from_json_bytes(bytes).unwrap()
    }

    fn assert_invalid(result: Result<crate::SliceOptions, SliceError>) {
        assert!(matches!(result, Err(SliceError::InvalidInput(_))));
    }
}
