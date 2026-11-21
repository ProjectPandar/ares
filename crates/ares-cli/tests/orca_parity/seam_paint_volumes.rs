use std::{fs, path::Path};

use crate::{
    golden,
    runner::{native_api_cli::NativeCase, native_stream_case, repo_root},
};

const UPSTREAM: &[&str] = &["GCode/SeamPlacer.cpp:1077-1156"];

fn load(printer: &str, variant: &str) -> (String, Vec<u8>, &'static str) {
    let label = format!("{printer}-{variant}");
    let file = format!("{label}.3mf");
    let digest = include_str!("../../../../tests/parity/seam-paint-volumes/SHA256SUMS")
        .lines()
        .filter_map(|line| line.split_once("  "))
        .find_map(|(sha, name)| (name == file).then_some(sha))
        .unwrap();
    let project = fs::read(
        repo_root()
            .join(Path::new("tests/parity/seam-paint-volumes"))
            .join(file),
    )
    .unwrap();
    (label, project, digest)
}

fn native(printer: &str, variant: &str) -> NativeCase {
    let (label, project, digest) = load(printer, variant);
    native_stream_case::qualify(
        ("seam-paint-volumes", UPSTREAM),
        &label,
        &project,
        digest,
        false,
    )
    .0
}

fn first_inner_wall(gcode: &str, accepts: impl Fn([f64; 3]) -> bool) -> [f64; 2] {
    let (mut inner, mut travel) = (false, false);
    let mut position = [0.0; 3];
    for line in gcode.lines() {
        if let Some(role) = line
            .strip_prefix(";TYPE:")
            .or_else(|| line.strip_prefix("; FEATURE: "))
        {
            inner = role == "Inner wall";
        }
        let Some(movement) = line.strip_prefix("G1 ") else {
            continue;
        };
        let mut next = position;
        let (mut xy, mut extrusion) = (false, None);
        for word in movement.split(';').next().unwrap().split_ascii_whitespace() {
            let (axis, value) = word.split_at(1);
            match axis {
                "X" | "Y" | "Z" => {
                    let i = usize::from(axis.as_bytes()[0] - b'X');
                    next[i] = value.parse().unwrap();
                    xy |= i < 2;
                }
                "E" => extrusion = value.parse::<f64>().ok(),
                _ => (),
            }
        }
        if xy {
            if inner && travel && extrusion.is_some_and(|e| e > 0.0) && accepts(position) {
                return [position[0], position[1]];
            }
            travel = extrusion.is_none();
        }
        position = next;
    }
    panic!("Native must start an inner wall in the requested region");
}

fn assert_native_behavior(printer: &str) {
    let plain = native(printer, "stack-plain");
    let painted = native(printer, "stack-painted");
    for z_range in [0.1..0.3, 3.4..3.8] {
        let layer = |[_, _, z]: [f64; 3]| z_range.contains(&z);
        let unpainted = first_inner_wall(&plain.native, layer);
        let selected = first_inner_wall(&painted.native, layer);
        println!("{printer} slab {z_range:?}: {unpainted:?} -> {selected:?}");
        assert_ne!(unpainted, selected, "{printer}: {:?}", painted.root);
        assert!(
            selected[1] < 93.0,
            "expected the transformed front patch: {selected:?}"
        );
    }

    let plain = native(printer, "hole-plain");
    let painted = native(printer, "hole-painted");
    let (hole_y, front_limit) = if printer == "x1c" {
        (93.0..105.0, 94.0)
    } else {
        (95.0..107.0, 96.0)
    };
    let hole = |[x, y, z]: [f64; 3]| {
        (99.5..112.5).contains(&x) && hole_y.contains(&y) && (0.1..0.3).contains(&z)
    };
    let unpainted = first_inner_wall(&plain.native, hole);
    let selected = first_inner_wall(&painted.native, hole);
    println!("{printer} hole: {unpainted:?} -> {selected:?}");
    assert_ne!(unpainted, selected, "{printer}: {:?}", painted.root);
    assert!(
        selected[1] < front_limit,
        "expected the painted hole wall: {selected:?}"
    );

    let normalize = |case: &NativeCase| {
        golden::normalize_one_generator_line(case.native.as_bytes(), golden::GeneratorKind::Orca)
            .unwrap()
    };
    let plain = native(printer, "residual-plain");
    let reference = normalize(&plain);
    for variant in ["residual-modifier", "residual-helper"] {
        let painted = native(printer, variant);
        assert!(
            golden::first_difference(&reference, &normalize(&painted)).is_none(),
            "{printer} {variant}: {:?}",
            painted.root
        );
    }
}

fn compare(printer: &str, variant: &str) {
    let (label, project, digest) = load(printer, variant);
    native_stream_case::check(
        ("seam-paint-volumes", UPSTREAM),
        &label,
        &project,
        digest,
        (false, |_, _| {}),
    );
}

macro_rules! printer_cases {
    ($printer:ident) => {
        mod $printer {
            #[test]
            fn native_changes_both_slabs_and_hole_but_ignores_residual_paint() {
                super::assert_native_behavior(stringify!($printer));
            }

            printer_cases!(@compare $printer, transformed_parts, "stack-painted");
            printer_cases!(@compare $printer, transformed_parts_unpainted, "stack-plain");
            printer_cases!(@compare $printer, negative_hole_wall, "hole-painted");
            printer_cases!(@compare $printer, negative_hole_unpainted, "hole-plain");
            printer_cases!(@compare $printer, residual_paint_on_modifier, "residual-modifier");
            printer_cases!(@compare $printer, residual_paint_on_helper, "residual-helper");
            printer_cases!(@compare $printer, residual_paint_unpainted, "residual-plain");
        }
    };
    (@compare $printer:ident, $test:ident, $variant:literal) => {
        #[test]
        fn $test() {
            super::compare(stringify!($printer), $variant);
        }
    };
}

printer_cases!(x1c);
printer_cases!(k1);
printer_cases!(mk4);
