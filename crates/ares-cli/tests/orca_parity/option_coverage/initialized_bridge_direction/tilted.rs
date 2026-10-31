use super::*;
use std::{fmt::Write as _, io::Write as _};

#[test]
fn initialized_tilted_mesh_matches_complete_output() {
    if !crate::runner::reference_enabled("ARES_PARITY_BRIDGE_DIRECTION_REFERENCE") {
        return;
    }
    let runner = OrcaRunner::from_env().unwrap().unwrap();
    let directory = tempfile::tempdir().unwrap();
    let model = directory.path().join("tilted-overhang.3mf");
    std::fs::write(&model, project()).unwrap();
    let profiles = VendorProfiles::load(&profiles_root(), "Creality").unwrap();
    let selection =
        parity::select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    let machine = profiles.machine(&selection.printer).unwrap();
    let mut process = profiles.process(&selection.process).unwrap();
    parity::normalize_process_defaults(&machine, &mut process);
    let overrides = [
        ("enable_support", "0"),
        ("disable_m73", "1"),
        ("enable_prime_tower", "0"),
        ("wall_loops", "2"),
        ("infill_direction", "0"),
        ("detect_narrow_internal_solid_infill", "0"),
    ];
    for (key, value) in overrides {
        process.insert(key.into(), Value::String(value.into()));
    }
    let mut filaments = selection
        .filaments
        .iter()
        .map(|name| profiles.filament(name).unwrap())
        .collect::<Vec<_>>();
    parity::normalize_filament_defaults(&mut filaments);
    let exported = runner
        .export_case(
            &CaseInputs {
                label: "initialized-mesh/tilted-translation/no-m73",
                machine: &machine,
                process: &process,
                filaments: &filaments,
            },
            &parity::smoke_overrides(),
            &model,
        )
        .unwrap();
    for (key, value) in overrides {
        runner::application::verify_value(&exported.project, key, &Value::String(value.into()))
            .unwrap();
    }
    let loaded = ares_core::load_project(&exported.project).unwrap();
    let object = &loaded.objects()[0];
    let transform = object.instances()[0]
        .transform()
        .then(object.volumes()[0].transform());
    let origin = transform.transform_point(ares_core::Point3d::new(0.0, 0.0, 0.0));
    let x = transform.transform_point(ares_core::Point3d::new(1.0, 0.0, 0.0));
    let z = transform.transform_point(ares_core::Point3d::new(0.0, 0.0, 1.0));
    assert!((x.z - origin.z).abs() > 0.01);
    assert!((z.x - origin.x).abs() > 0.01);
    assert!((z.y - origin.y).abs() > 0.01);
    assert!(origin.x.abs() > 1.0 && origin.y.abs() > 1.0);
    let sliced = runner.slice_case(&exported).unwrap();
    assert!(
        std::str::from_utf8(&sliced.reference)
            .unwrap()
            .contains(";TYPE:Bridge\n")
    );
    let outcome = parity::compare_case(&sliced);
    assert_eq!(outcome.status, "PASS", "{}", outcome.detail);
}

fn project() -> Vec<u8> {
    let stl = super::rehash::polygon_overhang(0.99);
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut unique = std::collections::HashMap::new();
    for vertex in stl.lines().filter_map(|line| line.strip_prefix("vertex ")) {
        let xyz = vertex
            .split_whitespace()
            .map(|value| value.parse::<f32>().unwrap())
            .collect::<Vec<_>>();
        let point = [xyz[0], xyz[1], xyz[2]];
        let key = point.map(|value| if value == 0.0 { 0 } else { value.to_bits() });
        let index = *unique.entry(key).or_insert_with(|| {
            vertices.push(point);
            vertices.len() - 1
        });
        indices.push(index);
    }
    let mut model = String::from(
        r#"<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"><resources><object id="1" type="model"><mesh><vertices>"#,
    );
    for xyz in &vertices {
        write!(
            model,
            r#"<vertex x="{}" y="{}" z="{}"/>"#,
            xyz[0], xyz[1], xyz[2]
        )
        .unwrap();
    }
    model.push_str("</vertices><triangles>");
    for triangle in indices.as_chunks::<3>().0 {
        write!(
            model,
            r#"<triangle v1="{}" v2="{}" v3="{}"/>"#,
            triangle[0], triangle[1], triangle[2]
        )
        .unwrap();
    }
    model.push_str(
        r#"</triangles></mesh></object></resources><build><item objectid="1" transform="0.9998476951563913 0 -0.01745240643728351 0.00030458649045213493 0.9998476951563913 0.017449748351250485 0.017449748351250485 -0.01745240643728351 0.9996954135095479 3 7 0"/></build></model>"#,
    );
    let mut archive = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    for (path, content) in [
        (
            "[Content_Types].xml",
            r#"<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="model" ContentType="application/vnd.ms-package.3dmanufacturing-3dmodel+xml"/></Types>"#,
        ),
        (
            "_rels/.rels",
            r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rel0" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel" Target="/3D/3dmodel.model"/></Relationships>"#,
        ),
        ("3D/3dmodel.model", model.as_str()),
    ] {
        archive
            .start_file(path, zip::write::SimpleFileOptions::DEFAULT)
            .unwrap();
        archive.write_all(content.as_bytes()).unwrap();
    }
    archive.finish().unwrap().into_inner()
}
