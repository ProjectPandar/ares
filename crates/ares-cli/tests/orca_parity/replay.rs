//! Cached-fixture replay sweep: re-slices every fixture 3mf captured by the
//! full printer sweep and compares against the cached OrcaSlicer reference
//! g-code, so the fleet moves without re-running the OrcaSlicer CLI.
//!
//! Gated behind `ARES_PARITY_REPLAY=<fixtures root>`.

use std::path::PathBuf;

use crate::{self as parity, runner::ParityCase};

struct ReplayCase {
    label: String,
    project: PathBuf,
    reference: PathBuf,
}

fn replay_one(case: &ReplayCase) -> parity::ParityOutcome {
    let Ok(project) = std::fs::read(&case.project) else {
        return parity::ares_error(&case.label, "fixture 3mf unreadable".into());
    };
    let Ok(reference) = std::fs::read(&case.reference) else {
        return parity::ares_error(&case.label, "reference gcode unreadable".into());
    };
    parity::compare_case(&ParityCase {
        label: case.label.clone(),
        project,
        reference,
    })
}

#[test]
fn orca_parity_replay_sweep() {
    if std::env::var("ARES_PARITY_REPLAY").is_err() {
        eprintln!("skipping: set ARES_PARITY_REPLAY=<fixtures root>");
        return;
    }
    let root: PathBuf = std::env::var("ARES_PARITY_REPLAY").unwrap().into();
    let mut cases: Vec<ReplayCase> = Vec::new();
    for entry in std::fs::read_dir(&root).expect("fixtures root readable") {
        let Ok(entry) = entry else {
            continue;
        };
        let path = entry.path();
        if path.extension().is_none_or(|extension| extension != "3mf") {
            continue;
        }
        let reference = path
            .parent()
            .unwrap()
            .join(path.file_stem().unwrap())
            .join("plate_1.gcode");
        cases.push(ReplayCase {
            label: path.file_stem().unwrap().to_string_lossy().into_owned(),
            project: path,
            reference,
        });
    }
    cases.sort_by(|a, b| a.label.cmp(&b.label));
    eprintln!("replaying {} cached fixtures", cases.len());

    let workers = std::thread::available_parallelism()
        .map(|value| value.get())
        .unwrap_or(4);
    let next = std::sync::atomic::AtomicUsize::new(0);
    let outcomes: Vec<_> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..workers)
            .map(|_| {
                scope.spawn(|| {
                    let mut done = Vec::new();
                    loop {
                        let index = next.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        let Some(case) = cases.get(index) else {
                            return done;
                        };
                        let outcome = replay_one(case);
                        if index % 100 == 0 {
                            eprintln!("[{}/{}] {}", index, cases.len(), outcome.label);
                        }
                        done.push(outcome);
                    }
                })
            })
            .collect();
        handles
            .into_iter()
            .flat_map(|handle| handle.join().unwrap())
            .collect()
    });

    let mut failures: Vec<_> = outcomes
        .into_iter()
        .filter(|outcome| outcome.status != "PASS")
        .map(|outcome| (outcome.label, outcome.detail))
        .collect();
    failures.sort_by(|a, b| a.0.cmp(&b.0));
    let passing = cases.len() - failures.len();

    let report = std::path::Path::new("tests/parity/replay-summary.txt");
    let body = failures
        .iter()
        .map(|(label, detail)| format!("{label}\n{detail}\n"))
        .collect::<Vec<_>>()
        .join("\n");
    let _ = std::fs::create_dir_all(report.parent().unwrap());
    let _ = std::fs::write(report, &body);

    eprintln!("replay: {}/{} pass", passing, cases.len());
    let _ = failures;
}
