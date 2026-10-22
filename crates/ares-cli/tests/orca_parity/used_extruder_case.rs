//! Shared success-required public cases for writer-used extruder reductions.

use crate::{
    paint_color_activity as activity,
    runner::{
        native_api_cli::{NativeCase, sha},
        native_stream_case,
    },
};
use serde_json::json;
use std::fs;

pub(crate) struct Witness {
    pub(crate) label: &'static str,
    pub(crate) sha: &'static str,
    pub(crate) tool: &'static str,
}

impl Witness {
    fn input(&self, dir: &str) -> Vec<u8> {
        fs::read(
            crate::runner::repo_root()
                .join("tests/parity")
                .join(dir)
                .join(format!("{}.3mf", self.label)),
        )
        .unwrap()
    }

    pub(crate) fn check(&self, dir: &str, upstream: &[&str]) {
        native_stream_case::check(
            (dir, upstream),
            self.label,
            &self.input(dir),
            self.sha,
            (false, |reference, replay| {
                assert_native(reference, replay, self.tool);
            }),
        );
    }
}

pub(crate) fn preflight(dir: &str, upstream: &[&str], cases: &[Witness]) {
    let namespace = format!("{dir}-preflight");
    let mut qualified = Vec::new();
    for case in cases {
        let project = case.input(dir);
        let (reference, replay) = native_stream_case::qualify(
            (&namespace, upstream),
            case.label,
            &project,
            case.sha,
            false,
        );
        assert_native(&reference, &replay, case.tool);
        qualified.push(json!({"case":case.label,"input_sha256":sha(&project),
            "reference":reference.root,"replay":replay.root}));
    }
    fs::write(
        crate::artifacts::root_from_env()
            .unwrap()
            .join(format!("{namespace}.json")),
        serde_json::to_vec_pretty(&qualified).unwrap(),
    )
    .unwrap();
}

fn assert_native(reference: &NativeCase, replay: &NativeCase, tool: &str) {
    let mut activities = Vec::new();
    for native in [reference, replay] {
        let observed = activity::activity(native.native.as_bytes());
        let model = observed["positive_model_XYE_by_tool"].as_object().unwrap();
        assert!(model[tool].as_u64().unwrap() > 0);
        assert!(model.keys().all(|selected| selected == tool));
        fs::write(
            native.root.join("native-activity.json"),
            serde_json::to_vec_pretty(&observed).unwrap(),
        )
        .unwrap();
        activities.push(observed);
    }
    assert_eq!(
        activities[0], activities[1],
        "Native T{tool} model activity replay"
    );
}
