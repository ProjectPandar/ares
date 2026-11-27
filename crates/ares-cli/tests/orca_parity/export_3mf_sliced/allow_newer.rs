use std::{fs, path::Path};

use super::super::export_3mf_sliced_rejection::rewrite_project;
use super::{compare_sliced_archives, invoke, write_saved_input};
use crate::OrcaRunner;

#[test]
fn direct_sliced_export_rejects_newer_project_by_default() {
    check(None, 232);
}

#[test]
fn direct_sliced_export_rejects_newer_project_with_false_flag() {
    check(Some("--allow-newer-file=0"), 232);
}

#[test]
fn direct_sliced_export_allows_newer_project_with_flag() {
    check(Some("--allow-newer-file"), 0);
}

fn check(flag: Option<&str>, expected: i32) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = runner.work_dir();
    let source = root.join("source/cwd");
    fs::create_dir_all(source.join("out")).unwrap();
    write_saved_input(&runner, &source);
    let project = rewrite_project(&fs::read(source.join("saved.3mf")).unwrap(), |name, xml| {
        if name != "3D/3dmodel.model" {
            return xml.to_owned();
        }
        let version = r#"<metadata name="OrcaSlicer">2.5.0</metadata>"#;
        assert_eq!(xml.matches(version).count(), 1);
        xml.replace(version, r#"<metadata name="OrcaSlicer">3.5.0</metadata>"#)
    });
    let mut argv = vec!["--slice", "1", "--export-3mf", "export.3mf"];
    argv.extend(flag);
    argv.push("saved.3mf");
    let native = root.join("native/cwd");
    let ares = root.join("ares/cwd");
    let variants: [(&Path, &Path); 2] = [
        (runner.bin(), &native),
        (Path::new(env!("CARGO_BIN_EXE_ares")), &ares),
    ];
    for (bin, cwd) in variants {
        fs::create_dir_all(cwd.join("out")).unwrap();
        fs::write(cwd.join("saved.3mf"), &project).unwrap();
        assert_eq!(invoke(bin, cwd, &argv), Some(expected), "{cwd:?}");
        assert_eq!(cwd.join("export.3mf").exists(), expected == 0, "{cwd:?}");
    }
    if expected == 0 {
        compare_sliced_archives(&native.join("export.3mf"), &ares.join("export.3mf"));
    }
}
