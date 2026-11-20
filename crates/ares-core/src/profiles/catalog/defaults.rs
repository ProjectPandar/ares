//! PresetCollection / PresetBundle FFF construction: Preset.cpp:1766-1824,
//! PresetBundle.cpp:484-554. Defaults are loaded records, not vendor resources.

use super::{Catalog, ProfileDocument, Record, Scope};
use crate::profiles::{ProfileKind, project_refresh::default_preset_fields};

pub(super) fn records() -> Vec<Record> {
    [
        (
            ProfileKind::Process,
            "process",
            "Default Setting",
            "print_settings_id",
        ),
        (
            ProfileKind::Filament,
            "filament",
            "Default Filament",
            "filament_settings_id",
        ),
        (
            ProfileKind::Machine,
            "machine",
            "Default Printer",
            "printer_settings_id",
        ),
    ]
    .into_iter()
    .map(|(kind, wire_kind, name, identity)| {
        // The existing owner applies declared membership, registry wire defaults,
        // and the nullable filament constructor overrides, not full-config filtering.
        let mut fields = default_preset_fields(kind)
            .expect("registered collection defaults have valid configuration values");
        let definition = crate::option_definition(identity)
            .expect("constructor identity has a registered definition");
        let value = crate::decode_config_value_by_key(identity, definition.default_wire_value)
            .expect("registered constructor identity has a valid wire default");
        fields.insert(
            identity.to_owned(),
            serde_json::from_str(&value).expect("decoded identity is a configuration value"),
        );
        Record {
            scope: Scope::Default,
            kind,
            document: ProfileDocument {
                kind: wire_kind.to_owned(),
                name: name.to_owned(),
                inherits: Some(String::new()),
                instantiation: None,
                includes: Vec::new(),
                fields,
            },
            source_identity: None,
            direct: false,
        }
    })
    .collect()
}

impl Catalog {
    pub(in crate::profiles) fn default_position(&self, kind: ProfileKind) -> usize {
        self.records
            .iter()
            .position(|record| record.scope == Scope::Default && record.kind == kind)
            .expect("each FFF collection has an initialized default prefix")
    }
}
