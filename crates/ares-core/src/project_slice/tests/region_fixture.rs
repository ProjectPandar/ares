use super::support::KsrArchive;

pub(super) fn modifier_projects() -> (Vec<u8>, Vec<u8>) {
    (modifier_project(true), modifier_project(false))
}

fn modifier_project(with_override: bool) -> Vec<u8> {
    let settings = if with_override {
        MODIFIER_SETTINGS
    } else {
        CONTROL_SETTINGS
    };
    modifier_archive(settings).bytes()
}

fn modifier_archive(settings: &str) -> KsrArchive {
    let mut archive = KsrArchive::new();
    for (path, text) in [
        ("3D/3dmodel.model", ROOT_MODEL),
        ("3D/_rels/3dmodel.model.rels", RELATIONSHIPS),
        ("3D/Objects/ksr_fdmtest_v4.drc_2.model", NORMAL_LEAF),
        ("3D/Objects/task22j_modifier.model", MODIFIER_LEAF),
        ("Metadata/model_settings.config", settings),
    ] {
        archive.insert_text(path, text);
    }
    archive
}

#[rustfmt::skip]
const ROOT_MODEL: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<model unit=\"millimeter\" xmlns=\"http://schemas.microsoft.com/3dmanufacturing/core/2015/02\" xmlns:p=\"http://schemas.microsoft.com/3dmanufacturing/production/2015/06\" requiredextensions=\"p\">\n <metadata name=\"OrcaSlicer\">2.4.2</metadata>\n <resources><object id=\"2\" type=\"model\"><components>\n  <component p:path=\"/3D/Objects/ksr_fdmtest_v4.drc_2.model\" objectid=\"1\"/>\n  <component p:path=\"/3D/Objects/task22j_modifier.model\" objectid=\"3\"/>\n </components></object></resources>\n <build><item objectid=\"2\" transform=\"1 0 0 0 1 0 0 0 1 0 0 0\" printable=\"1\" auto_drop=\"1\"/></build>\n</model>";
#[rustfmt::skip]
const RELATIONSHIPS: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\n <Relationship Target=\"/3D/Objects/ksr_fdmtest_v4.drc_2.model\" Id=\"normal\" Type=\"http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel\"/>\n <Relationship Target=\"/3D/Objects/task22j_modifier.model\" Id=\"modifier\" Type=\"http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel\"/>\n</Relationships>";
#[rustfmt::skip]
const NORMAL_LEAF: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<model unit=\"millimeter\" xmlns=\"http://schemas.microsoft.com/3dmanufacturing/core/2015/02\">\n <resources><object id=\"1\" type=\"model\"><mesh><vertices>\n<vertex x=\"0\" y=\"0\" z=\"0\"/><vertex x=\"20\" y=\"0\" z=\"0\"/><vertex x=\"20\" y=\"2\" z=\"0\"/><vertex x=\"0\" y=\"2\" z=\"0\"/>\n<vertex x=\"0\" y=\"0\" z=\"0.4\"/><vertex x=\"20\" y=\"0\" z=\"0.4\"/><vertex x=\"20\" y=\"2\" z=\"0.4\"/><vertex x=\"0\" y=\"2\" z=\"0.4\"/>\n</vertices><triangles>\n<triangle v1=\"0\" v2=\"2\" v3=\"1\"/><triangle v1=\"0\" v2=\"3\" v3=\"2\"/>\n<triangle v1=\"4\" v2=\"5\" v3=\"6\"/><triangle v1=\"4\" v2=\"6\" v3=\"7\"/>\n<triangle v1=\"0\" v2=\"1\" v3=\"5\"/><triangle v1=\"0\" v2=\"5\" v3=\"4\"/>\n<triangle v1=\"1\" v2=\"2\" v3=\"6\"/><triangle v1=\"1\" v2=\"6\" v3=\"5\"/>\n<triangle v1=\"2\" v2=\"3\" v3=\"7\"/><triangle v1=\"2\" v2=\"7\" v3=\"6\"/>\n<triangle v1=\"3\" v2=\"0\" v3=\"4\"/><triangle v1=\"3\" v2=\"4\" v3=\"7\"/>\n</triangles></mesh></object></resources><build/>\n</model>";
#[rustfmt::skip]
const MODIFIER_LEAF: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<model unit=\"millimeter\" xmlns=\"http://schemas.microsoft.com/3dmanufacturing/core/2015/02\">\n <resources><object id=\"3\" type=\"model\"><mesh><vertices>\n<vertex x=\"5\" y=\"0\" z=\"0\"/><vertex x=\"15\" y=\"0\" z=\"0\"/><vertex x=\"15\" y=\"2\" z=\"0\"/><vertex x=\"5\" y=\"2\" z=\"0\"/>\n<vertex x=\"5\" y=\"0\" z=\"0.4\"/><vertex x=\"15\" y=\"0\" z=\"0.4\"/><vertex x=\"15\" y=\"2\" z=\"0.4\"/><vertex x=\"5\" y=\"2\" z=\"0.4\"/>\n</vertices><triangles>\n<triangle v1=\"0\" v2=\"2\" v3=\"1\"/><triangle v1=\"0\" v2=\"3\" v3=\"2\"/>\n<triangle v1=\"4\" v2=\"5\" v3=\"6\"/><triangle v1=\"4\" v2=\"6\" v3=\"7\"/>\n<triangle v1=\"0\" v2=\"1\" v3=\"5\"/><triangle v1=\"0\" v2=\"5\" v3=\"4\"/>\n<triangle v1=\"1\" v2=\"2\" v3=\"6\"/><triangle v1=\"1\" v2=\"6\" v3=\"5\"/>\n<triangle v1=\"2\" v2=\"3\" v3=\"7\"/><triangle v1=\"2\" v2=\"7\" v3=\"6\"/>\n<triangle v1=\"3\" v2=\"0\" v3=\"4\"/><triangle v1=\"3\" v2=\"4\" v3=\"7\"/>\n</triangles></mesh></object></resources><build/>\n</model>";
#[rustfmt::skip]
const MODIFIER_SETTINGS: &str = "<config><object id=\"2\"><part id=\"1\" subtype=\"normal_part\"/><part id=\"3\" subtype=\"modifier_part\"><metadata key=\"bridge_angle\" value=\"37\"/></part></object><plate><metadata key=\"plater_id\" value=\"1\"/><model_instance><metadata key=\"object_id\" value=\"2\"/><metadata key=\"instance_id\" value=\"0\"/><metadata key=\"identify_id\" value=\"22001\"/></model_instance></plate><assemble><assemble_item object_id=\"2\" instance_id=\"0\" transform=\"1 0 0 0 1 0 0 0 1 0 0 0\" offset=\"0 0 0\"/></assemble></config>";
#[rustfmt::skip]
const CONTROL_SETTINGS: &str = "<config><object id=\"2\"><part id=\"1\" subtype=\"normal_part\"/><part id=\"3\" subtype=\"modifier_part\"/></object><plate><metadata key=\"plater_id\" value=\"1\"/><model_instance><metadata key=\"object_id\" value=\"2\"/><metadata key=\"instance_id\" value=\"0\"/><metadata key=\"identify_id\" value=\"22001\"/></model_instance></plate><assemble><assemble_item object_id=\"2\" instance_id=\"0\" transform=\"1 0 0 0 1 0 0 0 1 0 0 0\" offset=\"0 0 0\"/></assemble></config>";
