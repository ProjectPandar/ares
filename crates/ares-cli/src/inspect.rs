use std::{fs, io, path::Path};

use ares_core::{ConfigOverrides, inspection};
use clap::{ArgMatches, parser::ValueSource};
use serde_json::json;

use crate::{args::SliceArgs, failure::CliFailure};

pub(super) fn run(args: &SliceArgs, matches: &ArgMatches) -> Result<(), CliFailure> {
    let outputdir = args.outputdir.as_deref().unwrap_or_else(|| {
        args.output
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."))
    });
    let output = outputdir.join("result.json");
    let has_input = matches.value_source("input") == Some(ValueSource::CommandLine);
    if args.inspect_paint {
        let conflicting = [
            ("info", "info"),
            ("slice_plate", "slice"),
            ("plate", "slice"),
            ("export_stl", "export-stl"),
            ("export_3mf", "export-3mf"),
            ("dump_moves", "dump-moves"),
        ]
        .into_iter()
        .filter(|&(id, _)| matches.value_source(id) == Some(ValueSource::CommandLine))
        .min_by_key(|&(id, _)| matches.index_of(id));
        let message = match conflicting {
            Some((_, other)) => Some(format!("--inspect-paint cannot be combined with --{other}")),
            None if !has_input => {
                Some("--inspect-paint needs an input file or --load-assemble-list".to_owned())
            }
            None => None,
        };
        if let Some(message) = message {
            eprintln!("{message}");
            println!("run found error, return -2, exit...");
            receipt(&output, -2);
            std::process::exit(-2);
        }
    }
    if !args.inspect_paint && (args.export_stl || args.export_3mf.is_some()) {
        return Err(
            io::Error::new(io::ErrorKind::Unsupported, "model export is not supported").into(),
        );
    }
    if has_input {
        let source = args.input.to_string_lossy();
        let name = args
            .input
            .file_name()
            .unwrap_or(args.input.as_os_str())
            .to_string_lossy();
        let bytes = fs::read(&args.input).map_err(|error| {
            if error.kind() == io::ErrorKind::NotFound {
                CliFailure::missing_input(
                    &args.input,
                    #[cfg(target_os = "linux")]
                    &output,
                )
            } else {
                error.into()
            }
        })?;
        let overrides = args
            .overrides
            .as_deref()
            .map_or_else(
                || Ok(ConfigOverrides::default()),
                |raw| ConfigOverrides::from_json(raw.as_bytes()),
            )
            .map_err(io::Error::other)?;
        let map_error = |error| {
            crate::slice_error_to_failure(
                error,
                &output,
                #[cfg(target_os = "linux")]
                0,
            )
        };
        let model =
            inspection::load(&bytes, &name, &overrides, &args.transforms).map_err(map_error)?;
        if args.inspect_paint {
            let (objects, summary) = model.inspect_paint().map_err(map_error)?;
            print_json(json!({
                "frame": "mesh_local",
                "note": "Coordinates are mesh-local (each volume's own frame). Paint gizmos operate in this frame.",
                "objects": objects,
                "sources": [source],
                "summary": summary,
            }))?;
        } else {
            for object in model.objects() {
                print_info(&name, object.mesh_statistics().map_err(map_error)?);
            }
        }
    }
    receipt(&output, 0);
    Ok(())
}

fn print_json(value: serde_json::Value) -> Result<(), io::Error> {
    println!(
        "{}",
        serde_json::to_string_pretty(&value).map_err(io::Error::other)?
    );
    Ok(())
}

fn print_info(name: &str, stats: inspection::MeshStatistics) {
    println!("[{name}]");
    for (key, triple) in [
        ("size", stats.bounds.size),
        ("min", stats.bounds.min),
        ("max", stats.bounds.max),
    ] {
        for (axis, value) in ["x", "y", "z"].into_iter().zip(triple) {
            println!("{key}_{axis} = {value:.6}");
        }
    }
    println!("number_of_facets = {}", stats.facets);
    println!(
        "manifold = {}",
        if stats.open_edges == 0 { "yes" } else { "no" }
    );
    if stats.open_edges != 0 {
        println!("open_edges = {}", stats.open_edges);
    }
    println!("number_of_parts =  {}", stats.parts);
    println!("volume = {:.6}", stats.volume);
}

fn receipt(_output: &Path, _return_code: i32) {
    #[cfg(target_os = "linux")]
    let _ = crate::result_json::ResultJson {
        return_code: _return_code,
        error_string: if _return_code == 0 {
            "Success."
        } else {
            "Invalid parameters to the slicer."
        },
        plate_index: 0,
        layer_height: 0.0,
        wall_loops: 0,
        sparse_infill_density: 0.0,
        upward_machines: &[],
        sliced_plates: &[],
        warnings: &[],
    }
    .write(_output);
}
