use super::super::super::{
    ArchiveLimits, PackagePath, ProjectArchive, Transform3d, model_settings::PartSettings,
    model_xml::ModelDocument,
};
use super::{Instance, MODEL, cli_object_metadata, deferred, invalid};
use super::{Mesh, Object, Part, SavedModel};
use crate::SliceError;

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

pub(crate) fn load_saved(input: &[u8]) -> Result<SavedModel, SliceError> {
    let project = crate::load_project(input)?;
    from_saved_project(
        ProjectArchive::open(input, ArchiveLimits::PROJECT)?,
        &project,
    )
}

pub(crate) fn from_saved_project(
    mut archive: ProjectArchive<'_>,
    project: &crate::Project,
) -> Result<SavedModel, SliceError> {
    let paths: std::collections::BTreeSet<_> = project
        .objects()
        .iter()
        .flat_map(|object| object.volumes().iter().map(|part| part.source_model_path()))
        .collect();

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
            let [instance] = object.instances() else {
                return Err(deferred("expected one saved instance per object"));
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
                instances: vec![Instance::at([offset[12], offset[13], offset[14]])],
                metadata: cli_object_metadata(),
                auto_drop: true,
                parts: object
                    .volumes()
                    .iter()
                    .map(|volume| saved_part(volume, &settings.parts))
                    .collect::<Result<_, _>>()?,
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
    settings: &[PartSettings],
) -> Result<Part, SliceError> {
    let part = settings
        .iter()
        .find(|entry| entry.id == volume.id())
        .ok_or_else(|| deferred("expected saved part settings"))?;
    if volume.volume_type() != crate::ProjectVolumeType::ModelPart {
        return Err(deferred("expected ordinary model parts"));
    }
    Ok(model_part(volume, Some(part)))
}

pub(in crate::project::export) fn model_part(
    volume: &crate::ProjectVolume,
    part: Option<&PartSettings>,
) -> Part {
    let metadata = |key| {
        part.into_iter()
            .flat_map(|part| &part.retained_metadata)
            .find(|entry| entry.key == key)
            .map_or("", |entry| entry.value.as_str())
    };
    Part {
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
            use_f32_vertices: false,
            placement: [0.0; 3],
            source_offset: ["source_offset_x", "source_offset_y", "source_offset_z"]
                .map(|key| metadata(key).parse().unwrap_or(0.0)),
        },
        transform: volume.transform(),
        matrix: volume.transform().then(volume.source_transform()),
        source_file: metadata("source_file").to_owned(),
        source_object: metadata("source_object_id").parse().unwrap_or(-1),
        source_volume: metadata("source_volume_id").parse().unwrap_or(-1),
        metadata: Vec::new(),
        mesh_stat: None,
    }
}
