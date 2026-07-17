use crate::project::{
    ArchiveLimits, PackagePath, ProjectArchive,
    model_xml::{ModelDocument, ModelObjectType, ModelUnit},
    xml::{XmlRole, deserialize_xml},
};

use super::fixture::FIXTURE;

#[test]
fn project_model_deserializes_root_and_leaf_without_precision_loss() {
    let mut archive = ProjectArchive::open(FIXTURE, ArchiveLimits::PROJECT).unwrap();
    let root = model(&mut archive, "3D/3dmodel.model");
    let leaf = model(&mut archive, "3D/Objects/ksr_fdmtest_v4.drc_2.model");

    assert_eq!(root.unit, ModelUnit::Millimeter);
    assert_eq!(root.required_extensions, "p");
    assert_eq!(
        root.metadata
            .iter()
            .find(|entry| entry.name == "OrcaSlicer")
            .unwrap()
            .value,
        "2.4.2"
    );
    assert_eq!(root.resources.objects.len(), 1);
    let root_object = &root.resources.objects[0];
    assert_eq!(root_object.id, 2);
    assert_eq!(root_object.object_type, ModelObjectType::Model);
    assert!(root_object.mesh.is_none());
    let component = &root_object.components.as_ref().unwrap().components[0];
    assert_eq!(
        component.path.as_deref(),
        Some("/3D/Objects/ksr_fdmtest_v4.drc_2.model")
    );
    assert_eq!(component.object_id, 1);
    assert_eq!(root.build.items.len(), 1);
    let item = &root.build.items[0];
    assert_eq!(item.object_id, 2);
    assert!(item.printable);
    assert!(item.auto_drop);

    assert_eq!(leaf.resources.objects.len(), 1);
    let leaf_object = &leaf.resources.objects[0];
    assert_eq!(leaf_object.id, 1);
    assert_eq!(leaf_object.object_type, ModelObjectType::Model);
    let mesh = leaf_object.mesh.as_ref().unwrap();
    assert_eq!(mesh.vertices.vertices.len(), 6_109);
    assert_eq!(mesh.triangles.triangles.len(), 12_234);
    let vertices = &mesh.vertices.vertices;
    assert_eq!(
        [vertices[0].x, vertices[0].y, vertices[0].z],
        [17.652_542, -26.396_576, -45.5]
    );
    assert_eq!(
        [vertices[1].x, vertices[1].y, vertices[1].z],
        [17.5, -25.900_002, -46.0]
    );
    assert_eq!(
        [vertices[2].x, vertices[2].y, vertices[2].z],
        [18.065_765, -25.844_276, -46.0]
    );
    let last = vertices.last().unwrap();
    assert_eq!(
        [last.x, last.y, last.z],
        [-18.425_941, 25.669_678, -16.276_539]
    );
    let triangles = &mesh.triangles.triangles;
    assert_eq!(
        [triangles[0].v1, triangles[0].v2, triangles[0].v3],
        [2, 0, 1]
    );
    assert_eq!(
        [triangles[1].v1, triangles[1].v2, triangles[1].v3],
        [1, 0, 4]
    );
    assert_eq!(
        [triangles[2].v1, triangles[2].v2, triangles[2].v3],
        [0, 5, 4]
    );
    let last = triangles.last().unwrap();
    assert_eq!([last.v1, last.v2, last.v3], [60, 67, 59]);
    assert_eq!(
        mesh.triangles
            .triangles
            .iter()
            .flat_map(|triangle| [triangle.v1, triangle.v2, triangle.v3])
            .max(),
        Some(6_108)
    );
    assert_eq!(
        bounds(vertices.iter().map(|vertex| [vertex.x, vertex.y, vertex.z])),
        [[-37.5, -35.0, -46.0], [37.5, 35.0, 46.0],]
    );
    assert!(leaf.build.items.is_empty());
}

#[test]
fn project_model_defaults_unit_type_transform_and_build_flags() {
    let document: ModelDocument = deserialize_xml(
        br#"<model xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"><resources><object id="1"><components><component objectid="2"/></components></object></resources><build><item objectid="1"/></build></model>"#,
        XmlRole::Model,
    )
    .unwrap();

    assert_eq!(document.unit, ModelUnit::Millimeter);
    assert_eq!(
        document.resources.objects[0].object_type,
        ModelObjectType::Model
    );
    assert_eq!(
        document.resources.objects[0]
            .components
            .as_ref()
            .unwrap()
            .components[0]
            .transform,
        crate::Transform3d::IDENTITY
    );
    assert_eq!(
        document.build.items[0].transform,
        crate::Transform3d::IDENTITY
    );
    assert!(document.build.items[0].printable);
    assert!(document.build.items[0].auto_drop);
}

#[test]
fn project_model_build_flags_accept_only_absent_zero_or_one() {
    for (attributes, expected) in [
        ("", (true, true)),
        (r#" printable="0" auto_drop="0""#, (false, false)),
        (r#" printable="1" auto_drop="1""#, (true, true)),
    ] {
        let xml = format!(
            r#"<model xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"><resources/><build><item objectid="1"{attributes}/></build></model>"#
        );
        let document: ModelDocument = deserialize_xml(xml.as_bytes(), XmlRole::Model).unwrap();
        assert_eq!(
            (
                document.build.items[0].printable,
                document.build.items[0].auto_drop
            ),
            expected
        );
    }
}

fn model(archive: &mut ProjectArchive<'_>, path: &str) -> ModelDocument {
    let bytes = archive
        .read(&PackagePath::entry(path.as_bytes()).unwrap())
        .unwrap();
    deserialize_xml(&bytes, XmlRole::Model).unwrap()
}

fn bounds(points: impl Iterator<Item = [f32; 3]>) -> [[f32; 3]; 2] {
    points.fold(
        [[f32::INFINITY; 3], [f32::NEG_INFINITY; 3]],
        |mut bounds, point| {
            for axis in 0..3 {
                bounds[0][axis] = bounds[0][axis].min(point[axis]);
                bounds[1][axis] = bounds[1][axis].max(point[axis]);
            }
            bounds
        },
    )
}
