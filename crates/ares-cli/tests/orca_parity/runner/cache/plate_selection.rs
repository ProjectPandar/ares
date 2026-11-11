use super::{
    ExportedCase, OrcaRunner, ParityCase, Stage, StageError, cache_identity, fs, io, lock, publish,
    receipt, stages, trace,
};
use serde_json::{Value, json};

impl OrcaRunner {
    /// OrcaSlicer.cpp:5636-5638,5683,6210-6215: preserve selected plate identity.
    pub(crate) fn slice_selected_plate_case(
        &self,
        exported: &ExportedCase,
        plate: u32,
    ) -> Result<ParityCase, StageError> {
        let producer = self.producer()?;
        let project = stages::read(&exported.project_path, Stage::Slice)?;
        let filename = format!("plate_{plate}.gcode");
        let components = json!({
            "recipe": "slice-v2/selected-plate",
            "plate": plate,
            "output_file": filename,
            "project_sha256": cache_identity::raw_digest(&project),
            "project_name": exported.project_path.file_name().unwrap().to_str().unwrap(),
        });
        let request =
            cache_identity::digest(&[b"slice-v2", &serde_json::to_vec(&components).unwrap()]);
        let parent = self
            .cache
            .join("native-cache-v2")
            .join(&producer.digest)
            .join("slice");
        fs::create_dir_all(&parent).map_err(|error| io(Stage::Slice, error))?;
        let entry = parent.join(&request);
        let _lock = lock(&entry.with_extension("lock"), Stage::Slice)?;
        let reference_path = entry.join(&filename);
        let hit = entry.join("receipt.json").is_file() && reference_path.is_file();
        if !hit {
            let staging = tempfile::Builder::new()
                .prefix("selected-plate-")
                .tempdir_in(&parent)
                .map_err(|error| io(Stage::Slice, error))?;
            stages::run_live(
                &self.bin,
                [
                    "--slice".to_owned(),
                    plate.to_string(),
                    "--outputdir".to_owned(),
                    exported.output_dir.to_str().unwrap().to_owned(),
                    exported.project_path.to_str().unwrap().to_owned(),
                ],
                Stage::Slice,
                &self.work,
            )?;
            let reference = stages::read(&exported.output_dir.join(&filename), Stage::Slice)?;
            fs::write(staging.path().join(&filename), &reference)
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
        let record: Value =
            serde_json::from_slice(&stages::read(&entry.join("receipt.json"), Stage::Slice)?)
                .map_err(|error| io(Stage::Slice, error))?;
        if record["producer_digest"] != producer.digest
            || record["request_digest"] != request
            || record["request_components"] != components
            || record["output_sha256"] != cache_identity::raw_digest(&reference)
        {
            return Err(io(
                Stage::Slice,
                "selected plate cache identity/output mismatch",
            ));
        }
        trace(
            &self.work,
            producer,
            &request,
            Stage::Slice,
            if hit { "hit" } else { "miss" },
        )?;
        Ok(ParityCase {
            label: exported.label.clone(),
            project: exported.project.clone(),
            reference,
        })
    }
}
