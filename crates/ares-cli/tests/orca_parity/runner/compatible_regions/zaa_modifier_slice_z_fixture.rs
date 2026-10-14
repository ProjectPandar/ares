use super::super::fixture;
use serde_json::{Value, json};
use std::io::{Cursor, Read, Write};

#[derive(Clone, Copy, Debug)]
pub(super) enum Case {
    Disabled,
    Equal,
    ModifierOnly,
    Smaller,
}

impl Case {
    pub(super) fn name(self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::Equal => "equal",
            Self::ModifierOnly => "modifier-only",
            Self::Smaller => "smaller-min-z",
        }
    }

    pub(super) fn active(self) -> bool {
        !matches!(self, Self::Disabled)
    }

    fn options(self) -> (&'static str, &'static str, &'static str) {
        match self {
            Self::Disabled => ("0", "0", "0.05"),
            Self::Equal => ("1", "1", "0.05"),
            Self::ModifierOnly => ("0", "1", "0.05"),
            Self::Smaller => ("1", "1", "0.02"),
        }
    }
}

pub(super) fn prepare(base: &[u8], case: Case) -> (Vec<u8>, Value) {
    let (parent, enabled, min_z) = case.options();
    let (roof, roof_proof) = super::super::zaa_contour_z::wedge(base, parent == "1");
    let mut input = zip::ZipArchive::new(Cursor::new(&roof)).unwrap();
    let mut annotation = String::new();
    input
        .by_name("3D/Objects/cube10.stl_1.model")
        .unwrap()
        .read_to_string(&mut annotation)
        .unwrap();
    let vertices = regex::Regex::new(r#"(<vertex x="[^"]+" y="[^"]+" z=")([^"]+)("/>)"#).unwrap();
    assert_eq!(vertices.find_iter(&annotation).count(), 8);
    let annotation = vertices
        .replace_all(&annotation, |m: &regex::Captures<'_>| {
            format!(
                "{}{}{}",
                &m[1],
                if m[2].parse::<f64>().unwrap() <= -5.0 {
                    "-5"
                } else {
                    "2"
                },
                &m[3]
            )
        })
        .replace("id=\"1\"", "id=\"3\"")
        .replace("00010000-", "00030000-");
    let component = r#"<component p:path="/3D/Objects/zaa-modifier-3.model" objectid="3" transform="0.5 0 0 0 1 0 0 0 1 -2.5 0 0"/>"#;
    let relationship = r#"<Relationship Target="/3D/Objects/zaa-modifier-3.model" Id="rel-zaa-modifier-3" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/>"#;
    let part = format!(
        r#"<part id="3" subtype="modifier_part"><metadata key="name" value="ZAA roof modifier"/><metadata key="matrix" value="0.5 0 0 -2.5 0 1 0 0 0 0 1 0 0 0 0 1"/><metadata key="zaa_enabled" value="{enabled}"/><metadata key="zaa_min_z" value="{min_z}"/></part>"#
    );
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let mut unchanged = serde_json::Map::new();
    for index in 0..input.len() {
        let mut file = input.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        bytes = match file.name() {
            "Metadata/project_settings.config" => {
                let mut settings: Value = serde_json::from_slice(&bytes).unwrap();
                settings["zaa_min_z"] = json!("0.05");
                serde_json::to_vec(&settings).unwrap()
            }
            "3D/3dmodel.model" => String::from_utf8(bytes)
                .unwrap()
                .replace("</components>", &format!("{component}</components>"))
                .into_bytes(),
            "3D/_rels/3dmodel.model.rels" => String::from_utf8(bytes)
                .unwrap()
                .replace(
                    "</Relationships>",
                    &format!("{relationship}</Relationships>"),
                )
                .into_bytes(),
            "Metadata/model_settings.config" => String::from_utf8(bytes)
                .unwrap()
                .replace("</object>", &format!("{part}</object>"))
                .into_bytes(),
            _ => {
                unchanged.insert(file.name().into(), json!(fixture::hash(&bytes)));
                bytes
            }
        };
        output
            .start_file(file.name(), zip::write::SimpleFileOptions::DEFAULT)
            .unwrap();
        output.write_all(&bytes).unwrap();
    }
    output
        .start_file(
            "3D/Objects/zaa-modifier-3.model",
            zip::write::SimpleFileOptions::DEFAULT,
        )
        .unwrap();
    output.write_all(annotation.as_bytes()).unwrap();
    let bytes = output.finish().unwrap().into_inner();
    let loaded = ares_core::load_project(&bytes).unwrap();
    assert_eq!(loaded.objects().len(), 1);
    let object = loaded.objects().first().unwrap();
    assert_eq!(object.instances().len(), 1);
    assert_eq!(object.volumes().len(), 2);
    assert_eq!(
        loaded.settings().process.region.zaa_enabled.0,
        parent == "1"
    );
    assert_eq!(loaded.settings().process.region.zaa_min_z.0, 0.05);
    let modifier = object.volumes().iter().find(|v| v.id() == 3).unwrap();
    assert_eq!(
        modifier.volume_type(),
        ares_core::ProjectVolumeType::ParameterModifier
    );
    let mut min = [f64::INFINITY; 3];
    let mut max = [f64::NEG_INFINITY; 3];
    for &vertex in modifier.mesh().vertices() {
        let p = modifier.transform().transform_point(vertex);
        for (axis, value) in [p.x, p.y, p.z].into_iter().enumerate() {
            min[axis] = min[axis].min(value);
            max[axis] = max[axis].max(value);
        }
    }
    assert_eq!(min, [-5.0, -5.0, -5.0]);
    assert_eq!(max, [0.0, 5.0, 2.0]);
    let mut final_input = zip::ZipArchive::new(Cursor::new(&bytes)).unwrap();
    for (name, expected) in &unchanged {
        let mut member = Vec::new();
        final_input
            .by_name(name)
            .unwrap()
            .read_to_end(&mut member)
            .unwrap();
        assert_eq!(json!(fixture::hash(&member)), *expected);
    }
    let proof = json!({"case":case.name(),"parent_enabled":parent,"parent_min_z":"0.05",
        "modifier_enabled":enabled,"modifier_min_z":min_z,"modifier_metadata":part,
        "loaded_modifier_min":min,"loaded_modifier_max":max,"roof":roof_proof,
        "unchanged_roof_members":unchanged,"modifier_mesh_sha256":fixture::hash(annotation.as_bytes())});
    (bytes, proof)
}
