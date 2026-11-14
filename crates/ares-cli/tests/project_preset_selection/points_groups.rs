use serde_json::json;

use super::{input, support::Case};

// Two-nozzle printers with one printable area per extruder (Bambu Lab H2D and X2D
// machine profiles): the project stores one group string per extruder.
const H2D: [&str; 2] = ["0x0,325x0,325x320,0x320", "25x0,350x0,350x320,25x320"];
const X2D: [&str; 2] = ["0x0,256x0,256x256,0x256", "20.5x0,256x0,256x256,20.5x256"];

#[tokio::test]
async fn h2d_extruder_printable_areas_survive_the_always_selection() {
    extruder_printable_area("selection-points-groups-h2d", H2D).await;
}

#[tokio::test]
async fn x2d_extruder_printable_areas_survive_the_always_selection() {
    extruder_printable_area("selection-points-groups-x2d", X2D).await;
}

async fn extruder_printable_area(label: &str, groups: [&str; 2]) {
    let mut settings = input::settings("Missing Process", "0.2");
    settings["nozzle_diameter"] = json!(["0.4", "0.4"]);
    settings["extruder_printable_area"] = json!(groups);
    let Some(case) = Case::with_catalog(label, settings, input::catalog(&[], &[])) else {
        return;
    };
    // Native premise: the project-local printer keeps one group per extruder
    // (Config.cpp:1043-1044 joins the JSON groups with '#'; Config.hpp:1725-1735
    // serializes them the same way).
    let native = &case.native["projects"][0]["result"]["printer"];
    assert_eq!(native["selected"]["name"], "Missing Printer(selection.3mf)");
    let area = &native["edited"]["config"]["extruder_printable_area"];
    assert_eq!(area["type"], 16_394);
    assert_eq!(area["serialized"], groups.join("#"));
    case.write(
        "native-premise.json",
        &json!({"passed": true, "extruder_printable_area": area["serialized"]}),
    );
    let view = case
        .api()
        .await
        .expect("valid Always request must load the project");
    // One string per group, as ConfigOptionPointsGroups::vserialize (Config.hpp:1737-1746) lists them.
    let native_groups = area["serialized"]
        .as_str()
        .unwrap()
        .split('#')
        .collect::<Vec<_>>();
    assert_eq!(
        view["settings"]["extruder_printable_area"],
        json!(native_groups)
    );
}
