#[path = "cli_substitution_routes/support.rs"]
mod support;

use support::check;

const SLICE_ONE: &[&str] = &["--slice", "1", "--outputdir", "out"];
const SLICE_TWO: &[&str] = &["--slice", "2", "--outputdir", "out"];
const SLICE_THREE: &[&str] = &["--slice", "3", "--outputdir", "out"];
const SLICE_ALL: &[&str] = &["--slice", "0", "--outputdir", "out"];
const SUBCOMMAND_ONE: &[&str] = &["slice", "--plate", "1", "-o", "out/plate_1.gcode"];
const EXPORT_SLICED: &[&str] = &["--slice", "1", "--export-3mf", "exports/project.3mf"];
const EXPORT_UNSLICED: &[&str] = &["--export-3mf", "exports/project.3mf"];
const SETTINGS_FILE: &[&str] = &["--export-settings", "exports/settings.json"];
const SETTINGS_STDOUT: &[&str] = &["--export-settings", "-"];
const SETTINGS_MISSING_PRESET: &[&str] = &[
    "--export-settings",
    "exports/settings.json",
    "--load-settings",
    "missing-machine.json;missing-process.json",
];
#[derive(Clone, Copy)]
enum Saved {
    One,
    Multi,
    Invalid,
    NonprintableSecond,
    Newer,
    Legacy,
}

#[derive(Clone, Copy)]
enum Outcome {
    Gcode(&'static [&'static str]),
    Archive,
    SettingsFile,
    SettingsStdout,
    Fail(i32),
    BlockedArchive,
    MissingPreset,
}

#[test]
fn direct_one_success() {
    check(
        "direct-one",
        Saved::One,
        SLICE_ONE,
        SLICE_ONE,
        Outcome::Gcode(&["plate_1.gcode"]),
    );
}

#[test]
fn direct_one_absolute_spelling_is_preserved() {
    check(
        "direct-absolute",
        Saved::One,
        SLICE_ONE,
        SLICE_ONE,
        Outcome::Gcode(&["plate_1.gcode"]),
    );
}

#[test]
fn direct_one_validation_failure() {
    check(
        "direct-invalid",
        Saved::Invalid,
        SLICE_ONE,
        SLICE_ONE,
        Outcome::Fail(205),
    );
}

#[test]
fn direct_selected_success() {
    check(
        "direct-selected",
        Saved::Multi,
        SLICE_TWO,
        SLICE_TWO,
        Outcome::Gcode(&["plate_2.gcode"]),
    );
}

#[test]
fn direct_selected_missing_plate() {
    check(
        "direct-missing-plate",
        Saved::Multi,
        SLICE_THREE,
        SLICE_THREE,
        Outcome::Fail(254),
    );
}

#[test]
fn direct_all_success() {
    check(
        "direct-all",
        Saved::Multi,
        SLICE_ALL,
        SLICE_ALL,
        Outcome::Gcode(&["plate_1.gcode", "plate_2.gcode"]),
    );
}

#[test]
fn direct_all_later_nonprintable_plate() {
    check(
        "direct-all-invalid",
        Saved::NonprintableSecond,
        SLICE_ALL,
        SLICE_ALL,
        Outcome::Fail(206),
    );
}

#[test]
fn sliced_export_success() {
    check(
        "export-sliced",
        Saved::One,
        EXPORT_SLICED,
        EXPORT_SLICED,
        Outcome::Archive,
    );
}

#[test]
fn sliced_export_write_failure() {
    check(
        "export-sliced-blocked",
        Saved::One,
        EXPORT_SLICED,
        EXPORT_SLICED,
        Outcome::BlockedArchive,
    );
}

#[test]
fn unsliced_export_success() {
    check(
        "export-unsliced",
        Saved::One,
        EXPORT_UNSLICED,
        EXPORT_UNSLICED,
        Outcome::Archive,
    );
}

#[test]
fn unsliced_export_write_failure() {
    check(
        "export-unsliced-blocked",
        Saved::One,
        EXPORT_UNSLICED,
        EXPORT_UNSLICED,
        Outcome::BlockedArchive,
    );
}

#[test]
fn settings_export_file_success() {
    check(
        "settings-file",
        Saved::One,
        SETTINGS_FILE,
        SETTINGS_FILE,
        Outcome::SettingsFile,
    );
}

#[test]
fn settings_export_stdout_success() {
    check(
        "settings-stdout",
        Saved::One,
        SETTINGS_STDOUT,
        SETTINGS_STDOUT,
        Outcome::SettingsStdout,
    );
}

#[test]
fn settings_export_missing_preset() {
    check(
        "settings-preset-failure",
        Saved::One,
        SETTINGS_MISSING_PRESET,
        SETTINGS_MISSING_PRESET,
        Outcome::MissingPreset,
    );
}

#[test]
fn subcommand_one_success() {
    check(
        "subcommand-one",
        Saved::One,
        SLICE_ONE,
        SUBCOMMAND_ONE,
        Outcome::Gcode(&["plate_1.gcode"]),
    );
}

#[test]
fn subcommand_one_validation_failure() {
    check(
        "subcommand-invalid",
        Saved::Invalid,
        SLICE_ONE,
        SUBCOMMAND_ONE,
        Outcome::Fail(205),
    );
}

#[test]
fn subcommand_batch_selected_success() {
    check(
        "subcommand-selected",
        Saved::Multi,
        SLICE_TWO,
        &[&["slice"], SLICE_TWO].concat(),
        Outcome::Gcode(&["plate_2.gcode"]),
    );
}

#[test]
fn subcommand_batch_all_success() {
    check(
        "subcommand-all",
        Saved::Multi,
        SLICE_ALL,
        &[&["slice"], SLICE_ALL].concat(),
        Outcome::Gcode(&["plate_1.gcode", "plate_2.gcode"]),
    );
}

#[test]
fn subcommand_batch_missing_plate() {
    check(
        "subcommand-missing-plate",
        Saved::Multi,
        SLICE_THREE,
        &[&["slice"], SLICE_THREE].concat(),
        Outcome::Fail(254),
    );
}

#[test]
fn newer_file_rejected_before_reporting() {
    check(
        "newer-project",
        Saved::Newer,
        SLICE_ONE,
        SLICE_ONE,
        Outcome::Fail(232),
    );
}

#[test]
fn translated_enum_is_not_reported_as_substitution() {
    check(
        "legacy-enum",
        Saved::Legacy,
        SLICE_ONE,
        SLICE_ONE,
        Outcome::Gcode(&["plate_1.gcode"]),
    );
}
