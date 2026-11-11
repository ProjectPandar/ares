//! `PrintBase::set_status` statuses of a project slice (`PrintBase.cpp:123-128`) against the
//! statuses OrcaSlicer's CLI logs through `default_status_callback` (`OrcaSlicer.cpp:468-476`).
//! Failure inventory: a status missing, extra or out of `Print::process` order; per-object
//! statuses not interleaved like upstream's object loops; a wrong formatted text (Z contour
//! counts, tree drop percents, G-code layer numbers restarting per object); `RELOAD_SCENE` on the
//! wrong status; a callback changing the G-code.

#[path = "../../../tests/support/native_environment.rs"]
mod native_environment;

use std::{
    fs,
    io::{Cursor, Read, Write},
    path::PathBuf,
    process::Command,
};

const OVERHANG: &[u8] =
    include_bytes!("../../ares-cli/tests/organic_merge_affinity/overhang-organic.3mf");
const TWO_CUBES: &[u8] =
    include_bytes!("../../../tests/parity/aligned-seam-rotated-object/rotated-two-cubes.3mf");
const RELOAD_SCENE: u32 = 1 << 1;

type Status = (i32, String);

/// The project with string settings of `Metadata/project_settings.config` replaced.
fn variant(project: &[u8], edits: &[(&str, &str)]) -> Vec<u8> {
    let mut reader = zip::ZipArchive::new(Cursor::new(project)).unwrap();
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..reader.len() {
        let mut file = reader.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == "Metadata/project_settings.config" {
            let mut text = String::from_utf8(bytes).unwrap();
            for (key, value) in edits {
                let prefix = format!("\"{key}\": \"");
                let start = text.find(&prefix).unwrap() + prefix.len();
                assert_eq!(text[start..].find(&prefix), None, "{key} is set once");
                let end = start + text[start..].find('"').unwrap();
                text.replace_range(start..end, value);
            }
            bytes = text.into_bytes();
        }
        writer
            .start_file(
                file.name(),
                zip::write::SimpleFileOptions::default().compression_method(file.compression()),
            )
            .unwrap();
        writer.write_all(&bytes).unwrap();
    }
    writer.finish().unwrap().into_inner()
}

/// The statuses OrcaSlicer reports slicing plate 1, read from its debug log; `None` without the
/// oracle.
fn native_statuses(name: &str, project: &[u8]) -> Option<Vec<Status>> {
    let bin = native_environment::binary("ARES_ORCA_BIN")?;
    let root = std::env::var_os("ARES_PARITY_ARTIFACT_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    let directory = root.join(format!("slicing-status-{name}-{}", std::process::id()));
    fs::create_dir_all(directory.join("native")).unwrap();
    let input = directory.join("input.3mf");
    fs::write(&input, project).unwrap();
    let log = directory.join("orca.log");
    let output = Command::new(bin)
        .args(["--debug", "4", "--logfile"])
        .arg(&log)
        .args(["--slice", "1", "--outputdir"])
        .arg(directory.join("native"))
        .arg(&input)
        .output()
        .unwrap();
    fs::write(directory.join("native.stdout"), &output.stdout).unwrap();
    fs::write(directory.join("native.stderr"), &output.stderr).unwrap();
    assert!(
        output.status.success(),
        "native slice failed; artifacts {directory:?}"
    );
    let statuses = fs::read_to_string(&log)
        .unwrap()
        .lines()
        .filter_map(|line| {
            let rest = line.split_once("default_status_callback: percent=")?.1;
            let (percent, rest) = rest.split_once(", warning_step=")?;
            let (step, rest) = rest.split_once(", message=")?;
            let (text, _) = rest.rsplit_once(", message_type=")?;
            (step == "-1").then(|| (percent.parse().unwrap(), text.to_owned()))
        })
        .collect::<Vec<_>>();
    assert!(
        !statuses.is_empty(),
        "no native statuses; artifacts {directory:?}"
    );
    Some(statuses)
}

/// Slices plate 1 with a recording callback and again without one.
async fn ares_statuses(project: &[u8]) -> Vec<(i32, String, u32)> {
    let mut statuses = Vec::new();
    let mut record = |status: &ares_core::SlicingStatus<'_>| {
        statuses.push((status.percent, status.text.to_owned(), status.flags));
    };
    let mut context = ares_core::SlicerContext::new(0);
    context.set_status_callback(&mut record);
    let reported = slice(project, context).await;
    let silent = slice(project, ares_core::SlicerContext::new(0)).await;
    assert!(reported == silent, "the status callback changed the G-code");
    statuses
}

async fn slice(project: &[u8], context: ares_core::SlicerContext<'_>) -> Vec<u8> {
    ares_core::slice_project_plate(
        project,
        ares_core::GenerationMetadata::default(),
        None,
        &ares_core::ConfigOverrides::default(),
        context,
    )
    .await
    .unwrap()
}

async fn check(name: &str, project: &[u8]) -> Vec<Status> {
    let reported = ares_statuses(project).await;
    for (percent, text, flags) in &reported {
        let expected = if text == "Slicing mesh" {
            RELOAD_SCENE
        } else {
            0
        };
        assert_eq!(*flags, expected, "flags of {percent} {text}");
    }
    let reported = reported
        .into_iter()
        .map(|(percent, text, _)| (percent, text))
        .collect::<Vec<_>>();
    if let Some(native) = native_statuses(name, project) {
        assert_eq!(reported, native);
    }
    reported
}

fn count(statuses: &[Status], text: &str) -> usize {
    statuses
        .iter()
        .filter(|(_, status)| status.starts_with(text))
        .count()
}

#[tokio::test]
async fn organic_tree_support_reports_the_3d_generator_stages() {
    let statuses = check("organic", OVERHANG).await;
    assert!(statuses.contains(&(69, "Generating support".to_owned())));
}

#[tokio::test]
async fn hybrid_tree_support_reports_each_dropped_layer() {
    let project = variant(OVERHANG, &[("support_style", "tree_hybrid")]);
    let statuses = check("hybrid", &project).await;
    assert!(statuses.contains(&(56, "Support: generate contact points".to_owned())));
}

#[tokio::test]
async fn normal_support_reports_one_support_stage() {
    let project = variant(
        OVERHANG,
        &[
            ("support_type", "normal(manual)"),
            ("support_style", "grid"),
        ],
    );
    let statuses = check("grid", &project).await;
    assert_eq!(count(&statuses, "Generating support"), 1);
}

#[tokio::test]
async fn unsupported_object_reports_the_necessity_check_and_z_contouring() {
    let project = variant(OVERHANG, &[("enable_support", "0"), ("zaa_enabled", "1")]);
    let statuses = check("contour", &project).await;
    assert_eq!(count(&statuses, "Checking support necessity"), 1);
    assert!(count(&statuses, "Z contoured layer ") > 1);
}

#[tokio::test]
async fn two_objects_by_layer_interleave_per_object_stages() {
    let statuses = check("two-by-layer", TWO_CUBES).await;
    assert_eq!(count(&statuses, "Optimizing toolpath"), 6);
}

#[tokio::test]
async fn two_objects_by_object_number_layers_per_object() {
    let project = variant(TWO_CUBES, &[("print_sequence", "by object")]);
    let statuses = check("two-by-object", &project).await;
    let first_layers = statuses
        .iter()
        .filter(|status| *status == &(80, "Generating G-code: layer 1".to_owned()))
        .count();
    assert_eq!(first_layers, 2);
}
