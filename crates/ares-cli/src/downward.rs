use std::{fs, path::Path};

use ares_core::{ConfigSubstitution, DownwardMachine};

use crate::{SliceArgs, args::ProjectActionArgs, failure::CliFailure};

pub(super) fn load(
    args: &ProjectActionArgs,
    output: &Path,
) -> Result<Vec<DownwardMachine>, CliFailure> {
    if !args.downward_check {
        return Ok(Vec::new());
    }
    args.downward_settings
        .as_deref()
        .unwrap_or_default()
        .split(';')
        .filter(|path| !path.is_empty())
        .map(|path| load_one(Path::new(path), output))
        .collect()
}

pub(super) fn report(
    args: &SliceArgs,
    plate: &ares_core::SlicedPlate,
    source: (&str, &[ConfigSubstitution]),
    upward: &[String],
) -> Result<Vec<String>, CliFailure> {
    crate::failure::slicing::check(args, plate, source.0, source.1, upward)?;
    let machines = load(&args.project_actions, args.output())
        .map_err(|error| error.with_upward_report(upward.to_vec()))?;
    if machines.is_empty() {
        return Ok(Vec::new());
    }
    Ok(ares_core::compatible_downward_machines(
        plate
            .object_size
            .expect("downward targets request plate dimensions"),
        &machines,
        false,
    ))
}

fn load_one(path: &Path, output: &Path) -> Result<DownwardMachine, CliFailure> {
    let bytes = fs::read(path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            CliFailure::downward_missing(path, output)
        } else {
            CliFailure::downward_config(output)
        }
    })?;
    DownwardMachine::from_machine_json(&bytes).map_err(|_| CliFailure::downward_config(output))
}
