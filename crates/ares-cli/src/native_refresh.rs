use std::{
    fs, io,
    path::{Component, Path},
};

use ares_core::{NativeRefreshFile, NativeRefreshPresets, ProfileKind, UpwardMachineSource};

use crate::{
    SliceArgs,
    failure::{CliFailure, slice_error_to_failure},
    split_native_list,
};

pub(super) fn load(
    args: &SliceArgs,
    input: &[u8],
    load_filaments: &[&str],
) -> Result<Option<NativeRefreshPresets>, CliFailure> {
    if !args.uptodate
        || args.project_preset_catalog.is_some()
        || !args
            .input()
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("3mf"))
    {
        return Ok(None);
    }
    ares_core::validate_load_settings_input(input).map_err(|error| {
        slice_error_to_failure(
            error,
            args.output(),
            #[cfg(target_os = "linux")]
            args.plate.unwrap_or(1),
        )
    })?;
    let Some(source) = UpwardMachineSource::from_project(input)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error.to_string()))?
    else {
        return Ok(None);
    };
    let resources = crate::installed_resources::resolve(args.resources_dir.as_deref())?;
    let bbl = resources.join("profiles/BBL");
    let mut presets = NativeRefreshPresets::default();
    let settings = list(args.uptodate_settings.as_deref());
    if settings.is_empty() {
        presets.machine = resource(args, &bbl, "machine_full", source.system_printer())?;
        presets.process = resource(args, &bbl, "process_full", source.system_process())?;
    } else {
        for path in settings {
            let file = explicit(args, path)?;
            let (kind, name, target) = match file.kind() {
                ProfileKind::Machine => ("machine", source.system_printer(), &mut presets.machine),
                ProfileKind::Process => ("process", source.system_process(), &mut presets.process),
                ProfileKind::Filament => {
                    return Err(config_error(
                        args,
                        format!("found invalid config type filament from config {path}"),
                    ));
                }
            };
            if file.name() != name {
                return Err(config_error(
                    args,
                    format!(
                        "wrong {kind} config file {} loaded, current {kind} config name {name}",
                        file.name()
                    ),
                ));
            }
            *target = Some(file);
        }
    }

    if !load_filaments.iter().any(|label| !label.is_empty()) {
        presets.filaments = load_retained_filaments(args, &bbl, source.system_filaments())?;
    }
    if (presets.machine.is_none()
        || presets.process.is_none()
        || presets.filaments.iter().any(Option::is_none))
        && resources.join("profiles").is_dir()
    {
        presets.system_catalog = Some(crate::project_presets::load_resources_catalog(&resources)?);
    }
    Ok(Some(presets))
}

fn load_retained_filaments(
    args: &SliceArgs,
    bbl: &Path,
    names: &[String],
) -> Result<Vec<Option<NativeRefreshFile>>, CliFailure> {
    let files = list(args.uptodate_filaments.as_deref());
    if files.is_empty() {
        return names
            .iter()
            .map(|name| resource(args, bbl, "filament_full", name))
            .collect();
    }
    if files.len() != names.len() {
        return Err(CliFailure::invalid_parameters(
            #[cfg(target_os = "linux")]
            args.output(),
            format!(
                "uptodate_filaments size {} not equal to filament_count {}\n",
                files.len(),
                names.len()
            ),
        ));
    }
    let mut filaments = Vec::with_capacity(files.len());
    for (index, (path, name)) in files.into_iter().zip(names).enumerate() {
        let file = explicit(args, path)?;
        if file.kind() != ProfileKind::Filament {
            return Err(config_error(
                args,
                format!(
                    "unknown config type {} of file {path} in load-filaments",
                    type_name(file.kind())
                ),
            ));
        }
        if file.name() != name {
            return Err(config_error(
                args,
                format!(
                    "wrong filament config file {} loaded, current filament config name {name}, index {index}",
                    file.name()
                ),
            ));
        }
        filaments.push(Some(file));
    }
    Ok(filaments)
}

fn list(value: Option<&str>) -> Vec<&str> {
    value
        .filter(|value| !value.is_empty())
        .map(split_native_list)
        .unwrap_or_default()
}

fn explicit(args: &SliceArgs, label: &str) -> Result<NativeRefreshFile, CliFailure> {
    let bytes = fs::read(label).map_err(|error| {
        config_error(
            args,
            format!("Loading setting file \"{label}\" failed: {error}"),
        )
    })?;
    parse(args, &bytes, Path::new(label))
}

fn resource(
    args: &SliceArgs,
    bbl: &Path,
    kind: &str,
    name: &str,
) -> Result<Option<NativeRefreshFile>, CliFailure> {
    let mut components = Path::new(name).components();
    if !matches!(
        (components.next(), components.next()),
        (Some(Component::Normal(_)), None)
    ) {
        return Ok(None);
    }
    let path = bbl.join(kind).join(format!("{name}.json"));
    match fs::read(&path) {
        Ok(bytes) => {
            let file = parse(args, &bytes, &path)?;
            if kind == "filament_full" && file.kind() != ProfileKind::Filament {
                return Err(config_error(
                    args,
                    format!(
                        "unknown config type {} of file {} in load-filaments",
                        type_name(file.kind()),
                        path.display()
                    ),
                ));
            }
            Ok(Some(file))
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(config_error(
            args,
            format!(
                "Loading setting file \"{}\" failed: {error}",
                path.display()
            ),
        )),
    }
}

fn type_name(kind: ProfileKind) -> &'static str {
    match kind {
        ProfileKind::Machine => "machine",
        ProfileKind::Process => "process",
        ProfileKind::Filament => "filament",
    }
}

fn parse(args: &SliceArgs, bytes: &[u8], path: &Path) -> Result<NativeRefreshFile, CliFailure> {
    NativeRefreshFile::from_json(bytes).map_err(|error| {
        config_error(
            args,
            format!(
                "Loading setting file \"{}\" failed: {error}",
                path.display()
            ),
        )
    })
}

fn config_error(_args: &SliceArgs, message: String) -> CliFailure {
    CliFailure::config_file_error(
        #[cfg(target_os = "linux")]
        _args.output(),
        format!("{message}\n"),
    )
}
