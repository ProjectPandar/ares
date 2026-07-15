use std::{
    collections::BTreeMap,
    io::{Cursor, Read, Write},
};

use zip::{CompressionMethod, ZipArchive, ZipWriter, write::SimpleFileOptions};

pub(super) const FIXTURE: &[u8] =
    include_bytes!("../../../../../../tests/ksr_fdmtest_v4/ksr_fdmtest_v4.project.3mf");

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

pub(super) const ROOT_MODEL: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02" xmlns:p="http://schemas.microsoft.com/3dmanufacturing/production/2015/06" requiredextensions="p">
 <metadata name="OrcaSlicer">2.4.2</metadata>
 <resources>
  <object id="2" type="model"><components><component p:path="/3D/leaf.model" objectid="1" transform="1 0 0 0 1 0 0 0 1 0 0 0"/></components></object>
 </resources>
 <build><item objectid="2" transform="1 0 0 0 1 0 0 0 1 10 20 30" printable="1" auto_drop="1"/></build>
</model>"#;

pub(super) const LEAF_MODEL: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
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

const SLICE_INFO: &str =
    r#"<config><header><header_item key="OrcaSlicer-Version" value="2.4.2"/></header></config>"#;
const FILAMENT_SEQUENCE: &str =
    r#"{"plate_1":{"sequence":[],"nozzle_sequence":[],"optimal_assignment":[]}}"#;
const PLATE_JSON: &str = r#"{"bbox_all":[10,20,11,21],"bbox_objects":[{"area":0.5,"bbox":[10,20,11,21],"id":147,"layer_height":0.2,"name":"triangle"}],"bed_type":"hot_plate","filament_colors":[],"filament_ids":[],"first_extruder":0,"first_layer_time":1.0,"is_seq_print":false,"nozzle_diameter":0.4,"version":2}"#;

pub(in crate::project::tests) struct ProjectParts {
    entries: BTreeMap<String, Vec<u8>>,
}

impl ProjectParts {
    pub(in crate::project::tests) fn valid() -> Self {
        let mut entries = BTreeMap::new();
        for (path, text) in [
            ("[Content_Types].xml", CONTENT_TYPES),
            ("_rels/.rels", ROOT_RELATIONSHIPS),
            ("3D/root.model", ROOT_MODEL),
            ("3D/_rels/root.model.rels", MODEL_RELATIONSHIPS),
            ("3D/leaf.model", LEAF_MODEL),
            ("Metadata/model_settings.config", MODEL_SETTINGS),
            ("Metadata/slice_info.config", SLICE_INFO),
            ("Metadata/filament_sequence.json", FILAMENT_SEQUENCE),
            ("Metadata/plate_1.json", PLATE_JSON),
        ] {
            entries.insert(path.to_owned(), text.as_bytes().to_vec());
        }
        entries.insert(
            "Metadata/project_settings.config".to_owned(),
            br#"{"layer_height":"0.2"}"#.to_vec(),
        );
        entries.insert("Metadata/plate_1.png".to_owned(), b"opaque png".to_vec());
        Self { entries }
    }

    pub(in crate::project::tests) fn fixture() -> Self {
        let mut archive = ZipArchive::new(Cursor::new(FIXTURE)).unwrap();
        let mut entries = BTreeMap::new();
        for index in 0..archive.len() {
            let mut file = archive.by_index(index).unwrap();
            if file.is_dir() {
                continue;
            }
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes).unwrap();
            entries.insert(file.name().to_owned(), bytes);
        }
        Self { entries }
    }

    pub(in crate::project::tests) fn replace(&mut self, path: &str, from: &str, to: &str) {
        let text = String::from_utf8(self.entries.remove(path).unwrap()).unwrap();
        assert!(text.contains(from), "{path} does not contain {from:?}");
        self.entries
            .insert(path.to_owned(), text.replace(from, to).into_bytes());
    }

    pub(super) fn remove(&mut self, path: &str) {
        assert!(self.entries.remove(path).is_some(), "missing {path}");
    }

    pub(in crate::project::tests) fn insert_text(&mut self, path: &str, text: &str) {
        self.entries
            .insert(path.to_owned(), text.as_bytes().to_vec());
    }

    pub(in crate::project::tests) fn make_single_model(&mut self, model: &str) {
        self.insert_text("3D/root.model", model);
        self.remove("3D/_rels/root.model.rels");
        self.remove("3D/leaf.model");
    }

    pub(in crate::project::tests) fn set_model_settings_objects(
        &mut self,
        objects: &str,
        build_ids: &[u32],
    ) {
        let mut instance_counts = BTreeMap::<u32, u32>::new();
        let mut instances = String::new();
        for (index, object_id) in build_ids.iter().copied().enumerate() {
            let instance_id = instance_counts.entry(object_id).or_default();
            instances.push_str(&format!(
                r#"<model_instance><metadata key="object_id" value="{object_id}"/><metadata key="instance_id" value="{}"/><metadata key="identify_id" value="{}"/></model_instance>"#,
                *instance_id,
                index + 1_000
            ));
            *instance_id += 1;
        }
        self.insert_text(
            "Metadata/model_settings.config",
            &format!(
                r#"<config>{objects}<plate><metadata key="plater_id" value="1"/>{instances}</plate></config>"#
            ),
        );
    }

    pub(super) fn reuse_object_id_across_build_paths(&mut self) {
        self.insert_text(
            "3D/root.model",
            r#"<model xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02" xmlns:p="http://schemas.microsoft.com/3dmanufacturing/production/2015/06" requiredextensions="p"><resources/><build><item p:path="/3D/a.model" objectid="1"/><item p:path="/3D/b.model" objectid="1"/></build></model>"#,
        );
        self.insert_text(
            "3D/_rels/root.model.rels",
            r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Target="/3D/a.model" Id="a" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/><Relationship Target="/3D/b.model" Id="b" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/></Relationships>"#,
        );
        self.insert_text("3D/a.model", LEAF_MODEL);
        self.insert_text(
            "3D/b.model",
            &LEAF_MODEL.replace(
                r#"<vertex x="1" y="0" z="0"/>"#,
                r#"<vertex x="9" y="0" z="0"/>"#,
            ),
        );
        self.insert_text(
            "Metadata/model_settings.config",
            r#"<config><object id="1"><part id="1" subtype="normal_part"><metadata key="matrix" value="1 0 0 0 0 1 0 0 0 0 1 0 0 0 0 1"/></part></object><plate><metadata key="plater_id" value="1"/><model_instance><metadata key="object_id" value="1"/><metadata key="instance_id" value="0"/><metadata key="identify_id" value="700"/></model_instance><model_instance><metadata key="object_id" value="1"/><metadata key="instance_id" value="1"/><metadata key="identify_id" value="701"/></model_instance></plate><assemble><assemble_item object_id="1" instance_id="0" transform="1 0 0 0 1 0 0 0 1 0 0 0" offset="0 0 0"/><assemble_item object_id="1" instance_id="1" transform="1 0 0 0 1 0 0 0 1 0 0 0" offset="0 0 0"/></assemble></config>"#,
        );
    }

    pub(super) fn use_distinct_object_ids_across_build_paths(&mut self) {
        self.reuse_object_id_across_build_paths();
        self.replace(
            "3D/root.model",
            r#"p:path="/3D/b.model" objectid="1""#,
            r#"p:path="/3D/b.model" objectid="2""#,
        );
        self.replace("3D/b.model", r#"object id="1""#, r#"object id="2""#);
        self.insert_text(
            "Metadata/model_settings.config",
            r#"<config><object id="1"><part id="1" subtype="normal_part"><metadata key="matrix" value="1 0 0 0 0 1 0 0 0 0 1 0 0 0 0 1"/></part></object><object id="2"><part id="2" subtype="normal_part"><metadata key="matrix" value="1 0 0 0 0 1 0 0 0 0 1 0 0 0 0 1"/></part></object><plate><metadata key="plater_id" value="1"/><model_instance><metadata key="object_id" value="1"/><metadata key="instance_id" value="0"/><metadata key="identify_id" value="700"/></model_instance><model_instance><metadata key="object_id" value="2"/><metadata key="instance_id" value="0"/><metadata key="identify_id" value="701"/></model_instance></plate><assemble><assemble_item object_id="1" instance_id="0" transform="1 0 0 0 1 0 0 0 1 0 0 0" offset="0 0 0"/><assemble_item object_id="2" instance_id="0" transform="1 0 0 0 1 0 0 0 1 0 0 0" offset="0 0 0"/></assemble></config>"#,
        );
    }

    pub(super) fn add_second_instance_of_same_build_identity(&mut self) {
        self.replace(
            "3D/root.model",
            "</build>",
            r#"<item objectid="2" transform="1 0 0 0 1 0 0 0 1 40 50 60"/></build>"#,
        );
        self.replace(
            "Metadata/model_settings.config",
            "</plate>",
            r#"<model_instance><metadata key="object_id" value="2"/><metadata key="instance_id" value="1"/><metadata key="identify_id" value="902"/></model_instance></plate>"#,
        );
        self.replace(
            "Metadata/model_settings.config",
            "</assemble>",
            r#"<assemble_item object_id="2" instance_id="1" transform="1 0 0 0 1 0 0 0 1 0 0 60" offset="0 0 0"/></assemble>"#,
        );
    }

    pub(in crate::project::tests) fn bytes(self) -> Vec<u8> {
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
        for (path, bytes) in self.entries {
            writer.start_file(path, options).unwrap();
            writer.write_all(&bytes).unwrap();
        }
        writer.finish().unwrap().into_inner()
    }
}
