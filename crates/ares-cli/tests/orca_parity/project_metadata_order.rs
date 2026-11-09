//! bbs_3mf.cpp:3582-3593,4142-4160,3655-3673,4577-4626:
//! metadata dispatch depends on parent state, not sibling adjacency.

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
const CONTROL_SHA: &str = "b1a3ee564b1e78d1b87f6957f2177f6cd528906ce4790218a0dca410b4191bb8";
const UPSTREAM: &[&str] = &[
    "bbs_3mf.cpp:3582-3593,4142-4160,3655-3673,4577-4626",
    "bbs_3mf.cpp:4577-4584",
];

#[test]
fn contiguous_project_metadata_complete_parity() {
    native_stream_case::check(
        ("project-metadata-order", UPSTREAM),
        "contiguous",
        CONTROL,
        CONTROL_SHA,
        (false, |_, _| {
            assert_imports(CONTROL, None);
        }),
    );
}

#[test]
fn model_metadata_after_build_complete_parity() {
    check_order("model", "3D/3dmodel.model", "model", "resources", true);
}

#[test]
fn object_metadata_after_part_imports() {
    check_order(
        "object",
        "Metadata/model_settings.config",
        "object",
        "part",
        false,
    );
}

#[test]
fn part_metadata_after_mesh_stat_imports() {
    check_order(
        "part",
        "Metadata/model_settings.config",
        "part",
        "mesh_stat",
        false,
    );
}

#[test]
fn object_keyless_face_count_complete_parity() {
    check_object_metadata("keyless-object", "");
}

#[test]
fn object_empty_key_face_count_complete_parity() {
    check_object_metadata("empty-key-object", " key=\"\"");
}

#[test]
fn part_and_plate_missing_metadata_keys_still_reject() {
    const MEMBER: &str = "Metadata/model_settings.config";
    let mut archive = zip::ZipArchive::new(Cursor::new(CONTROL)).unwrap();
    let mut xml = String::new();
    archive
        .by_name(MEMBER)
        .unwrap()
        .read_to_string(&mut xml)
        .unwrap();
    for parent in ["part", "plate"] {
        let closing = format!("</{parent}>");
        assert!(xml.contains(&closing));
        let changed = xml.replacen(
            &closing,
            &format!("<metadata face_count=\"12\"/>{closing}"),
            1,
        );
        let project = replace_member(MEMBER, &changed);
        let error = ares_core::load_project(&project).unwrap_err().to_string();
        assert_eq!(
            error,
            "invalid project model settings XML: missing field `@key`"
        );
        assert!(
            ares_core::UpwardMachineSource::from_project(&project)
                .unwrap()
                .is_some()
        );
    }
}

fn check_object_metadata(label: &str, key: &str) {
    const MEMBER: &str = "Metadata/model_settings.config";
    let mut archive = zip::ZipArchive::new(Cursor::new(CONTROL)).unwrap();
    let mut xml = String::new();
    archive
        .by_name(MEMBER)
        .unwrap()
        .read_to_string(&mut xml)
        .unwrap();
    let changed = xml.replacen(
        "</object>",
        &format!("<metadata{key} face_count=\"12\"/></object>"),
        1,
    );
    check_project(label, replace_member(MEMBER, &changed), true);
}

fn check_order(label: &str, member: &str, parent: &str, sibling: &str, slice: bool) {
    assert_eq!(sha(CONTROL), CONTROL_SHA);
    check_project(label, permute_metadata(member, parent, sibling), slice);
}

fn check_project(label: &str, project: Vec<u8>, slice: bool) {
    let (reference, _) = native_stream_case::qualify(
        ("project-metadata-order", UPSTREAM),
        label,
        &project,
        &sha(&project),
        false,
    );
    let runner = OrcaRunner::from_env().unwrap().unwrap();
    let control = NativeCase::slice(&runner, CONTROL, "project-metadata-order-control-");
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
        reference.root.join("native-contiguous-comparison.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "control_sha256": CONTROL_SHA, "permuted_sha256": sha(&project),
            "control_root": control.root, "permuted_root": reference.root,
            "native_contiguous_difference": difference,
            "complete_ordered_comparison": true,
        }))
        .unwrap(),
    )
    .unwrap();
    assert!(
        difference.is_none(),
        "Native ordering changed output: {difference:?} at {:?}",
        reference.root
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
    if slice {
        reference.compare(ares_core::GenerationMetadata::deterministic(
            2026, 8, 27, 0, 0, 0,
        ));
    }
    assert_imports(&project, Some((imported, identity)));
}

type Imports = (
    Result<ares_core::Project, ares_core::SliceError>,
    Result<Option<ares_core::UpwardMachineSource>, ares_core::SliceError>,
);

fn assert_imports(project: &[u8], imports: Option<Imports>) {
    let (imported, identity) = imports.unwrap_or_else(|| {
        (
            ares_core::load_project(project),
            ares_core::UpwardMachineSource::from_project(project),
        )
    });
    let expected = ares_core::load_project(CONTROL).unwrap();
    let actual = imported.unwrap();
    assert_eq!(actual.models(), expected.models());
    assert_eq!(actual.objects(), expected.objects());
    assert_eq!(actual.plates(), expected.plates());
    assert_eq!(
        identity.unwrap().is_some(),
        ares_core::UpwardMachineSource::from_project(CONTROL)
            .unwrap()
            .is_some()
    );
}

fn permute_metadata(member: &str, parent: &str, sibling: &str) -> Vec<u8> {
    let mut archive = zip::ZipArchive::new(Cursor::new(CONTROL)).unwrap();
    let mut xml = String::new();
    archive
        .by_name(member)
        .unwrap()
        .read_to_string(&mut xml)
        .unwrap();
    let parent_start = xml.find(&format!("<{parent} ")).unwrap();
    let closing = parent_start + xml[parent_start..].find(&format!("</{parent}>")).unwrap();
    let structural = parent_start
        + xml[parent_start..closing]
            .find(&format!("<{sibling}"))
            .unwrap();
    let start = xml[parent_start..structural].rfind("<metadata ").unwrap() + parent_start;
    let open_end = start + xml[start..].find('>').unwrap() + 1;
    let end = if xml[..open_end].ends_with("/>") {
        open_end
    } else {
        open_end + xml[open_end..].find("</metadata>").unwrap() + "</metadata>".len()
    };
    let moved = format!(
        "{}{}{}{}",
        &xml[..start],
        &xml[end..closing],
        &xml[start..end],
        &xml[closing..]
    );
    assert_eq!(moved.len(), xml.len());
    replace_member(member, &moved)
}

fn replace_member(member: &str, xml: &str) -> Vec<u8> {
    let mut archive = zip::ZipArchive::new(Cursor::new(CONTROL)).unwrap();
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..archive.len() {
        let entry = archive.by_index(index).unwrap();
        if entry.name() == member {
            writer
                .start_file(member, zip::write::SimpleFileOptions::default())
                .unwrap();
            writer.write_all(xml.as_bytes()).unwrap();
        } else {
            writer.raw_copy_file(entry).unwrap();
        }
    }
    writer.finish().unwrap().into_inner()
}
