use std::io::{Cursor, Write};

use crate::{OrcaRunner, runner::native_api_cli::NativeCase};

use super::plater_archive::rewrite_archive;

const PROJECT: &[u8] = include_bytes!("../../../../../../tests/parity/bbl-type1-tower/enabled.3mf");

#[derive(Clone, Copy)]
enum Assignment {
    WholeObject,
    SparseInfill,
    SolidInfill,
    HeightRange,
}

#[test]
fn short_sparse_role_delays_the_second_tool_change() {
    check("short-sparse", &project(true, Assignment::SparseInfill));
}

#[test]
fn full_height_sparse_role_delays_the_second_tool_change() {
    check("sparse", &project(false, Assignment::SparseInfill));
}

#[test]
fn height_range_delays_the_second_tool_change() {
    check(
        "range",
        &with_filament_range(project(false, Assignment::HeightRange)),
    );
}

#[test]
fn solid_role_delays_the_second_tool_change() {
    check("solid", &project(true, Assignment::SolidInfill));
}

#[test]
fn two_object_two_filament_control() {
    check(
        "two-filament-control",
        &project(false, Assignment::WholeObject),
    );
}

fn check(label: &str, project: &[u8]) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        crate::native_environment::skipped("Native runner unavailable for Type1 tool-change count");
        return;
    };
    let reference = NativeCase::slice(
        &runner,
        project,
        &format!("type1-toolchange-count-{label}-"),
    );
    assert!(reference.native.lines().any(|line| line == "T0"));
    assert!(reference.native.lines().any(|line| line == "T1"));
    assert!(
        reference
            .native
            .lines()
            .any(|line| line == "; get travel path for change filament")
    );
    reference.compare(ares_core::GenerationMetadata::deterministic(
        2026, 8, 27, 0, 0, 0,
    ));
}

fn project(short: bool, assignment: Assignment) -> Vec<u8> {
    rewrite_archive(PROJECT, |name, text| match name {
        "Metadata/project_settings.config" => {
            let mut settings: serde_json::Value = serde_json::from_str(text).unwrap();
            settings["farthest_point_timelapse"] = "0".into();
            if matches!(assignment, Assignment::SolidInfill) {
                // Keep the bottom solid infill without re-entering it near the top shell.
                settings["top_shell_layers"] = "0".into();
            }
            Some(serde_json::to_string(&settings).unwrap())
        }
        "Metadata/model_settings.config" if !matches!(assignment, Assignment::WholeObject) => {
            let (before, object) = text.split_once("<object id=\"4\">").unwrap();
            let (object, after) = object.split_once("</object>").unwrap();
            let (head, tail) = object
                .split_once("<metadata key=\"extruder\" value=\"2\"/>")
                .unwrap();
            let role = match assignment {
                Assignment::SparseInfill => {
                    "\n    <metadata key=\"sparse_infill_filament_id\" value=\"2\"/>"
                }
                Assignment::SolidInfill => {
                    "\n    <metadata key=\"internal_solid_filament_id\" value=\"2\"/>"
                }
                Assignment::HeightRange => "",
                Assignment::WholeObject => unreachable!(),
            };
            Some(format!(
                "{before}<object id=\"4\">{head}<metadata key=\"extruder\" value=\"1\"/>{role}{tail}</object>{after}"
            ))
        }
        "3D/3dmodel.model" if short => {
            let (model, build) = text.split_once("<build").unwrap();
            let build = build
                .replace(" 1 128 134 5\"", " 0.2 128 134 1\"")
                .replace(" 1 128 122 5\"", " 0.2 128 122 1\"");
            Some(format!("{model}<build{build}"))
        }
        _ => None,
    })
}

fn with_filament_range(project: Vec<u8>) -> Vec<u8> {
    let mut archive = zip::ZipWriter::new_append(Cursor::new(project)).unwrap();
    archive
        .start_file(
            "Metadata/layer_config_ranges.xml",
            zip::write::SimpleFileOptions::DEFAULT,
        )
        .unwrap();
    archive
        .write_all(br#"<?xml version="1.0" encoding="UTF-8"?>
<objects><object id="1"><range min_z="2" max_z="5"><option opt_key="layer_height">0.2</option><option opt_key="extruder">2</option></range></object></objects>
"#)
        .unwrap();
    archive.finish().unwrap().into_inner()
}
