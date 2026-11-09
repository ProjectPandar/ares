//! bbs_3mf.cpp:367,525-541,4023-4052,5927: unbound face_property.
//! Orient.cpp:209-227,427-428: appearance affects orientation, not saved transforms.

use std::{
    fs,
    io::{Cursor, Read, Write},
};

use crate::{
    OrcaRunner, golden,
    runner::{
        native_api_cli::{NativeCase, sha},
        native_stream_case,
    },
};

const CONTROL: &[u8] = include_bytes!(
    "../../../../tests/parity/machine-acceleration-cache-rounding/stock-control.3mf"
);
const UPSTREAM: &[&str] = &[
    "bbs_3mf.cpp:367,525-541,4023-4052,5927",
    "Orient.cpp:209-227,427-428",
];

#[test]
fn neutral_face_property_fixed_transform_complete_parity() {
    check("neutral", "");
}

#[test]
fn exterior_face_property_fixed_transform_complete_parity() {
    // EnumFaceTypes::eExteriorAppearance, deps_src/admesh/stl.h:51-56.
    check("exterior", "1");
}

#[test]
fn face_property_namespace_and_element_rejections_are_retained() {
    for (element, attribute) in [
        ("triangle", "xml:face_property=\"\""),
        ("vertex", "face_property=\"\""),
    ] {
        let project = with_attribute(element, attribute);
        let load = ares_core::load_project(&project).unwrap_err().to_string();
        assert_eq!(
            load,
            "invalid project model XML: attribute namespace does not match its 3MF meaning"
        );
        // Identity scans the main model, not the referenced geometry resource.
        assert!(
            ares_core::UpwardMachineSource::from_project(&project)
                .unwrap()
                .is_some()
        );
    }
}

fn check(label: &str, value: &str) {
    let project = with_attribute("triangle", &format!("face_property=\"{value}\""));
    let (reference, _) = native_stream_case::qualify(
        ("project-face-property", UPSTREAM),
        label,
        &project,
        &sha(&project),
        false,
    );
    let runner = OrcaRunner::from_env().unwrap().unwrap();
    let control = NativeCase::slice(&runner, CONTROL, "project-face-property-control-");
    let native = golden::normalize_one_generator_line(
        reference.native.as_bytes(),
        golden::GeneratorKind::Orca,
    )
    .unwrap();
    let contiguous = golden::normalize_one_generator_line(
        control.native.as_bytes(),
        golden::GeneratorKind::Orca,
    )
    .unwrap();
    let difference = golden::first_difference(&contiguous, &native).map(|value| value.to_string());
    fs::write(
        reference.root.join("native-control-comparison.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "control_sha256": sha(CONTROL), "variant_sha256": sha(&project),
            "control_root": control.root, "variant_root": reference.root,
            "native_control_difference": difference,
            "complete_ordered_comparison": true, "orient_requested": false,
        }))
        .unwrap(),
    )
    .unwrap();
    assert!(
        difference.is_none(),
        "Native fixed-transform difference: {difference:?}"
    );
    let imported = ares_core::load_project(&project);
    let identity = ares_core::UpwardMachineSource::from_project(&project);
    fs::write(
        reference.root.join("imports.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "project_error": imported.as_ref().err().map(ToString::to_string),
            "identity_error": identity.as_ref().err().map(ToString::to_string),
        }))
        .unwrap(),
    )
    .unwrap();
    reference.compare(ares_core::GenerationMetadata::deterministic(
        2026, 8, 27, 0, 0, 0,
    ));
    let imported = imported.unwrap();
    let mesh = imported.objects()[0].volumes()[0].mesh();
    let properties = mesh.face_properties();
    assert_eq!(properties.len(), mesh.triangles().len());
    assert_eq!(properties[0].as_ref(), value);
    assert!(properties[1..].iter().all(|value| value.is_empty()));
    assert_eq!(mesh.clone().face_properties(), properties);
    assert!(identity.unwrap().is_some());
}

fn with_attribute(element: &str, attribute: &str) -> Vec<u8> {
    const MEMBER: &str = "3D/Objects/cube10.stl_1.model";
    let mut archive = zip::ZipArchive::new(Cursor::new(CONTROL)).unwrap();
    let mut xml = String::new();
    archive
        .by_name(MEMBER)
        .unwrap()
        .read_to_string(&mut xml)
        .unwrap();
    let original = format!("<{element} ");
    assert!(xml.contains(&original));
    let changed = xml.replacen(&original, &format!("<{element} {attribute} "), 1);
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..archive.len() {
        let entry = archive.by_index(index).unwrap();
        if entry.name() == MEMBER {
            writer
                .start_file(MEMBER, zip::write::SimpleFileOptions::default())
                .unwrap();
            writer.write_all(changed.as_bytes()).unwrap();
        } else {
            writer.raw_copy_file(entry).unwrap();
        }
    }
    writer.finish().unwrap().into_inner()
}
