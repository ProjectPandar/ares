//! Runtime Native `PrintStatistics` oracle (`tools/chain-probe/print-statistics.nix`):
//! `print.print_statistics()` after export (`GCode.cpp:1904-1940`) and its
//! `GCodeProcessorResult` sources. Outputs never enter the source tree.
use super::{OrcaRunner, Stage, StageError, cache_identity, fs, io, lock, publish, stages};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

const KEY: &str = "ARES_PRINT_STATISTICS_BIN";
const OUTPUTS: [&str; 2] = ["plate_1.print_statistics.json", "plate_1.gcode"];
/// The probe is built from the seed-replay source, which reads this seed
/// unconditionally (`GCode.cpp:2707`); 0 is the seeded reference's seed.
const SEED: (&str, &str) = ("ORCA_REPLAY_SEED", "0");

impl OrcaRunner {
    /// The dump and the cache entry holding it, or `None` when the producer is not configured.
    pub(crate) fn print_statistics_reference(
        &self,
        input: &Path,
    ) -> Result<Option<(Value, PathBuf)>, StageError> {
        let Some(bin) = crate::native_environment::binary(KEY) else {
            return Ok(None);
        };
        let stage = Stage::Slice;
        // A multi-threaded Native picks a different travel order than the
        // single-CPU reference Ares matches; run under ares-pin-runner.
        let parallelism = std::thread::available_parallelism()
            .map_err(|error| io(stage, error))?
            .get();
        if parallelism != 1 {
            return Err(io(
                stage,
                format!("{KEY} needs a process pinned to one CPU, found {parallelism}"),
            ));
        }
        let bin = bin.canonicalize().map_err(|error| io(stage, error))?;
        let input = input.canonicalize().map_err(|error| io(stage, error))?;
        // The Nix launcher embeds the wrapped ELF path; hash the ELF it runs.
        let native = bin.with_file_name(".orca-slicer-wrapped");
        let patch = super::super::repo_root().join("tools/chain-probe/print-statistics.patch");
        let components = json!({
            "recipe": "print-statistics-v1",
            "input": cache_identity::raw_digest(&stages::read(&input, stage)?),
            "input_name": input.file_name().map(|name| name.to_string_lossy()),
            "launcher_path": bin,
            "launcher": cache_identity::raw_digest(&stages::read(&bin, stage)?),
            "native_path": native,
            "native": cache_identity::raw_digest(&stages::read(&native, stage)?),
            "patch": cache_identity::raw_digest(&stages::read(&patch, stage)?),
            "parallelism": parallelism,
            "environment": [SEED],
        });
        let producer = self.producer()?;
        let request = cache_identity::digest(&[&serde_json::to_vec(&components).unwrap()]);
        let parent = self
            .cache
            .join("native-cache-v2")
            .join(&producer.digest)
            .join("print-statistics");
        fs::create_dir_all(&parent).map_err(|error| io(stage, error))?;
        let entry = parent.join(&request);
        let _lock = lock(&entry.with_extension("lock"), stage)?;
        if !entry.join("receipt.json").is_file() {
            let staging = tempfile::Builder::new()
                .prefix("print-statistics-")
                .tempdir_in(&parent)
                .map_err(|error| io(stage, error))?;
            stages::run_live_with_env(
                &bin,
                [
                    "--slice".to_owned(),
                    "0".to_owned(),
                    "--outputdir".to_owned(),
                    staging.path().to_string_lossy().into_owned(),
                    input.to_string_lossy().into_owned(),
                ],
                &[SEED],
                stage,
                &self.work,
            )?;
            let mut hashes = serde_json::Map::new();
            for name in OUTPUTS {
                let bytes = stages::read(&staging.path().join(name), stage)?;
                hashes.insert(name.into(), cache_identity::raw_digest(&bytes).into());
            }
            let receipt = json!({"schema": "print-statistics-v1", "request": request,
                "components": components, "producer": producer.receipt, "outputs": hashes,
                "command_work": self.work});
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
            return Err(io(stage, "print statistics cache request mismatch"));
        }
        for name in OUTPUTS {
            let bytes = stages::read(&entry.join(name), stage)?;
            if receipt["outputs"][name] != cache_identity::raw_digest(&bytes) {
                return Err(io(
                    stage,
                    format!("print statistics cache output mismatch: {name}"),
                ));
            }
        }
        let dump = serde_json::from_slice(&stages::read(&entry.join(OUTPUTS[0]), stage)?)
            .map_err(|error| io(stage, error))?;
        Ok(Some((dump, entry)))
    }
}
