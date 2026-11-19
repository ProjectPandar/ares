//! Brim.cpp:549–576 and 592–607: group width changes and separate contour/hole offsets.

use std::{
    fmt::Write as _,
    io::{Cursor, Read, Write},
};

use crate::{OrcaRunner, auto_brim_object_speed::brim_moves, runner::native_api_cli::NativeCase};

const BASE: &[u8] =
    include_bytes!("../../../../tests/parity/auto-brim-exact-group-assignment/single-auto.3mf");

#[test]
fn auto_brim_widens_a_tall_part_with_little_adhesion() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    // The same footprint has zero group width at both heights. Only the
    // taller group qualifies for the addition (28 / 32 mm < 1.1 mm).
    let short = plate((14., 2., 9.), None, "auto_brim", "0.48");
    let control = NativeCase::slice(&runner, &short, "auto-brim-adhesion-short-");
    assert!(brim_moves(&control.native).is_empty());
    let tall = plate((14., 2., 10.4), None, "auto_brim", "0.48");
    let case = NativeCase::slice(&runner, &tall, "auto-brim-adhesion-addition-");
    assert_extent(&case, 104.8..105.0);
    compare(&case);
    compare(&control);
}

#[test]
fn auto_brim_caps_width_at_the_largest_footprint_side() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    // At this height the group width reaches 1.5 times the diagonal,
    // but the contour offset is limited by the 2.742474 mm side.
    let project = plate((2.742474, 2., 25.), None, "auto_brim", "0.50");
    let case = NativeCase::slice(&runner, &project, "auto-brim-bbox-clamp-");
    assert_extent(&case, 106.4..106.6);
    compare(&case);
}

#[test]
fn combined_brim_preserves_outer_spacings_and_extrudes_inside_the_hole() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let project = plate((20., 18., 3.2), Some((10., 8.)), "outer_and_inner", "0.50");
    let case = NativeCase::slice(&runner, &project, "combined-brim-outer-width-");
    assert_extent(&case, 98.4..98.6);
    assert!(brim_moves(&case.native).into_iter().any(|line| {
        (105.0..115.0).contains(&axis(line, 'X')) && (108.7..108.8).contains(&axis(line, 'Y'))
    }));
    compare(&case);
}

fn axis(line: &str, axis: char) -> f64 {
    line.split_ascii_whitespace()
        .find_map(|word| word.strip_prefix(axis))
        .and_then(|value| value.parse().ok())
        .unwrap_or(f64::NAN)
}

fn assert_extent(case: &NativeCase, expected: std::ops::Range<f64>) {
    let min_y = brim_moves(&case.native)
        .into_iter()
        .map(|line| axis(line, 'Y'))
        .fold(f64::INFINITY, f64::min);
    assert!(
        expected.contains(&min_y),
        "Native brim min Y {min_y}; expected {expected:?} at {:?}",
        case.root
    );
}

fn compare(case: &NativeCase) {
    case.compare(ares_core::GenerationMetadata::deterministic(
        1980, 1, 1, 0, 0, 0,
    ));
}

fn plate(
    size: (f64, f64, f64),
    hole: Option<(f64, f64)>,
    brim_type: &str,
    line_width: &str,
) -> Vec<u8> {
    let mut source = zip::ZipArchive::new(Cursor::new(BASE)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..source.len() {
        let mut file = source.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        match file.name() {
            "Metadata/project_settings.config" => {
                let mut config: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
                for (key, value) in [
                    ("brim_type", brim_type),
                    ("initial_layer_line_width", line_width),
                    ("elefant_foot_compensation", "0"),
                    ("brim_width", "3.6"),
                ] {
                    config[key] = value.into();
                }
                bytes = serde_json::to_vec(&config).unwrap();
            }
            "3D/3dmodel.model" => {
                bytes = String::from_utf8(bytes)
                    .unwrap()
                    .replace("0 0 50 110 110 0", "0 0 1 110 110 0")
                    .into_bytes();
            }
            "3D/Objects/cube10.stl_1.model" => {
                let mut xml = String::from_utf8(bytes).unwrap();
                let start = xml.find("<mesh>").unwrap();
                let end = xml.find("</mesh>").unwrap() + "</mesh>".len();
                xml.replace_range(start..end, &mesh(size, hole));
                bytes = xml.into_bytes();
            }
            _ => {}
        }
        output
            .start_file(file.name(), zip::write::SimpleFileOptions::DEFAULT)
            .unwrap();
        output.write_all(&bytes).unwrap();
    }
    output.finish().unwrap().into_inner()
}

fn mesh((width, depth, height): (f64, f64, f64), hole: Option<(f64, f64)>) -> String {
    let mut mesh = String::from("<mesh><vertices>");
    for (w, d) in std::iter::once((width, depth)).chain(hole) {
        for z in [0., height] {
            for (x, y) in [
                (-w / 2., -d / 2.),
                (w / 2., -d / 2.),
                (w / 2., d / 2.),
                (-w / 2., d / 2.),
            ] {
                write!(mesh, "<vertex x=\"{x}\" y=\"{y}\" z=\"{z}\"/>").unwrap();
            }
        }
    }
    mesh.push_str("</vertices><triangles>");
    let mut quad = |[a, b, c, d]: [usize; 4]| {
        write!(mesh, "<triangle v1=\"{a}\" v2=\"{b}\" v3=\"{c}\"/><triangle v1=\"{a}\" v2=\"{c}\" v3=\"{d}\"/>").unwrap();
    };
    for a in 0..4 {
        let b = (a + 1) % 4;
        quad([a, b, b + 4, a + 4]);
        if hole.is_some() {
            quad([b + 8, a + 8, a + 12, b + 12]);
            quad([b, a, a + 8, b + 8]);
            quad([a + 4, b + 4, b + 12, a + 12]);
        }
    }
    if hole.is_none() {
        quad([0, 3, 2, 1]);
        quad([4, 5, 6, 7]);
    }
    mesh.push_str("</triangles></mesh>");
    mesh
}
