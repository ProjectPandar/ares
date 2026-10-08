//! Runtime Native GCodeProcessor oracle; outputs never enter the source tree.
use super::{OrcaRunner, Stage, StageError, cache_identity, fs, io, lock, publish, stages};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

const NATIVE: &str = "/nix/store/1bq61dyqxb8yjdsaaq697ii6hbhb5cxm-orca-slicer-procgcode-2.4.2/bin/.orca-slicer-wrapped";
const OUTPUTS: [&str; 3] = ["role-map.json", "moves.csv", "metadata.json"];

impl OrcaRunner {
    pub(crate) fn processor_reference(&self, input: &Path) -> Result<PathBuf, StageError> {
        let stage = Stage::Slice;
        let input = input.canonicalize().map_err(|error| io(stage, error))?;
        let tools = super::super::repo_root().join("tools/chain-probe");
        let launcher = tools.join("role-filament-map.sh");
        let reader = tools.join("role-filament-map.py");
        let components = json!({
            "recipe": "processor-role-map-v1",
            "input": cache_identity::raw_digest(&stages::read(&input, stage)?),
            "native": cache_identity::raw_digest(&stages::read(Path::new(NATIVE), stage)?),
            "reader": cache_identity::raw_digest(&stages::read(&reader, stage)?),
            "launcher": cache_identity::raw_digest(&stages::read(&launcher, stage)?),
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
            stages::run(
                &launcher,
                [&input, &capture, &reader].map(|path| path.to_string_lossy().into_owned()),
                stage,
                &self.work,
            )?;
            let mut outputs = serde_json::Map::new();
            for filename in OUTPUTS {
                let bytes = stages::read(&capture.join(filename), stage)?;
                outputs.insert(filename.into(), cache_identity::raw_digest(&bytes).into());
            }
            let receipt = json!({"schema":"processor-role-map-v1", "request":request,
                "components":components, "producer":producer.receipt, "outputs":outputs,
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
        for filename in OUTPUTS {
            let bytes = stages::read(&capture.join(filename), stage)?;
            if receipt["outputs"][filename] != cache_identity::raw_digest(&bytes) {
                return Err(io(
                    stage,
                    format!("processor cache output mismatch: {filename}"),
                ));
            }
        }
        Ok(capture)
    }
}
