//! Existing processor diagnostics: raw color metadata, stable marker indices, dynamics.
use super::{OrcaRunner, Stage, StageError, cache_identity, fs, io, lock, publish, stages};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

enum Recipe {
    RawColor,
    StableColor,
    Dynamics,
    StableDynamics,
}

impl Recipe {
    fn key(&self) -> &'static str {
        match self {
            Self::RawColor => "ARES_PROCGCODE_BIN",
            Self::StableColor => "ARES_PROCGCODE_STABLE_BIN",
            Self::Dynamics => "ARES_PROCGCODE_DYNAMICS_BIN",
            Self::StableDynamics => "ARES_PROCGCODE_DYNAMICS_STABLE_BIN",
        }
    }

    fn patches(&self) -> &'static [&'static str] {
        match self {
            Self::RawColor => &["procgcode.patch"],
            Self::StableColor => &["procgcode.patch", "procgcode-stable-options.patch"],
            Self::Dynamics => &["procgcode.patch", "procgcode-dynamics.patch"],
            Self::StableDynamics => &[
                "procgcode.patch",
                "procgcode-dynamics.patch",
                "procgcode-stable-options.patch",
            ],
        }
    }

    fn products(&self) -> &'static [&'static str] {
        match self {
            Self::RawColor | Self::StableColor => &["native.json", "native.csv"],
            Self::Dynamics | Self::StableDynamics => &["native-dynamics.csv", "native.csv"],
        }
    }
}

impl OrcaRunner {
    pub(crate) fn processor_raw_color_reference(
        &self,
        input: &Path,
    ) -> Result<Option<PathBuf>, StageError> {
        self.diagnostic_reference(input, Recipe::RawColor)
    }

    pub(crate) fn processor_stable_color_reference(
        &self,
        input: &Path,
    ) -> Result<Option<PathBuf>, StageError> {
        self.diagnostic_reference(input, Recipe::StableColor)
    }

    pub(crate) fn processor_dynamics_reference(
        &self,
        input: &Path,
    ) -> Result<Option<PathBuf>, StageError> {
        self.diagnostic_reference(input, Recipe::Dynamics)
    }

    pub(crate) fn processor_stable_dynamics_reference(
        &self,
        input: &Path,
    ) -> Result<Option<PathBuf>, StageError> {
        self.diagnostic_reference(input, Recipe::StableDynamics)
    }

    fn diagnostic_reference(
        &self,
        input: &Path,
        recipe: Recipe,
    ) -> Result<Option<PathBuf>, StageError> {
        let Some(bin) = crate::native_environment::binary(recipe.key()) else {
            return Ok(None);
        };
        let stage = Stage::Slice;
        let bin = bin.canonicalize().map_err(|error| io(stage, error))?;
        let input = input.canonicalize().map_err(|error| io(stage, error))?;
        // These approved diagnostic launchers embed the pinned wrapped ELF path.
        // Validate that link; hashing a Nix launcher alone misses the actual processor.
        let native = bin.with_file_name(".orca-slicer-wrapped");
        let launcher_bytes = stages::read(&bin, stage)?;
        let native_name = native.as_os_str().as_encoded_bytes();
        if bin != native
            && !launcher_bytes
                .windows(native_name.len())
                .any(|part| part == native_name)
        {
            return Err(io(
                stage,
                format!("{} does not declare wrapped ELF {native:?}", recipe.key()),
            ));
        }
        let native_bytes = stages::read(&native, stage)?;
        let tools = super::super::super::repo_root().join("tools/chain-probe");
        let mut patches = serde_json::Map::new();
        for patch in recipe.patches() {
            patches.insert(
                (*patch).into(),
                cache_identity::raw_digest(&stages::read(&tools.join(patch), stage)?).into(),
            );
        }
        let components = json!({
            "recipe": "processor-diagnostic-v1", "diagnostic": recipe.key(),
            "input": cache_identity::raw_digest(&stages::read(&input, stage)?),
            "input_name": input.file_name(), "launcher_path": bin,
            "launcher": cache_identity::raw_digest(&launcher_bytes),
            "native_path": native, "native": cache_identity::raw_digest(&native_bytes),
            "patches": patches, "products": recipe.products(),
        });
        let producer = self.producer()?;
        let request = cache_identity::digest(&[&serde_json::to_vec(&components).unwrap()]);
        let parent = self
            .cache
            .join("native-cache-v2")
            .join(&producer.digest)
            .join("processor-diagnostic");
        fs::create_dir_all(&parent).map_err(|error| io(stage, error))?;
        let entry = parent.join(&request);
        let _lock = lock(&entry.with_extension("lock"), stage)?;
        let products = recipe.products().iter().copied().chain([
            "stdout.bin",
            "stderr.bin",
            "status.json",
            "command.json",
            "lifecycle.json",
        ]);
        let hit = entry.join("receipt.json").is_file();
        if !hit {
            let staging = tempfile::Builder::new()
                .prefix("diagnostic-")
                .tempdir_in(&parent)
                .map_err(|error| io(stage, error))?;
            let commands = tempfile::Builder::new()
                .prefix("processor-diagnostic-")
                .tempdir_in(&self.work)
                .map_err(|error| io(stage, error))?
                .keep();
            let option = match recipe {
                Recipe::RawColor | Recipe::StableColor => "--dump-color-change",
                Recipe::Dynamics | Recipe::StableDynamics => "--dump-move-dynamics",
            };
            stages::run(
                &bin,
                [
                    "--process-gcode".to_owned(),
                    input.to_string_lossy().into_owned(),
                    option.to_owned(),
                    staging
                        .path()
                        .join(recipe.products()[0])
                        .to_string_lossy()
                        .into_owned(),
                    "--dump-moves".to_owned(),
                    staging
                        .path()
                        .join("native.csv")
                        .to_string_lossy()
                        .into_owned(),
                ],
                stage,
                &commands,
            )?;
            let command = fs::read_dir(&commands)
                .map_err(|error| io(stage, error))?
                .next()
                .unwrap()
                .map_err(|error| io(stage, error))?
                .path();
            for name in [
                "stdout.bin",
                "stderr.bin",
                "status.json",
                "command.json",
                "lifecycle.json",
            ] {
                fs::copy(command.join(name), staging.path().join(name))
                    .map_err(|error| io(stage, error))?;
            }
            let mut hashes = serde_json::Map::new();
            for name in products.clone() {
                hashes.insert(
                    name.into(),
                    cache_identity::raw_digest(&stages::read(&staging.path().join(name), stage)?)
                        .into(),
                );
            }
            fs::write(staging.path().join("receipt.json"), serde_json::to_vec_pretty(&json!({
                "schema": "processor-diagnostic-v1", "request": request, "components": components,
                "producer": producer.receipt, "outputs": hashes, "command_work": commands,
            })).unwrap()).map_err(|error| io(stage, error))?;
            publish(staging.keep(), &entry, stage)?;
        }
        let receipt: Value =
            serde_json::from_slice(&stages::read(&entry.join("receipt.json"), stage)?)
                .map_err(|error| io(stage, error))?;
        if receipt["request"] != request || receipt["components"] != components {
            return Err(io(stage, "processor diagnostic cache request mismatch"));
        }
        for name in products {
            let bytes = stages::read(&entry.join(name), stage)?;
            if receipt["outputs"][name] != cache_identity::raw_digest(&bytes) {
                return Err(io(
                    stage,
                    format!("processor diagnostic cache output mismatch: {name}"),
                ));
            }
        }
        fs::write(
            self.work.join(format!("{}.json", recipe.key())),
            serde_json::to_vec_pretty(&json!({
                "request": request, "entry": entry, "cache_hit": hit,
                "input": input, "diagnostic": recipe.key(),
            }))
            .unwrap(),
        )
        .map_err(|error| io(stage, error))?;
        Ok(Some(entry))
    }
}
