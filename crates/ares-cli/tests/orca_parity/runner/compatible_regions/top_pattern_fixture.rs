use crate::runner::compatible_regions::fixture::{object_start, part_end};
use crate::{
    CaseInputs, VendorProfiles, normalize_filament_defaults, normalize_process_defaults,
    runner::{self, ExportedCase, OrcaRunner},
    select_printer, smoke_overrides,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{Cursor, Read, Write},
};

#[derive(Clone, Copy)]
pub(super) enum Scope {
    Object,
    Part,
}

impl Scope {
    pub(super) fn name(self) -> &'static str {
        match self {
            Self::Object => "object",
            Self::Part => "part",
        }
    }
}

pub(super) fn prepare(
    runner: &OrcaRunner,
    scope: Scope,
    pattern: &str,
    low: bool,
) -> (ExportedCase, Value) {
    let profiles = VendorProfiles::load(
        &runner::repo_root().join("OrcaSlicer/resources/profiles"),
        "Creality",
    )
    .unwrap();
    let selection = select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    let machine = profiles.machine(&selection.printer).unwrap();
    let mut process = profiles.process(&selection.process).unwrap();
    normalize_process_defaults(&machine, &mut process);
    let mut filaments = selection
        .filaments
        .iter()
        .map(|name| profiles.filament(name).unwrap())
        .collect::<Vec<_>>();
    normalize_filament_defaults(&mut filaments);
    let requested = [
        ("sparse_infill_pattern", "rectilinear"),
        ("sparse_infill_density", "20%"),
        ("top_surface_pattern", "monotonicline"),
        ("internal_solid_infill_pattern", "monotonic"),
        ("layer_height", "0.2"),
        ("top_shell_layers", "5"),
        ("bottom_shell_layers", "3"),
        ("wall_loops", "2"),
        ("minimum_sparse_infill_area", "0"),
        ("dont_filter_internal_bridges", "disabled"),
        ("enable_support", "0"),
        ("enable_prime_tower", "0"),
        ("exclude_object", "0"),
        ("gcode_comments", "1"),
    ];
    for (key, value) in requested {
        process.insert(key.into(), Value::from(value));
    }
    let source = runner::repo_root().join("tests/parity/cube10.stl");
    let mesh_bytes = fs::read(&source).unwrap();
    assert_eq!(
        hash(&mesh_bytes),
        "d66a155ffff024e9d09130d5394be47aaef509b31dbd4febddf32f9b88145968"
    );
    let mesh = if low {
        let slab = runner
            .work
            .join(format!("top-slab-{}-{pattern}.stl", scope.name()));
        let mut transformed = mesh_bytes.clone();
        let triangles = u32::from_le_bytes(mesh_bytes[80..84].try_into().unwrap()) as usize;
        for triangle in 0..triangles {
            for offset in [20, 32, 44] {
                let start = 84 + 50 * triangle + offset;
                let z = f32::from_le_bytes(mesh_bytes[start..start + 4].try_into().unwrap());
                transformed[start..start + 4].copy_from_slice(&(z * 0.05).to_le_bytes());
            }
        }
        fs::write(&slab, transformed).unwrap();
        slab
    } else {
        source
    };
    let label = format!("bridge-local-top/{}/{pattern}/low-{low}", scope.name());
    let exported = runner
        .export_case(
            &CaseInputs {
                label: &label,
                machine: &machine,
                process: &process,
                filaments: &filaments,
            },
            &smoke_overrides(),
            &mesh,
        )
        .unwrap();
    for (key, value) in requested {
        runner::application::verify_value(&exported.project, key, &Value::from(value)).unwrap();
    }
    let mut input = zip::ZipArchive::new(Cursor::new(&exported.project)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let mut unchanged = serde_json::Map::new();
    let mut metadata = String::new();
    for index in 0..input.len() {
        let mut file = input.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == "Metadata/model_settings.config" {
            let text = String::from_utf8(bytes).unwrap();
            assert!(!text.contains("key=\"top_surface_pattern\""));
            let start = match scope {
                Scope::Object => object_start(&text),
                Scope::Part => part_end(&text),
            };
            metadata = format!(
                "{}\n      <metadata key=\"top_surface_pattern\" value=\"{pattern}\"/>{}",
                &text[..start],
                &text[start..]
            );
            bytes = metadata.as_bytes().to_vec();
        } else {
            unchanged.insert(file.name().into(), json!(hash(&bytes)));
        }
        output
            .start_file(
                file.name(),
                zip::write::SimpleFileOptions::DEFAULT.compression_method(file.compression()),
            )
            .unwrap();
        output.write_all(&bytes).unwrap();
    }
    let project = output.finish().unwrap().into_inner();
    let mut final_input = zip::ZipArchive::new(Cursor::new(&project)).unwrap();
    for (name, expected) in &unchanged {
        let mut bytes = Vec::new();
        final_input
            .by_name(name)
            .unwrap()
            .read_to_end(&mut bytes)
            .unwrap();
        assert_eq!(json!(hash(&bytes)), *expected);
    }
    let project_path = runner
        .work
        .join(format!("top-{}-{pattern}-low-{low}.3mf", scope.name()));
    let output_dir = runner
        .work
        .join(format!("top-{}-{pattern}-low-{low}-native", scope.name()));
    fs::write(&project_path, &project).unwrap();
    fs::create_dir(&output_dir).unwrap();
    let proof = json!({"scope":scope.name(),"pattern":pattern,"low_slab":low,
        "unchanged_members":unchanged,"local_metadata":metadata,"input_sha256":hash(&project),
        "source_mesh_sha256":hash(&mesh_bytes),"mesh_sha256":hash(&fs::read(mesh).unwrap()),"z_scale":if low {0.05} else {1.0}});
    (
        ExportedCase {
            label,
            project,
            project_path,
            output_dir,
        },
        proof,
    )
}

pub(super) fn hash(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
