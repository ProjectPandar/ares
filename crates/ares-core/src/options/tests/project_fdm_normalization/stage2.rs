use crate::options::{
    OrcaBool, OrcaString, ProcessPrintSequence, ProcessTimelapseType, ProjectSettings,
    project_fdm_normalization::{ProjectFdmNormalizationKey, normalize_fdm_2},
};

use ProjectFdmNormalizationKey::{EnablePrimeTower, IndependentSupportLayerHeight};

#[test]
fn zero_used_filaments_is_an_exact_full_struct_noop() {
    for (num_objects, print_sequence, timelapse_type, wrapping) in [
        (
            0,
            ProcessPrintSequence::ByLayer,
            ProcessTimelapseType::Traditional,
            false,
        ),
        (
            1,
            ProcessPrintSequence::ByObject,
            ProcessTimelapseType::Traditional,
            false,
        ),
        (
            2,
            ProcessPrintSequence::ByObject,
            ProcessTimelapseType::Smooth,
            true,
        ),
    ] {
        let mut settings = settings_with_controls(
            true,
            true,
            print_sequence,
            timelapse_type,
            wrapping,
        );
        let original = settings.clone();

        let changed = normalize_fdm_2(&mut settings, num_objects, 0);

        assert_eq!(changed, []);
        assert_eq!(settings, original);
    }
}

#[test]
fn one_used_filament_disables_a_true_tower_only() {
    let mut settings = settings_with_controls(
        true,
        true,
        ProcessPrintSequence::ByLayer,
        ProcessTimelapseType::Traditional,
        false,
    );
    let original = settings.clone();

    let changed = normalize_fdm_2(&mut settings, 1, 1);

    assert_eq!(changed, [EnablePrimeTower]);
    assert_eq!(settings.process.print.enable_prime_tower, OrcaBool(false));
    assert_eq!(
        settings.process.print.independent_support_layer_height,
        OrcaBool(true)
    );
    assert_only_stage2_write_set_changed(&original, &settings);
}

#[test]
fn many_filaments_by_layer_preserves_tower_and_disables_independent_support() {
    let mut settings = settings_with_controls(
        true,
        true,
        ProcessPrintSequence::ByLayer,
        ProcessTimelapseType::Traditional,
        false,
    );
    let original = settings.clone();

    let changed = normalize_fdm_2(&mut settings, 8, 3);

    assert_eq!(changed, [IndependentSupportLayerHeight]);
    assert_eq!(settings.process.print.enable_prime_tower, OrcaBool(true));
    assert_eq!(
        settings.process.print.independent_support_layer_height,
        OrcaBool(false)
    );
    assert_only_stage2_write_set_changed(&original, &settings);
}

#[test]
fn many_filaments_by_object_disables_tower_only_for_multiple_objects() {
    for (num_objects, expected_key, expected_tower, expected_support) in [
        (
            1,
            IndependentSupportLayerHeight,
            OrcaBool(true),
            OrcaBool(false),
        ),
        (
            2,
            EnablePrimeTower,
            OrcaBool(false),
            OrcaBool(true),
        ),
    ] {
        let mut settings = settings_with_controls(
            true,
            true,
            ProcessPrintSequence::ByObject,
            ProcessTimelapseType::Traditional,
            false,
        );
        let original = settings.clone();

        let changed = normalize_fdm_2(&mut settings, num_objects, 4);

        assert_eq!(changed, [expected_key]);
        assert_eq!(settings.process.print.enable_prime_tower, expected_tower);
        assert_eq!(
            settings.process.print.independent_support_layer_height,
            expected_support
        );
        assert_only_stage2_write_set_changed(&original, &settings);
    }
}

#[test]
fn smooth_timelapse_and_wrapping_each_prevent_tower_disabling() {
    for (timelapse_type, wrapping) in [
        (ProcessTimelapseType::Smooth, false),
        (ProcessTimelapseType::Traditional, true),
        (ProcessTimelapseType::Smooth, true),
    ] {
        let mut settings = settings_with_controls(
            true,
            true,
            ProcessPrintSequence::ByObject,
            timelapse_type,
            wrapping,
        );
        let original = settings.clone();

        let changed = normalize_fdm_2(&mut settings, 3, 1);

        assert_eq!(changed, [IndependentSupportLayerHeight]);
        assert_eq!(settings.process.print.enable_prime_tower, OrcaBool(true));
        assert_eq!(
            settings.process.print.independent_support_layer_height,
            OrcaBool(false)
        );
        assert_only_stage2_write_set_changed(&original, &settings);
    }
}

#[test]
fn already_false_values_are_never_reenabled_or_reported() {
    let mut tower_disabled = settings_with_controls(
        false,
        true,
        ProcessPrintSequence::ByObject,
        ProcessTimelapseType::Traditional,
        false,
    );
    let tower_disabled_original = tower_disabled.clone();

    let changed = normalize_fdm_2(&mut tower_disabled, 3, 1);

    assert_eq!(changed, []);
    assert_eq!(tower_disabled, tower_disabled_original);

    let mut support_disabled = settings_with_controls(
        true,
        false,
        ProcessPrintSequence::ByLayer,
        ProcessTimelapseType::Traditional,
        false,
    );
    let support_disabled_original = support_disabled.clone();

    let changed = normalize_fdm_2(&mut support_disabled, 3, 2);

    assert_eq!(changed, []);
    assert_eq!(support_disabled, support_disabled_original);

    let mut both_disabled = settings_with_controls(
        false,
        false,
        ProcessPrintSequence::ByLayer,
        ProcessTimelapseType::Traditional,
        false,
    );
    let both_disabled_original = both_disabled.clone();

    let changed = normalize_fdm_2(&mut both_disabled, 3, 2);

    assert_eq!(changed, []);
    assert_eq!(both_disabled, both_disabled_original);
}

#[test]
fn changed_keys_have_exact_observable_names_and_no_third_variant() {
    let keys = [EnablePrimeTower, IndependentSupportLayerHeight];

    assert_eq!(keys[0].as_ref(), "enable_prime_tower");
    assert_eq!(keys[1].as_ref(), "independent_support_layer_height");
    assert_eq!(
        keys.map(|key| key.to_string()),
        ["enable_prime_tower", "independent_support_layer_height"]
    );

    for key in keys {
        match key {
            EnablePrimeTower | IndependentSupportLayerHeight => {}
        }
    }
}

fn settings_with_controls(
    enable_prime_tower: bool,
    independent_support_layer_height: bool,
    print_sequence: ProcessPrintSequence,
    timelapse_type: ProcessTimelapseType,
    enable_wrapping_detection: bool,
) -> ProjectSettings {
    let mut settings = ProjectSettings::default();
    settings.process.print.enable_prime_tower = OrcaBool(enable_prime_tower);
    settings.process.print.independent_support_layer_height =
        OrcaBool(independent_support_layer_height);
    settings.process.print.print_sequence = print_sequence;
    settings.process.print.timelapse_type = timelapse_type;
    settings.process.gcode.enable_wrapping_detection = OrcaBool(enable_wrapping_detection);
    settings.process.print.notes = OrcaString("stage-two-sentinel".to_owned());
    settings.metadata.name = "stage-two-metadata".to_owned();
    settings
}

fn assert_only_stage2_write_set_changed(original: &ProjectSettings, actual: &ProjectSettings) {
    let mut restored = actual.clone();
    restored.process.print.enable_prime_tower = original.process.print.enable_prime_tower;
    restored.process.print.independent_support_layer_height =
        original.process.print.independent_support_layer_height;

    assert_eq!(&restored, original);
}
