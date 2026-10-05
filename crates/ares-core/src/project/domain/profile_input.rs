//! STL Model import (`Format/STL.cpp:17-40`, `Model.cpp:485-500,1254-1264,2600-2615`).

use std::collections::HashMap;

use super::{
    Point3d, Project, ProjectInstance, ProjectMesh, ProjectModel, ProjectObject, ProjectVolume,
    ProjectVolumeType, Transform3d,
    documents::{PlateMetadata, ProjectDocuments},
};
use crate::{
    ProjectSettings, SliceError,
    options::ProjectSettingsBuilder,
    project::{
        filament_sequence::FilamentSequences,
        model_settings::ModelSettings,
        raw_settings::{ProjectSettingsRaw, RawConfigValue},
        slice_info::{SliceInfo, SliceInfoHeader},
    },
};
use serde::Deserialize as _;

impl Project {
    pub(crate) fn initialize_cli_loaded_project_config(&mut self) {
        // OrcaSlicer.cpp:1769: the CLI's 3MF import reads this dynamic option
        // with create=true. STL --load-settings does not perform that read.
        let raw = &mut self.documents.project_settings_raw;
        if !raw.contains_key("upward_compatible_machine") {
            raw.insert_strings(
                "upward_compatible_machine",
                &self.settings.printer.remaining.upward_compatible_machine.0,
            );
        }
    }

    /// 3MF selected-profile entry (`OrcaSlicer.cpp:1624-1640` loads the
    /// embedded config as the full-config base; `:2750-2812 update_full_config`
    /// overlays every key the loaded presets supply and keeps the rest).
    ///
    /// `settings` is the selected profile's fully composed config; `profile_raw`
    /// is the chain-expanded presence view and `filament_raws` carries one
    /// merged raw per selected filament for the `:3301` per-slot `set_at`
    /// overlay. Upstream indexes `different_settings_to_system` and
    /// `inherits_group` by the embedded `filament_count` (`:2817-2818`), so
    /// selecting more filaments than the embedded count is rejected rather
    /// than silently truncated or allowed to write out of bounds.
    pub(crate) fn from_3mf_with_profiles(
        input: &[u8],
        settings: &ProjectSettings,
        profile_raw: &ProjectSettingsRaw,
        filament_raws: &[ProjectSettingsRaw],
    ) -> Result<Self, SliceError> {
        let mut project = crate::load_project(input)?;
        let mut merged = project.documents.project_settings_raw.clone();
        let embedded_raw = merged.clone();

        let selected = settings.project.preset.filament_settings_id.0.len();
        let embedded_count = project
            .settings()
            .project
            .preset
            .filament_settings_id
            .0
            .len();
        if embedded_count == 0 {
            return Err(SliceError::InvalidInput(
                "project embeds no filament selection".to_owned(),
            ));
        }
        if selected > embedded_count {
            return Err(SliceError::InvalidInput(format!(
                "selection has {selected} filaments but the project embeds {embedded_count}"
            )));
        }

        let filament_keys: std::collections::BTreeSet<&String> = filament_raws
            .iter()
            .flat_map(|raw| raw.entries().map(|(key, _)| key))
            .collect();
        // Dynamic groups whose slot layout follows the embedded filament
        // count; they are rebuilt below.
        let dynamic_groups = [
            "different_settings_to_system",
            "inherits_group",
            "compatible_machine_expression_group",
            "compatible_process_expression_group",
        ];
        for (key, value) in profile_raw.entries() {
            if filament_keys.contains(key) || dynamic_groups.contains(&key.as_str()) {
                continue;
            }
            merged.insert(key.clone(), value.clone());
        }

        // Filament keys overlay per loaded slot (`set_at(src, index - 1, 0)`):
        // supplied slots take the filament's first element, the rest keep the
        // embedded element (front-copy for growth, `Config.hpp:427-447`). A key
        // absent from the embedded config leaves `Null` placeholders in slots
        // no filament supplies; Orca-exported projects embed the full config,
        // so that case is bounded to synthetic inputs.
        let mut inserts: Vec<(String, RawConfigValue)> = Vec::new();
        for key in &filament_keys {
            let embedded_len = embedded_raw
                .get(key.as_str())
                .map_or(0, RawConfigValue::len);
            let len = embedded_len.max(selected);
            let values = (0..len)
                .map(|index| filament_slot(&embedded_raw, filament_raws, key, index, selected))
                .collect();
            inserts.push(((*key).clone(), RawConfigValue::List(values)));
        }

        // Settings IDs (`:2048,:2088,:3155-3158`) carry the selected names into
        // the covered slots; the tail keeps embedded entries.
        inserts.push((
            "printer_settings_id".to_owned(),
            RawConfigValue::Text(settings.project.preset.printer_settings_id.0.clone()),
        ));
        inserts.push((
            "print_settings_id".to_owned(),
            RawConfigValue::Text(settings.project.preset.print_settings_id.0.clone()),
        ));
        for (key, loaded) in [
            (
                "filament_settings_id",
                &settings.project.preset.filament_settings_id.0,
            ),
            ("filament_ids", &settings.project.gcode.filament_ids.0),
        ] {
            let embedded_len = embedded_raw.get(key).map_or(0, RawConfigValue::len);
            let len = embedded_len.max(embedded_count);
            let values = (0..len)
                .map(|index| name_slot(&embedded_raw, key, loaded, index))
                .collect();
            inserts.push((key.to_owned(), RawConfigValue::List(values)));
        }
        // `:2980-2984`: the loaded process's `compatible_printers` overwrites
        // `print_compatible_printers` even when it is absent or empty.
        inserts.push((
            "print_compatible_printers".to_owned(),
            RawConfigValue::List(
                settings
                    .project
                    .preset
                    .print_compatible_printers
                    .0
                    .iter()
                    .map(|value| RawConfigValue::Text(value.clone()))
                    .collect(),
            ),
        ));
        // `:3117-3337`: single-variant selections rebuild `filament_self_index`
        // as a plain 1..=filament_count identity.
        inserts.push((
            "filament_self_index".to_owned(),
            RawConfigValue::List(
                (1..=embedded_count)
                    .map(|index| RawConfigValue::Number(serde_json::Number::from(index)))
                    .collect(),
            ),
        ));

        // `:2817-2818`: group vectors are embedded_count + 2, indexed
        // process | filaments | machine; only the loaded slots are written.
        let inherits: Vec<String> = (0..selected + 2)
            .filter_map(|index| {
                match profile_raw
                    .get("inherits_group")
                    .and_then(|value| value.element(index))
                {
                    Some(RawConfigValue::Text(text)) => Some(text.clone()),
                    _ => None,
                }
            })
            .collect();
        if !inherits.is_empty() {
            let len = embedded_count + 2;
            let values = (0..len)
                .map(|index| {
                    inherits_slot(
                        &inherits,
                        embedded_raw.get("inherits_group"),
                        index,
                        len,
                        selected,
                    )
                })
                .collect();
            inserts.push(("inherits_group".to_owned(), RawConfigValue::List(values)));
        }
        // `:2838,:2978,:3161`: the process slot, loaded filament slots, and the
        // machine slot are cleared; unloaded filament slots stay embedded.
        // `:2275,:3680-3699`: mapping every embedded filament to a single
        // preset disables the prime tower (unless a smooth timelapse needs
        // it) and records the key in the process slot.
        let single_preset_mapping = settings
            .project
            .preset
            .filament_settings_id
            .0
            .first()
            .is_some_and(|first| {
                settings
                    .project
                    .preset
                    .filament_settings_id
                    .0
                    .iter()
                    .all(|name| name == first)
            });
        fn truthy(value: Option<&RawConfigValue>) -> bool {
            match value {
                Some(RawConfigValue::Text(value)) => value == "1",
                Some(RawConfigValue::Flag(value)) => *value,
                _ => false,
            }
        }
        let smooth_timelapse = truthy(merged.get("enable_timelapse"))
            && matches!(merged.get("timelapse_type"), Some(RawConfigValue::Text(value)) if value == "1");
        let prime_tower_disabled = single_preset_mapping && !smooth_timelapse;
        if prime_tower_disabled {
            inserts.push((
                "enable_prime_tower".to_owned(),
                RawConfigValue::Text("0".to_owned()),
            ));
        }
        {
            let len = embedded_count + 2;
            let values = (0..len)
                .map(|index| {
                    different_settings_slot(
                        embedded_raw.get("different_settings_to_system"),
                        index,
                        len,
                        selected,
                        prime_tower_disabled,
                    )
                })
                .collect();
            inserts.push((
                "different_settings_to_system".to_owned(),
                RawConfigValue::List(values),
            ));
        }
        // `:3259-3271`: expression groups are embedded_count-indexed with
        // reserved process/machine slots; loaded filament conditions fill the
        // middle slots.
        // `cli_metadata::filament_conditions` lays the loaded conditions at
        // [process | filaments | machine] for the machine group and
        // [filaments] for the process group.
        for (key, len, offset) in [
            (
                "compatible_machine_expression_group",
                embedded_count + 2,
                1usize,
            ),
            (
                "compatible_process_expression_group",
                embedded_count,
                0usize,
            ),
        ] {
            let Some(group) = profile_raw.get(key) else {
                continue;
            };
            let values = (0..len)
                .map(|index| {
                    expression_group_slot(group, embedded_raw.get(key), index, offset, selected)
                })
                .collect();
            inserts.push((key.to_owned(), RawConfigValue::List(values)));
        }

        for (key, value) in inserts {
            merged.insert(key, value);
        }

        project.settings = ProjectSettingsBuilder::deserialize(merged.clone().into_object())
            .map_err(|error| {
                SliceError::InvalidInput(format!("merged project settings are invalid: {error}"))
            })?
            .resolve();
        project.documents.project_settings_raw = merged;
        project.initialize_cli_loaded_project_config();
        Ok(project)
    }

    pub(crate) fn from_stl_with_profiles(
        input: &[u8],
        filename: &str,
        settings: ProjectSettings,
        raw: ProjectSettingsRaw,
    ) -> Result<Self, SliceError> {
        let model = crate::load_model(input)?;
        let (vertices, triangles) = index_stl_vertices(&model);
        let mut min = [f64::INFINITY; 3];
        let mut max = [f64::NEG_INFINITY; 3];
        for vertex in &vertices {
            for axis in 0..3 {
                let coordinate = f64::from(vertex[axis]);
                min[axis] = min[axis].min(coordinate);
                max[axis] = max[axis].max(coordinate);
            }
        }
        let center: [f64; 3] = std::array::from_fn(|axis| (min[axis] + max[axis]) * 0.5);
        let vertices = vertices
            .into_iter()
            .map(|vertex| {
                let shifted: [f64; 3] =
                    std::array::from_fn(|axis| f64::from(vertex[axis] - center[axis] as f32));
                Point3d::new(shifted[0], shifted[1], shifted[2])
            })
            .collect();
        let transform =
            Transform3d::IDENTITY.pretranslated(Point3d::new(center[0], center[1], center[2]));
        let volume = ProjectVolume::new(
            filename.to_owned(),
            0,
            ProjectMesh::new(vertices, triangles),
            transform,
            (
                filename.to_owned(),
                ProjectVolumeType::ModelPart,
                Default::default(),
                Transform3d::IDENTITY,
            ),
        );
        let instance = ProjectInstance::new([1, 0, 0], true, false, Transform3d::IDENTITY);
        let object = ProjectObject::new(
            filename.to_owned(),
            1,
            (
                filename.to_owned(),
                String::new(),
                Default::default(),
                Default::default(),
            ),
            vec![volume],
            vec![instance],
        );
        Ok(Self::new(
            vec![ProjectModel::new(filename.to_owned(), vec![1])],
            vec![object],
            vec![PlateMetadata::new(1, vec![[1, 0, 0]])],
            settings,
            ProjectDocuments {
                model_settings: ModelSettings {
                    objects: Vec::new(),
                    plates: Vec::new(),
                    assemble: None,
                },
                project_settings_raw: raw,
                slice_info: SliceInfo {
                    header: SliceInfoHeader { items: Vec::new() },
                },
                filament_sequences: FilamentSequences(Default::default()),
                plate_documents: Vec::new(),
                has_painted_layer_height_profile: false,
            },
        ))
    }
}

/// One slot of a filament-vector overlay (`OrcaSlicer.cpp:3301` `set_at`).
fn filament_slot(
    embedded_raw: &ProjectSettingsRaw,
    filament_raws: &[ProjectSettingsRaw],
    key: &str,
    index: usize,
    selected: usize,
) -> RawConfigValue {
    if index < selected
        && let Some(value) = filament_raws[index].get(key)
        && let Some(element) = value.element(0)
    {
        return element.clone();
    }
    embedded_raw
        .get(key)
        .and_then(|value| value.element(index).or_else(|| value.element(0)))
        .cloned()
        .unwrap_or(RawConfigValue::Null)
}

/// One slot of a per-slot name vector (`filament_settings_id`, `filament_ids`).
fn name_slot(
    embedded_raw: &ProjectSettingsRaw,
    key: &str,
    loaded: &[String],
    index: usize,
) -> RawConfigValue {
    if index < loaded.len() {
        return RawConfigValue::Text(loaded[index].clone());
    }
    embedded_raw
        .get(key)
        .and_then(|value| value.element(index))
        .cloned()
        .unwrap_or_else(|| RawConfigValue::Text(String::new()))
}

/// One slot of the `inherits_group` vector (`OrcaSlicer.cpp:2817`).
fn inherits_slot(
    inherits: &[String],
    embedded: Option<&RawConfigValue>,
    index: usize,
    len: usize,
    selected: usize,
) -> RawConfigValue {
    let loaded = if index == 0 {
        inherits.first()
    } else if index == len - 1 {
        inherits.last()
    } else if index <= selected {
        inherits.get(index)
    } else {
        None
    };
    loaded
        .map(|text| RawConfigValue::Text(text.clone()))
        .or_else(|| embedded.and_then(|value| value.element(index)).cloned())
        .unwrap_or_else(|| RawConfigValue::Text(String::new()))
}

/// One slot of the `different_settings_to_system` vector
/// (`OrcaSlicer.cpp:2838,:2978,:3161,:3680-3699`).
fn different_settings_slot(
    embedded: Option<&RawConfigValue>,
    index: usize,
    len: usize,
    selected: usize,
    prime_tower_disabled: bool,
) -> RawConfigValue {
    if index == 0 {
        let text = if prime_tower_disabled {
            "enable_prime_tower"
        } else {
            ""
        };
        return RawConfigValue::Text(text.to_owned());
    }
    if index == len - 1 || index <= selected {
        return RawConfigValue::Text(String::new());
    }
    embedded
        .and_then(|value| value.element(index))
        .cloned()
        .unwrap_or_else(|| RawConfigValue::Text(String::new()))
}

/// One slot of a `compatible_*_expression_group` vector
/// (`OrcaSlicer.cpp:3259-3271`).
fn expression_group_slot(
    group: &RawConfigValue,
    embedded: Option<&RawConfigValue>,
    index: usize,
    offset: usize,
    selected: usize,
) -> RawConfigValue {
    if index >= offset
        && index - offset < selected
        && let Some(value) = group.element(index)
    {
        return value.clone();
    }
    embedded
        .and_then(|value| value.element(index))
        .cloned()
        .unwrap_or_else(|| RawConfigValue::Text(String::new()))
}

fn index_stl_vertices(model: &crate::Model) -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
    let mut vertices = Vec::new();
    let mut index = HashMap::new();
    let mut triangles = Vec::with_capacity(model.triangles().len());
    for triangle in model.triangles() {
        let ids = triangle.vertices().map(|vertex| {
            let coordinates = [vertex.x, vertex.y, vertex.z];
            *index.entry(vertex_key(coordinates)).or_insert_with(|| {
                let id = vertices.len() as u32;
                vertices.push(coordinates);
                id
            })
        });
        triangles.push(ids);
    }
    (vertices, triangles)
}

fn vertex_key(coordinates: [f32; 3]) -> [u32; 3] {
    coordinates.map(|value| if value == 0.0 { 0 } else { value.to_bits() })
}

#[cfg(test)]
mod tests {
    use std::io::Write as _;

    use super::*;
    use crate::project::raw_settings::RawConfigValue;

    const CONTENT_TYPES: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
 <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
 <Default Extension="model" ContentType="application/vnd.ms-package.3dmanufacturing-3dmodel+xml"/>
 <Default Extension="png" ContentType="image/png"/>
</Types>"#;

    const ROOT_RELATIONSHIPS: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
 <Relationship Target="/3D/root.model" Id="r1" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/>
</Relationships>"#;

    const ROOT_MODEL: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02" xmlns:p="http://schemas.microsoft.com/3dmanufacturing/production/2015/06" requiredextensions="p">
 <metadata name="OrcaSlicer">2.4.2</metadata>
 <resources>
  <object id="2" type="model"><components><component p:path="/3D/leaf.model" objectid="1" transform="1 0 0 0 1 0 0 0 1 0 0 0"/></components></object>
 </resources>
 <build><item objectid="2" transform="1 0 0 0 1 0 0 0 1 10 20 30" printable="1" auto_drop="1"/></build>
</model>"#;

    const LEAF_MODEL: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02" xmlns:p="http://schemas.microsoft.com/3dmanufacturing/production/2015/06" requiredextensions="p">
 <resources><object id="1" type="model"><mesh><vertices>
  <vertex x="0" y="0" z="0"/><vertex x="1" y="0" z="0"/><vertex x="0" y="1" z="0"/>
 </vertices><triangles><triangle v1="0" v2="1" v3="2"/></triangles></mesh></object></resources>
 <build/>
</model>"#;

    const MODEL_RELATIONSHIPS: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
 <Relationship Target="/3D/leaf.model" Id="r1" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/>
</Relationships>"#;

    const MODEL_SETTINGS: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<config>
 <object id="2"><part id="1" subtype="normal_part"><metadata key="matrix" value="1 0 0 0 0 1 0 0 0 0 1 0 0 0 0 1"/></part></object>
 <plate><metadata key="plater_id" value="1"/><model_instance><metadata key="object_id" value="2"/><metadata key="instance_id" value="0"/><metadata key="identify_id" value="133"/></model_instance></plate>
 <assemble><assemble_item object_id="2" instance_id="0" transform="1 0 0 0 1 0 0 0 1 0 0 30" offset="0 0 0"/></assemble>
</config>"#;

    const SLICE_INFO: &str = r#"<config><header><header_item key="OrcaSlicer-Version" value="2.4.2"/></header></config>"#;
    const FILAMENT_SEQUENCE: &str =
        r#"{"plate_1":{"sequence":[],"nozzle_sequence":[],"optimal_assignment":[]}}"#;

    fn archive(settings_json: &str) -> Vec<u8> {
        let mut buffer = std::io::Cursor::new(Vec::new());
        {
            let mut writer = zip::ZipWriter::new(&mut buffer);
            let options = zip::write::SimpleFileOptions::default();
            for (path, bytes) in [
                ("[Content_Types].xml", CONTENT_TYPES.as_bytes()),
                ("_rels/.rels", ROOT_RELATIONSHIPS.as_bytes()),
                ("3D/root.model", ROOT_MODEL.as_bytes()),
                ("3D/_rels/root.model.rels", MODEL_RELATIONSHIPS.as_bytes()),
                ("3D/leaf.model", LEAF_MODEL.as_bytes()),
                ("Metadata/model_settings.config", MODEL_SETTINGS.as_bytes()),
                ("Metadata/slice_info.config", SLICE_INFO.as_bytes()),
                (
                    "Metadata/filament_sequence.json",
                    FILAMENT_SEQUENCE.as_bytes(),
                ),
                ("Metadata/project_settings.config", settings_json.as_bytes()),
            ] {
                writer.start_file(path, options).unwrap();
                writer.write_all(bytes).unwrap();
            }
            writer.finish().unwrap();
        }
        buffer.into_inner()
    }

    fn composed(settings_json: &str) -> ProjectSettings {
        serde_json::from_str(settings_json).unwrap()
    }

    fn raw(settings_json: &str) -> ProjectSettingsRaw {
        serde_json::from_str(settings_json).unwrap()
    }

    fn text(value: Option<&RawConfigValue>, index: usize) -> String {
        match value.and_then(|value| value.element(index)) {
            Some(RawConfigValue::Text(text)) => text.clone(),
            Some(RawConfigValue::Number(number)) => number.to_string(),
            other => panic!("expected text at {index}, got {other:?}"),
        }
    }

    fn overlay(
        settings_json: &str,
        profile_settings_json: &str,
        profile_json: &str,
        filament_jsons: &[&str],
    ) -> Result<Project, SliceError> {
        let filament_raws = filament_jsons
            .iter()
            .map(|json| raw(json))
            .collect::<Vec<_>>();
        Project::from_3mf_with_profiles(
            &archive(settings_json),
            &composed(profile_settings_json),
            &raw(profile_json),
            &filament_raws,
        )
    }

    #[test]
    fn overlay_preserves_embedded_keys_and_filament_tail() {
        // OrcaSlicer.cpp:2750-2812: keys the presets do not supply keep the
        // embedded value; filament slots beyond the selection keep theirs
        // (:3301 set_at writes only the loaded indices).
        let settings = r#"{
            "printer_settings_id": "Embedded Machine",
            "print_settings_id": "Embedded Process",
            "filament_settings_id": ["Embedded Filament", "Second Filament"],
            "filament_diameter": [3.0, 2.85],
            "layer_height": "0.32"
        }"#;
        let profile = r#"{
            "filament_diameter": [1.75],
            "layer_height": "0.2"
        }"#;
        let filament = r#"{"filament_diameter": [1.75]}"#;
        let selection = r#"{
            "printer_settings_id": "Embedded Machine",
            "print_settings_id": "Embedded Process",
            "filament_settings_id": ["Embedded Filament"]
        }"#;
        let project = overlay(settings, selection, profile, &[filament]).unwrap();
        let merged = project.documents().project_settings_raw.clone();

        assert_eq!(
            text(merged.get("filament_diameter"), 0),
            "1.75",
            "supplied slot takes the selected filament value"
        );
        assert_eq!(
            text(merged.get("filament_diameter"), 1),
            "2.85",
            "unselected slot keeps the embedded value"
        );
        assert_eq!(
            text(merged.get("layer_height"), 0),
            "0.2",
            "process key supplied by the preset is overlaid"
        );
        assert_eq!(
            project.settings().project.preset.printer_settings_id.0,
            "Embedded Machine"
        );
    }

    #[test]
    fn overlay_rejects_more_selected_filaments_than_embedded() {
        let settings = r#"{
            "printer_settings_id": "M", "print_settings_id": "P",
            "filament_settings_id": ["Only One"]
        }"#;
        let selection = r#"{
            "filament_settings_id": ["A", "B"]
        }"#;
        let error = overlay(settings, selection, "{}", &["{}", "{}"]).unwrap_err();
        assert!(
            error.to_string().contains("selection has 2 filaments"),
            "{error}"
        );
    }

    #[test]
    fn overlay_overwrites_print_compatible_printers_unconditionally() {
        // OrcaSlicer.cpp:2980-2984: the loaded process's compatible_printers
        // replaces print_compatible_printers even when it is absent or empty.
        let settings = r#"{
            "printer_settings_id": "M", "print_settings_id": "P",
            "filament_settings_id": ["F"],
            "print_compatible_printers": ["Embedded Machine"]
        }"#;
        let project = overlay(
            settings,
            "{\"filament_settings_id\":[\"F\"]}",
            "{}",
            &["{}"],
        )
        .unwrap();
        let merged = project.documents().project_settings_raw.clone();
        assert!(merged.get("print_compatible_printers").unwrap().len() == 0);
        assert!(
            project
                .settings()
                .project
                .preset
                .print_compatible_printers
                .0
                .is_empty()
        );
    }

    #[test]
    fn single_preset_mapping_disables_prime_tower_and_records_the_key() {
        // OrcaSlicer.cpp:2275,3680-3699.
        let settings = r#"{
            "printer_settings_id": "M", "print_settings_id": "P",
            "filament_settings_id": ["Same", "Same"],
            "enable_prime_tower": "1"
        }"#;
        let project = overlay(
            settings,
            "{\"filament_settings_id\":[\"Same\",\"Same\"]}",
            "{}",
            &["{}", "{}"],
        )
        .unwrap();
        let merged = project.documents().project_settings_raw.clone();
        assert_eq!(text(merged.get("enable_prime_tower"), 0), "0");
        assert_eq!(
            text(merged.get("different_settings_to_system"), 0),
            "enable_prime_tower"
        );
    }

    #[test]
    fn smooth_timelapse_keeps_the_prime_tower() {
        let settings = r#"{
            "printer_settings_id": "M", "print_settings_id": "P",
            "filament_settings_id": ["Same"],
            "enable_prime_tower": "1",
            "enable_timelapse": "1",
            "timelapse_type": "1"
        }"#;
        let project = overlay(
            settings,
            "{\"filament_settings_id\":[\"Same\"]}",
            "{}",
            &["{}"],
        )
        .unwrap();
        let merged = project.documents().project_settings_raw.clone();
        assert_eq!(text(merged.get("enable_prime_tower"), 0), "1");
        assert_eq!(text(merged.get("different_settings_to_system"), 0), "");
    }

    #[test]
    fn distinct_filaments_keep_the_prime_tower() {
        let settings = r#"{
            "printer_settings_id": "M", "print_settings_id": "P",
            "filament_settings_id": ["A", "B"],
            "enable_prime_tower": "1"
        }"#;
        let project = overlay(
            settings,
            "{\"filament_settings_id\":[\"A\",\"B\"]}",
            "{}",
            &["{}", "{}"],
        )
        .unwrap();
        let merged = project.documents().project_settings_raw.clone();
        assert_eq!(text(merged.get("enable_prime_tower"), 0), "1");
        assert_eq!(text(merged.get("different_settings_to_system"), 0), "");
    }
}
