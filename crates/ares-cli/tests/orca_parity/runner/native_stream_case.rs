//! Native double replay before the canonical main public API/CLI assertions.

use crate::{
    OrcaRunner, golden,
    runner::native_api_cli::{NativeCase, sha},
};
use serde_json::json;
use std::fs;

pub(crate) fn check(
    (namespace, upstream): (&str, &[&str]),
    label: &str,
    project: &[u8],
    expected_sha: &str,
    (assert_two_tools, assert_native): (bool, impl Fn(&NativeCase, &NativeCase)),
) {
    let (reference, replay) = qualify(
        (namespace, upstream),
        label,
        project,
        expected_sha,
        assert_two_tools,
    );
    assert_native(&reference, &replay);
    let (api, cli) = reference.compare(ares_core::GenerationMetadata::deterministic(
        2026, 8, 27, 0, 0, 0,
    ));
    let path = reference.root.join("comparison.json");
    let mut comparison: serde_json::Value =
        serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    let mut qualification: serde_json::Value =
        serde_json::from_slice(&fs::read(reference.root.join("native-replay.json")).unwrap())
            .unwrap();
    comparison["case"] = json!(label);
    comparison["native_replay_comparison"] = qualification["native_replay_comparison"].take();
    comparison["api_cli_equal"] = json!(api == cli);
    comparison["strict_comparator"] = json!("golden::compare_ordered_bytes_generator_only");
    fs::write(path, serde_json::to_vec_pretty(&comparison).unwrap()).unwrap();
}

pub(crate) fn qualify(
    (namespace, upstream): (&str, &[&str]),
    label: &str,
    project: &[u8],
    expected_sha: &str,
    assert_two_tools: bool,
) -> (NativeCase, NativeCase) {
    let runner = OrcaRunner::from_env().unwrap().unwrap();
    assert_eq!(sha(project), expected_sha, "frozen input SHA for {label}");
    let reference = NativeCase::slice(&runner, project, &format!("{namespace}-{label}-"));
    let replay = NativeCase::slice(&runner, project, &format!("{namespace}-{label}-replay-"));
    let first_native = golden::normalize_one_generator_line(
        reference.native.as_bytes(),
        golden::GeneratorKind::Orca,
    )
    .unwrap();
    let replay_native =
        golden::normalize_one_generator_line(replay.native.as_bytes(), golden::GeneratorKind::Orca)
            .unwrap();
    let native_replay = golden::first_difference(&first_native, &replay_native)
        .map_or(Ok(()), |difference| Err(difference.to_string()));
    fs::write(
        reference.root.join("native-replay.json"),
        serde_json::to_vec_pretty(&json!({
            "case": label, "input_sha256": expected_sha, "upstream": upstream,
            "reference_root": reference.root, "replay_root": replay.root,
            "native_replay_comparison": native_replay
        }))
        .unwrap(),
    )
    .unwrap();
    native_replay.as_ref().unwrap_or_else(|error| {
        panic!(
            "Native nondeterminism {label}: {error}; {:?}",
            reference.root
        )
    });
    if assert_two_tools {
        assert!(
            reference.native.lines().any(|line| line.starts_with("T0")),
            "Native target must use T0: {:?}",
            reference.root
        );
        assert!(
            reference.native.lines().any(|line| line.starts_with("T1")),
            "Native target must use T1: {:?}",
            reference.root
        );
    }
    (reference, replay)
}
