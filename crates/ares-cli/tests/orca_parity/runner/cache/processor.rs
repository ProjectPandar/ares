//! Runtime Native GCodeProcessor oracle; outputs never enter the source tree.
#[path = "processor_diagnostic.rs"]
mod processor_diagnostic;

use super::{OrcaRunner, Stage, StageError, cache_identity, fs, io, lock, publish, stages};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

fn native_bin() -> PathBuf {
    PathBuf::from(
        std::env::var_os("ARES_PROCGCODE_NATIVE_BIN")
            .expect("ARES_PROCGCODE_NATIVE_BIN must select the pinned v2.5.0 processor ELF"),
    )
}
impl OrcaRunner {
    pub(crate) fn processor_reference(&self, input: &Path) -> Result<Option<PathBuf>, StageError> {
        self.processor_capture(
            input,
            "process",
            &["role-map.json", "moves.csv", "metadata.json"],
        )
    }

    pub(crate) fn role_slice_reference(&self, input: &Path) -> Result<Option<PathBuf>, StageError> {
        self.processor_capture(input, "slice", &["role-map.json", "plate_1.gcode"])
    }

    fn processor_capture(
        &self,
        input: &Path,
        mode: &str,
        outputs: &[&str],
    ) -> Result<Option<PathBuf>, StageError> {
        let Some(gdb) = crate::native_environment::binary("ARES_GDB_BIN") else {
            return Ok(None);
        };
        let stage = Stage::Slice;
        let gdb = gdb.canonicalize().map_err(|error| io(stage, error))?;
        let input = input.canonicalize().map_err(|error| io(stage, error))?;
        let tools = super::super::repo_root().join("tools/chain-probe");
        let launcher = tools.join("role-filament-map.sh");
        let reader = tools.join("role-filament-map.py");
        let components = json!({
            "recipe": "processor-role-map-v2",
            "mode": mode,
            "input": cache_identity::raw_digest(&stages::read(&input, stage)?),
            "input_name": input.file_name().map(|name| name.to_string_lossy()),
            "native_path": native_bin(),
            "native": cache_identity::raw_digest(&stages::read(&native_bin(), stage)?),
            "reader": cache_identity::raw_digest(&stages::read(&reader, stage)?),
            "launcher": cache_identity::raw_digest(&stages::read(&launcher, stage)?),
            "launcher_path": launcher,
            "gdb_path": gdb,
            "gdb": cache_identity::raw_digest(&stages::read(&gdb, stage)?),
        });
        let producer = self.producer()?;
        let request = cache_identity::digest(&[&serde_json::to_vec(&components).unwrap()]);
        let parent = self
            .cache
            .join("native-cache-v2")
            .join(&producer.digest)
            .join("processor");
        fs::create_dir_all(&parent).map_err(|error| io(stage, error))?;
        let entry = parent.join(&request);
        let _lock = lock(&entry.with_extension("lock"), stage)?;
        if !entry.join("receipt.json").is_file() {
            let staging = tempfile::Builder::new()
                .prefix("processor-")
                .tempdir_in(&parent)
                .map_err(|error| io(stage, error))?;
            let capture = staging.path().join("capture");
            stages::run_live(
                &launcher,
                [&input, &capture, &reader, &gdb]
                    .map(|path| path.to_string_lossy().into_owned())
                    .into_iter()
                    .chain([mode.to_owned()]),
                stage,
                &self.work,
            )?;
            let mut output_hashes = serde_json::Map::new();
            for &filename in outputs {
                let bytes = stages::read(&capture.join(filename), stage)?;
                output_hashes.insert(filename.into(), cache_identity::raw_digest(&bytes).into());
            }
            let receipt = json!({"schema":"processor-role-map-v2", "request":request,
                "components":components, "producer":producer.receipt, "outputs":output_hashes,
                "command_work":self.work});
            fs::write(
                staging.path().join("receipt.json"),
                serde_json::to_vec_pretty(&receipt).unwrap(),
            )
            .map_err(|error| io(stage, error))?;
            publish(staging.keep(), &entry, stage)?;
        }
        let receipt: Value =
            serde_json::from_slice(&stages::read(&entry.join("receipt.json"), stage)?)
                .map_err(|error| io(stage, error))?;
        if receipt["request"] != request || receipt["components"] != components {
            return Err(io(stage, "processor cache request mismatch"));
        }
        let capture = entry.join("capture");
        for &filename in outputs {
            let bytes = stages::read(&capture.join(filename), stage)?;
            if receipt["outputs"][filename] != cache_identity::raw_digest(&bytes) {
                return Err(io(
                    stage,
                    format!("processor cache output mismatch: {filename}"),
                ));
            }
        }
        Ok(Some(capture))
    }
}
