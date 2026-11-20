use std::collections::HashMap;

use super::super::{
    ArchiveLimits, PackagePath, ProjectArchive, Transform3d, model_xml::ModelDocument,
};
use super::{MODEL, deferred, invalid};
use crate::SliceError;

pub(super) struct Mesh {
    pub name: String,
    pub vertices: Vec<[f32; 3]>,
    pub triangles: Vec<[usize; 3]>,
    pub placement: [f64; 3],
    pub source_offset: [f32; 3],
}

pub(super) struct Object {
    pub name: String,
    pub placement: [f64; 3],
    pub parts: Vec<Part>,
}

pub(super) struct Part {
    pub mesh: Mesh,
    pub transform: Transform3d,
    pub matrix: Transform3d,
    pub source_file: String,
    pub source_object: i32,
    pub source_volume: i32,
}

pub(super) struct SavedModel {
    pub objects: Vec<Object>,
    pub orca_version: Option<String>,
}

impl Object {
    pub(super) fn unsliced(mesh: Mesh, filename: &str) -> Self {
        Self {
            name: mesh.name.clone(),
            placement: mesh.placement,
            parts: vec![Part {
                mesh,
                transform: Transform3d::IDENTITY,
                matrix: Transform3d::IDENTITY,
                source_file: filename.to_owned(),
                source_object: 0,
                source_volume: 0,
            }],
        }
    }
}

fn has_painted_triangles(model: &[u8]) -> Result<bool, SliceError> {
    use quick_xml::events::Event;

    let mut reader = quick_xml::Reader::from_reader(model);
    loop {
        let element = match reader
            .read_event()
            .map_err(|err| invalid(err.to_string()))?
        {
            Event::Start(element) | Event::Empty(element) => element,
            Event::Eof => return Ok(false),
            _ => continue,
        };
        if element.local_name().as_ref() != b"triangle" {
            continue;
        }
        // bbs_3mf.cpp:355–363: the writer does not preserve paint attributes.
        for attribute in element.attributes() {
            let attribute = attribute.map_err(|err| invalid(err.to_string()))?;
            if attribute.key.local_name().as_ref().starts_with(b"paint_")
                && !attribute.value.is_empty()
            {
                return Ok(true);
            }
        }
    }
}

pub(super) fn load_saved(input: &[u8]) -> Result<SavedModel, SliceError> {
    let project = crate::load_project(input)?;
    let paths: std::collections::BTreeSet<_> = project
        .objects()
        .iter()
        .flat_map(|object| object.volumes().iter().map(|part| part.source_model_path()))
        .collect();
    let mut archive = ProjectArchive::open(input, ArchiveLimits::PROJECT)?;
    if archive.paths().any(|path| {
        !paths.contains(path.as_str())
            && !matches!(
                path.as_str(),
                MODEL
                    | "3D/_rels/3dmodel.model.rels"
                    | "_rels/.rels"
                    | "[Content_Types].xml"
                    | "Metadata/project_settings.config"
                    | "Metadata/model_settings.config"
                    | "Metadata/slice_info.config"
                    | "Metadata/filament_sequence.json"
            )
    }) {
        return Err(deferred("additional saved archive members"));
    }
    for path in paths {
        let bytes = archive.read(&PackagePath::entry(path.as_bytes())?)?;
        if has_painted_triangles(&bytes)? {
            return Err(deferred("painted saved mesh"));
        }
    }
    let model_bytes = archive.read(&PackagePath::entry(MODEL.as_bytes())?)?;
    let doc: ModelDocument = quick_xml::de::from_reader(model_bytes.as_slice())
        .map_err(|err| invalid(err.to_string()))?;
    let settings = &project.documents().model_settings.objects;
    let objects = project
        .objects()
        .iter()
        .map(|object| {
            let ([volume], [instance]) = (object.volumes(), object.instances()) else {
                return Err(deferred(
                    "expected one saved volume and instance per object",
                ));
            };
            if !instance.printable()
                || instance.transform().without_translation() != Transform3d::IDENTITY
                || object.name().contains(['/', '\\', '\0'])
            {
                return Err(deferred("saved model metadata or transforms"));
            }
            let settings = settings
                .iter()
                .find(|entry| entry.id == object.id())
                .ok_or_else(|| deferred("expected saved object settings"))?;
            let offset = instance.transform().to_column_major();
            Ok(Object {
                name: object.name().to_owned(),
                placement: [offset[12], offset[13], offset[14]],
                parts: vec![saved_part(volume, &settings.parts)?],
            })
        })
        .collect::<Result<_, SliceError>>()?;
    Ok(SavedModel {
        objects,
        orca_version: doc
            .metadata
            .into_iter()
            .find(|m| m.name == "OrcaSlicer")
            .map(|m| m.value),
    })
}

fn saved_part(
    volume: &crate::ProjectVolume,
    settings: &[super::super::model_settings::PartSettings],
) -> Result<Part, SliceError> {
    let part = settings
        .iter()
        .find(|entry| entry.id == volume.id())
        .ok_or_else(|| deferred("expected saved part settings"))?;
    if part.subtype != "normal_part" || volume.transform() != Transform3d::IDENTITY {
        return Err(deferred("saved part type or transform"));
    }
    let metadata = |key| {
        part.retained_metadata
            .iter()
            .find(|entry| entry.key == key)
            .map_or("", |entry| entry.value.as_str())
    };
    Ok(Part {
        mesh: Mesh {
            name: volume.name().to_owned(),
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
            placement: [0.0; 3],
            source_offset: ["source_offset_x", "source_offset_y", "source_offset_z"]
                .map(|key| metadata(key).parse().unwrap_or(0.0)),
        },
        transform: volume.transform(),
        matrix: volume.transform().then(volume.source_transform()),
        source_file: metadata("source_file").to_owned(),
        source_object: metadata("source_object_id").parse().unwrap_or(-1),
        source_volume: metadata("source_volume_id").parse().unwrap_or(-1),
    })
}

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
    let model = crate::stl::load(input)?;
    let mut mesh = Mesh {
        name: filename.to_owned(),
        vertices: Vec::new(),
        triangles: Vec::with_capacity(model.triangles().len()),
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
