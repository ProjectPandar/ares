use super::{
    OrcaRunner, Stage, StageError, cache_identity, fs, io, lock, publish, receipt, stages, trace,
};
use serde_json::json;
use std::path::Path;

impl OrcaRunner {
    pub(crate) fn slice_stl_with_settings(
        &self,
        settings: [&Path; 3],
        model: &Path,
    ) -> Result<Vec<u8>, StageError> {
        let producer = self.producer()?;
        let mut inputs = Vec::new();
        for path in settings.into_iter().chain([model]) {
            let bytes = stages::read(path, Stage::Slice)?;
            inputs.push(json!({
                "name": path.file_name().unwrap().to_str().unwrap(),
                "sha256": cache_identity::raw_digest(&bytes),
            }));
        }
        let components = json!({"recipe": "direct-stl-v1/arrange-0/plate-0", "inputs": inputs});
        let request =
            cache_identity::digest(&[b"direct-stl-v1", &serde_json::to_vec(&components).unwrap()]);
        let parent = self
            .cache
            .join("native-cache-v2")
            .join(&producer.digest)
            .join("slice");
        fs::create_dir_all(&parent).map_err(|error| io(Stage::Slice, error))?;
        let entry = parent.join(&request);
        let _lock = lock(&entry.with_extension("lock"), Stage::Slice)?;
        let reference_path = entry.join("plate_1.gcode");
        let hit = entry.join("receipt.json").is_file() && reference_path.is_file();
        let output = self.work.join("native");
        fs::create_dir_all(&output).map_err(|error| io(Stage::Slice, error))?;
        if !hit {
            let staging = tempfile::Builder::new()
                .prefix("direct-stl-")
                .tempdir_in(&parent)
                .map_err(|error| io(Stage::Slice, error))?;
            stages::run_live(
                &self.bin,
                [
                    "--load-settings".to_owned(),
                    format!(
                        "{};{}",
                        settings[0].to_str().unwrap(),
                        settings[1].to_str().unwrap(),
                    ),
                    "--load-filaments".to_owned(),
                    settings[2].to_str().unwrap().to_owned(),
                    "--arrange".to_owned(),
                    "0".to_owned(),
                    "--slice".to_owned(),
                    "0".to_owned(),
                    "--outputdir".to_owned(),
                    output.to_str().unwrap().to_owned(),
                    model.to_str().unwrap().to_owned(),
                ],
                Stage::Slice,
                &self.work,
            )?;
            let reference = stages::read(&output.join("plate_1.gcode"), Stage::Slice)?;
            fs::write(staging.path().join("plate_1.gcode"), &reference)
                .map_err(|error| io(Stage::Slice, error))?;
            receipt(
                staging.path(),
                producer,
                (&request, Stage::Slice, &components),
                &reference,
                &self.work,
            )?;
            publish(staging.keep(), &entry, Stage::Slice)?;
        }
        let reference = stages::read(&reference_path, Stage::Slice)?;
        fs::write(output.join("plate_1.gcode"), &reference)
            .map_err(|error| io(Stage::Slice, error))?;
        trace(
            &self.work,
            producer,
            &request,
            Stage::Slice,
            if hit { "hit" } else { "miss" },
        )?;
        Ok(reference)
    }
}
