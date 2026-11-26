//! Original-machine resource bytes for OrcaSlicer.cpp:2517-2536.
use std::{fs, io, path::Path};

use ares_core::UpwardMachineSource;

use crate::failure::CliFailure;

pub(super) fn read_machine_full(
    source: Option<&UpwardMachineSource>,
    resources: &Path,
) -> Result<Option<Vec<u8>>, CliFailure> {
    let Some(original) = source else {
        return Ok(None);
    };
    if original.system_printer().is_empty() {
        return Ok(None);
    }
    let path = resources
        .join("profiles")
        .join("BBL")
        .join("machine_full")
        .join(format!("{}.json", original.system_printer()));
    match fs::read(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}
