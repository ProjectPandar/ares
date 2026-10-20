//! Shared frozen-project admission, repeat/activity and strict case lifecycle.

use std::{fs, io::Read};

use ares_core::Project;

use super::{complete, observe};
use crate::runner::{OrcaRunner, repo_root};

#[derive(Clone, Copy)]
pub(super) struct Input<'a> {
    pub(super) name: &'a str,
    pub(super) path: &'a str,
    pub(super) sha: &'a str,
}

pub(super) fn check(
    label: &str,
    input: Input<'_>,
    admit: impl FnOnce(&Project, &str),
    activity: impl Fn(&complete::Recorded),
) {
    let Some(mut runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let project = fs::read(repo_root().join("tests/parity").join(input.path)).unwrap();
    assert_eq!(observe::hash(&project), input.sha, "frozen input");
    let loaded = ares_core::load_project(&project).unwrap();
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(&project)).unwrap();
    let mut metadata = String::new();
    archive
        .by_name("Metadata/model_settings.config")
        .unwrap()
        .read_to_string(&mut metadata)
        .unwrap();
    admit(&loaded, &metadata);
    let runs = (1..=2)
        .map(|repeat| complete::run_once(&mut runner, label, input.name, repeat, &project))
        .collect::<Vec<_>>();
    complete::assert_repeated(&runs);
    for run in &runs {
        activity(run);
    }
    for run in &runs {
        complete::assert_complete(run);
    }
}

pub(super) fn modifier(metadata: &str) -> &str {
    metadata
        .split_once("<part id=\"3\" subtype=\"modifier_part\">")
        .unwrap()
        .1
        .split_once("</part>")
        .unwrap()
        .0
}
