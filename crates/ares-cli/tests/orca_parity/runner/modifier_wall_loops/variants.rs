use std::io::{Cursor, Read, Write};

use super::{compare, native_runner};

#[derive(Clone, Copy)]
enum Variant {
    Range(i32),
    Zero,
    Outside,
    Nested,
}

macro_rules! case {
    ($name:ident, $backend:literal, $variant:expr) => {
        #[tokio::test]
        async fn $name() {
            let Some(runner) = native_runner() else {
                return;
            };
            let input: &[u8] = match $backend {
                "classic" => include_bytes!(
                    "../../../../../../tests/parity/validation/modifier-wall-loops-classic.3mf"
                ),
                "arachne" => include_bytes!(
                    "../../../../../../tests/parity/validation/modifier-wall-loops-arachne.3mf"
                ),
                _ => unreachable!(),
            };
            compare(&runner, stringify!($name), &rewrite(input, $variant)).await;
        }
    };
}

case!(
    classic_range_parent3_modifier4,
    "classic",
    Variant::Range(4)
);
case!(
    arachne_range_parent3_modifier4,
    "arachne",
    Variant::Range(4)
);
case!(
    classic_equal_parent2_range3_modifier2,
    "classic",
    Variant::Range(2)
);
case!(
    arachne_equal_parent2_range3_modifier2,
    "arachne",
    Variant::Range(2)
);
case!(classic_modifier_zero_walls, "classic", Variant::Zero);
case!(arachne_modifier_zero_walls, "arachne", Variant::Zero);
case!(classic_nonoverlapping_modifier, "classic", Variant::Outside);
case!(arachne_nonoverlapping_modifier, "arachne", Variant::Outside);
case!(classic_nested_modifier4_then5, "classic", Variant::Nested);
case!(arachne_nested_modifier4_then5, "arachne", Variant::Nested);

fn rewrite(input: &[u8], variant: Variant) -> Vec<u8> {
    let mut archive = zip::ZipArchive::new(Cursor::new(input)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let mut nested_mesh = None;
    for index in 0..archive.len() {
        let mut file = archive.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        let name = file.name();
        if name == "Metadata/model_settings.config" {
            let mut xml = String::from_utf8(bytes).unwrap();
            match variant {
                Variant::Range(value) => {
                    xml = xml.replace(
                        "<metadata key=\"wall_loops\" value=\"4\"/>",
                        &format!("<metadata key=\"wall_loops\" value=\"{value}\"/>"),
                    );
                }
                Variant::Zero => {
                    xml = xml.replace(
                        "<metadata key=\"wall_loops\" value=\"4\"/>",
                        "<metadata key=\"wall_loops\" value=\"0\"/>",
                    );
                }
                Variant::Outside => xml = xml.replace("120.5", "150.5"),
                Variant::Nested => {
                    xml = xml.replace("</object>", "<part id=\"4\" subtype=\"modifier_part\"><metadata key=\"name\" value=\"nested-wall5\"/><metadata key=\"matrix\" value=\"0.25 0 0 121.75 0 1 0 138 0 0 1 5 0 0 0 1\"/><metadata key=\"wall_loops\" value=\"5\"/></part></object>");
                }
            }
            bytes = xml.into_bytes();
        } else if name == "3D/3dmodel.model" {
            let mut xml = String::from_utf8(bytes).unwrap();
            match variant {
                Variant::Outside => xml = xml.replace("120.5", "150.5"),
                Variant::Nested => {
                    xml = xml.replace("</components>", "<component p:path=\"/3D/Objects/wall-nested.model\" objectid=\"4\" p:UUID=\"00040000-b206-40ff-9872-83e8017abed1\" transform=\"0.25 0 0 0 1 0 0 0 1 121.75 138 5\"/></components>");
                }
                Variant::Range(_) | Variant::Zero => {}
            }
            bytes = xml.into_bytes();
        } else if name == "3D/_rels/3dmodel.model.rels" && matches!(variant, Variant::Nested) {
            bytes = String::from_utf8(bytes).unwrap().replace("</Relationships>", "<Relationship Target=\"/3D/Objects/wall-nested.model\" Id=\"rel-4\" Type=\"http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel\"/></Relationships>").into_bytes();
        } else if name == "3D/Objects/epsilon-modifier.model" && matches!(variant, Variant::Nested)
        {
            nested_mesh = Some(
                String::from_utf8(bytes.clone())
                    .unwrap()
                    .replace("id=\"3\"", "id=\"4\"")
                    .replace("00030000", "00040000")
                    .into_bytes(),
            );
        }
        output
            .start_file(name, zip::write::SimpleFileOptions::DEFAULT)
            .unwrap();
        output.write_all(&bytes).unwrap();
    }
    if let Variant::Range(_) = variant {
        output
            .start_file(
                "Metadata/layer_config_ranges.xml",
                zip::write::SimpleFileOptions::DEFAULT,
            )
            .unwrap();
        output.write_all(b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<objects><object id=\"1\"><range min_z=\"2\" max_z=\"6\"><option opt_key=\"layer_height\">0.2</option><option opt_key=\"wall_loops\">3</option></range></object></objects>\n").unwrap();
    }
    if let Some(mesh) = nested_mesh {
        output
            .start_file(
                "3D/Objects/wall-nested.model",
                zip::write::SimpleFileOptions::DEFAULT,
            )
            .unwrap();
        output.write_all(&mesh).unwrap();
    }
    output.finish().unwrap().into_inner()
}
