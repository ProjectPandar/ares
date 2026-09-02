use std::collections::BTreeSet;

use serde::{Serialize, Serializer, ser::SerializeMap};

use crate::{ProjectSettings, options::config_export::collector};

use super::super::project_fixture::project_settings_bytes;

#[test]
fn config_export_inventory_collects_the_four_typed_groups_once() {
    let settings: ProjectSettings = serde_json::from_slice(&project_settings_bytes()).unwrap();
    let entries = collector::collect_config_entries(&settings).unwrap();

    assert_eq!(entries.len(), 650);
    assert!(entries.windows(2).all(|pair| pair[0].key < pair[1].key));
    assert_eq!(
        entries
            .iter()
            .map(|entry| entry.key.as_str())
            .collect::<BTreeSet<_>>()
            .len(),
        650
    );
    for metadata_key in ["from", "name", "version"] {
        assert!(entries.iter().all(|entry| entry.key != metadata_key));
    }
}

#[test]
fn config_export_inventory_preserves_complete_group_json_bytes() {
    let settings: ProjectSettings = serde_json::from_slice(&project_settings_bytes()).unwrap();
    let actual = [
        ("printer", serde_json::to_vec(&settings.printer).unwrap()),
        ("process", serde_json::to_vec(&settings.process).unwrap()),
        ("filament", serde_json::to_vec(&settings.filament).unwrap()),
        ("project", serde_json::to_vec(&settings.project).unwrap()),
    ]
    .map(|(name, bytes)| (name, bytes.len(), fnv1a64(&bytes)));

    assert_eq!(
        actual,
        [
            ("printer", 32_945, 28_122_473_580_188_592_08),
            ("process", 10_823, 12_386_891_994_242_524_541),
            ("filament", 6_802, 4_742_530_254_364_489_637),
            ("project", 1_828, 13_102_796_653_342_733_105),
        ]
    );
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf29ce484222325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
    })
}

#[test]
fn config_export_inventory_rejects_duplicate_canonical_keys() {
    struct DuplicateKeys;

    impl Serialize for DuplicateKeys {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let mut map = serializer.serialize_map(Some(2))?;
            map.serialize_entry("duplicate", "first")?;
            map.serialize_entry("duplicate", "second")?;
            map.end()
        }
    }

    let error = collector::collect_serializable_for_test(&DuplicateKeys).unwrap_err();
    assert_eq!(error.to_string(), "duplicate config key duplicate");
}
