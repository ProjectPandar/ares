use crate::load_project;

use super::fixture::ProjectParts;

#[test]
fn task22a_painted_layer_height_profile_presence_is_case_insensitive_and_opaque() {
    for (path, payload) in [
        ("Metadata/layer_heights_profile.txt", ""),
        (
            "metadata/LAYER_HEIGHTS_PROFILE.TXT",
            "not a decoded height profile",
        ),
    ] {
        let mut parts = ProjectParts::valid();
        parts.insert_text(path, payload);

        assert!(
            load_project(parts.bytes())
                .unwrap()
                .has_painted_layer_height_profile()
        );
    }

    let mut parts = ProjectParts::valid();
    parts.insert_text("Metadata/layer_heights_profile.txt", "");
    parts.insert_text("metadata/LAYER_HEIGHTS_PROFILE.TXT", "opaque duplicate");
    assert!(
        load_project(parts.bytes())
            .unwrap()
            .has_painted_layer_height_profile()
    );
}

#[test]
fn task22a_painted_layer_height_profile_absence_is_false() {
    assert!(
        !load_project(ProjectParts::valid().bytes())
            .unwrap()
            .has_painted_layer_height_profile()
    );
}
