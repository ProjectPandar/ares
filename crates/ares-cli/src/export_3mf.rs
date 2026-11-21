use std::{fs, path::Path};

use ares_core::{LoadSettingsFile, LoadSettingsRequest};

use crate::{CliFailure, SliceArgs, read_preset_files, split_native_list};

pub(super) async fn run(args: &SliceArgs, output: &Path) -> Result<(), CliFailure> {
    let output = args.outputdir.as_ref().map_or_else(
        || output.to_owned(),
        |dir| {
            // Native concatenates the string rather than treating an absolute
            // export filename as overriding --outputdir.
            let mut path = dir.as_os_str().to_owned();
            path.push("/");
            path.push(output);
            path.into()
        },
    );
    let sliced = args.slice_for_export == Some(1);
    let unsupported = [
        (args.slice_for_export.is_some() && !sliced, "--slice"),
        (sliced && args.outputdir.is_some(), "--outputdir"),
        (sliced && args.load_settings.is_some(), "--load-settings"),
        (sliced && args.load_filaments.is_some(), "--load-filaments"),
        (args.min_save.is_some_and(|value| value != 0), "--min-save"),
        (args.load_slicedata.is_some(), "--load-slicedata"),
        (args.export_slicedata.is_some(), "--export-slicedata"),
        (args.arrange.is_some_and(|value| value != 0), "--arrange"),
        (args.orient != 0, "--orient"),
        (args.plate.is_some(), "--plate"),
        (args.dump_moves.is_some(), "--dump-moves"),
        (args.overrides.is_some(), "--overrides"),
        (args.options.is_some(), "--options"),
        (args.output_path.is_some(), "-o"),
        (
            args.project_preset_catalog.is_some(),
            "--project-preset-catalog",
        ),
        (args.resources_dir.is_some(), "--resources-dir"),
        (args.progress, "--progress"),
    ];
    if let Some((_, option)) = unsupported.into_iter().find(|(present, _)| *present) {
        return Err(CliFailure::project_export(
            &output,
            true,
            format!("unsupported 3MF export option {option}"),
        ));
    }
    let failed = |detail| CliFailure::project_export(&output, false, detail);
    let input = match fs::read(&args.input) {
        Ok(bytes) => bytes,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            return Err(CliFailure::missing_input(
                &args.input,
                #[cfg(target_os = "linux")]
                &output,
            ));
        }
        Err(err) => return Err(failed(err.to_string())),
    };
    let filename = args
        .input
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| failed("invalid source filename".to_owned()))?;
    if sliced {
        if !filename.to_ascii_lowercase().ends_with(".3mf") {
            return Err(CliFailure::project_export(
                &output,
                true,
                "sliced export requires a saved 3MF project".to_owned(),
            ));
        }
        let project = ares_core::load_project(&input).map_err(|err| failed(err.to_string()))?;
        let print_object_id =
            saved_print_object_id(&project).map_err(|detail| failed(detail.into()))?;
        let processed = ares_core::slice_project_plate_with_archive(
            &input,
            ares_core::GenerationMetadata::default(),
            Some(1),
            &ares_core::ConfigOverrides::default(),
            ares_core::SlicerContext::new(0),
        )
        .await
        .map_err(|err| failed(err.to_string()))?;
        let bytes = ares_core::export_sliced_3mf(&input, filename, processed, print_object_id)
            .map_err(|err| failed(err.to_string()))?;
        return fs::write(&output, bytes).map_err(|err| failed(err.to_string()));
    }
    let settings = read_preset_files(
        &args
            .load_settings
            .as_deref()
            .map_or_else(Vec::new, split_native_list),
    )?;
    let filaments = read_preset_files(
        &args
            .load_filaments
            .as_deref()
            .map_or_else(Vec::new, split_native_list),
    )?;
    let bytes = ares_core::export_3mf(
        &input,
        filename,
        LoadSettingsRequest {
            settings: &sources(&settings),
            filaments: &sources(&filaments),
        },
    )
    .map_err(|err| failed(err.to_string()))?;
    fs::write(&output, bytes).map_err(|err| failed(err.to_string()))
}

fn saved_print_object_id(project: &ares_core::Project) -> Result<u32, &'static str> {
    if project.plates().len() != 1 {
        return Err("sliced export requires one saved plate");
    }
    let objects = project.objects();
    let mut volumes = 0;
    for object in objects {
        let [instance] = object.instances() else {
            return Err("sliced export requires one saved instance per object");
        };
        if !instance.printable()
            || object
                .volumes()
                .iter()
                .any(|volume| volume.volume_type() != ares_core::ProjectVolumeType::ModelPart)
        {
            return Err("sliced export requires printable instances of ordinary model parts");
        }
        volumes += object.volumes().len() as u32;
    }

    // The saved-plate path starts with a fresh ObjectBase counter.
    let model = 2; // Model + ModelWipeTower (Model.hpp:1704,1753).
    let config = 1; // ModelConfigObject (Model.hpp:86–104).
    let object = 1 + config + 1; // ModelObject + config + LayerHeightProfile (582–588).
    let instance = 1; // ModelInstance (1562–1567).
    let volume = 1 + config + 4 + 8; // ModelVolume, config and FacetsAnnotations (904–935).
    let input = 2 * model + (object + instance) * objects.len() as u32 + volume * volumes;
    // The reader's return and the CLI vector insertion each construct a tower
    // but preserve the existing model ID (Model.cpp:489–516; OrcaSlicer.cpp:2008).
    let model_moves = 2;
    let print = 1 + model; // PrintBase and its private model (PrintBase.hpp:390,561).
    let initial_list = 3 + print; // PartPlateList and two PartPlates (PartPlate.cpp:5966–5995).
    let saved_plate = 1 + print; // Replacement plate + Print (PartPlate.cpp:6589–6605).
    let slice_copy = model; // Default construct before copying IDs (OrcaSlicer.cpp:6466–6470).
    let print_object = 1; // Print::apply (PrintApply.cpp:1977).
    Ok(input + model_moves + initial_list + saved_plate + slice_copy + print_object)
}

fn sources(files: &[(String, String)]) -> Vec<LoadSettingsFile<'_>> {
    files
        .iter()
        .map(|(label, json)| LoadSettingsFile { label, json })
        .collect()
}
