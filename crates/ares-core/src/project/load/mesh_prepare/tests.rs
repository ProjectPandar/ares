use crate::{Point3d, load_project};

use super::fixture::ProjectParts;

#[test]
fn task22b_import_winding_uses_face_order_f32_and_strict_negative_flip() {
    let positive = load_tetra(false);
    let reversed = load_tetra(true);

    assert_eq!(
        positive.objects()[0].volumes()[0].mesh().triangles(),
        &[[0, 2, 1], [0, 1, 3], [0, 3, 2], [1, 2, 3]]
    );
    assert_eq!(
        reversed.objects()[0].volumes()[0].mesh().triangles(),
        positive.objects()[0].volumes()[0].mesh().triangles()
    );
}

#[test]
fn task22b_import_winding_preserves_positive_zero_negative_zero_nan_and_degenerate_faces() {
    for (vertices, triangles) in [
        (
            "<vertex x=\"0.0\" y=\"-0.0\" z=\"0.0\"/><vertex x=\"1\" y=\"0\" z=\"0\"/><vertex x=\"0\" y=\"1\" z=\"0\"/>",
            "<triangle v1=\"0\" v2=\"1\" v3=\"2\"/>",
        ),
        (
            "<vertex x=\"-3e38\" y=\"-3e38\" z=\"-3e38\"/><vertex x=\"3e38\" y=\"-3e38\" z=\"3e38\"/><vertex x=\"-3e38\" y=\"3e38\" z=\"3e38\"/><vertex x=\"3e38\" y=\"3e38\" z=\"-3e38\"/>",
            "<triangle v1=\"0\" v2=\"1\" v3=\"2\"/><triangle v1=\"0\" v2=\"0\" v3=\"0\"/>",
        ),
    ] {
        let project = load_mesh(vertices, triangles, None);
        let mesh = project.objects()[0].volumes()[0].mesh();
        assert_eq!(mesh.triangles()[0], [0, 1, 2]);
    }

    let project = load_mesh(
        "<vertex x=\"-0.0\" y=\"0.0\" z=\"-0.0\"/><vertex x=\"0.0\" y=\"-0.0\" z=\"0.0\"/>",
        "<triangle v1=\"0\" v2=\"1\" v3=\"1\"/>",
        None,
    );
    let vertices = project.objects()[0].volumes()[0].mesh().vertices();
    assert_eq!(vertices[0].x.to_bits(), f64::from(-0.0_f32).to_bits());
    assert_eq!(vertices[0].y.to_bits(), f64::from(0.0_f32).to_bits());
    assert_eq!(vertices[0].z.to_bits(), f64::from(-0.0_f32).to_bits());

    let project = load_mesh(
        "<vertex x=\"0\" y=\"0\" z=\"0\"/><vertex x=\"1\" y=\"0\" z=\"0\"/><vertex x=\"0\" y=\"1\" z=\"0\"/><vertex x=\"0\" y=\"0\" z=\"1\"/><vertex x=\"2\" y=\"0\" z=\"0\"/>",
        "<triangle v1=\"0\" v2=\"1\" v3=\"4\"/><triangle v1=\"0\" v2=\"1\" v3=\"2\"/><triangle v1=\"0\" v2=\"3\" v3=\"1\"/><triangle v1=\"0\" v2=\"2\" v3=\"3\"/><triangle v1=\"1\" v2=\"3\" v3=\"2\"/>",
        None,
    );
    assert_eq!(
        project.objects()[0].volumes()[0].mesh().triangles(),
        &[[0, 4, 1], [0, 2, 1], [0, 1, 3], [0, 3, 2], [1, 2, 3]]
    );
}

#[test]
fn task22b_fresh_mesh_centering_keeps_f32_subtraction_and_f64_shift() {
    let project = load_mesh(
        "<vertex x=\"128.0\" y=\"0\" z=\"0\"/><vertex x=\"128.00001525878906\" y=\"0\" z=\"0\"/><vertex x=\"128.0\" y=\"1\" z=\"0\"/>",
        "<triangle v1=\"0\" v2=\"1\" v3=\"2\"/>",
        None,
    );
    let volume = &project.objects()[0].volumes()[0];
    let x = volume
        .mesh()
        .vertices()
        .iter()
        .map(|point| point.x)
        .collect::<Vec<_>>();
    assert_eq!(x, vec![0.0, 0.000_015_258_789_062_5, 0.0]);

    let shift = volume
        .transform()
        .transform_point(Point3d::new(0.0, 0.0, 0.0));
    assert_eq!(shift.x, 128.000_007_629_394_53);
    let reconstructed = x
        .iter()
        .map(|x| {
            volume
                .transform()
                .transform_point(Point3d::new(*x, -0.5, 0.0))
                .x
        })
        .collect::<Vec<_>>();
    assert_eq!(
        reconstructed,
        vec![
            128.000_007_629_394_53,
            128.000_022_888_183_6,
            128.000_007_629_394_53
        ]
    );
    let raw_center = ((reconstructed[0] + reconstructed[1]) * 0.5 * 1_000_000.0) as i64;
    assert_eq!(raw_center, 128_000_015);
    assert_ne!(raw_center, 128_000_007);
}

#[test]
fn task22b_fresh_mesh_compensation_is_component_then_shift_and_metadata_stays_provenance() {
    let transform = "2 0 0 0 3 0 0 0 4 10 20 30";
    let mut parts = mesh_parts(
        "<vertex x=\"0\" y=\"0\" z=\"0\"/><vertex x=\"2\" y=\"0\" z=\"0\"/><vertex x=\"0\" y=\"4\" z=\"0\"/><vertex x=\"0\" y=\"0\" z=\"6\"/>",
        "<triangle v1=\"0\" v2=\"2\" v3=\"1\"/><triangle v1=\"0\" v2=\"1\" v3=\"3\"/><triangle v1=\"0\" v2=\"3\" v3=\"2\"/><triangle v1=\"1\" v2=\"2\" v3=\"3\"/>",
    );
    parts.replace(
        "3D/root.model",
        "transform=\"1 0 0 0 1 0 0 0 1 0 0 0\"",
        &format!("transform=\"{transform}\""),
    );
    parts.replace(
        "Metadata/model_settings.config",
        "value=\"1 0 0 0 0 1 0 0 0 0 1 0 0 0 0 1\"",
        "value=\"1 0 0 99 0 1 0 98 0 0 1 97 0 0 0 1\"",
    );
    let project = load_project(parts.bytes()).unwrap();
    let volume = &project.objects()[0].volumes()[0];

    assert_eq!(
        volume
            .transform()
            .transform_point(Point3d::new(0.0, 0.0, 0.0)),
        Point3d::new(12.0, 26.0, 42.0)
    );
    assert_eq!(
        volume
            .transform()
            .transform_point(volume.mesh().vertices()[0]),
        Point3d::new(10.0, 20.0, 30.0)
    );
    assert_eq!(
        volume
            .source_transform()
            .transform_point(Point3d::new(0.0, 0.0, 0.0)),
        Point3d::new(99.0, 98.0, 97.0)
    );
}

fn load_tetra(reversed: bool) -> crate::Project {
    let faces = [[0, 2, 1], [0, 1, 3], [0, 3, 2], [1, 2, 3]];
    let triangles = faces
        .map(|[a, b, c]| {
            let [b, c] = if reversed { [c, b] } else { [b, c] };
            format!("<triangle v1=\"{a}\" v2=\"{b}\" v3=\"{c}\"/>")
        })
        .join("");
    load_mesh(
        "<vertex x=\"0\" y=\"0\" z=\"0\"/><vertex x=\"1\" y=\"0\" z=\"0\"/><vertex x=\"0\" y=\"1\" z=\"0\"/><vertex x=\"0\" y=\"0\" z=\"1\"/>",
        &triangles,
        None,
    )
}

fn load_mesh(vertices: &str, triangles: &str, unit: Option<&str>) -> crate::Project {
    let mut parts = mesh_parts(vertices, triangles);
    if let Some(unit) = unit {
        parts.replace(
            "3D/leaf.model",
            "unit=\"millimeter\"",
            &format!("unit=\"{unit}\""),
        );
    }
    load_project(parts.bytes()).unwrap()
}

fn mesh_parts(vertices: &str, triangles: &str) -> ProjectParts {
    let mut parts = ProjectParts::valid();
    parts.insert_text(
        "3D/leaf.model",
        &format!(
            r#"<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"><resources><object id="1" type="model"><mesh><vertices>{vertices}</vertices><triangles>{triangles}</triangles></mesh></object></resources><build/></model>"#,
        ),
    );
    parts
}
