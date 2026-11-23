use std::{fs, path::Path};

use ares_core::{
    ConfigOverrides, GenerationMetadata, ImportedStl, LoadSettingsFile, LoadSettingsRequest,
    LoadSettingsSliceRequest, SlicedPlate, SlicerContext,
};

use crate::{
    CliFailure, args::DirectSliceArgs, preset_sources::read_preset_files, split_native_list,
};

fn destination(output: &Path, outputdir: Option<&Path>) -> std::path::PathBuf {
    outputdir.map_or_else(
        || output.to_owned(),
        |dir| {
            // Native concatenates the string rather than treating an absolute
            // export filename as overriding --outputdir.
            let mut path = dir.as_os_str().to_owned();
            path.push("/");
            path.push(output);
            path.into()
        },
    )
}

pub(super) async fn run_direct(args: &DirectSliceArgs, output: &Path) -> Result<(), CliFailure> {
    let output = destination(output, args.outputdir.as_deref());
    let stl = args.inputs.iter().all(|path| {
        path.extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("stl"))
    });
    let sliced = args.selection.is_some();
    let placement = if args.arrange.arrange == Some(0) {
        ares_core::Placement::None
    } else {
        ares_core::Placement::Arrange
    };
    let arranged_stl = stl
        && args.inputs.len() > 1
        && args.selection == Some(0)
        && args.arrange.arrange == Some(1);
    let unsupported = [
        (
            args.selection
                .is_some_and(|plate| plate != 1 && !(stl && plate == 0)),
            "--slice",
        ),
        (sliced && args.outputdir.is_some(), "--outputdir"),
        (
            sliced && !stl && args.load_settings.is_some(),
            "--load-settings",
        ),
        (
            sliced && !stl && args.load_filaments.is_some(),
            "--load-filaments",
        ),
        (!stl && args.arrange.arrange == Some(1), "--arrange"),
        (!stl && args.convert_unit, "--convert-unit"),
        (args.orient.is_some_and(|value| value != 0), "--orient"),
        (args.load_defaultfila, "--load-defaultfila"),
        (args.enable_timelapse, "--enable-timelapse"),
        (args.export_statistics, "--export-statistics"),
    ];
    reject_unsupported(&output, &unsupported)?;
    let failed = |detail| CliFailure::project_export(&output, false, detail);
    if args.inputs.len() != 1 && !arranged_stl {
        return Err(failed(
            "3MF export requires exactly one input model".to_owned(),
        ));
    }
    let models = args
        .inputs
        .iter()
        .map(|path| read_model(path, &output))
        .collect::<Result<Vec<_>, _>>()?;
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
    let presets = LoadSettingsRequest {
        loaded_sources: None,
        settings: &sources(&settings),
        filaments: &sources(&filaments),
    };
    let archive = if stl {
        export_stl(
            models,
            presets,
            (args.selection, placement, args.convert_unit),
        )
        .await
    } else {
        let (input, filename) = &models[0];
        if sliced {
            export_saved_sliced(input, filename).await
        } else {
            ares_core::export_3mf(input, filename, presets)
        }
    }
    .map_err(|err| failed(err.to_string()))?;
    fs::write(&output, archive).map_err(|err| failed(err.to_string()))
}

async fn export_stl(
    inputs: Vec<(Vec<u8>, &str)>,
    presets: LoadSettingsRequest<'_>,
    (selection, placement, convert): (Option<i32>, ares_core::Placement, bool),
) -> Result<Vec<u8>, ares_core::SliceError> {
    let filename = inputs[0].1;
    let models = inputs
        .into_iter()
        .map(|(bytes, name)| {
            let mut model = ImportedStl::read(&bytes, name)?;
            if convert && let Some(unit) = model.infer_unit() {
                model.convert_to_millimeters(unit);
            }
            Ok(model)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let [model] = models.as_slice() else {
        let plates = ares_core::slice_stl_models_with_load_settings(
            models,
            stl_slice_request(
                filename,
                presets,
                placement,
                &mut SlicedPlate::default(),
                &ConfigOverrides::default(),
            ),
            0,
            SlicerContext::new(0),
            false,
        )
        .map_err(|failure| *failure.error)?
        .map(|plate| plate.map_err(|failure| *failure.error))
        .collect::<Result<Vec<_>, _>>()?;
        return ares_core::export_arranged_sliced_3mf(plates);
    };
    if selection.is_none() {
        return ares_core::export_imported_stl_3mf(model, presets, placement);
    }
    let processed = ares_core::slice_stl_with_load_settings_for_export(
        model,
        stl_slice_request(
            filename,
            presets,
            placement,
            &mut SlicedPlate::default(),
            &ConfigOverrides::default(),
        ),
        GenerationMetadata::default(),
        SlicerContext::new(0),
    )
    .await?;
    ares_core::export_sliced_3mf(&[], filename, processed, print_object_id(1, 1, false))
}

fn stl_slice_request<'a>(
    filename: &'a str,
    presets: LoadSettingsRequest<'a>,
    placement: ares_core::Placement,
    sliced_plate: &'a mut SlicedPlate,
    overrides: &'a ConfigOverrides,
) -> LoadSettingsSliceRequest<'a> {
    LoadSettingsSliceRequest {
        filename,
        catalog_json: None,
        loaded_sources: presets.loaded_sources,
        uptodate: false,
        settings: presets.settings,
        filaments: presets.filaments,
        plate: None,
        overrides,
        placement,
        orient: false,
        sliced_plate,
    }
}

async fn export_saved_sliced(
    input: &[u8],
    filename: &str,
) -> Result<Vec<u8>, ares_core::SliceError> {
    let project = ares_core::load_project(input)?;
    let first_id = saved_print_object_id(&project)
        .map_err(|detail| ares_core::SliceError::InvalidInput(detail.into()))?;
    let processed = ares_core::slice_project_plate_with_archive(
        input,
        GenerationMetadata::default(),
        Some(1),
        &ConfigOverrides::default(),
        SlicerContext::new(0),
    )
    .await?;
    ares_core::export_sliced_3mf(input, filename, processed, first_id)
}

fn reject_unsupported(output: &Path, unsupported: &[(bool, &str)]) -> Result<(), CliFailure> {
    if let Some((_, option)) = unsupported.iter().find(|(present, _)| *present) {
        return Err(CliFailure::project_export(
            output,
            true,
            format!("unsupported 3MF export option {option}"),
        ));
    }
    Ok(())
}

fn read_model<'a>(path: &'a Path, output: &Path) -> Result<(Vec<u8>, &'a str), CliFailure> {
    let failed = |detail| CliFailure::project_export(output, false, detail);
    let input = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            return Err(CliFailure::missing_input(
                path,
                #[cfg(target_os = "linux")]
                output,
            ));
        }
        Err(err) => return Err(failed(err.to_string())),
    };
    let filename = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| failed("invalid source filename".to_owned()))?;
    Ok((input, filename))
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

    Ok(print_object_id(objects.len() as u32, volumes, true))
}

fn print_object_id(objects: u32, volumes: u32, saved: bool) -> u32 {
    // The direct STL path uses one fresh model object and one volume. The saved
    // path starts with the same ObjectBase allocation sequence.
    let model = 2; // Model + ModelWipeTower (Model.hpp:1704,1753).
    let config = 1; // ModelConfigObject (Model.hpp:86–104).
    let object = 1 + config + 1; // ModelObject + config + LayerHeightProfile (582–588).
    let instance = 1; // ModelInstance (1562–1567).
    let volume = 1 + config + 4 + 8; // ModelVolume, config and FacetsAnnotations (904–935).
    let input = 2 * model + (object + instance) * objects + volume * volumes;
    // The reader's return and the CLI vector insertion each construct a tower
    // but preserve the existing model ID (Model.cpp:489–516; OrcaSlicer.cpp:2008).
    let model_moves = 2;
    let print = 1 + model; // PrintBase and its private model (PrintBase.hpp:390,561).
    let initial_list = 3 + print; // PartPlateList and two PartPlates (PartPlate.cpp:5966–5995).
    let plate_preparation = if saved {
        1 + print // Replacement plate + Print (PartPlate.cpp:6589–6605).
    } else {
        2 // Two more model allocations are avoided when no saved plate is restored.
    };
    let slice_copy = model; // Default construct before copying IDs (OrcaSlicer.cpp:6466–6470).
    let print_object = 1; // Print::apply (PrintApply.cpp:1977).
    input + model_moves + initial_list + plate_preparation + slice_copy + print_object
}

fn sources(
    files: &[(String, String, Option<ares_core::SourceIdentity>)],
) -> Vec<LoadSettingsFile<'_>> {
    files
        .iter()
        .map(|(label, json, source_identity)| LoadSettingsFile {
            label,
            json,
            source_identity: source_identity.as_ref(),
        })
        .collect()
}
