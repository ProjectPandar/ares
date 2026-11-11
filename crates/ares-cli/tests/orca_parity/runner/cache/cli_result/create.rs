use super::*;

impl OrcaRunner {
    pub(super) fn create_cli_result_reference(
        &self,
        project: &Path,
        request: ReferenceRequest<'_>,
        entry: &Path,
        components: &Value,
    ) -> Result<(), StageError> {
        let stage = Stage::Slice;
        let ReferenceRequest {
            kind,
            load_settings,
        } = request;
        let producer = self.producer()?;
        let request = cache_identity::digest(&[&serde_json::to_vec(components).unwrap()]);
        let staging = tempfile::Builder::new()
            .prefix("cli-result-")
            .tempdir_in(entry.parent().unwrap())
            .map_err(|error| io(stage, error))?;
        let output = staging.path().join("native");
        fs::create_dir(&output).map_err(|error| io(stage, error))?;
        let command_work = staging.path().join("command");
        fs::create_dir(&command_work).map_err(|error| io(stage, error))?;
        let mut argv = vec!["--slice".to_owned(), kind.slice().to_string()];
        if let Some(lists) = load_settings {
            argv.push("--load-settings".to_owned());
            argv.push(lists.settings.join(";"));
            argv.push("--load-filaments".to_owned());
            argv.push(lists.filaments.join(";"));
        }
        argv.push("--outputdir".to_owned());
        argv.push(output.to_string_lossy().into_owned());
        argv.push(project.to_string_lossy().into_owned());
        let outcome = stages::run_live(&self.bin, argv, stage, &command_work);
        let expected = if cfg!(unix) {
            kind.code().rem_euclid(256)
        } else {
            kind.code()
        };
        let (captured, evidence) = match outcome {
            Err(failure) if failure.kind == stages::FailureKind::Process => {
                if kind.code() == 0
                    || failure.captured.as_ref().map(|c| c.code) != Some(Some(expected))
                {
                    return Err(failure);
                }
                (*failure.captured.unwrap(), failure.evidence.unwrap())
            }
            Err(failure) => return Err(failure),
            Ok(()) if kind.code() == 0 => {
                // The isolated work directory contains exactly this successful command.
                let evidence = fs::read_dir(&command_work)
                    .map_err(|error| io(stage, error))?
                    .next()
                    .unwrap()
                    .map_err(|error| io(stage, error))?
                    .path();
                let status: Value =
                    serde_json::from_slice(&stages::read(&evidence.join("status.json"), stage)?)
                        .map_err(|error| io(stage, error))?;
                if status["code"] != 0 || status["snapshot_incomplete"] != false {
                    return Err(io(stage, "incomplete Native successful command capture"));
                }
                (
                    stages::Captured {
                        code: Some(0),
                        status: status["native_status"].to_string(),
                        stdout: stages::read(&evidence.join("stdout.bin"), stage)?,
                        stderr: stages::read(&evidence.join("stderr.bin"), stage)?,
                    },
                    evidence,
                )
            }
            Ok(()) => {
                return Err(io(
                    stage,
                    format!("expected Native CLI failure {}", kind.code()),
                ));
            }
        };
        let mut files = fs::read_dir(&output)
            .map_err(|error| io(stage, error))?
            .map(|entry| entry.map(|entry| entry.file_name()))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| io(stage, error))?;
        files.sort();
        if files
            != kind
                .native_products()
                .iter()
                .map(std::ffi::OsString::from)
                .collect::<Vec<_>>()
        {
            return Err(io(
                stage,
                format!(
                    "unexpected Native {} product inventory: {files:?}",
                    kind.code()
                ),
            ));
        }
        let result = stages::read(&output.join("result.json"), stage)?;
        let parsed: Value = serde_json::from_slice(&result).map_err(|error| io(stage, error))?;
        if parsed["return_code"] != kind.code() {
            return Err(io(stage, "Native result.json return code mismatch"));
        }
        if matches!(
            kind,
            ResultKind::SequentialLast | ResultKind::CustomGCodeSuccess
        ) {
            let gcode = stages::read(&output.join("plate_1.gcode"), stage)?;
            if gcode.is_empty() {
                return Err(io(stage, "Native last-object control emitted no G-code"));
            }
            fs::write(staging.path().join("plate_1.gcode"), gcode)
                .map_err(|error| io(stage, error))?;
        }
        if matches!(kind, ResultKind::RetainedFirst | ResultKind::RetainedSecond) {
            let filename = kind.native_products()[0];
            let gcode = stages::read(&output.join(filename), stage)?;
            if gcode.is_empty() || parsed["plate_index"] != if kind.slice() == 2 { 2 } else { 1 } {
                return Err(io(stage, "Native retained output/plate identity mismatch"));
            }
            fs::write(staging.path().join(filename), gcode).map_err(|error| io(stage, error))?;
        }
        fs::write(staging.path().join("stdout"), &captured.stdout)
            .map_err(|error| io(stage, error))?;
        fs::write(staging.path().join("stderr"), &captured.stderr)
            .map_err(|error| io(stage, error))?;
        fs::write(staging.path().join("result.json"), result).map_err(|error| io(stage, error))?;
        let mut products = serde_json::Map::new();
        for file in ["stdout", "stderr"]
            .into_iter()
            .chain(kind.native_products().iter().copied())
        {
            products.insert(
                file.into(),
                cache_identity::raw_digest(&stages::read(&staging.path().join(file), stage)?)
                    .into(),
            );
        }
        // Paths in command evidence must refer to the published directory, not vanished staging.
        let published_evidence = entry.join(evidence.strip_prefix(staging.path()).unwrap());
        let receipt = json!({"schema":kind.schema(),"request":request,
            "components":components,"producer":producer.receipt,"exit":captured.code,
            "command_evidence":published_evidence,"outputs":products,"native_products":kind.native_products()});
        fs::write(
            staging.path().join("receipt.json"),
            serde_json::to_vec_pretty(&receipt).unwrap(),
        )
        .map_err(|error| io(stage, error))?;
        publish(staging.keep(), entry, stage)?;
        Ok(())
    }
}
