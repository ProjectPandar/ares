use ares_core::{
    ComposedProfile, FilamentOptions, MergedProfile, MergedProfileMetadata, PrinterOptions,
    ProcessOptions, ProfileFragment, ProfileGroupMetadata, ProfileKind, ProfileSelection,
    ProjectSettings, compose_profile_fragments, merge_profile_fragments,
};

#[test]
fn public_profile_api_is_externally_usable() {
    let fragments = [
        br#"{"type":"machine","name":"m","printer_model":"M"}"#.as_slice(),
        br#"{"type":"process","name":"p","layer_height":0.2,"compatible_printers":["m",""],"compatible_printers_condition":"process-machine"}"#,
        br#"{"type":"filament","name":"base-f","filament_id":"BASE-ID"}"#,
        br#"{"type":"filament","name":"f","inherits":"base-f","from":"user","version":"01.002.000.00","setting_id":"F-SETTING","instantiation":"true","description":"public filament","url":"https://example.invalid/f","renamed_from":"legacy-f","filament_id":"CHILD-ID","compatible_printers":["m"],"compatible_printers_condition":"filament-machine","compatible_prints":["p"],"compatible_prints_condition":"filament-process","filament_diameter":[1.75]}"#,
    ]
    .into_iter()
    .map(ProfileFragment::from_json_bytes)
    .collect::<Result<Vec<_>, _>>()
    .unwrap();

    let fragment = &fragments[3];
    assert_eq!(fragment.kind(), ProfileKind::Filament);
    assert_eq!(fragment.name(), "f");
    assert_eq!(fragment.inherits(), Some("base-f"));
    assert_eq!(fragment.from(), Some("user"));
    assert_eq!(fragment.version(), Some("01.002.000.00"));
    assert_eq!(fragment.setting_id(), Some("F-SETTING"));
    assert_eq!(fragment.instantiation(), Some("true"));
    assert_eq!(fragment.description(), Some("public filament"));
    assert_eq!(fragment.url(), Some("https://example.invalid/f"));
    assert_eq!(fragment.renamed_from(), Some("legacy-f"));
    assert_eq!(fragment.filament_id(), Some("CHILD-ID"));

    for (kind, name) in [
        (ProfileKind::Machine, "m"),
        (ProfileKind::Process, "p"),
        (ProfileKind::Filament, "f"),
    ] {
        let merged = merge_profile_fragments(&fragments, kind, name).unwrap();
        assert_eq!(inspect_merged_profile(merged), kind);
    }

    let selection = ProfileSelection::new("p", "m", ["f"]).unwrap();
    assert_eq!(selection.process(), "p");
    assert_eq!(selection.machine(), "m");
    assert_eq!(selection.filaments(), ["f"]);
    let composed: ComposedProfile = compose_profile_fragments(&fragments, &selection).unwrap();
    let settings: &ProjectSettings = composed.settings();
    let metadata: &ProfileGroupMetadata = composed.metadata();

    assert_eq!(composed.process_name(), "p");
    assert_eq!(composed.machine_name(), "m");
    assert_eq!(composed.filament_names(), ["f"]);
    assert_eq!(settings.process.object.layer_height.0, 0.2);
    assert_eq!(metadata.inherits_group().unwrap(), ["", "base-f", ""]);
    assert_eq!(
        metadata.compatible_machine_expression_group().unwrap(),
        ["process-machine", "filament-machine"]
    );
    assert_eq!(
        metadata.compatible_process_expression_group().unwrap(),
        ["filament-process"]
    );
    let settings: ProjectSettings = composed.into_settings();
    assert_eq!(settings.project.preset.print_settings_id.0, "p");
}

fn inspect_merged_profile(merged: MergedProfile) -> ProfileKind {
    match merged {
        MergedProfile::Machine { metadata, options } => {
            let metadata: &MergedProfileMetadata = &metadata;
            let options: &PrinterOptions = &options;
            assert_eq!(metadata.name(), "m");
            assert_eq!(options.remaining.printer_model.0, "M");
            ProfileKind::Machine
        }
        MergedProfile::Process { metadata, options } => {
            let metadata: &MergedProfileMetadata = &metadata;
            let options: &ProcessOptions = &options;
            assert_eq!(metadata.name(), "p");
            assert_eq!(options.object.layer_height.0, 0.2);
            ProfileKind::Process
        }
        MergedProfile::Filament { metadata, options } => {
            let metadata: &MergedProfileMetadata = &metadata;
            let options: &FilamentOptions = &options;
            assert_eq!(metadata.name(), "f");
            assert_eq!(metadata.inherits(), Some("base-f"));
            assert_eq!(metadata.from(), Some("user"));
            assert_eq!(metadata.version(), Some("01.002.000.00"));
            assert_eq!(metadata.setting_id(), Some("F-SETTING"));
            assert_eq!(metadata.instantiation(), Some("true"));
            assert_eq!(metadata.description(), Some("public filament"));
            assert_eq!(metadata.url(), Some("https://example.invalid/f"));
            assert_eq!(metadata.renamed_from(), Some("legacy-f"));
            assert_eq!(metadata.filament_id(), Some("BASE-ID"));
            assert_eq!(metadata.compatible_printers().unwrap(), ["m"]);
            assert_eq!(
                metadata.compatible_printers_condition(),
                Some("filament-machine")
            );
            assert_eq!(metadata.compatible_prints().unwrap(), ["p"]);
            assert_eq!(
                metadata.compatible_prints_condition(),
                Some("filament-process")
            );
            assert_eq!(options.gcode.filament_diameter.0[0].0, 1.75);
            ProfileKind::Filament
        }
    }
}
