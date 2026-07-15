mod archive;
mod association;
mod invalid;

use std::{
    collections::BTreeMap,
    io::{Cursor, Write},
};

use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};

const CONTENT_TYPES: &str = r#"<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="model" ContentType="application/vnd.ms-package.3dmanufacturing-3dmodel+xml"/><Default Extension="png" ContentType="image/png"/></Types>"#;
const ROOT_RELATIONSHIPS: &str = r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Target="/3D/root.model" Id="r1" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/></Relationships>"#;
const SLICE_INFO: &str =
    r#"<config><header><header_item key="OrcaSlicer-Version" value="2.4.2"/></header></config>"#;
const FILAMENT_SEQUENCE: &str =
    r#"{"plate_1":{"sequence":[],"nozzle_sequence":[],"optimal_assignment":[]}}"#;
const PLATE_JSON: &str = r#"{"bbox_all":[0,0,1,1],"bbox_objects":[],"bed_type":"hot_plate","filament_colors":[],"filament_ids":[],"first_extruder":0,"first_layer_time":1.0,"is_seq_print":false,"nozzle_diameter":0.4,"version":2}"#;

struct LayerProject {
    entries: BTreeMap<String, Vec<u8>>,
}

impl LayerProject {
    fn with_build_order(object_ids: &[u32]) -> Self {
        let resources = object_ids
            .iter()
            .rev()
            .map(|id| {
                format!(
                    r#"<object id="{id}" type="model"><mesh><vertices><vertex x="0" y="0" z="0"/><vertex x="1" y="0" z="0"/><vertex x="0" y="1" z="0"/></vertices><triangles><triangle v1="0" v2="1" v3="2"/></triangles></mesh></object>"#
                )
            })
            .collect::<String>();
        let build = object_ids
            .iter()
            .map(|id| format!(r#"<item objectid="{id}"/>"#))
            .collect::<String>();
        let root_model = format!(
            r#"<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"><resources>{resources}</resources><build>{build}</build></model>"#
        );
        let instances = object_ids
            .iter()
            .enumerate()
            .map(|(index, id)| {
                format!(
                    r#"<model_instance><metadata key="object_id" value="{id}"/><metadata key="instance_id" value="0"/><metadata key="identify_id" value="{}"/></model_instance>"#,
                    index + 1
                )
            })
            .collect::<String>();
        let model_settings = format!(
            r#"<config><plate><metadata key="plater_id" value="1"/>{instances}</plate></config>"#
        );

        let mut entries = BTreeMap::new();
        for (path, bytes) in [
            ("[Content_Types].xml", CONTENT_TYPES.as_bytes()),
            ("_rels/.rels", ROOT_RELATIONSHIPS.as_bytes()),
            ("3D/root.model", root_model.as_bytes()),
            ("Metadata/model_settings.config", model_settings.as_bytes()),
            ("Metadata/slice_info.config", SLICE_INFO.as_bytes()),
            (
                "Metadata/filament_sequence.json",
                FILAMENT_SEQUENCE.as_bytes(),
            ),
            ("Metadata/plate_1.json", PLATE_JSON.as_bytes()),
            (
                "Metadata/project_settings.config",
                br#"{"layer_height":"0.2"}"#,
            ),
        ] {
            entries.insert(path.to_owned(), bytes.to_vec());
        }
        Self { entries }
    }

    fn one_object() -> Self {
        Self::with_build_order(&[42])
    }

    fn insert_ranges(&mut self, path: &str, xml: &str) {
        self.entries
            .insert(path.to_owned(), xml.as_bytes().to_vec());
    }

    fn bytes(self) -> Vec<u8> {
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
        for (path, bytes) in self.entries {
            writer.start_file(path, options).unwrap();
            writer.write_all(&bytes).unwrap();
        }
        writer.finish().unwrap().into_inner()
    }
}

fn error_message(project: LayerProject) -> String {
    crate::load_project(project.bytes())
        .unwrap_err()
        .to_string()
}

fn assert_bounded(message: &str) {
    assert!(
        message.len() <= 512,
        "unbounded error ({} bytes)",
        message.len()
    );
}
