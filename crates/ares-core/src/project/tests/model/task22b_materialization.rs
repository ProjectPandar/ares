use crate::{
    GenerationMetadata, Point3d, SliceError, load_project, project::model_xml::ModelDocument,
    slice_project,
};

use super::fixture::{FIXTURE, ProjectParts};
use crate::project::{
    ArchiveLimits, PackagePath, ProjectArchive,
    xml::{XmlRole, deserialize_xml},
};

#[test]
fn task22b_vertex_units_materialize_through_f32_before_promotion() {
    for (unit, expected) in [
        ("micron", 0.0003_f32),
        ("millimeter", 0.3_f32),
        ("centimeter", 3.0_f32),
        ("inch", 7.620_000_4_f32),
        ("foot", 91.44_f32),
        ("meter", 300.0_f32),
    ] {
        let project = unit_project(unit, "0.3");
        let volume = &project.objects()[0].volumes()[0];
        let reconstructed = volume
            .transform()
            .transform_point(volume.mesh().vertices()[1]);
        assert_eq!(reconstructed.x, f64::from(expected), "{unit}");
        if unit == "inch" {
            let shift = volume
                .transform()
                .transform_point(Point3d::new(0.0, 0.0, 0.0));
            assert_eq!((shift.x * 1_000_000.0) as i64, 3_810_000);
            assert_ne!((shift.x * 1_000_000.0) as i64, 3_809_999);
        }
    }
}

#[tokio::test]
async fn task22b_vertex_unit_product_nonfinite_precedes_effective_config() {
    let mut parts = unit_parts("meter", "3.4e38");
    parts.replace(
        "Metadata/project_settings.config",
        r#"{"layer_height":"0.2"}"#,
        r#"{"layer_height":"0.2","filament_map":[]}"#,
    );
    let bytes = parts.bytes();
    let expected = SliceError::InvalidInput("project mesh vertices must be finite".to_owned());

    assert_eq!(load_project(&bytes).unwrap_err(), expected);
    assert_eq!(
        slice_project(
            bytes,
            GenerationMetadata::deterministic(2026, 7, 16, 1, 2, 3)
        )
        .await
        .unwrap_err(),
        expected
    );
}

#[test]
fn task22b_empty_geometry_is_omitted_before_volume_metadata_association() {
    for empty_mesh in [
        r#"<vertices/><triangles/>"#,
        r#"<vertices/><triangles><triangle v1="0" v2="1" v3="2"/></triangles>"#,
        r#"<vertices><vertex x="0" y="0" z="0"/></vertices><triangles/>"#,
    ] {
        let mut parts = ProjectParts::valid();
        parts.insert_text(
            "3D/root.model",
            r#"<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02" xmlns:p="http://schemas.microsoft.com/3dmanufacturing/production/2015/06" requiredextensions="p"><resources><object id="2" type="model"><components><component p:path="/3D/empty.model" objectid="1"/><component p:path="/3D/full.model" objectid="1"/></components></object></resources><build><item objectid="2"/></build></model>"#,
        );
        parts.insert_text(
            "3D/_rels/root.model.rels",
            r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Target="/3D/empty.model" Id="empty" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/><Relationship Target="/3D/full.model" Id="full" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/></Relationships>"#,
        );
        parts.remove("3D/leaf.model");
        parts.insert_text(
            "3D/empty.model",
            &format!(
                r#"<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"><resources><object id="1" type="model"><mesh>{empty_mesh}</mesh></object></resources><build/></model>"#,
            ),
        );
        parts.insert_text(
            "3D/full.model",
            r#"<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"><resources><object id="1" type="model"><mesh><vertices><vertex x="0" y="0" z="0"/><vertex x="1" y="0" z="0"/><vertex x="0" y="1" z="0"/></vertices><triangles><triangle v1="0" v2="1" v3="2"/></triangles></mesh></object></resources><build/></model>"#,
        );
        parts.insert_text(
            "Metadata/model_settings.config",
            r#"<config><object id="2"><part id="1" subtype="normal_part"><metadata key="name" value="First"/></part><part id="1" subtype="normal_part"><metadata key="name" value="Later"/></part></object><plate><metadata key="plater_id" value="1"/><model_instance><metadata key="object_id" value="2"/><metadata key="instance_id" value="0"/><metadata key="identify_id" value="133"/></model_instance></plate></config>"#,
        );

        let project = load_project(parts.bytes()).unwrap();
        let volumes = project.objects()[0].volumes();
        assert_eq!(volumes.len(), 1);
        assert_eq!(volumes[0].id(), 1);
        assert_eq!(volumes[0].name(), "First");
    }
}

#[test]
fn task22b_ksr_import_preparation_has_exact_mesh_facts() {
    let project = load_project(FIXTURE).unwrap();
    let volume = &project.objects()[0].volumes()[0];
    assert_eq!(volume.mesh().vertices().len(), 6_109);
    assert_eq!(volume.mesh().triangles().len(), 12_234);
    assert_eq!(volume.mesh().triangles()[0], [2, 0, 1]);

    let source = fixture_source_mesh();
    assert!(signed_volume(&source) > 0.0);
    let bounds = f32_bounds(&source);
    assert_eq!(bounds, [[-37.5, -35.0, -46.0], [37.5, 35.0, 46.0]]);
    assert_eq!(
        volume
            .transform()
            .transform_point(Point3d::new(0.0, 0.0, 0.0)),
        Point3d::new(0.0, 0.0, 0.0)
    );
    for (actual, source) in volume.mesh().vertices().iter().zip(&source.0) {
        for (actual, source) in [actual.x, actual.y, actual.z].into_iter().zip(*source) {
            assert_eq!(actual.to_bits(), f64::from(source).to_bits());
        }
    }
}

fn unit_project(unit: &str, x: &str) -> crate::Project {
    load_project(unit_parts(unit, x).bytes()).unwrap()
}

fn unit_parts(unit: &str, x: &str) -> ProjectParts {
    let mut parts = ProjectParts::valid();
    parts.insert_text(
        "3D/leaf.model",
        &format!(
            r#"<model unit="{unit}" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"><resources><object id="1" type="model"><mesh><vertices><vertex x="0" y="0" z="0"/><vertex x="{x}" y="0" z="0"/><vertex x="0" y="1" z="0"/></vertices><triangles><triangle v1="0" v2="1" v3="2"/></triangles></mesh></object></resources><build/></model>"#,
        ),
    );
    parts
}

fn fixture_source_mesh() -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
    let mut archive = ProjectArchive::open(FIXTURE, ArchiveLimits::PROJECT).unwrap();
    let bytes = archive
        .read(&PackagePath::entry(b"3D/Objects/ksr_fdmtest_v4.drc_2.model").unwrap())
        .unwrap();
    let document: ModelDocument = deserialize_xml(&bytes, XmlRole::Model).unwrap();
    let mesh = document.resources.objects[0].mesh.as_ref().unwrap();
    (
        mesh.vertices
            .vertices
            .iter()
            .map(|vertex| [vertex.x, vertex.y, vertex.z])
            .collect(),
        mesh.triangles
            .triangles
            .iter()
            .map(|face| [face.v1, face.v2, face.v3])
            .collect(),
    )
}

fn signed_volume(mesh: &(Vec<[f32; 3]>, Vec<[u32; 3]>)) -> f32 {
    let p0 = mesh.0[0];
    mesh.1.iter().fold(0.0_f32, |volume, face| {
        let [a, b, c] = face.map(|index| mesh.0[index as usize]);
        let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let v = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let cross = [
            u[1] * v[2] - u[2] * v[1],
            u[2] * v[0] - u[0] * v[2],
            u[0] * v[1] - u[1] * v[0],
        ];
        let norm = (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt();
        let normal = if norm == 0.0 {
            cross
        } else {
            cross.map(|component| component / norm)
        };
        let height =
            normal[0] * (a[0] - p0[0]) + normal[1] * (a[1] - p0[1]) + normal[2] * (a[2] - p0[2]);
        volume + 0.5 * norm * height / 3.0
    })
}

fn f32_bounds(mesh: &(Vec<[f32; 3]>, Vec<[u32; 3]>)) -> [[f32; 3]; 2] {
    mesh.0.iter().fold(
        [[f32::INFINITY; 3], [f32::NEG_INFINITY; 3]],
        |mut bounds, vertex| {
            for (axis, value) in vertex.iter().copied().enumerate() {
                bounds[0][axis] = bounds[0][axis].min(value);
                bounds[1][axis] = bounds[1][axis].max(value);
            }
            bounds
        },
    )
}
