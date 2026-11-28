use super::super::super::{
    ArchiveLimits, PackagePath, ProjectArchive, Transform3d,
    model_settings::{ObjectSettings, PartSettings},
    model_xml::ModelDocument,
};
use super::super::saved_assets::{self, SavedArchiveExtras};
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
    archive: ProjectArchive<'_>,
    project: &crate::Project,
) -> Result<SavedModel, SliceError> {
    from_saved_project_with_extras(archive, project, None).map(|(model, _)| model)
}

pub(in crate::project::export) fn from_saved_project_with_extras(
    mut archive: ProjectArchive<'_>,
    project: &crate::Project,
    full_cli_source_filename: Option<&str>,
) -> Result<(SavedModel, SavedArchiveExtras), SliceError> {
    let full_cli_slice = full_cli_source_filename.is_some();
    let paths: std::collections::BTreeSet<_> = project
        .objects()
        .iter()
        .flat_map(|object| object.volumes().iter().map(|part| part.source_model_path()))
        .collect();

    let additional = {
        let mut additional = archive.paths().filter(|path| {
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
        });
        if full_cli_slice {
            additional.cloned().collect::<Vec<_>>()
        } else if additional.next().is_some() {
            return Err(deferred("additional saved archive members"));
        } else {
            Vec::new()
        }
    };
    let extras = if additional.is_empty() {
        SavedArchiveExtras::default()
    } else {
        saved_assets::load(&mut archive, project, additional)?
    };
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
        .enumerate()
        .map(|(object_index, object)| {
            let [instance] = object.instances() else {
                return Err(deferred("expected one saved instance per object"));
            };
            let transform = instance.transform();
            let translation_only = transform.without_translation() == Transform3d::IDENTITY;
            if !instance.printable()
                || !safe_name(object.name())
                || (!translation_only && !full_cli_slice)
            {
                return Err(deferred("saved model metadata or transforms"));
            }
            let settings = settings
                .iter()
                .find(|entry| entry.id == object.id())
                .ok_or_else(|| deferred("expected saved object settings"))?;
            let mut saved_instance = if translation_only {
                let offset = transform.to_column_major();
                Instance::at([offset[12], offset[13], offset[14]])
            } else {
                if !instance.auto_drop() || !supports_build_transform(object, settings, transform) {
                    return Err(deferred("saved model metadata or transforms"));
                }
                Instance {
                    transform,
                    printable: instance.printable(),
                    assemble: None,
                }
            };
            saved_instance.assemble = instance.assemble();
            Ok(Object {
                name: object.name().to_owned(),
                instances: vec![saved_instance],
                metadata: cli_object_metadata(),
                auto_drop: true,
                parts: object
                    .volumes()
                    .iter()
                    .enumerate()
                    .map(|(volume_index, volume)| {
                        let mut part = saved_part(volume, &settings.parts, full_cli_slice)?;
                        if let Some(filename) = full_cli_source_filename {
                            project_source_provenance(
                                &mut part,
                                filename,
                                object_index,
                                volume_index,
                            );
                        }
                        Ok(part)
                    })
                    .collect::<Result<_, SliceError>>()?,
            })
        })
        .collect::<Result<_, SliceError>>()?;
    Ok((
        SavedModel {
            objects,
            orca_version: doc
                .metadata
                .into_iter()
                .find(|m| m.name == "OrcaSlicer")
                .map(|m| m.value),
        },
        extras,
    ))
}

fn project_source_provenance(
    part: &mut Part,
    filename: &str,
    object_index: usize,
    volume_index: usize,
) {
    // bbs_3mf.cpp:2360–2374 fills only unset indices on fallback.
    if part.source_file.is_empty() {
        part.source_file = filename.to_owned();
        if part.source_object == -1 {
            part.source_object = object_index as i32;
        }
        if part.source_volume == -1 {
            part.source_volume = volume_index as i32;
        }
    } else if let Some(separator) = part.source_file.rfind(['/', '\\']) {
        // bbs_3mf.cpp:8249–8259,8374–8387: write the basename.
        part.source_file = part.source_file[separator + 1..].to_owned();
    }
}

fn safe_name(name: &str) -> bool {
    !name.contains(['/', '\\', '\0'])
}

fn supports_build_transform(
    object: &crate::ProjectObject,
    settings: &ObjectSettings,
    transform: Transform3d,
) -> bool {
    let ([volume], [part]) = (object.volumes(), settings.parts.as_slice()) else {
        return false;
    };
    // A nonzero volume offset can be folded into the build matrix by the importer
    // (bbs_3mf.cpp:2337–2358). Keep that case out of this projection.
    volume.transform() == Transform3d::IDENTITY
        && volume.source_transform() == Transform3d::IDENTITY
        && settings
            .auto_drop
            .as_deref()
            .is_none_or(|value| value == "1")
        && settings.module.is_empty()
        && settings
            .options
            .iter()
            .all(|entry| entry.key == "extruder" && entry.value == "1")
        && safe_name(volume.name())
        && part.id == volume.id()
        && part.options.is_empty()
        && !part.has_parametric_shape
        && supported_provenance(part)
        && native_invertible_affine(transform)
}

fn supported_provenance(part: &PartSettings) -> bool {
    let mut filename = None;
    let [mut object, mut volume, mut x, mut y, mut z] = [false; 5];
    for entry in &part.retained_metadata {
        let value = entry.value.as_str();
        let (seen, valid) = match entry.key.as_str() {
            "name" | "matrix" => continue,
            "source_file" => {
                if filename.replace(value).is_some() {
                    return false;
                }
                continue;
            }
            "source_object_id" => (&mut object, value.parse::<i32>().is_ok()),
            "source_volume_id" => (&mut volume, value.parse::<i32>().is_ok()),
            "source_offset_x" => (&mut x, value.parse::<f32>().is_ok_and(f32::is_finite)),
            "source_offset_y" => (&mut y, value.parse::<f32>().is_ok_and(f32::is_finite)),
            "source_offset_z" => (&mut z, value.parse::<f32>().is_ok_and(f32::is_finite)),
            _ => return false,
        };
        if *seen || !valid {
            return false;
        }
        *seen = true;
    }
    filename.is_none_or(str::is_empty) || (object && volume && x && y && z)
}

// bbs_3mf.cpp:4464–4472, Eigen FullPivLU.h:321–342,489–539.
fn native_invertible_affine(transform: Transform3d) -> bool {
    let matrix = transform.as_matrix();
    if !matrix.iter().flatten().all(|value| value.is_finite()) {
        return false;
    }
    let mut lu: [[f64; 3]; 3] = [0, 1, 2].map(|col| [0, 1, 2].map(|row| matrix[row][col]));
    let mut pivots = [0.0; 3];
    for (k, magnitude) in pivots.iter_mut().enumerate() {
        let (row, col, score) = largest_pivot(&lu, k);
        *magnitude = score;
        if score == 0.0 || !score.is_finite() {
            return false;
        }
        for column in &mut lu {
            column.swap(k, row);
        }
        lu.swap(k, col);
        let (pivot_column, trailing) = lu[k..].split_first_mut().unwrap();
        let pivot = pivot_column[k];
        for value in &mut pivot_column[k + 1..] {
            *value /= pivot;
        }
        for column in trailing {
            let factor = column[k];
            for (value, multiplier) in column[k + 1..].iter_mut().zip(&pivot_column[k + 1..]) {
                *value -= multiplier * factor;
            }
        }
    }
    let cutoff = pivots.into_iter().reduce(f64::max).unwrap() * (3.0 * f64::EPSILON);
    pivots.into_iter().all(|pivot| pivot > cutoff)
}

fn largest_pivot(lu: &[[f64; 3]; 3], start: usize) -> (usize, usize, f64) {
    // Eigen visits each remaining corner in column-major order, keeping the first tie.
    let mut best = (start, start, lu[start][start].abs());
    for (col, column) in lu.iter().enumerate().skip(start) {
        for (row, coefficient) in column.iter().enumerate().skip(start) {
            let score = coefficient.abs();
            if score > best.2 {
                best = (row, col, score);
            }
        }
    }
    best
}

fn saved_part(
    volume: &crate::ProjectVolume,
    settings: &[PartSettings],
    full_cli_slice: bool,
) -> Result<Part, SliceError> {
    let part = settings
        .iter()
        .find(|entry| entry.id == volume.id())
        .ok_or_else(|| deferred("expected saved part settings"))?;
    if volume.volume_type() != crate::ProjectVolumeType::ModelPart {
        return Err(deferred("expected ordinary model parts"));
    }
    let mut saved = model_part(volume, Some(part));
    if full_cli_slice {
        saved.mesh_stat = Some(volume.mesh().repair_statistics().clone());
    }
    Ok(saved)
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
