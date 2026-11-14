use std::io::{Cursor, Read, Write};

const CONTROL: &[u8] = include_bytes!(
    "../../../../../tests/parity/machine-acceleration-cache-rounding/stock-control.3mf"
);

const MEMBER: &str = "3D/Objects/cube10.stl_1.model";
const ROOT: &str = "3D/3dmodel.model";
const SETTINGS: &str = "Metadata/model_settings.config";

pub(super) fn member(input: &[u8], name: &str) -> String {
    let mut archive = zip::ZipArchive::new(Cursor::new(input)).unwrap();
    let mut text = String::new();
    archive
        .by_name(name)
        .unwrap()
        .read_to_string(&mut text)
        .unwrap();
    text
}

pub(super) fn replace_members(input: &[u8], replacements: &[(&str, String)]) -> Vec<u8> {
    let mut archive = zip::ZipArchive::new(Cursor::new(input)).unwrap();
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..archive.len() {
        let entry = archive.by_index(index).unwrap();
        if let Some((_, text)) = replacements.iter().find(|(name, _)| *name == entry.name()) {
            writer
                .start_file(entry.name(), zip::write::SimpleFileOptions::default())
                .unwrap();
            writer.write_all(text.as_bytes()).unwrap();
        } else {
            writer.raw_copy_file(entry).unwrap();
        }
    }
    writer.finish().unwrap().into_inner()
}

pub(super) fn geometry(coplanar: bool) -> Vec<u8> {
    let input = asymmetric_project(None);
    let mut xml = member(&input, MEMBER);
    let start = xml.find("<mesh>").unwrap();
    let end = xml.find("</mesh>").unwrap() + "</mesh>".len();
    let mesh = if coplanar {
        r#"<mesh><vertices>
<vertex x="0" y="0" z="-5"/><vertex x="11" y="0" z="-5"/>
<vertex x="1" y="7" z="-5"/><vertex x="1" y="0.5" z="3.5"/>
<vertex x="6.5" y="0.5" z="3.5"/><vertex x="1.5" y="4" z="3.5"/>
</vertices><triangles>
<triangle v1="0" v2="2" v3="1"/><triangle v1="3" v2="4" v3="5"/>
<triangle v1="0" v2="1" v3="4"/><triangle v1="0" v2="4" v3="3"/>
<triangle v1="1" v2="2" v3="5"/><triangle v1="1" v2="5" v3="4"/>
<triangle v1="2" v2="0" v3="3"/><triangle v1="2" v2="3" v3="5"/>
</triangles></mesh>"#
    } else {
        r#"<mesh><vertices>
<vertex x="0" y="0" z="-5"/><vertex x="11" y="0" z="-5"/>
<vertex x="1" y="7" z="-5"/><vertex x="2" y="1" z="12"/>
<vertex x="16" y="0" z="-5"/><vertex x="22" y="0" z="-5"/>
<vertex x="17" y="4" z="-5"/><vertex x="35" y="1" z="4"/>
</vertices><triangles>
<triangle v1="0" v2="2" v3="1"/><triangle v1="0" v2="1" v3="3"/>
<triangle v1="1" v2="2" v3="3"/><triangle v1="2" v2="0" v3="3"/>
<triangle v1="4" v2="6" v3="5"/><triangle v1="4" v2="5" v3="7"/>
<triangle v1="5" v2="6" v3="7"/><triangle v1="6" v2="4" v3="7"/>
</triangles></mesh>"#
    };
    xml.replace_range(start..end, mesh);
    replace_members(&input, &[(MEMBER, xml)])
}

pub(super) fn transformed(volume: bool, mirror: bool) -> Vec<u8> {
    let input = asymmetric_project(None);
    let input = if volume && mirror {
        input
    } else {
        let mut xml = member(&input, MEMBER);
        let start = xml.find("<mesh>").unwrap();
        let end = xml.find("</mesh>").unwrap() + 7;
        xml.replace_range(
            start..end,
            r#"<mesh><vertices>
<vertex x="0" y="0" z="-5"/><vertex x="11" y="0" z="-5"/>
<vertex x="11" y="7" z="-5"/><vertex x="0" y="7" z="-5"/>
<vertex x="0.5" y="0.5" z="12"/><vertex x="9.5" y="0.5" z="12"/>
<vertex x="9.5" y="5.5" z="12"/><vertex x="0.5" y="5.5" z="12"/>
</vertices><triangles>
<triangle v1="0" v2="2" v3="1"/><triangle v1="0" v2="3" v3="2"/>
<triangle v1="4" v2="5" v3="6"/><triangle v1="4" v2="6" v3="7"/>
<triangle v1="0" v2="1" v3="5"/><triangle v1="0" v2="5" v3="4"/>
<triangle v1="1" v2="2" v3="6"/><triangle v1="1" v2="6" v3="5"/>
<triangle v1="2" v2="3" v3="7"/><triangle v1="2" v2="7" v3="6"/>
<triangle v1="3" v2="0" v3="4"/><triangle v1="3" v2="4" v3="7"/>
</triangles></mesh>"#,
        );
        replace_members(&input, &[(MEMBER, xml)])
    };
    let root = member(&input, ROOT);
    let linear = if mirror {
        "-1.25 0 0 0 0.75 0 0 0 1.5"
    } else {
        "0 1.25 0 -0.75 0 0 0 0 1.5"
    };
    let root = if volume {
        root.replace("1 0 0 0 1 0 0 0 1 0 0 0", &format!("{linear} 0 0 0"))
            .replace("110 110 5", "110 110 7.5")
    } else {
        root.replace(
            "1 0 0 0 1 0 0 0 1 110 110 5",
            &format!("{linear} 110 110 7.5"),
        )
    };
    replace_members(&input, &[(ROOT, root)])
}

pub(super) fn multiple(kind: &str) -> Vec<u8> {
    let input = asymmetric_project(None);
    let mut root = member(&input, ROOT);
    let mut settings = member(&input, SETTINGS);
    let mut mesh = member(&input, MEMBER);
    match kind {
        "volumes" => {
            let object = mesh[mesh.find("<object ").unwrap()..mesh.find("</object>").unwrap() + 9]
                .to_owned();
            let second = object
                .replace("id=\"1\"", "id=\"3\"")
                .replace("00010000-", "00030000-");
            mesh = mesh.replace("</resources>", &format!("{second}</resources>"));
            root = root.replace("</components>", r#"<component p:path="/3D/Objects/cube10.stl_1.model" objectid="3" p:UUID="00030000-b206-40ff-9872-83e8017abed1" transform="0.5 0 0 0 0.75 0 0 0 1 18 0 0"/></components>"#);
            let part = settings
                [settings.find("<part ").unwrap()..settings.find("</part>").unwrap() + 7]
                .to_owned();
            settings = settings.replacen(
                "</object>",
                &format!("{}\n</object>", part.replace("id=\"1\"", "id=\"3\"")),
                1,
            );
        }
        "instances" => {
            root = root.replace("</build>", r#"<item objectid="2" p:UUID="00000003-b1ec-4553-aec9-835e5b724bb4" transform="1 0 0 0 1 0 0 0 1 140 110 5" printable="1" auto_drop="1"/></build>"#);
            settings = settings.replace("</plate>", r#"<model_instance><metadata key="object_id" value="2"/><metadata key="instance_id" value="1"/><metadata key="identify_id" value="15"/></model_instance></plate>"#);
        }
        "objects" | "plates" => {
            let object = root[root.find("<object ").unwrap()..root.find("</object>").unwrap() + 9]
                .to_owned();
            root = root.replace(
                "</resources>",
                &format!(
                    "{}</resources>",
                    object
                        .replace("id=\"2\"", "id=\"4\"")
                        .replace("00000001-", "00000004-")
                ),
            );
            root = root.replace("</build>", r#"<item objectid="4" p:UUID="00000004-b1ec-4553-aec9-835e5b724bb4" transform="0 0.5 0 -0.5 0 0 0 0 0.5 150 110 2.5" printable="1" auto_drop="1"/></build>"#);
            let object = settings
                [settings.find("<object ").unwrap()..settings.find("</object>").unwrap() + 9]
                .to_owned();
            settings = settings.replacen(
                "<plate>",
                &format!("{}<plate>", object.replace("id=\"2\"", "id=\"4\"")),
                1,
            );
            let instance = r#"<model_instance><metadata key="object_id" value="4"/><metadata key="instance_id" value="0"/><metadata key="identify_id" value="15"/></model_instance>"#;
            settings = if kind == "plates" {
                settings.replace(
                    "</plate>",
                    &format!(
                        "</plate><plate><metadata key=\"plater_id\" value=\"2\"/>{instance}</plate>"
                    ),
                )
            } else {
                settings.replace("</plate>", &format!("{instance}</plate>"))
            };
        }
        _ => unreachable!(),
    }
    let mut replacements = vec![(ROOT, root), (SETTINGS, settings), (MEMBER, mesh)];
    if kind == "plates" {
        let path = "Metadata/filament_sequence.json";
        let mut sequences: serde_json::Value = serde_json::from_str(&member(&input, path)).unwrap();
        sequences["plate_2"] = sequences["plate_1"].clone();
        replacements.push((path, serde_json::to_string(&sequences).unwrap()));
    }
    replace_members(&input, &replacements)
}

pub(super) fn asymmetric_project(vertices: Option<&str>) -> Vec<u8> {
    const MEMBER: &str = "3D/Objects/cube10.stl_1.model";
    let mut archive = zip::ZipArchive::new(Cursor::new(CONTROL)).unwrap();
    let mut xml = String::new();
    archive
        .by_name(MEMBER)
        .unwrap()
        .read_to_string(&mut xml)
        .unwrap();
    let start = xml.find("<mesh>").unwrap();
    let end = xml.find("</mesh>").unwrap() + "</mesh>".len();
    xml.replace_range(
        start..end,
        r#"<mesh><vertices>
<vertex x="0" y="0" z="-5"/><vertex x="11" y="0" z="-5"/>
<vertex x="1" y="7" z="-5"/><vertex x="2" y="1" z="12"/>
</vertices><triangles>
<triangle v1="0" v2="2" v3="1"/><triangle v1="0" v2="1" v3="3"/>
<triangle v1="1" v2="2" v3="3"/><triangle v1="2" v2="0" v3="3"/>
</triangles></mesh>"#,
    );
    if let Some(vertices) = vertices {
        let start = xml.find("<vertices>").unwrap() + "<vertices>".len();
        let end = xml[start..].find("</vertices>").unwrap() + start;
        xml.replace_range(start..end, vertices);
    }
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..archive.len() {
        let entry = archive.by_index(index).unwrap();
        if entry.name() == MEMBER {
            writer
                .start_file(MEMBER, zip::write::SimpleFileOptions::default())
                .unwrap();
            writer.write_all(xml.as_bytes()).unwrap();
        } else {
            writer.raw_copy_file(entry).unwrap();
        }
    }
    writer.finish().unwrap().into_inner()
}
