use super::super::{fixture, zaa_contour_z};
use serde_json::json;
use std::io::{Cursor, Read, Write};

#[derive(Clone, Copy)]
pub(super) enum Case {
    Alternate,
    NonAlternating,
    DisabledZaa,
}

impl Case {
    pub(super) fn name(self) -> &'static str {
        match self {
            Self::Alternate => "flag-false",
            Self::NonAlternating => "flag-true",
            Self::DisabledZaa => "disabled-zaa-flag-true",
        }
    }

    pub(super) fn enabled(self) -> bool {
        !matches!(self, Self::DisabledZaa)
    }

    pub(super) fn flag(self) -> bool {
        !matches!(self, Self::Alternate)
    }
}

pub(super) fn prepare(base: &[u8], case: Case) -> (Vec<u8>, serde_json::Value) {
    let (roof, roof_proof) = zaa_contour_z::wedge(base, case.enabled());
    let mut input = zip::ZipArchive::new(Cursor::new(&roof)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let mut unchanged = serde_json::Map::new();
    for index in 0..input.len() {
        let mut file = input.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == "Metadata/project_settings.config" {
            let mut settings: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
            settings["zaa_dont_alternate_fill_direction"] =
                json!(if case.flag() { "1" } else { "0" });
            settings["zaa_min_z"] = json!("0.05");
            settings["zaa_minimize_perimeter_height"] = json!("35");
            bytes = serde_json::to_vec(&settings).unwrap();
        } else {
            unchanged.insert(file.name().into(), json!(fixture::hash(&bytes)));
        }
        output
            .start_file(file.name(), zip::write::SimpleFileOptions::DEFAULT)
            .unwrap();
        output.write_all(&bytes).unwrap();
    }
    let project = output.finish().unwrap().into_inner();
    let loaded = ares_core::load_project(&project).unwrap();
    assert_eq!(loaded.objects().len(), 1);
    assert_eq!(loaded.objects().first().unwrap().instances().len(), 1);
    assert_eq!(
        loaded.settings().process.region.zaa_enabled.0,
        case.enabled()
    );
    assert_eq!(
        loaded
            .settings()
            .process
            .region
            .zaa_dont_alternate_fill_direction
            .0,
        case.flag()
    );
    let mut final_input = zip::ZipArchive::new(Cursor::new(&project)).unwrap();
    for (name, expected) in &unchanged {
        let mut bytes = Vec::new();
        final_input
            .by_name(name)
            .unwrap()
            .read_to_end(&mut bytes)
            .unwrap();
        assert_eq!(json!(fixture::hash(&bytes)), *expected);
    }
    let proof = json!({"case":case.name(),"zaa_enabled":case.enabled(),
        "dont_alternate_fill_direction":case.flag(),"roof":roof_proof,
        "unchanged_roof_members":unchanged});
    (project, proof)
}
