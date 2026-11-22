use std::{
    fs, io,
    path::{Component, Path},
};

use ares_core::{NativeRefreshFile, NativeRefreshPresets, ProfileKind, UpwardMachineSource};

use crate::{SliceArgs, failure::CliFailure, split_native_list};

pub(super) struct RefreshInputs<'a> {
    pub resources_dir: Option<&'a Path>,
    pub settings: Option<&'a str>,
    pub filaments: Option<&'a str>,
    #[cfg(target_os = "linux")]
    pub output: &'a Path,
}

pub(super) fn requested(args: &SliceArgs) -> bool {
    args.uptodate
        && args.project_preset_catalog.is_none()
        && args
            .input()
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("3mf"))
}

pub(super) fn load_admitted(
    args: &SliceArgs,
    input: &ares_core::CliPresetProject<'_>,
    load_filaments: &[&str],
) -> Result<Option<NativeRefreshPresets>, CliFailure> {
    if !requested(args) {
        return Ok(None);
    }
    let Some(source) = UpwardMachineSource::from_project(input.input())
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error.to_string()))?
    else {
        return Ok(None);
    };
    let inputs = RefreshInputs {
        resources_dir: args.resources_dir.as_deref(),
        settings: args.uptodate_settings.as_deref(),
        filaments: args.uptodate_filaments.as_deref(),
        #[cfg(target_os = "linux")]
        output: args.output(),
    };
    load(&inputs, &source, load_filaments).map(Some)
}

pub(super) fn load(
    inputs: &RefreshInputs<'_>,
    source: &UpwardMachineSource,
    load_filaments: &[&str],
) -> Result<NativeRefreshPresets, CliFailure> {
    let resources = crate::installed_resources::resolve(inputs.resources_dir)?;
    let bbl = resources.join("profiles/BBL");
    let mut presets = NativeRefreshPresets::default();
    let settings = list(inputs.settings);
    if settings.is_empty() {
        presets.machine = resource(inputs, &bbl, "machine_full", source.system_printer())?;
        presets.process = resource(inputs, &bbl, "process_full", source.system_process())?;
    } else {
        for path in settings {
            let file = explicit(inputs, path)?;
            let (kind, name, target) = match file.kind() {
                ProfileKind::Machine => ("machine", source.system_printer(), &mut presets.machine),
                ProfileKind::Process => ("process", source.system_process(), &mut presets.process),
                ProfileKind::Filament => {
                    return Err(config_error(
                        inputs,
                        format!("found invalid config type filament from config {path}"),
                    ));
                }
            };
            if file.name() != name {
                return Err(config_error(
                    inputs,
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
        presets.filaments = load_retained_filaments(inputs, &bbl, source.system_filaments())?;
    }
    if (presets.machine.is_none()
        || presets.process.is_none()
        || presets.filaments.iter().any(Option::is_none))
        && resources.join("profiles").is_dir()
    {
        presets.system_catalog = Some(crate::project_presets::load_resources_catalog(&resources)?);
    }
    Ok(presets)
}

fn load_retained_filaments(
    inputs: &RefreshInputs<'_>,
    bbl: &Path,
    names: &[String],
) -> Result<Vec<Option<NativeRefreshFile>>, CliFailure> {
    let files = list(inputs.filaments);
    if files.is_empty() {
        return names
            .iter()
            .map(|name| resource(inputs, bbl, "filament_full", name))
            .collect();
    }
    if files.len() != names.len() {
        return Err(CliFailure::invalid_parameters(
            #[cfg(target_os = "linux")]
            inputs.output,
            format!(
                "uptodate_filaments size {} not equal to filament_count {}\n",
                files.len(),
                names.len()
            ),
        ));
    }
    let mut filaments = Vec::with_capacity(files.len());
    for (index, (path, name)) in files.into_iter().zip(names).enumerate() {
        let file = explicit(inputs, path)?;
        if file.kind() != ProfileKind::Filament {
            return Err(config_error(
                inputs,
                format!(
                    "unknown config type {} of file {path} in load-filaments",
                    type_name(file.kind())
                ),
            ));
        }
        if file.name() != name {
            return Err(config_error(
                inputs,
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

fn explicit(inputs: &RefreshInputs<'_>, label: &str) -> Result<NativeRefreshFile, CliFailure> {
    let bytes = fs::read(label).map_err(|error| {
        config_error(
            inputs,
            format!("Loading setting file \"{label}\" failed: {error}"),
        )
    })?;
    parse(inputs, &bytes, Path::new(label))
}

fn resource(
    inputs: &RefreshInputs<'_>,
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
            let file = parse(inputs, &bytes, &path)?;
            if kind == "filament_full" && file.kind() != ProfileKind::Filament {
                return Err(config_error(
                    inputs,
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
            inputs,
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

fn parse(
    inputs: &RefreshInputs<'_>,
    bytes: &[u8],
    path: &Path,
) -> Result<NativeRefreshFile, CliFailure> {
    NativeRefreshFile::from_json(bytes).map_err(|error| {
        config_error(
            inputs,
            format!(
                "Loading setting file \"{}\" failed: {error}",
                path.display()
            ),
        )
    })
}

fn config_error(inputs: &RefreshInputs<'_>, message: String) -> CliFailure {
    #[cfg(not(target_os = "linux"))]
    let _ = inputs;
    CliFailure::config_file_error(
        #[cfg(target_os = "linux")]
        inputs.output,
        format!("{message}\n"),
    )
}
