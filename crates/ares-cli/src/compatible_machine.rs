//! Original-machine resource bytes for OrcaSlicer.cpp:2517-2536.
use std::{fs, io, path::Path};

use ares_core::UpwardMachineSource;

use crate::failure::CliFailure;

pub(super) fn load(
    input: &[u8],
    resources: &Path,
) -> Result<(Option<UpwardMachineSource>, Option<Vec<u8>>), CliFailure> {
    let source = UpwardMachineSource::from_project(input)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error.to_string()))?;
    let Some(original) = &source else {
        return Ok((source, None));
    };
    let bytes = if original.system_printer().is_empty() {
        None
    } else {
        let path = resources
            .join("profiles")
            .join("BBL")
            .join("machine_full")
            .join(format!("{}.json", original.system_printer()));
        match fs::read(path) {
            Ok(bytes) => Some(bytes),
            Err(error) if error.kind() == io::ErrorKind::NotFound => None,
            Err(error) => return Err(error.into()),
        }
    };
    Ok((source, bytes))
}
