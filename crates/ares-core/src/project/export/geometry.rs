use std::collections::HashMap;

use super::super::{
    ArchiveLimits, PackagePath, ProjectArchive, Transform3d,
    model_settings::{MeshStatistics, Metadata},
    model_xml::ModelDocument,
};
use super::{MODEL, deferred, invalid};
use crate::SliceError;

#[derive(Clone, Debug, PartialEq)]
pub(super) struct Mesh {
    pub name: String,
    pub vertices: Vec<[f32; 3]>,
    pub triangles: Vec<[usize; 3]>,
    pub use_f32_vertices: bool,
    pub placement: [f64; 3],
    pub source_offset: [f32; 3],
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Object {
    pub name: String,
    pub(super) instances: Vec<Instance>,
    pub(super) parts: Vec<Part>,
    pub(super) metadata: Vec<Metadata>,
    pub(super) auto_drop: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct Instance {
    pub transform: Transform3d,
    pub printable: bool,
}

impl Instance {
    fn at([x, y, z]: [f64; 3]) -> Self {
        Self {
            transform: Transform3d::from_rows([
                [1.0, 0.0, 0.0, x],
                [0.0, 1.0, 0.0, y],
                [0.0, 0.0, 1.0, z],
                [0.0, 0.0, 0.0, 1.0],
            ]),
            printable: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct Part {
    pub mesh: Mesh,
    pub transform: Transform3d,
    pub matrix: Transform3d,
    pub use_f32_transform: bool,
    pub source_file: String,
    pub source_object: i32,
    pub source_volume: i32,
    pub metadata: Vec<Metadata>,
    pub mesh_stat: Option<MeshStatistics>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SavedModel {
    pub objects: Vec<Object>,
    pub orca_version: Option<String>,
}

impl Object {
    fn from_placed_stl_object(object: &crate::ProjectObject) -> Result<Self, SliceError> {
        let ([instance], [volume]) = (object.instances(), object.volumes()) else {
            return Err(deferred("expected one STL instance and volume"));
        };
        let name = object.name();
        let instance_placement = instance.transform().to_column_major();
        let volume_transform = Transform3d::from_rows(std::array::from_fn(|row| {
            std::array::from_fn(|column| volume.transform().as_matrix()[row][column] as f32 as f64)
        }));
        let volume_placement = volume_transform.to_column_major();
        let mesh = Mesh {
            name: name.to_owned(),
            vertices: volume
                .mesh()
                .vertices()
                .iter()
                .map(|p| [p.x as f32, p.y as f32, p.z as f32])
                .collect(),
            triangles: volume
                .mesh()
                .triangles()
                .iter()
                .map(|tri| tri.map(|i| i as usize))
                .collect(),
            use_f32_vertices: true,
            placement: [
                volume_placement[12],
                volume_placement[13],
                volume_placement[14],
            ],
            source_offset: [
                volume_placement[12] as f32,
                volume_placement[13] as f32,
                volume_placement[14] as f32,
            ],
        };
        Ok(Self {
            name: name.to_owned(),
            instances: vec![Instance::at([
                instance_placement[12],
                instance_placement[13],
                instance_placement[14],
            ])],
            metadata: cli_object_metadata(),
            auto_drop: true,
            parts: vec![Part {
                mesh,
                transform: volume_transform,
                matrix: volume_transform,
                use_f32_transform: true,
                source_file: name.to_owned(),
                source_object: 0,
                source_volume: 0,
                metadata: Vec::new(),
                mesh_stat: None,
            }],
        })
    }

    pub(super) fn unsliced(mesh: Mesh, filename: &str) -> Self {
        Self {
            name: mesh.name.clone(),
            instances: vec![Instance::at(mesh.placement)],
            metadata: cli_object_metadata(),
            auto_drop: true,
            parts: vec![Part {
                mesh,
                transform: Transform3d::IDENTITY,
                matrix: Transform3d::IDENTITY,
                use_f32_transform: false,
                source_file: filename.to_owned(),
                source_object: 0,
                source_volume: 0,
                metadata: Vec::new(),
                mesh_stat: None,
            }],
        }
    }
}

fn cli_object_metadata() -> Vec<Metadata> {
    vec![Metadata {
        key: "extruder".into(),
        value: "1".into(),
    }]
}

pub(crate) fn placed_stl_model(project: &crate::Project) -> Result<SavedModel, SliceError> {
    Ok(SavedModel {
        objects: project
            .objects()
            .iter()
            .map(Object::from_placed_stl_object)
            .collect::<Result<_, _>>()?,
        orca_version: None,
    })
}

#[path = "saved.rs"]
mod saved;
pub(super) use saved::load_saved;
pub(super) use saved::model_part;

pub(super) fn load(input: &[u8], filename: &str) -> Result<Mesh, SliceError> {
    if filename.to_ascii_lowercase().ends_with(".stl") {
        return stl(input, filename);
    }
    if !filename.to_ascii_lowercase().ends_with(".3mf") {
        return Err(deferred("expected an STL or a plain 3MF"));
    }
    plain(input, filename)
}

fn stl(input: &[u8], filename: &str) -> Result<Mesh, SliceError> {
    let (model, _) = crate::stl::load(input)?;
    let mut mesh = Mesh {
        name: filename.to_owned(),
        vertices: Vec::new(),
        triangles: Vec::with_capacity(model.triangles().len()),
        use_f32_vertices: false,
        placement: [0.0; 3],
        source_offset: [0.0; 3],
    };
    // ADMesh indexes the facets in encounter order (`stl_generate_shared_vertices`).
    let mut index = HashMap::new();
    for triangle in model.triangles() {
        mesh.triangles.push(triangle.vertices().map(|p| {
            let vertex = [p.x, p.y, p.z];
            let key = vertex.map(|value| if value == 0.0 { 0 } else { value.to_bits() });
            *index.entry(key).or_insert_with(|| {
                mesh.vertices.push(vertex);
                mesh.vertices.len() - 1
            })
        }));
    }
    mesh.center(false)?;
    Ok(mesh)
}

fn plain(input: &[u8], filename: &str) -> Result<Mesh, SliceError> {
    let mut archive = ProjectArchive::open(input, ArchiveLimits::PROJECT)?;
    if let Some(part) = archive
        .paths()
        .find(|path| !matches!(path.as_str(), MODEL | "_rels/.rels" | "[Content_Types].xml"))
    {
        return Err(deferred(&format!("archive member {}", part.as_str())));
    }
    let relationships = archive.read(&PackagePath::entry(b"_rels/.rels")?)?;
    let relationships: super::super::relationships::Relationships =
        quick_xml::de::from_reader(relationships.as_slice())
            .map_err(|err| invalid(err.to_string()))?;
    if relationships.relationships.len() != 1
        || relationships
            .resolve_required(
                &PackagePath::root(),
                super::super::relationships::MODEL_RELATIONSHIP_TYPE,
            )?
            .as_str()
            != MODEL
    {
        return Err(deferred("expected one root model relationship"));
    }
    let model_bytes = archive.read(&PackagePath::entry(MODEL.as_bytes())?)?;
    let doc: ModelDocument = quick_xml::de::from_reader(model_bytes.as_slice())
        .map_err(|err| invalid(err.to_string()))?;
    doc.validate()?;
    let [object] = doc.resources.objects.as_slice() else {
        return Err(deferred("expected one object"));
    };
    let [item] = doc.build.items.as_slice() else {
        return Err(deferred("expected one build item"));
    };
    let Some(geometry) = &object.mesh else {
        return Err(deferred("object components"));
    };
    if !doc.required_extensions.is_empty()
        || !doc.metadata.is_empty()
        || !doc.resources.color_groups.is_empty()
        || object.pid != 0
        || object.object_type != super::super::model_xml::ModelObjectType::Model
        || item.object_id != object.id
        || item.path.is_some()
        || !item.printable
        || item.transform.without_translation() != Transform3d::default()
        || geometry.triangles.triangles.iter().any(|tri| {
            tri.paint_supports != Default::default()
                || tri.paint_color != Default::default()
                || tri.paint_fuzzy_skin != Default::default()
                || !tri.face_property.is_empty()
        })
    {
        return Err(deferred(
            "advanced model metadata, transforms, or facet properties",
        ));
    }
    let factor = doc.unit.millimeter_factor();
    let name = if object.name.is_empty() {
        filename
    } else {
        &object.name
    };
    if name.contains(['/', '\\', '\0']) {
        return Err(deferred("object name cannot be used as a package path"));
    }
    let offset = item.transform.to_column_major();
    let mut mesh = Mesh {
        name: name.to_owned(),
        vertices: geometry
            .vertices
            .vertices
            .iter()
            .map(|p| [p.x * factor, p.y * factor, p.z * factor])
            .collect(),
        triangles: geometry
            .triangles
            .triangles
            .iter()
            .map(|tri| [tri.v1 as usize, tri.v2 as usize, tri.v3 as usize])
            .collect(),
        use_f32_vertices: false,
        placement: [offset[12], offset[13], offset[14]],
        source_offset: [0.0; 3],
    };
    mesh.center(true)?;
    Ok(mesh)
}

impl Mesh {
    // Model::center_around_origin and ModelObject::ensure_on_bed: preserve
    // input XY placement when loading 3MF; create an origin instance for STL.
    fn center(&mut self, loaded_placement: bool) -> Result<(), SliceError> {
        let Some(&first) = self.vertices.first() else {
            return Err(invalid("empty mesh"));
        };
        if self.triangles.is_empty() {
            return Err(invalid("empty mesh"));
        }
        for axis in 0..3 {
            let (min, max) = self
                .vertices
                .iter()
                .fold((first[axis], first[axis]), |(min, max), p| {
                    (min.min(p[axis]), max.max(p[axis]))
                });
            let center = (min + max) * 0.5;
            for vertex in &mut self.vertices {
                vertex[axis] -= center;
            }
            if loaded_placement {
                self.source_offset[axis] = center;
                self.placement[axis] += f64::from(center);
            }
            if axis == 2 {
                self.placement[2] = f64::from(center - min);
            }
        }
        Ok(())
    }
}
