use super::{OrcaRunner, stages};
use std::{fs, path::Path, time::Duration};

#[test]
fn direct_content_cache_child() {
    let Some(directory) = std::env::var_os("CONTENT_CACHE_ROOT") else {
        return;
    };
    let root = Path::new(&directory);
    let step: usize = std::env::var("CONTENT_CACHE_STEP")
        .unwrap()
        .parse()
        .unwrap();
    let change = std::env::var("CONTENT_CACHE_CHANGE").unwrap();
    let work = root.join(step.to_string());
    let output = work.join("native");
    fs::create_dir(&output).unwrap();
    let runner = OrcaRunner::from_env().unwrap().unwrap();
    let mode = match (change.as_str(), step) {
        ("rejection" | "tampered-rejection", _) => "3",
        ("timeout", 3..) => "1",
        ("crash", 3..) => "2",
        ("generic", 3..) => "4",
        _ => "0",
    };
    let args = if change == "export" {
        vec![
            "--export-3mf".into(),
            work.join("export.3mf").to_str().unwrap().into(),
            work.join("input.3mf").to_str().unwrap().into(),
        ]
    } else {
        vec![
            "--slice".into(),
            mode.into(),
            "--datadir".into(),
            work.join("data").to_str().unwrap().into(),
            "--outputdir".into(),
            output.to_str().unwrap().into(),
            work.join("input.3mf").to_str().unwrap().into(),
        ]
    };
    let stage = if change == "export" {
        stages::Stage::Export
    } else {
        stages::Stage::Slice
    };
    let result = if mode == "1" {
        stages::run_with_timeout(runner.bin(), args, stage, &work, Duration::from_millis(30))
    } else if change == "live" && step >= 3 {
        stages::run_with_timeout(runner.bin(), args, stage, &work, Duration::from_secs(5))
    } else if change == "explicit" {
        stages::run_with_env(
            runner.bin(),
            args,
            &[(
                "CONTENT_EXPLICIT",
                if step == 3 { "changed" } else { "original" },
            )],
            stage,
            &work,
        )
    } else {
        stages::run(runner.bin(), args, stage, &work)
    };
    match mode {
        "0" => {
            result.unwrap();
            let input = fs::read(work.join("input.3mf")).unwrap();
            let expected = [
                input,
                if change == "explicit" {
                    if step == 3 {
                        b"changed".to_vec()
                    } else {
                        b"original".to_vec()
                    }
                } else {
                    vec![]
                },
            ]
            .concat();
            let file = if change == "export" {
                work.join("export.3mf")
            } else {
                output.join("nested/result.bin")
            };
            assert_eq!(fs::read(file).unwrap(), expected);
        }
        "1" => assert_eq!(result.unwrap_err().kind, stages::FailureKind::Timeout),
        "2" => {
            let error = result.unwrap_err();
            assert_eq!(error.kind, stages::FailureKind::Process);
            assert_eq!(error.captured.unwrap().code, None);
        }
        "3" => {
            let error = result.unwrap_err();
            assert_eq!(error.kind, stages::FailureKind::Process);
            assert!(error.input_rejection("layer_height"));
            let captured = error.captured.unwrap();
            assert_eq!(captured.code, Some(238));
            assert_eq!(captured.stdout, b"native stdout\n");
            assert_eq!(
                captured.stderr,
                b"native stderr\nParam values in 3mf/config error:\nlayer_height: invalid value\n"
            );
        }
        "4" => {
            let error = result.unwrap_err();
            assert_eq!(error.kind, stages::FailureKind::Process);
            assert!(!error.input_rejection("layer_height"));
        }
        _ => unreachable!(),
    }
}
