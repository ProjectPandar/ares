use std::{fs::File, io::Write, panic::catch_unwind, path::Path};

use zip::{ZipWriter, write::SimpleFileOptions};

use super::{archive_members, compare_archive_members};

const SETTINGS: &str = "Metadata/model_settings.config";
const OTHER: &str = "Metadata/other.config";
const FIRST: &str = "  <object id=\"3\">\n    <part id=\"1\"/>\n  </object>\n";
const SECOND: &str = "  <object id=\"5\">\n    <part id=\"2\"/>\n  </object>\n";
const PLATE_A: &str = "  <plate>\n    <model_instance><metadata key=\"object_id\" value=\"3\"/></model_instance>\n  </plate>\n";
const PLATE_B: &str = "  <plate>\n    <model_instance><metadata key=\"object_id\" value=\"5\"/></model_instance>\n  </plate>\n";
const ASSEMBLE_FIRST: &str = "    <assemble_item object_id=\"3\"/>\n";
const ASSEMBLE_SECOND: &str = "    <assemble_item object_id=\"5\"/>\n";

#[test]
fn compares_complete_model_settings_objects_without_ordering_them() {
    let reference = config(&[FIRST, SECOND, PLATE_A, PLATE_B]);
    let swapped = config(&[SECOND, FIRST, PLATE_A, PLATE_B]);
    let temp = temp_dir();
    assert!(compare(temp.path(), &[(&reference, ""), (&swapped, "")]));
}

#[test]
fn rejects_changes_to_model_settings_object_bytes_or_membership() {
    let changed = FIRST.replace("part id=\"1\"", "part id=\"9\"");
    let two = config(&[FIRST, SECOND, PLATE_A, PLATE_B]);
    let one = config(&[FIRST, PLATE_A, PLATE_B]);
    let zero = config(&[PLATE_A, PLATE_B]);
    let temp = temp_dir();
    for unchanged in [&zero, &one] {
        assert!(compare(temp.path(), &[(unchanged, ""), (unchanged, "")]));
    }
    for (left, right, other) in [
        (&two, &config(&[SECOND, &changed, PLATE_A, PLATE_B]), ""),
        (&two, &one, ""),
        (&one, &zero, ""),
        (&two, &one, SECOND),
    ] {
        assert!(!compare(temp.path(), &[(left, ""), (right, other)]));
    }
}

#[test]
fn keeps_model_settings_structure_and_other_members_ordered() {
    let assembly = assemble(ASSEMBLE_FIRST, ASSEMBLE_SECOND);
    let reversed_assembly = assemble(ASSEMBLE_SECOND, ASSEMBLE_FIRST);
    let reference = config(&[FIRST, SECOND, PLATE_A, PLATE_B, &assembly]);
    let objects = config(&[FIRST, SECOND]);
    let swapped_objects = config(&[SECOND, FIRST]);
    let temp = temp_dir();
    for changed in [
        config(&[SECOND, FIRST, PLATE_B, PLATE_A, &assembly]),
        config(&[SECOND, PLATE_A, FIRST, PLATE_B, &assembly]),
        config(&[SECOND, FIRST, PLATE_A, PLATE_B, &reversed_assembly]),
        config(&[SECOND, "\n", FIRST, PLATE_A, PLATE_B, &assembly]),
    ] {
        assert!(!compare(temp.path(), &[(&reference, ""), (&changed, "")]));
    }
    assert!(!compare(
        temp.path(),
        &[(&reference, &objects), (&reference, &swapped_objects)],
    ));
}

fn assemble(first: &str, second: &str) -> String {
    format!("  <assemble>\n{first}{second}  </assemble>\n")
}

fn config(children: &[&str]) -> String {
    format!("<config>\n{}</config>\n", children.concat())
}

fn temp_dir() -> tempfile::TempDir {
    tempfile::tempdir_in(crate::artifacts::root_from_env().unwrap()).unwrap()
}

fn compare(root: &Path, archives: &[(&str, &str); 2]) -> bool {
    let [reference, actual] = ["native.3mf", "ares.3mf"].map(|name| root.join(name));
    for (path, (settings, other)) in [&reference, &actual].into_iter().zip(archives) {
        let mut zip = ZipWriter::new(File::create(path).unwrap());
        for (name, bytes) in [(SETTINGS, settings), (OTHER, other)] {
            zip.start_file(name, SimpleFileOptions::default()).unwrap();
            zip.write_all(bytes.as_bytes()).unwrap();
        }
        zip.finish().unwrap();
    }
    let expected = archive_members(&reference);
    let actual_members = archive_members(&actual);
    catch_unwind(|| compare_archive_members(&expected, &actual_members, &actual)).is_ok()
}
