//! Shared public Native → API → CLI whole-output parity seam.

use crate::{OrcaRunner, golden, runner::stages};
use sha2::{Digest, Sha256};
use std::{fs, path::PathBuf, process::Command};

pub(crate) struct NativeCase {
    pub(crate) root: PathBuf,
    pub(crate) native: String,
    project: Vec<u8>,
}

pub(crate) fn sha(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

impl NativeCase {
    pub(crate) fn slice(runner: &OrcaRunner, project: &[u8], prefix: &str) -> Self {
        let root = tempfile::Builder::new()
            .prefix(prefix)
            .tempdir_in(crate::artifacts::root_from_env().unwrap())
            .unwrap()
            .keep();
        let input = root.join("input.3mf");
        let output = root.join("native");
        fs::write(&input, project).unwrap();
        fs::create_dir(&output).unwrap();
        stages::run(
            runner.bin(),
            [
                "--slice".to_owned(),
                "0".to_owned(),
                "--outputdir".to_owned(),
                output.to_str().unwrap().to_owned(),
                input.to_str().unwrap().to_owned(),
            ],
            stages::Stage::Slice,
            &root,
        )
        .unwrap();
        let bytes = fs::read(output.join("plate_1.gcode")).unwrap();
        fs::write(root.join("native.gcode"), &bytes).unwrap();
        Self {
            root,
            native: String::from_utf8(bytes).unwrap(),
            project: project.to_vec(),
        }
    }

    pub(crate) fn compare(&self, metadata: ares_core::GenerationMetadata) -> (Vec<u8>, Vec<u8>) {
        let api = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(ares_core::slice_project_plate_validated(
                &self.project,
                metadata,
                None,
                &ares_core::ConfigOverrides::default(),
                ares_core::SlicerContext::new(0),
            ));
        if let Ok(bytes) = &api {
            fs::write(self.root.join("ares.api.gcode"), bytes).unwrap();
        }
        let cli_path = self.root.join("ares.cli.gcode");
        let cli = Command::new(env!("CARGO_BIN_EXE_ares"))
            .args(["slice", "-o"])
            .arg(&cli_path)
            .arg(self.root.join("input.3mf"))
            .output()
            .unwrap();
        fs::write(self.root.join("ares.cli.stdout"), &cli.stdout).unwrap();
        fs::write(self.root.join("ares.cli.stderr"), &cli.stderr).unwrap();
        let cli_bytes = fs::read(cli_path);
        let api_comparison = api
            .as_ref()
            .map_err(|error| format!("API: {error:?}"))
            .and_then(|bytes| {
                golden::compare_ordered_bytes_generator_only(self.native.as_bytes(), bytes)
            });
        let cli_comparison = cli_bytes
            .as_ref()
            .map_err(|error| format!("CLI output: {error}"))
            .and_then(|bytes| {
                golden::compare_ordered_bytes_generator_only(self.native.as_bytes(), bytes)
            });
        fs::write(
            self.root.join("comparison.json"),
            serde_json::to_vec_pretty(&serde_json::json!({
                "input_sha256": sha(&self.project),
                "native_sha256": sha(self.native.as_bytes()),
                "native_brim_blocks": self.native.matches(";TYPE:Brim\n").count(),
                "native_skirt_blocks": self.native.matches(";TYPE:Skirt\n").count(),
                "api_error": api.as_ref().err().map(|error| format!("{error:?}")),
                "api_comparison": api_comparison,
                "cli_code": cli.status.code(),
                "cli_comparison": cli_comparison,
                "strict_parity_required": true,
            }))
            .unwrap(),
        )
        .unwrap();
        api_comparison.unwrap_or_else(|error| panic!("API at {:?}: {error}", self.root));
        assert!(cli.status.success(), "CLI at {:?}", self.root);
        cli_comparison.unwrap_or_else(|error| panic!("CLI at {:?}: {error}", self.root));
        let api = api.unwrap();
        let cli = cli_bytes.unwrap();
        assert_eq!(api, cli, "API/CLI raw equality");
        (api, cli)
    }
}
