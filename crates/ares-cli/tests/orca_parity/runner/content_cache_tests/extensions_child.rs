use super::{find_traces, single_trace, slice_args};
use crate::runner::{CaseInputs, OrcaRunner, stages};
use serde_json::{Value, json};
use std::{fs, path::Path};

pub(super) fn run(case: &str, root: &Path, runner: &OrcaRunner, step: &str) {
    match case {
        "producer" => child_producer(root, runner),
        "datadir" => child_datadir(root, runner, step),
        "flags" => child_flags(root, runner, step),
        "repeat" => child_repeat(root, runner),
        "cross-repeat" => child_cross_repeat(root, runner),
        "isolated-cwd" => child_isolated_cwd(root, runner),
        "isolated-cwd-hit" => child_isolated_cwd_hit(root, runner),
        _ => unreachable!(),
    }
}

fn child_producer(root: &Path, runner: &OrcaRunner) {
    let machine = serde_json::from_value(json!({"name": "machine"})).unwrap();
    let process = serde_json::from_value(json!({"name": "process"})).unwrap();
    let inputs = CaseInputs {
        label: "ext-producer",
        machine: &machine,
        process: &process,
        filaments: &[],
    };
    let exported = runner
        .export_case(&inputs, &serde_json::Map::new(), &root.join("model.stl"))
        .unwrap();
    let _ = runner.slice_case(&exported).unwrap();
}

fn child_datadir(root: &Path, runner: &OrcaRunner, step: &str) {
    let work = root.join(format!("work-{step}"));
    let out = work.join("out");
    fs::create_dir_all(&out).unwrap();
    let input = work.join("input.3mf");
    fs::write(&input, b"project-bytes").unwrap();
    let mut args = vec!["--slice".into(), "0".into()];
    if step == "3" {
        let writable = work.join("writable-data");
        fs::create_dir_all(&writable).unwrap();
        fs::write(writable.join("item"), "x").unwrap();
        args.extend(["--datadir".into(), writable.to_str().unwrap().into()]);
    }
    args.extend([
        "--outputdir".into(),
        out.to_str().unwrap().into(),
        input.to_str().unwrap().into(),
    ]);
    stages::run(runner.bin(), args, stages::Stage::Slice, &work).unwrap();
    let value = single_trace(&work);
    match step {
        "1" => {
            assert_eq!(value["outcome"], "miss");
            assert_eq!(value["profile"]["datadir_hashes"], 0);
        }
        "2" => {
            assert_eq!(value["outcome"], "hit");
            assert_eq!(value["profile"]["datadir_hashes"], 0);
        }
        "3" => {
            assert_eq!(value["outcome"], "miss");
            assert_eq!(value["profile"]["datadir_hashes"], 1);
        }
        _ => unreachable!(),
    }
}

fn child_flags(root: &Path, runner: &OrcaRunner, step: &str) {
    let work = root.join(format!("flags-{step}"));
    let out = work.join("out");
    fs::create_dir_all(&out).unwrap();
    let input = work.join("input.3mf");
    fs::write(&input, b"flag-project").unwrap();
    let mut args: Vec<String> = match step {
        "scalar-1" | "scalar-2" => vec!["--layer-height".into(), "0.2".into()],
        "bool-eq-1" | "bool-eq-2" => vec!["--use-firmware-retraction=1".into()],
        "bare-bool" => vec!["--wipe".into()],
        "path-opt" => vec!["--post-process".into(), "/nonexistent/script.sh".into()],
        _ => unreachable!(),
    };
    args.extend(slice_args(&out, &input));
    stages::run(runner.bin(), args, stages::Stage::Slice, &work).unwrap();
    let value = single_trace(&work);
    match step {
        "scalar-1" | "bool-eq-1" => assert_eq!(value["outcome"], "miss"),
        "scalar-2" | "bool-eq-2" => assert_eq!(value["outcome"], "hit"),
        "bare-bool" | "path-opt" => {
            assert_eq!(value["outcome"], "bypass-request");
            assert_eq!(value["profile"]["datadir_hashes"], 0);
        }
        _ => unreachable!(),
    }
}

fn child_repeat(root: &Path, runner: &OrcaRunner) {
    let work = root.join("repeat-work");
    let first = work.join("first");
    let second = work.join("second");
    fs::create_dir_all(&first).unwrap();
    fs::create_dir_all(&second).unwrap();
    let input = work.join("input.3mf");
    fs::write(&input, b"repeat-project").unwrap();
    for out in [&first, &second] {
        stages::run(
            runner.bin(),
            slice_args(out, &input),
            stages::Stage::Slice,
            &work,
        )
        .unwrap();
    }
    let mut traces = Vec::new();
    find_traces(&work, &mut traces);
    let outcomes: Vec<String> = traces
        .iter()
        .map(|path| {
            let value: Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
            value["outcome"].as_str().unwrap().to_owned()
        })
        .collect();
    assert!(outcomes.iter().any(|outcome| outcome == "miss"));
    assert!(outcomes.iter().any(|outcome| outcome == "bypass-repeat"));
}

fn child_cross_repeat(root: &Path, runner: &OrcaRunner) {
    let work_a = root.join("cross-repeat-a");
    let work_b = root.join("cross-repeat-b");
    for (work, expected) in [(&work_a, "miss"), (&work_b, "bypass-repeat")] {
        let out = work.join("out");
        fs::create_dir_all(&out).unwrap();
        let input = work.join("input.3mf");
        fs::write(&input, b"cross-repeat-project").unwrap();
        stages::run(
            runner.bin(),
            slice_args(&out, &input),
            stages::Stage::Slice,
            work,
        )
        .unwrap();
        assert_eq!(single_trace(work)["outcome"], expected);
    }
}

fn child_isolated_cwd(root: &Path, runner: &OrcaRunner) {
    let work = root.join("cwd-work");
    let out_a = work.join("out-a");
    let out_b = work.join("out-b");
    fs::create_dir_all(&out_a).unwrap();
    fs::create_dir_all(&out_b).unwrap();
    let input_a = work.join("input-a.3mf");
    let input_b = work.join("input-b.3mf");
    fs::write(&input_a, b"project-a").unwrap();
    fs::write(&input_b, b"project-b").unwrap();
    for (input, out) in [(&input_a, &out_a), (&input_b, &out_b)] {
        stages::run(
            runner.bin(),
            slice_args(out, input),
            stages::Stage::Slice,
            &work,
        )
        .unwrap();
    }
    assert!(work.join("result.json").is_file());
}

fn child_isolated_cwd_hit(root: &Path, runner: &OrcaRunner) {
    let work3 = root.join("cwd-work-hit");
    let out_b2 = work3.join("out-b2");
    fs::create_dir_all(&out_b2).unwrap();
    let input_b = root.join("cwd-work/input-b.3mf");
    stages::run(
        runner.bin(),
        slice_args(&out_b2, &input_b),
        stages::Stage::Slice,
        &work3,
    )
    .unwrap();
    assert_eq!(single_trace(&work3)["outcome"], "hit");
    assert_eq!(
        fs::read(out_b2.join("plate_1.gcode")).unwrap(),
        b"project-b"
    );
}
