use super::support::{KsrArchive, ksr_project, metadata};

const MODEL_SETTINGS: &str = "Metadata/model_settings.config";
const MODEL: &str = "3D/3dmodel.model";
const FILAMENTS: &str = "Metadata/filament_sequence.json";

/// Splits the KSR reference project into two plates: plate 1 keeps the
/// original instance; plate 2 receives a duplicated object placed 60 mm
/// along +X.
fn two_plate_archive() -> KsrArchive {
    let mut archive = KsrArchive::new();

    let model_settings = archive.entry_text(MODEL_SETTINGS);
    let object_start = model_settings.find("<object ").unwrap();
    let object_end = model_settings.find("</object>").unwrap() + "</object>".len();
    let second_object = model_settings[object_start..object_end]
        .replace("id=\"2\"", "id=\"3\"")
        .replace("ksr_fdmtest_v4.drc", "plate_two.drc");
    let plate_two = concat!(
        "  <plate>\n",
        "    <metadata key=\"plater_id\" value=\"2\"/>\n",
        "    <metadata key=\"plater_name\" value=\"\"/>\n",
        "    <metadata key=\"locked\" value=\"false\"/>\n",
        "    <metadata key=\"filament_map_mode\" value=\"Auto For Flush\"/>\n",
        "    <model_instance>\n",
        "      <metadata key=\"object_id\" value=\"3\"/>\n",
        "      <metadata key=\"instance_id\" value=\"0\"/>\n",
        "      <metadata key=\"identify_id\" value=\"211\"/>\n",
        "    </model_instance>\n",
        "  </plate>\n",
        "  <plate>",
    );
    archive.insert_text(
        MODEL_SETTINGS,
        format!(
            "{}\n  {}{}",
            &model_settings[..object_start],
            second_object,
            model_settings[object_start..].replace("  <plate>", plate_two),
        )
        .as_str(),
    );

    let model = archive.entry_text(MODEL);
    let (resources, build) = model.split_once("<build").unwrap();
    let object_start = resources.find("<object id=\"2\"").unwrap();
    let object_end = resources.rfind("</object>").unwrap() + "</object>".len();
    let second_object = resources[object_start..object_end]
        .replace("id=\"2\"", "id=\"3\"")
        .replace(
            "00000002-61cb-4c03-9d28-80fed5dfa1dc",
            "00000003-61cb-4c03-9d28-80fed5dfa1dc",
        );
    let item_two = "<item objectid=\"3\" transform=\"1 0 0 0 1 0 0 0 1 193.039205 115.992105 46\" printable=\"1\" auto_drop=\"1\"/>";
    archive.insert_text(
        MODEL,
        format!(
            "{}{}\n  {}<build{}",
            &resources[..object_start],
            second_object,
            &resources[object_start..],
            build.replace(" </build>", &format!("  {}\n </build>", item_two)),
        )
        .as_str(),
    );

    archive.insert_text(
        FILAMENTS,
        "{\"plate_1\":{\"nozzle_sequence\":[],\"optimal_assignment\":[],\"sequence\":[]},\"plate_2\":{\"nozzle_sequence\":[],\"optimal_assignment\":[],\"sequence\":[]}}",
    );
    archive
}

#[tokio::test]
async fn slice_project_slices_only_the_first_plate_by_default() {
    let output = crate::slice_project(two_plate_archive().bytes(), metadata())
        .await
        .unwrap();
    let text = String::from_utf8(output).unwrap();
    assert!(
        text.contains("ksr_fdmtest_v4.drc"),
        "plate 1 keeps the original object"
    );
    assert!(
        !text.contains("plate_two.drc"),
        "plate 2 object must not appear on plate 1"
    );
}

#[tokio::test]
async fn slice_project_plate_selects_instances_of_the_requested_plate() {
    let bytes = two_plate_archive().bytes();
    let plate_two = crate::slice_project_plate(bytes.as_slice(), metadata(), Some(2))
        .await
        .unwrap();
    let text = String::from_utf8(plate_two).unwrap();
    assert!(
        text.contains("printing object plate_two.drc"),
        "plate 2 slices its own object"
    );
    assert!(
        !text.contains("printing object ksr_fdmtest_v4.drc"),
        "plate 1 object must not leak into plate 2"
    );
    let plate_one = crate::slice_project_plate(bytes.as_slice(), metadata(), Some(1))
        .await
        .unwrap();
    assert!(
        String::from_utf8(plate_one)
            .unwrap()
            .contains("ksr_fdmtest_v4.drc")
    );
}

#[tokio::test]
async fn slice_project_plate_rejects_an_unknown_plate() {
    let error = crate::slice_project_plate(two_plate_archive().bytes(), metadata(), Some(7))
        .await
        .unwrap_err();
    assert_eq!(
        error.to_string(),
        crate::SliceError::InvalidInput("project has no plate 7".to_owned()).to_string()
    );
}

#[test]
fn single_plate_project_loads_all_its_instances() {
    // The reference project has one plate; selection must keep every instance.
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let output = runtime
        .block_on(crate::slice_project(ksr_project(), metadata()))
        .unwrap();
    assert!(
        String::from_utf8(output)
            .unwrap()
            .contains("; total layer number: 460")
    );
}
