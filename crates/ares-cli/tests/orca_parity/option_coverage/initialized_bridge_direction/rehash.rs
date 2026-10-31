use super::*;

#[test]
fn initialized_rehash_bridge_matches_complete_output() {
    compare_polygon("initialized-bridge/rehash-stretched/no-m73", 2.0);
}

#[test]
fn initialized_compressed_bridge_matches_complete_output() {
    compare_polygon("initialized-mesh/compressed/no-m73", 0.99);
}

fn compare_polygon(label: &str, x_scale: f32) {
    if !crate::runner::reference_enabled("ARES_PARITY_BRIDGE_DIRECTION_REFERENCE") {
        return;
    }
    let runner = OrcaRunner::from_env().unwrap().unwrap();
    let directory = tempfile::tempdir().unwrap();
    let model = directory
        .path()
        .join(format!("polygon-overhang-{x_scale}.stl"));
    std::fs::write(&model, polygon_overhang(x_scale)).unwrap();
    let profiles = VendorProfiles::load(&profiles_root(), "Creality").unwrap();
    let selection =
        parity::select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    let machine = profiles.machine(&selection.printer).unwrap();
    let mut process = profiles.process(&selection.process).unwrap();
    parity::normalize_process_defaults(&machine, &mut process);
    process.insert("enable_support".into(), Value::String("0".into()));
    process.insert("disable_m73".into(), Value::String("1".into()));
    process.insert("enable_prime_tower".into(), Value::String("0".into()));
    process.insert("wall_loops".into(), Value::String("0".into()));
    process.insert("infill_direction".into(), Value::String("0".into()));
    process.insert(
        "detect_narrow_internal_solid_infill".into(),
        Value::String("0".into()),
    );
    let mut filaments = selection
        .filaments
        .iter()
        .map(|name| profiles.filament(name).unwrap())
        .collect::<Vec<_>>();
    parity::normalize_filament_defaults(&mut filaments);
    let exported = runner
        .export_case(
            &CaseInputs {
                label,
                machine: &machine,
                process: &process,
                filaments: &filaments,
            },
            &parity::smoke_overrides(),
            &model,
        )
        .unwrap();
    for (key, value) in [
        ("enable_support", "0"),
        ("disable_m73", "1"),
        ("enable_prime_tower", "0"),
        ("wall_loops", "0"),
        ("infill_direction", "0"),
        ("detect_narrow_internal_solid_infill", "0"),
    ] {
        runner::application::verify_value(&exported.project, key, &Value::String(value.into()))
            .unwrap();
    }
    let sliced = runner.slice_case(&exported).unwrap();
    assert!(
        std::str::from_utf8(&sliced.reference)
            .unwrap()
            .contains(";TYPE:Bridge\n")
    );
    let outcome = parity::compare_case(&sliced);
    assert_eq!(outcome.status, "PASS", "{}", outcome.detail);
}

pub(super) fn polygon_overhang(x_scale: f32) -> String {
    use std::fmt::Write;

    let base = [(3.0, 0.0), (3.0, 1.5), (3.0, 3.0), (1.5, 3.0)];
    let roof = [(10.0, 0.0), (9.0, 4.0), (7.0, 7.0), (4.0, 9.0)];
    let rings = [(base, 0.0), (base, 1.0), (roof, 1.0), (roof, 1.2)].map(|(quarter, z)| {
        std::array::from_fn::<_, 16, _>(|index| {
            let (x, y) = quarter[index % 4];
            let (x, y) = match index / 4 {
                0 => (x, y),
                1 => (-y, x),
                2 => (-x, -y),
                3 => (y, -x),
                _ => unreachable!(),
            };
            [x * x_scale, y, z]
        })
    });
    let mut stl = String::from("solid polygon_overhang\n");
    let mut triangle = |vertices: [[f32; 3]; 3]| {
        stl.push_str("facet normal 0 0 0\nouter loop\n");
        for [x, y, z] in vertices {
            writeln!(stl, "vertex {x} {y} {z}").unwrap();
        }
        stl.push_str("endloop\nendfacet\n");
    };
    for index in 0..16 {
        let next = (index + 1) % 16;
        for pair in rings.windows(2) {
            triangle([pair[0][index], pair[0][next], pair[1][next]]);
            triangle([pair[0][index], pair[1][next], pair[1][index]]);
        }
        triangle([[0.0, 0.0, 0.0], rings[0][next], rings[0][index]]);
        triangle([[0.0, 0.0, 1.2], rings[3][index], rings[3][next]]);
    }
    stl.push_str("endsolid polygon_overhang\n");
    stl
}
