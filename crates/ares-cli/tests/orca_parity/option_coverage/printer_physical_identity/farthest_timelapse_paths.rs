//! `GCode.cpp:9974-9996,10118,10174,10321`: the farthest-point snapshot fires inside
//! arc-fitted, segmented, sloped and z-contoured outer walls, not at the layer end.

use crate::{OrcaRunner, runner::native_api_cli::NativeCase};
use serde_json::{Value, json};
use std::io::{Cursor, Read, Write};

const BASE: &[u8] =
    include_bytes!("../../../../../../tests/parity/printer-physical-identity/P0.3mf");
const OBJECT: &str = "3D/Objects/cube10.stl_1.model";

fn model(vertices: &[[f64; 3]], triangles: &[[usize; 3]]) -> String {
    let vertices = vertices
        .iter()
        .map(|v| {
            let [x, y, z] = v.map(|value| (value * 1e6).round() / 1e6);
            format!("     <vertex x=\"{x}\" y=\"{y}\" z=\"{z}\"/>\n")
        })
        .collect::<String>();
    let triangles = triangles
        .iter()
        .map(|[a, b, c]| format!("     <triangle v1=\"{a}\" v2=\"{b}\" v3=\"{c}\"/>\n"))
        .collect::<String>();
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<model unit=\"millimeter\" xml:lang=\"en-US\" xmlns=\"http://schemas.microsoft.com/3dmanufacturing/core/2015/02\" xmlns:BambuStudio=\"http://schemas.bambulab.com/package/2021\" xmlns:p=\"http://schemas.microsoft.com/3dmanufacturing/production/2015/06\" requiredextensions=\"p\">\n <metadata name=\"BambuStudio:3mfVersion\">1</metadata>\n <resources>\n  <object id=\"1\" p:UUID=\"00010000-81cb-4c03-9d28-80fed5dfa1dc\" type=\"model\">\n   <mesh>\n    <vertices>\n{vertices}    </vertices>\n    <triangles>\n{triangles}    </triangles>\n   </mesh>\n  </object>\n </resources>\n <build/>\n</model>\n"
    )
}

/// A closed prism from a bottom ring to a top ring with the same vertex count, z in [-5, 5].
fn prism(bottom: &[[f64; 2]], top: &[[f64; 2]]) -> String {
    let n = bottom.len();
    let vertices = bottom
        .iter()
        .map(|&[x, y]| [x, y, -5.0])
        .chain(top.iter().map(|&[x, y]| [x, y, 5.0]))
        .chain([[0.0, 0.0, -5.0], [0.0, 0.0, 5.0]])
        .collect::<Vec<_>>();
    let triangles = (0..n)
        .flat_map(|i| {
            let j = (i + 1) % n;
            [
                [2 * n, j, i],
                [2 * n + 1, n + i, n + j],
                [i, j, n + j],
                [i, n + j, n + i],
            ]
        })
        .collect::<Vec<_>>();
    model(&vertices, &triangles)
}

fn square(low: f64, high: f64) -> [[f64; 2]; 4] {
    [[low, low], [high, low], [high, high], [low, high]]
}

/// Two 64-segment arcs (half chord 10, sagitta 4) meeting at sharp tips on the object axis;
/// arc fitting turns each wall side into one G2/G3 move.
fn lens() -> Vec<[f64; 2]> {
    const SEGMENTS: usize = 64;
    let (half, sagitta) = (10.0_f64, 4.0_f64);
    let radius = (half * half + sagitta * sagitta) / (2.0 * sagitta);
    let span = (half / radius).asin();
    // P0 places the object at (175, 160): the lens axis points away from the bed origin.
    let (sin, cos) = 160.0_f64.atan2(175.0).sin_cos();
    let point = |side: f64, index: usize| {
        let t = -span + 2.0 * span * index as f64 / SEGMENTS as f64;
        let (u, v) = (
            radius * t.sin(),
            side * (radius * t.cos() - (radius - sagitta)),
        );
        [u * cos - v * sin, u * sin + v * cos]
    };
    (0..SEGMENTS)
        .map(|index| point(-1.0, index))
        .chain((0..SEGMENTS).map(|index| point(1.0, SEGMENTS - index)))
        .collect()
}

/// A 10 mm box whose top falls from z 5 at the origin-side corner to z 0 at the far corner,
/// so ZAA lowers the top-layer walls near the farthest point.
fn wedge() -> String {
    let ring = square(-5.0, 5.0);
    let top = |[x, y]: [f64; 2]| 5.0 - 5.0 * (x + 5.0 + y + 5.0) / 20.0;
    let vertices = ring
        .iter()
        .map(|&[x, y]| [x, y, -5.0])
        .chain(ring.iter().map(|&point| [point[0], point[1], top(point)]))
        .collect::<Vec<_>>();
    let triangles = [[0, 2, 1], [0, 3, 2], [4, 5, 6], [4, 6, 7]]
        .into_iter()
        .chain((0..4).flat_map(|i| {
            let j = (i + 1) % 4;
            [[i, j, 4 + j], [i, 4 + j, 4 + i]]
        }))
        .collect::<Vec<_>>();
    model(&vertices, &triangles)
}

/// P0 with the object mesh replaced, the farthest-point snapshot on, every feature on
/// filament 1 (no toolchange decides the snapshot head) and `extra` settings.
fn project(object: &str, extra: &[(&str, &str)]) -> Vec<u8> {
    let mut reader = zip::ZipArchive::new(Cursor::new(BASE)).unwrap();
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..reader.len() {
        let mut file = reader.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == OBJECT {
            bytes = object.as_bytes().to_vec();
        } else if file.name() == "Metadata/project_settings.config" {
            let mut settings: Value = serde_json::from_slice(&bytes).unwrap();
            assert_eq!(settings.get("farthest_point_timelapse"), None);
            settings["farthest_point_timelapse"] = json!("1");
            for key in [
                "bottom_surface_filament_id",
                "internal_solid_filament_id",
                "sparse_infill_filament_id",
                "top_surface_filament_id",
            ] {
                settings[key] = json!("1");
            }
            for (key, value) in extra {
                settings[*key] = json!(value);
            }
            bytes = serde_json::to_vec_pretty(&settings).unwrap();
        }
        writer
            .start_file(
                file.name(),
                zip::write::SimpleFileOptions::default().compression_method(file.compression()),
            )
            .unwrap();
        writer.write_all(&bytes).unwrap();
    }
    writer.finish().unwrap().into_inner()
}

/// Snapshot blocks followed by an extrusion before the next layer change.
fn in_layer_snapshots(gcode: &str) -> usize {
    let mut open = false;
    let mut count = 0;
    for line in gcode.lines() {
        if line == "; SKIPTYPE: timelapse" {
            open = true;
        } else if line == "; CHANGE_LAYER" {
            open = false;
        } else if open && ["G1 ", "G2 ", "G3 "].iter().any(|g| line.starts_with(g)) {
            let extrudes = line.split_ascii_whitespace().any(|word| {
                word.strip_prefix('E')
                    .and_then(|value| value.parse::<f64>().ok())
                    .is_some_and(|value| value > 0.0)
            });
            if extrudes {
                count += 1;
                open = false;
            }
        }
    }
    count
}

fn assert_in_layer_parity(prefix: &str, object: &str, extra: &[(&str, &str)], in_layer: usize) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let case = NativeCase::slice(&runner, &project(object, extra), prefix);
    assert_eq!(
        in_layer_snapshots(&case.native),
        in_layer,
        "{:?}",
        case.root
    );
    case.compare(ares_core::GenerationMetadata::deterministic(
        2026, 11, 13, 0, 0, 0,
    ));
}

#[test]
fn farthest_snapshot_fires_after_fitted_arc_wall() {
    assert_in_layer_parity("farthest-arc-", &prism(&lens(), &lens()), &[], 50);
}

#[test]
fn farthest_snapshot_fires_inside_slowed_overhang_wall() {
    let top = square(-5.0, 12.0);
    let bottom = square(-5.0, 5.0);
    assert_in_layer_parity("farthest-overhang-", &prism(&bottom, &top), &[], 50);
}

#[test]
fn farthest_snapshot_fires_on_scarf_sloped_loop() {
    let ring = square(-5.0, 5.0);
    let scarf = [
        ("seam_slope_type", "external"),
        ("seam_slope_conditional", "0"),
        ("seam_slope_entire_loop", "1"),
    ];
    assert_in_layer_parity("farthest-scarf-", &prism(&ring, &ring), &scarf, 50);
}

#[test]
fn farthest_snapshot_fires_on_zaa_contoured_wall() {
    assert_in_layer_parity("farthest-zaa-", &wedge(), &[("zaa_enabled", "1")], 49);
}
