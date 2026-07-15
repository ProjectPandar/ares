use crate::{OrcaInt, load_project};

use super::fixture::ProjectParts;

const SUB1: &str = r##"<model xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02" xmlns:m="http://schemas.microsoft.com/3dmanufacturing/material/2015/02" requiredextensions="m"><resources>
 <m:colorgroup id="2"><m:color color="#A"/><m:color color="#B"/></m:colorgroup>
 <m:colorgroup id="10"><m:color color="#C"/></m:colorgroup>
 <object id="1" type="model"><mesh><vertices><vertex x="0" y="0" z="0"/><vertex x="1" y="0" z="0"/><vertex x="0" y="1" z="0"/></vertices><triangles><triangle v1="0" v2="1" v3="2"/></triangles></mesh></object>
 </resources><build/></model>"##;

const SUB2: &str = r##"<model xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02" xmlns:m="http://schemas.microsoft.com/3dmanufacturing/material/2015/02" requiredextensions="m"><resources>
 <m:colorgroup id="2"><m:color color="#D"/></m:colorgroup>
 <m:colorgroup id="3"><m:color color="#A"/><m:color color="#B"/></m:colorgroup>
 <m:colorgroup id="10"><m:color color="#F"/></m:colorgroup>
 </resources><build/></model>"##;

fn root_object(id: u32, pid: Option<&str>, pindex: Option<&str>) -> String {
    let pid = pid.map_or_else(String::new, |pid| format!(r#" pid="{pid}""#));
    let pindex = pindex.map_or_else(String::new, |value| format!(r#" pindex="{value}""#));
    format!(
        r#"<object id="{id}" name="object-{id}"{pid}{pindex} type="model"><components><component p:path="/3D/sub1.model" objectid="1"/></components></object>"#
    )
}

fn color_project(settings_ids: &[u32]) -> ProjectParts {
    let definitions = [
        root_object(10, Some("2"), None),
        root_object(11, Some("3"), Some("999")),
        root_object(12, Some("5"), None),
        root_object(13, Some("10"), None),
        root_object(14, Some("6"), None),
        root_object(15, Some("oops"), None),
        root_object(16, None, None),
        root_object(17, Some("0"), None),
        root_object(18, Some("999"), None),
    ]
    .concat();
    let build = (10..=18)
        .map(|id| format!(r#"<item objectid="{id}"/>"#))
        .collect::<String>();
    let root = format!(
        r##"<model xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02" xmlns:p="http://schemas.microsoft.com/3dmanufacturing/production/2015/06" xmlns:m="http://schemas.microsoft.com/3dmanufacturing/material/2015/02" requiredextensions="p m"><resources>
        <m:colorgroup id="2"><m:color color="#old"/><m:color color="#E"/></m:colorgroup>
        <m:colorgroup id="5"><m:color color="#C"/></m:colorgroup>
        <m:colorgroup id="6"><m:color color="#c"/></m:colorgroup>
        <m:colorgroup id="10"/>{definitions}</resources><build>{build}</build></model>"##
    );
    let relationships = r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Target="/3D/sub1.model" Id="s1" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/><Relationship Target="/3D/sub2.model" Id="s2" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/></Relationships>"#;
    let mut parts = ProjectParts::valid();
    parts.insert_text("3D/root.model", &root);
    parts.insert_text("3D/_rels/root.model.rels", relationships);
    parts.insert_text("3D/sub1.model", SUB1);
    parts.insert_text("3D/sub2.model", SUB2);
    parts.remove("3D/leaf.model");
    let objects = settings_ids
        .iter()
        .map(|id| format!(r#"<object id="{id}"/>"#))
        .collect::<String>();
    parts.set_model_settings_objects(&objects, &(10..=18).collect::<Vec<_>>());
    parts
}

fn extruder(project: &crate::Project, index: usize) -> Option<OrcaInt> {
    project.objects()[index].region_overrides().extruder
}

#[test]
fn no_settings_color_map_is_deterministic_and_source_faithful() {
    let project = load_project(color_project(&[]).bytes()).unwrap();

    assert_eq!(extruder(&project, 0), Some(OrcaInt(1))); // root replaces submodel group 2
    assert_eq!(extruder(&project, 1), Some(OrcaInt(2))); // last group-3 color; pindex ignored
    assert_eq!(extruder(&project, 2), Some(OrcaInt(3)));
    assert_eq!(extruder(&project, 3), Some(OrcaInt(3))); // first submodel group 10 retained; exact dedup
    assert_eq!(extruder(&project, 4), Some(OrcaInt(4))); // exact strings remain case-sensitive
    assert_eq!(extruder(&project, 5), None); // invalid pid becomes zero
    assert_eq!(extruder(&project, 6), None); // missing pid becomes zero
    assert_eq!(extruder(&project, 7), None); // explicit zero
    assert_eq!(extruder(&project, 8), None); // unmapped group
}

#[test]
fn any_matching_settings_record_suppresses_color_fallback_without_extruder() {
    let project = load_project(color_project(&[10]).bytes()).unwrap();

    assert_eq!(extruder(&project, 0), None);
    assert_eq!(project.objects()[0].name(), "");
    assert_eq!(extruder(&project, 1), Some(OrcaInt(2)));
}
