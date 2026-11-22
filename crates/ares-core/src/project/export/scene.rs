use std::collections::BTreeMap;
use std::fmt::Write;

use crate::project::model_settings::{AssembleSettings, Metadata, PartSettings};
use crate::{Project, ProjectVolumeType, SliceError};

use super::{ObjectEntry, geometry, invalid, project_settings, write_package, xml};

#[derive(Clone, Copy)]
struct SceneInstance {
    object: usize,
    ordinal: usize,
    identify: u32,
}

struct ScenePlate<'a> {
    id: u32,
    metadata: &'a [Metadata],
    instances: Vec<SceneInstance>,
}

pub(super) struct SceneLayout<'a> {
    plates: Vec<ScenePlate<'a>>,
    instances: BTreeMap<(u32, u32), SceneInstance>,
    assemble: Option<&'a AssembleSettings>,
}

/// Writes an unsliced project from the model and project options already in memory.
pub fn export_project_3mf(project: &Project) -> Result<Vec<u8>, SliceError> {
    write(project, &project.documents().project_settings_raw, None)
}

pub(crate) fn write(
    project: &Project,
    raw: &crate::project::raw_settings::ProjectSettingsRaw,
    plates: Option<&[crate::PlaterPlate]>,
) -> Result<Vec<u8>, SliceError> {
    if !project.plates_custom_gcodes.is_empty() {
        return Err(unsupported("custom layer G-code"));
    }
    let docs = &project.documents().model_settings;
    let objects = project
        .objects()
        .iter()
        .map(|object| {
            if object.name().contains(['/', '\\', '\0']) {
                return Err(invalid("object name cannot be used as a 3MF member name"));
            }
            if !object.layer_height_profile().is_empty() || !object.layer_config_ranges().is_empty()
            {
                return Err(unsupported("layer height profiles or layer ranges"));
            }
            if !object.brim_points.is_empty() {
                return Err(unsupported("brim ear points"));
            }
            let settings = docs
                .objects
                .iter()
                .find(|settings| settings.id == object.id());
            let mut metadata = Vec::new();
            if !object.module().is_empty() {
                metadata.push(field("module", object.module()));
            }
            if let Some(settings) = settings {
                metadata.extend(
                    settings
                        .options
                        .iter()
                        .filter(|item| item.key != "extruder")
                        .cloned(),
                );
            }
            if let Some(extruder) = object.region_overrides().extruder {
                metadata.push(field("extruder", extruder.0.to_string()));
            }
            let parts = object
                .volumes()
                .iter()
                .map(|volume| {
                    let unsupported_type = match volume.volume_type() {
                        ProjectVolumeType::ModelPart => None,
                        ProjectVolumeType::NegativeVolume => Some("negative volumes"),
                        ProjectVolumeType::ParameterModifier => Some("parameter modifiers"),
                        ProjectVolumeType::SupportEnforcer => Some("support enforcers"),
                        ProjectVolumeType::SupportBlocker => Some("support blockers"),
                    };
                    if let Some(reason) = unsupported_type {
                        return Err(unsupported(reason));
                    }
                    let mesh = volume.mesh();
                    if [
                        &mesh.supported_facets,
                        &mesh.mmu_segmentation_facets,
                        &mesh.fuzzy_skin_facets,
                        &mesh.seam_facets,
                    ]
                    .into_iter()
                    .any(|facets| !facets.mapping.is_empty())
                    {
                        return Err(unsupported("painted facets"));
                    }
                    let settings = settings.and_then(|settings| {
                        settings.parts.iter().find(|part| part.id == volume.id())
                    });
                    let mut part = geometry::model_part(volume, settings);
                    part.source_file = part
                        .source_file
                        .rsplit(['/', '\\'])
                        .next()
                        .unwrap()
                        .to_owned();
                    if let Some(settings) = settings {
                        retain_part_settings(&mut part, settings)?;
                    }
                    if let Some(extruder) = volume.region_overrides().extruder {
                        part.metadata
                            .push(field("extruder", extruder.0.to_string()));
                    }
                    Ok(part)
                })
                .collect::<Result<Vec<_>, SliceError>>()?;
            Ok(geometry::Object {
                name: object.name().to_owned(),
                instances: object
                    .instances()
                    .iter()
                    .map(|instance| geometry::Instance {
                        transform: instance.transform(),
                        printable: instance.printable(),
                    })
                    .collect(),
                parts,
                metadata,
                auto_drop: object
                    .instances()
                    .first()
                    .is_none_or(crate::ProjectInstance::auto_drop),
            })
        })
        .collect::<Result<Vec<_>, SliceError>>()?;
    let scene = SceneLayout::new(project, plates);
    let settings = project_settings::write_secure(raw)?;
    write_package(&objects, &[], settings, None, Some(&scene))
}

fn retain_part_settings(
    part: &mut geometry::Part,
    settings: &PartSettings,
) -> Result<(), SliceError> {
    if settings.has_parametric_shape {
        return Err(unsupported("text or embossed shapes"));
    }
    part.metadata.extend(
        settings
            .options
            .iter()
            .filter(|item| item.key != "extruder")
            .cloned(),
    );
    part.metadata.extend(
        settings
            .retained_metadata
            .iter()
            .filter(|item| matches!(item.key.as_str(), "source_in_inches" | "source_in_meters"))
            .cloned(),
    );
    part.mesh_stat = settings.mesh_stat.clone();
    Ok(())
}

pub(super) fn unsupported(reason: &str) -> SliceError {
    invalid(format!("Cannot save a project containing {reason}."))
}

fn field(key: &str, value: impl Into<String>) -> Metadata {
    Metadata {
        key: key.into(),
        value: value.into(),
    }
}

impl<'a> SceneLayout<'a> {
    fn new(project: &'a Project, plater_plates: Option<&[crate::PlaterPlate]>) -> Self {
        let docs = &project.documents().model_settings;
        let instances: BTreeMap<_, _> = project
            .objects()
            .iter()
            .enumerate()
            .flat_map(|(object, entry)| {
                entry
                    .instances()
                    .iter()
                    .enumerate()
                    .map(move |(ordinal, instance)| {
                        let identify = instance
                            .saved_identify_id()
                            .filter(|&id| id != 0)
                            .unwrap_or_else(|| instance.labeled_id());
                        (
                            (entry.id(), instance.instance_id()),
                            SceneInstance {
                                object,
                                ordinal,
                                identify,
                            },
                        )
                    })
            })
            .collect();
        let metadata = |source: Option<usize>| {
            source
                .and_then(|index| docs.plates.get(index))
                .map_or(&[][..], |settings| &settings.metadata)
        };
        let plates = if let Some(plates) = plater_plates {
            plates
                .iter()
                .enumerate()
                .map(|(index, plate)| ScenePlate {
                    id: index as u32 + 1,
                    metadata: metadata(plate.source),
                    instances: plate
                        .instances
                        .iter()
                        .filter_map(|&(object, ordinal)| {
                            let object = &project.objects()[object];
                            let item = object.instances().get(ordinal)?;
                            instances.get(&(object.id(), item.instance_id())).copied()
                        })
                        .collect(),
                })
                .collect()
        } else if project.plates().is_empty() {
            vec![ScenePlate {
                id: 1,
                metadata: &[],
                instances: instances.values().copied().collect(),
            }]
        } else {
            project
                .plates()
                .iter()
                .enumerate()
                .map(|(index, plate)| ScenePlate {
                    id: plate.id(),
                    metadata: docs
                        .plates
                        .get(index)
                        .map_or(&[], |settings| &settings.metadata),
                    instances: plate
                        .instances()
                        .iter()
                        .filter_map(|&[object, instance, _]| {
                            instances.get(&(object, instance)).copied()
                        })
                        .collect(),
                })
                .collect()
        };
        Self {
            plates,
            instances,
            assemble: docs.assemble.as_ref(),
        }
    }

    pub(super) fn filament_sequence(&self) -> Vec<u8> {
        let entries: BTreeMap<_, _> = self.plates.iter().enumerate().map(|(index, _)| (
            format!("plate_{}", index + 1),
            serde_json::json!({ "nozzle_sequence": [], "optimal_assignment": [], "sequence": [] }),
        )).collect();
        serde_json::to_vec(&entries).expect("serializing an empty filament sequence")
    }

    pub(super) fn write_model_settings(&self, out: &mut String, objects: &[ObjectEntry<'_>]) {
        // A plate's G-code paths and thumbnails belong to a sliced or rendered result, not to this model.
        const PLATE_FIELDS: &[&str] = &[
            "bed_type",
            "print_sequence",
            "first_layer_print_sequence",
            "other_layers_print_sequence",
            "other_layers_print_sequence_nums",
            "spiral_vase_mode",
            "imex_parallel_mode",
            "imex_head_filament_map",
            "filament_map_mode",
            "filament_maps",
            "filament_volume_maps",
        ];
        for plate in &self.plates {
            let value = |key, default| {
                plate
                    .metadata
                    .iter()
                    .find(|item| item.key == key)
                    .map_or(default, |item| item.value.as_str())
            };
            out.push_str("  <plate>\n");
            xml::metadata(out, "    ", "plater_id", &plate.id.to_string());
            xml::metadata(out, "    ", "plater_name", value("plater_name", ""));
            xml::metadata(out, "    ", "locked", value("locked", "false"));
            for &key in PLATE_FIELDS {
                let item = plate.metadata.iter().find(|item| item.key == key);
                if let Some(item) = item {
                    xml::metadata(out, "    ", key, &item.value);
                }
            }
            for instance in &plate.instances {
                out.push_str("    <model_instance>\n");
                xml::metadata(
                    out,
                    "      ",
                    "object_id",
                    &objects[instance.object].id.to_string(),
                );
                xml::metadata(out, "      ", "instance_id", &instance.ordinal.to_string());
                xml::metadata(out, "      ", "identify_id", &instance.identify.to_string());
                out.push_str("    </model_instance>\n");
            }
            out.push_str("  </plate>\n");
        }
        out.push_str("  <assemble>\n");
        for item in self
            .assemble
            .into_iter()
            .flat_map(|assemble| &assemble.items)
        {
            let Some(instance) = self.instances.get(&(item.object_id, item.instance_id)) else {
                continue;
            };
            let id = objects[instance.object].id;
            let ordinal = instance.ordinal;
            let transform = quick_xml::escape::escape(&item.transform);
            let offset = quick_xml::escape::escape(&item.offset);
            writeln!(out, "   <assemble_item object_id=\"{id}\" instance_id=\"{ordinal}\" transform=\"{transform}\" offset=\"{offset}\" />").unwrap();
        }
        out.push_str("  </assemble>\n");
    }
}
