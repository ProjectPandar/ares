use std::{fs, io, io::Write, path::Path};

use crate::failure::{CliFailure, substitution_warnings};

pub(super) fn run(input: &Path, destination: &Path) -> Result<(), CliFailure> {
    if !input
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("3mf"))
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "--export-settings requires a saved 3MF project",
        )
        .into());
    }
    let project = fs::read(input)?;
    let json = ares_core::export_project_settings(&project).map_err(io::Error::other)?;
    let substitutions = crate::substitutions(&project);
    eprint!(
        "{}",
        substitution_warnings(&input.to_string_lossy(), &substitutions)
    );
    write(destination, &json)
}

pub(super) fn write(destination: &Path, json: &[u8]) -> Result<(), CliFailure> {
    if destination == Path::new("-") {
        io::stdout().lock().write_all(json)?;
    } else {
        // ConfigBase::save_to_json serializes before replacing the destination.
        let parent = destination
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        let mut file = tempfile::NamedTempFile::new_in(parent)?;
        file.write_all(json)?;
        file.persist(destination).map_err(|error| error.error)?;
    }
    Ok(())
}
