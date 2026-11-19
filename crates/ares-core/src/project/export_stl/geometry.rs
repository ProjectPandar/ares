use std::collections::HashMap;

use quick_xml::{Reader, events::Event};

use super::super::{
    ArchiveLimits, PackagePath, ProjectArchive, Transform3d,
    model_xml::{ModelDocument, ModelObjectType, ModelUnit},
    relationships::{MODEL_RELATIONSHIP_TYPE, Relationships},
};
use super::{
    Face, SliceError, Vertex, coordinate_key, cross, dot, invalid, leaf_name, minus, normalized,
    unsupported,
};

const MODEL: &str = "3D/3dmodel.model";

pub(super) struct Mesh {
    pub name: String,
    pub vertices: Vec<Vertex>,
    pub faces: Vec<Face>,
    center: [f64; 3],
    offset: [f64; 3],
}

impl Mesh {
    fn imported(
        name: String,
        mut vertices: Vec<Vertex>,
        faces: Vec<Face>,
        mut offset: [f64; 3],
    ) -> Result<Self, SliceError> {
        let (Some(&first), false) = (vertices.first(), faces.is_empty()) else {
            return Err(invalid("empty STL export mesh"));
        };
        let (min, max) = vertices.iter().fold((first, first), |(min, max), p| {
            (
                std::array::from_fn(|i| min[i].min(p[i])),
                std::array::from_fn(|i| max[i].max(p[i])),
            )
        });
        // ModelVolume::center_geometry_after_creation; then ModelObject::ensure_on_bed
        // (standalone import, before explicit repair). Its bounds are in double precision.
        let center = std::array::from_fn(|i| 0.5 * (f64::from(min[i]) + f64::from(max[i])));
        if center != [0.0; 3] {
            for point in &mut vertices {
                *point = std::array::from_fn(|i| point[i] + -(center[i] as f32));
            }
        }
        let min_z = vertices.iter().map(|p| p[2]).fold(f32::INFINITY, f32::min);
        offset[2] += -(f64::from(min_z) + (center[2] + offset[2]));
        Ok(Self {
            name,
            vertices,
            faces,
            center,
            offset,
        })
    }

    pub(super) fn placed_vertices(&self) -> Result<Vec<Vertex>, SliceError> {
        self.vertices.iter().map(|p| self.place_vertex(p)).collect()
    }

    fn place_vertex(&self, p: &Vertex) -> Result<Vertex, SliceError> {
        let mut out = [0.0; 3];
        for (i, component) in out.iter_mut().enumerate() {
            let restored = f64::from(p[i]) + self.center[i];
            let raw = restored as f32;
            // Exclude rounding-sensitive placements: require the volume
            // restoration to be exact before composing a nonzero shift.
            if self.offset[i] != 0.0 && f64::from(raw) != restored {
                return Err(unsupported(
                    "STL export placement requires exact volume restoration",
                ));
            }
            *component = (f64::from(raw) + self.offset[i]) as f32;
        }
        Ok(out)
    }
}

pub(super) fn load(input: &[u8], filename: &str) -> Result<Vec<Mesh>, SliceError> {
    let extension = filename.rsplit_once('.').map(|(_, ext)| ext);
    if extension.is_some_and(|ext| ext.eq_ignore_ascii_case("stl")) {
        return stl(input, filename).map(|mesh| vec![mesh]);
    }
    if extension.is_some_and(|ext| ext.eq_ignore_ascii_case("3mf")) {
        return plain(input, filename);
    }
    Err(unsupported("STL export expects STL or plain 3MF input"))
}

fn stl(input: &[u8], filename: &str) -> Result<Mesh, SliceError> {
    let model = crate::stl::load_closed(input)?;
    let mut index = HashMap::new();
    let mut vertices = Vec::new();
    let faces = model
        .triangles()
        .iter()
        .map(|triangle| {
            triangle.vertices().map(|p| {
                let point = [p.x, p.y, p.z];
                *index.entry(coordinate_key(point)).or_insert_with(|| {
                    vertices.push(point);
                    vertices.len() - 1
                })
            })
        })
        .collect();
    Mesh::imported(filename.to_owned(), vertices, faces, [0.0; 3])
}

fn plain(input: &[u8], filename: &str) -> Result<Vec<Mesh>, SliceError> {
    let mut archive = ProjectArchive::open(input, ArchiveLimits::PROJECT)?;
    if archive
        .paths()
        .any(|path| !matches!(path.as_str(), MODEL | "_rels/.rels" | "[Content_Types].xml"))
    {
        return Err(unsupported(
            "STL export does not accept project or auxiliary archive members",
        ));
    }
    let relationships = archive.read(&PackagePath::entry(b"_rels/.rels")?)?;
    let relationships: Relationships = quick_xml::de::from_reader(relationships.as_slice())
        .map_err(|err| invalid(err.to_string()))?;
    if relationships.relationships.len() != 1
        || relationships
            .resolve_required(&PackagePath::root(), MODEL_RELATIONSHIP_TYPE)?
            .as_str()
            != MODEL
    {
        return Err(unsupported(
            "STL export requires one root 3MF model relationship",
        ));
    }
    let bytes = archive.read(&PackagePath::entry(MODEL.as_bytes())?)?;
    check_plain_xml(&bytes)?;
    let doc: ModelDocument =
        quick_xml::de::from_reader(bytes.as_slice()).map_err(|err| invalid(err.to_string()))?;
    doc.validate()?;
    if doc.unit != ModelUnit::Millimeter || doc.build.items.is_empty() {
        return Err(unsupported(
            "STL export requires a nonempty millimetre 3MF build",
        ));
    }
    let count = doc.resources.objects.len();
    let mut objects = doc
        .resources
        .objects
        .into_iter()
        .map(|object| (object.id, object))
        .collect::<HashMap<_, _>>();
    if objects.len() != count || doc.build.items.len() != count {
        return Err(unsupported(
            "STL export requires one build item per model object",
        ));
    }
    doc.build.items.into_iter().map(|item| {
        let Some(object) = objects.remove(&item.object_id) else {
            return Err(unsupported("STL export requires one build item per model object"));
        };
        if !item.printable || !item.auto_drop || item.transform.without_translation() != Transform3d::default() || object.object_type != ModelObjectType::Model {
            return Err(unsupported("STL export requires a printable mesh with translation only and automatic bed placement"));
        }
        // Third-party single-object 3MFs always use the input stem (bbs_3mf.cpp:3758–3763).
            let name = if count == 1 {
            filename.rsplit_once('.').unwrap().0.to_owned()
        } else {
            object.name
        };
        if !leaf_name(&name) {
            return Err(unsupported("STL export requires safe named 3MF objects"));
        }
        let Some(geometry) = object.mesh else {
            return Err(unsupported("STL export requires a mesh, not components"));
        };
        let vertices = geometry.vertices.vertices.into_iter().map(|p| [p.x, p.y, p.z]).collect::<Vec<_>>();
        let mut faces = geometry.triangles.triangles.into_iter().map(|f| [f.v1 as usize, f.v2 as usize, f.v3 as usize]).collect::<Vec<_>>();
        orient_imported_mesh(&vertices, &mut faces);
        let m = item.transform.to_column_major();
        Mesh::imported(name, vertices, faces, [m[12], m[13], m[14]])
    }).collect()
}

// bbs_3mf.cpp:5343, TriangleMesh.cpp:1479 (native volume test before centering).
fn orient_imported_mesh(vertices: &[Vertex], faces: &mut [Face]) {
    let Some(&origin) = vertices.first() else {
        return;
    };
    let volume = faces.iter().fold(0.0, |sum, &face| {
        let [a, b, c] = face.map(|i| vertices[i]);
        let v = cross(minus(b, a), minus(c, a));
        let area = 0.5 * dot(v, v).sqrt();
        sum + area * dot(normalized(v), minus(a, origin)) / 3.0
    });
    if volume < 0.0 {
        for face in faces {
            face.swap(1, 2);
        }
    }
}

// Do not silently drop attributes or resources unknown to the shared 3MF reader.
fn check_plain_xml(bytes: &[u8]) -> Result<(), SliceError> {
    let mut reader = Reader::from_reader(bytes);
    loop {
        let event = reader
            .read_event()
            .map_err(|err| invalid(err.to_string()))?;
        let node = match event {
            Event::Start(node) | Event::Empty(node) => node,
            Event::Eof => return Ok(()),
            Event::DocType(_) => {
                return Err(unsupported(
                    "STL export does not accept a 3MF document type",
                ));
            }
            _ => continue,
        };
        let tag = node.name();
        let keys: &[&[u8]] = match tag.as_ref() {
            b"model" => &[b"xmlns", b"unit", b"xml:lang"],
            b"object" => &[b"id", b"name", b"type"],
            b"item" => &[b"objectid", b"transform", b"printable", b"auto_drop"],
            b"vertex" => &[b"x", b"y", b"z"],
            b"triangle" => &[b"v1", b"v2", b"v3"],
            b"build" | b"resources" | b"mesh" | b"vertices" | b"triangles" => &[],
            _ => {
                return Err(unsupported(
                    "STL export does not support advanced 3MF elements",
                ));
            }
        };
        for attribute in node.attributes() {
            let attr = attribute.map_err(|err| invalid(err.to_string()))?;
            if !keys.contains(&attr.key.as_ref()) {
                return Err(unsupported(
                    "STL export does not support advanced 3MF attributes",
                ));
            }
        }
    }
}
