//! Core-only E2E, without a Native comparison. `Orient.cpp:208-244,402-510`.
//! Failure modes: dropping direct-mesh properties, treating normal/empty properties
//! as appearance, using annotation area instead of facet area, or mutating input.
use ares_core::{
    load_project,
    plater::{OrientMesh, OrientParams, Orientation, auto_orient, auto_orient_mesh},
};
use sha2::{Digest, Sha256};
use std::fmt::Write as _;
use std::{
    fs,
    io::{Cursor, Read, Write},
    path::PathBuf,
};

const CONTROL: &[u8] =
    include_bytes!("../../../tests/parity/machine-acceleration-cache-rounding/stock-control.3mf");

#[test]
fn core_direct_mesh_appearance_weighting_uses_geometric_area() {
    // Analytic outward normals of faces 3 and 2, negated for the build direction.
    // Face 1 area is 11*sqrt(10)/2. Its appearance multiplier is 1+3=4;
    // this raises face-3 placement's overhang cost above face-2 placement.
    let baseline = [7., -1., -123.].map(|v| v / 15179_f64.sqrt());
    let marked = [-7., -10., 79.].map(|v| v / 6390_f64.sqrt());
    let cases = [
        ("unannotated", None, baseline),
        ("empty", Some(""), baseline),
        ("normal", Some("0 999999"), baseline),
        ("appearance-small-area", Some("3 1"), marked),
        ("appearance-large-area", Some("3 999999"), marked),
    ];
    let mut results = Vec::new();
    for (name, property, expected) in cases {
        let input = fixture(property);
        let project = load_project(&input).unwrap();
        let mesh = project.objects()[0].volumes()[0].mesh();
        let before = mesh.clone();
        let result = auto_orient_mesh(mesh).unwrap();
        retain(name, &input, &result, expected, None);
        assert_eq!(mesh, &before);
        for (actual, expected) in result.direction.into_iter().zip(expected) {
            assert!(
                (actual - expected).abs() <= 4. * f64::from(f32::EPSILON),
                "{name}: {result:?}"
            );
        }
        results.push(result);
    }
    assert_eq!(results[0], results[1]);
    assert_eq!(results[0], results[2]);
    assert_eq!(results[3], results[4]);
    assert_ne!(results[0], results[3]);
}

#[test]
fn core_direct_mesh_cooling_search_uses_weighted_overhang_projection() {
    let input = fixture(None);
    let project = load_project(&input).unwrap();
    let mesh = project.objects()[0].volumes()[0].mesh();
    let before = mesh.clone();
    let baseline = auto_orient_mesh(mesh).unwrap();
    // Rotating face 013's analytic normal gives (-.14237,-.29589,-.94455).
    // Its XY direction is 244.3047 degrees: the unique 2-degree maximum is 244.
    let angle = (244. * std::f64::consts::PI / 180.) as f32;
    let maximum = [f64::from(angle.cos()), f64::from(angle.sin()), 0.];
    for (name, machine, expected) in [
        ("core-cooling-maximum", [1., 0., 0.], maximum),
        (
            "core-cooling-machine",
            [-0.6, -0.8, 0.],
            [f64::from(-0.6_f32), f64::from(-0.8_f32), 0.],
        ),
    ] {
        let result = auto_orient(
            &[OrientMesh {
                mesh,
                overhang_angle: 60.,
                cooling_direction: Some(machine),
            }],
            &OrientParams::default(),
        )
        .unwrap()
        .remove(0);
        retain(
            name,
            &input,
            &result,
            baseline.direction,
            Some((machine, expected)),
        );
        assert_eq!(result.direction, baseline.direction);
        assert_eq!(result.cooling.as_ref().unwrap().direction, expected);
        assert_eq!(mesh, &before);
    }
}

fn retain(
    name: &str,
    input: &[u8],
    result: &Orientation,
    expected: [f64; 3],
    cooling_context: Option<([f64; 3], [f64; 3])>,
) {
    let root = std::env::var_os("ARES_PARITY_ARTIFACT_ROOT")
        .map_or_else(
            || PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/auto-orientation-core"),
            PathBuf::from,
        )
        .join("appearance");
    let mut input_sha256 = String::with_capacity(64);
    for byte in Sha256::digest(input).iter() {
        write!(&mut input_sha256, "{byte:02x}").unwrap();
    }
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join(format!("{name}.3mf")), input).unwrap();
    fs::write(
        root.join(format!("{name}.json")),
        serde_json::to_vec_pretty(&serde_json::json!({
            "comparison": "core-only; no Native comparison",
            "input_sha256": input_sha256,
            "expected_direction": expected,
            "direction": result.direction,
            "rotation_matrix": result.rotation_matrix,
            "axis": result.axis,
            "angle": result.angle,
            "euler_angles": result.euler_angles,
            "cooling_context": cooling_context.map(|(machine, expected)| serde_json::json!({"machine": machine, "expected": expected})),
            "cooling": result.cooling.as_ref().map(|cooling| serde_json::json!({
                "direction": cooling.direction, "axis": cooling.axis,
                "angle": cooling.angle, "rotation_matrix": cooling.rotation_matrix,
            })),
        }))
        .unwrap(),
    )
    .unwrap();
}

fn fixture(property: Option<&str>) -> Vec<u8> {
    const MEMBER: &str = "3D/Objects/cube10.stl_1.model";
    let mut archive = zip::ZipArchive::new(Cursor::new(CONTROL)).unwrap();
    let mut xml = String::new();
    archive
        .by_name(MEMBER)
        .unwrap()
        .read_to_string(&mut xml)
        .unwrap();
    let start = xml.find("<mesh>").unwrap();
    let end = xml.find("</mesh>").unwrap() + "</mesh>".len();
    let annotation = property.map_or_else(String::new, |p| format!(" face_property=\"{p}\""));
    xml.replace_range(
        start..end,
        &format!(
            r#"<mesh><vertices>
<vertex x="0" y="0" z="-5"/><vertex x="11" y="0" z="-5"/>
<vertex x="1" y="7" z="-5"/><vertex x="18" y="3" z="-4"/>
</vertices><triangles>
<triangle v1="0" v2="2" v3="1"/><triangle v1="0" v2="1" v3="3"{annotation}/>
<triangle v1="1" v2="2" v3="3"/><triangle v1="2" v2="0" v3="3"/>
</triangles></mesh>"#
        ),
    );
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..archive.len() {
        let entry = archive.by_index(index).unwrap();
        if entry.name() == MEMBER {
            writer
                .start_file(MEMBER, zip::write::SimpleFileOptions::default())
                .unwrap();
            writer.write_all(xml.as_bytes()).unwrap();
        } else {
            writer.raw_copy_file(entry).unwrap();
        }
    }
    writer.finish().unwrap().into_inner()
}
