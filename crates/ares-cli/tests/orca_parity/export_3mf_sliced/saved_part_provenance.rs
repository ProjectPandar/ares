use std::{collections::BTreeMap, fmt::Write, fs, path::Path};

use quick_xml::{Reader, events::Event};

use super::super::export_3mf_sliced_rejection::rewrite_project;
use super::{GCODE, comparator, invoke, write_saved_input};
use crate::OrcaRunner;

pub(super) const MODEL_SETTINGS: &str = "Metadata/model_settings.config";
const SOURCE: &str = "input/synthetic-witness.3mf";
const BASENAME: &str = "synthetic-witness.3mf";
const EXPORT: &str = "export.3mf";
pub(super) const KEYS: [&str; 6] = [
    "source_file",
    "source_object_id",
    "source_volume_id",
    "source_offset_x",
    "source_offset_y",
    "source_offset_z",
];
const COMPLETE: [&str; 6] = ["original.stl", "7", "11", "1.25", "-2.5", "3.75"];

#[test]
fn missing_provenance_uses_the_input_archive_and_loaded_indices() {
    check([None; 6], [BASENAME, "0", "0", "0", "0", "0"]);
}

#[test]
fn missing_filename_fills_only_unset_indices_and_keeps_offsets() {
    check(
        [
            None,
            Some("7"),
            Some("-1"),
            Some("1.25"),
            Some("-2.5"),
            None,
        ],
        [BASENAME, "7", "0", "1.25", "-2.5", "0"],
    );
}

#[test]
fn empty_filename_uses_the_input_archive_without_overwriting_volume_index() {
    check(
        [Some(""), Some("-1"), Some("11"), None, None, Some("3.75")],
        [BASENAME, "0", "11", "0", "0", "3.75"],
    );
}

#[test]
fn explicit_source_path_exports_only_the_basename() {
    let mut input = COMPLETE.map(Some);
    input[0] = Some(r"source/nested\original.stl");
    check(input, COMPLETE);
}

#[test]
fn explicit_filename_does_not_fill_unset_indices() {
    check(
        [Some("original.stl"), None, None, None, None, None],
        ["original.stl", "-1", "-1", "0", "0", "0"],
    );
}

#[test]
fn explicit_filename_keeps_complete_companions() {
    check(COMPLETE.map(Some), COMPLETE);
}

fn check(input: [Option<&str>; 6], expected: [&str; 6]) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = runner.work_dir();
    let source = root.join("source/cwd");
    fs::create_dir_all(&source).unwrap();
    write_saved_input(&runner, &source);
    let project = rewrite_project(&fs::read(source.join("saved.3mf")).unwrap(), |name, xml| {
        if name == MODEL_SETTINGS {
            with_provenance(xml, input)
        } else {
            xml.to_owned()
        }
    });

    let native = export(runner.bin(), &root.join("native/cwd"), &project);
    assert_eq!(
        native,
        KEYS.into_iter().zip(expected.map(str::to_owned)).collect(),
        "unexpected Native part provenance in {root:?}"
    );
    let ares = export(
        Path::new(env!("CARGO_BIN_EXE_ares")),
        &root.join("ares/cwd"),
        &project,
    );
    assert_eq!(ares, native, "saved part provenance differs in {root:?}");
}

pub(super) fn with_provenance(xml: &str, values: [Option<&str>; 6]) -> String {
    let start = "    <part id=\"1\" subtype=\"normal_part\">";
    let mut replacement = start.to_owned();
    for (key, value) in KEYS.into_iter().zip(values) {
        if let Some(value) = value {
            write!(
                replacement,
                "\n      <metadata key=\"{key}\" value=\"{}\"/>",
                quick_xml::escape::escape(value)
            )
            .unwrap();
        }
    }
    assert_eq!(xml.matches(start).count(), 1);
    assert_eq!(part_provenance(xml.as_bytes()).len(), KEYS.len());
    xml.lines()
        .filter(|line| {
            !KEYS
                .iter()
                .any(|key| line.contains(&format!("<metadata key=\"{key}\"")))
        })
        .collect::<Vec<_>>()
        .join("\n")
        .replace(start, &replacement)
}

fn export(bin: &Path, cwd: &Path, project: &[u8]) -> BTreeMap<&'static str, String> {
    fs::create_dir_all(cwd.join("input")).unwrap();
    fs::write(cwd.join(SOURCE), project).unwrap();
    let argv = ["--slice", "1", "--export-3mf", EXPORT, SOURCE];
    assert_eq!(invoke(bin, cwd, &argv), Some(0), "export in {cwd:?}");
    let members = comparator::read_members(&cwd.join(EXPORT));
    assert!(members.iter().any(|member| member.name == GCODE));
    let config = members
        .iter()
        .find(|member| member.name == MODEL_SETTINGS)
        .unwrap();
    part_provenance(&config.bytes)
}

pub(super) fn part_provenance(xml: &[u8]) -> BTreeMap<&'static str, String> {
    let mut reader = Reader::from_reader(xml);
    let mut part_count = 0;
    let mut in_part = false;
    let mut result = BTreeMap::new();
    loop {
        match reader.read_event().unwrap() {
            Event::Start(element) if element.name().as_ref() == b"part" => {
                part_count += 1;
                in_part = true;
            }
            Event::End(element) if element.name().as_ref() == b"part" => in_part = false,
            Event::Empty(element) if in_part && element.name().as_ref() == b"metadata" => {
                let key = element.try_get_attribute("key").unwrap().unwrap();
                if let Some(key) = KEYS
                    .into_iter()
                    .find(|name| name.as_bytes() == key.value.as_ref())
                {
                    let value = element
                        .try_get_attribute("value")
                        .unwrap()
                        .unwrap()
                        .decoded_and_normalized_value(
                            quick_xml::XmlVersion::Implicit1_0,
                            reader.decoder(),
                        )
                        .unwrap()
                        .into_owned();
                    assert!(result.insert(key, value).is_none(), "duplicate {key}");
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }
    assert_eq!(part_count, 1);
    result
}
