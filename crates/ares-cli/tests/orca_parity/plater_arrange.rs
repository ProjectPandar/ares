//! "Arrange all" on a plater session (`PlaterSession::arrange(All, settings)`, the GUI
//! `ArrangeJob` at v2.5.0-beta `345efec64`) places the instances like the official v2.5.0-beta
//! CLI global arrange (`OrcaSlicer.cpp:5619-6098`): the build items in order with their
//! transforms as `_add_build_to_model_stream` writes them (9 significant digits,
//! `bbs_3mf.cpp:7150-7151`) and the plate map. Each input is a committed fixture with literal
//! edits; the cases stay on the path the CLI and the GUI job share (the A3 slice plan in
//! `docs/architecture/web-plater-move-rotate-arrange.md`). The Native oracle must be the
//! official v2.5.0-beta CLI; a missing oracle fails the test.

use ares_core::{ArrangeScope, ArrangeSettings, PlaterSession};

use super::plater_archive::rewrite_archive;
use crate::runner::{OrcaRunner, repo_root, stages};

#[path = "plater_arrange/arranged.rs"]
mod arranged;
#[path = "plater_arrange/bed_exclusion.rs"]
mod bed_exclusion;
#[path = "plater_arrange/current_plate.rs"]
mod current_plate;
#[path = "plater_arrange/wipe_tower.rs"]
mod wipe_tower;
use arranged::Arranged;

const SINGLE: &str = "tests/parity/orca_cli_ender3.3mf";
/// Two 10 mm cubes on an Ender-3, the first with two instances; combined skirt, by object.
const CUBES: &str = "tests/parity/byobject-perobject-multi-copy-brim/combined-brim.3mf";
const MODEL: &str = "3D/3dmodel.model";
const PROJECT: &str = "Metadata/project_settings.config";
const OBJECTS: &str = "Metadata/model_settings.config";

/// `(archive entry, text, replacement)`.
type Edit = (&'static str, &'static str, &'static str);

/// Names the two cube objects apart; both load as `cube10.stl`.
const NAMED: [Edit; 2] = [
    (
        OBJECTS,
        "<object id=\"2\">\n    <metadata key=\"name\" value=\"cube10.stl\"/>",
        "<object id=\"2\">\n    <metadata key=\"name\" value=\"first-cube\"/>",
    ),
    (
        OBJECTS,
        "<object id=\"3\">\n    <metadata key=\"name\" value=\"cube10.stl\"/>",
        "<object id=\"3\">\n    <metadata key=\"name\" value=\"second-cube\"/>",
    ),
];
const BY_LAYER: Edit = (
    PROJECT,
    "\"print_sequence\": \"by object\"",
    "\"print_sequence\": \"by layer\"",
);
/// The first instance turned 30° about Z, the second stretched to 30 x 10 mm.
const TURNED_AND_LONG: [Edit; 2] = [
    (
        MODEL,
        "transform=\"1 0 0 0 1 0 0 0 1 50 110 2\"",
        "transform=\"0.866025404 0.5 0 -0.5 0.866025404 0 0 0 1 50 110 2\"",
    ),
    (
        MODEL,
        "transform=\"1 0 0 0 1 0 0 0 1 125 110 2\"",
        "transform=\"3 0 0 0 1 0 0 0 1 125 110 2\"",
    ),
];

const fn settings(enable_rotation: bool, align_to_y_axis: bool) -> ArrangeSettings {
    ArrangeSettings {
        distance: 0.0,
        enable_rotation,
        allow_multi_materials_on_same_plate: true,
        avoid_extrusion_cali_region: false,
        align_to_y_axis,
    }
}

fn flag(name: &str, value: bool) -> String {
    format!("--{name}={}", u8::from(value))
}

/// Native global arrange of the edited `base` and the session arrange with `settings` agree.
fn assert_arranged_like_native(label: &str, base: &str, edits: &[Edit], settings: ArrangeSettings) {
    assert_arranged_like_native_with(label, base, edits, settings, &[]);
}

/// `assert_arranged_like_native` with `cli` flags for Native that the session reads from the
/// project instead.
fn assert_arranged_like_native_with(
    label: &str,
    base: &str,
    edits: &[Edit],
    settings: ArrangeSettings,
    cli: &[&str],
) {
    assert_scope_arranged_like_native(label, (base, edits), settings, cli, ArrangeScope::All);
}

/// Native and the session arrange `scope` alike. Native arranges and exports only the plate
/// it slices (`--slice <n>`, `OrcaSlicer.cpp:5815-5960, 8399-8404`), so a current-plate
/// arrange compares that plate's instances.
fn assert_scope_arranged_like_native(
    label: &str,
    (base, edits): (&str, &[Edit]),
    settings: ArrangeSettings,
    cli: &[&str],
    scope: ArrangeScope,
) {
    let plate = match scope {
        ArrangeScope::All => None,
        ArrangeScope::CurrentPlate(plate) => Some(plate),
    };
    let runner = OrcaRunner::from_env()
        .unwrap()
        .expect("official v2.5.0-beta arrange oracle");
    let input = rewrite_archive(
        &std::fs::read(repo_root().join(base)).unwrap(),
        |name, text| {
            let mut edited = None::<String>;
            for (entry, from, to) in edits.iter().filter(|(entry, ..)| *entry == name) {
                let current = edited.as_deref().unwrap_or(text);
                assert!(current.contains(from), "{label}: {entry} has no {from:?}");
                edited = Some(current.replace(from, to));
            }
            edited
        },
    );
    let directory = runner.work.join(label);
    std::fs::create_dir_all(&directory).unwrap();
    let (input_path, output_path) = (directory.join("input.3mf"), directory.join("arranged.3mf"));
    std::fs::write(&input_path, &input).unwrap();
    // The CLI keeps the distance automatic and allows several materials on a plate
    // (`OrcaSlicer.cpp:1670, 5973`).
    assert_eq!(settings.distance, 0.0);
    assert!(settings.allow_multi_materials_on_same_plate);
    stages::run(
        runner.bin(),
        [
            "--arrange".to_owned(),
            "1".to_owned(),
            flag("allow-rotations", settings.enable_rotation),
            flag(
                "avoid-extrusion-cali-region",
                settings.avoid_extrusion_cali_region,
            ),
            flag("align-to-y-axis", settings.align_to_y_axis),
        ]
        .into_iter()
        .chain(cli.iter().map(|&arg| arg.to_owned()))
        .chain(
            plate
                .into_iter()
                .flat_map(|plate| ["--slice".to_owned(), (plate + 1).to_string()]),
        )
        .chain([
            "--export-3mf".to_owned(),
            output_path.to_str().unwrap().to_owned(),
            input_path.to_str().unwrap().to_owned(),
        ]),
        stages::Stage::Export,
        &directory,
    )
    .unwrap();
    let native = Arranged::from_native(&std::fs::read(&output_path).unwrap());
    let mut session = PlaterSession::load(input).unwrap();
    session.arrange(scope, settings).unwrap();
    assert_eq!(Arranged::from_session(&session, plate), native, "{label}");
}

#[test]
fn plater_arrange_centres_a_single_object_like_native() {
    let off_centre = (MODEL, " 110 110 5\"", " 40 60 5\"");
    assert_arranged_like_native("single", SINGLE, &[off_centre], settings(false, false));
}

#[test]
fn plater_arrange_packs_several_objects_with_and_without_rotation_like_native() {
    let edits = [
        NAMED[0],
        NAMED[1],
        BY_LAYER,
        TURNED_AND_LONG[0],
        TURNED_AND_LONG[1],
    ];
    assert_arranged_like_native("several", CUBES, &edits, settings(false, false));
    assert_arranged_like_native("several-rotated", CUBES, &edits, settings(true, false));
}

#[test]
fn plater_arrange_overflows_onto_a_new_plate_like_native() {
    let scaled = |position| -> Edit {
        match position {
            50 => (
                MODEL,
                "1 0 0 0 1 0 0 0 1 50 110 2\"",
                "8 0 0 0 8 0 0 0 1 50 110 2\"",
            ),
            125 => (
                MODEL,
                "1 0 0 0 1 0 0 0 1 125 110 2\"",
                "8 0 0 0 8 0 0 0 1 125 110 2\"",
            ),
            _ => (
                MODEL,
                "1 0 0 0 1 0 0 0 1 200 110 2\"",
                "8 0 0 0 8 0 0 0 1 200 110 2\"",
            ),
        }
    };
    // Two more 80 mm instances of the first cube: five do not fit one 220 mm plate.
    let more = (
        MODEL,
        "</build>",
        "<item objectid=\"2\" transform=\"8 0 0 0 8 0 0 0 1 60 40 2\" printable=\"1\"/><item objectid=\"2\" transform=\"8 0 0 0 8 0 0 0 1 160 40 2\" printable=\"1\"/></build>",
    );
    let edits = [
        NAMED[0],
        NAMED[1],
        BY_LAYER,
        scaled(50),
        scaled(125),
        scaled(200),
        more,
    ];
    assert_arranged_like_native("overflow", CUBES, &edits, settings(false, false));
}

#[test]
fn plater_arrange_aligns_long_sides_to_y_on_an_i3_printer_like_native() {
    let edits = [
        NAMED[0],
        NAMED[1],
        BY_LAYER,
        TURNED_AND_LONG[0],
        TURNED_AND_LONG[1],
    ];
    assert_arranged_like_native("align-to-y", CUBES, &edits, settings(false, true));
    assert_arranged_like_native("align-to-y-off", CUBES, &edits, settings(false, false));
}

/// The three instances scaled to 50 mm.
const FIFTY_MM: [Edit; 3] = [
    (
        MODEL,
        "1 0 0 0 1 0 0 0 1 50 110 2\"",
        "5 0 0 0 5 0 0 0 1 50 110 2\"",
    ),
    (
        MODEL,
        "1 0 0 0 1 0 0 0 1 125 110 2\"",
        "5 0 0 0 5 0 0 0 1 125 110 2\"",
    ),
    (
        MODEL,
        "1 0 0 0 1 0 0 0 1 200 110 2\"",
        "5 0 0 0 5 0 0 0 1 200 110 2\"",
    ),
];

#[test]
fn plater_arrange_keeps_clear_of_the_bed_exclude_area_like_native() {
    let excluded = (
        PROJECT,
        "\"bed_exclude_area\": [\n    \"0x0\"\n  ]",
        "\"bed_exclude_area\": [\n    \"0x0\",\n    \"120x0\",\n    \"120x120\",\n    \"0x120\"\n  ]",
    );
    let edits = [
        NAMED[0],
        NAMED[1],
        BY_LAYER,
        FIFTY_MM[0],
        FIFTY_MM[1],
        FIFTY_MM[2],
        excluded,
    ];
    assert_arranged_like_native("bed-exclude-area", CUBES, &edits, settings(false, false));
}

#[test]
fn plater_arrange_spaces_a_by_object_print_for_the_extruder_clearance_like_native() {
    assert_arranged_like_native("by-object", CUBES, &NAMED, settings(false, false));
}

/// A 32-gon of radius 110 mm about the bed centre; the bed shrink makes it a polygon bin.
const ROUND_BED: Edit = (
    PROJECT,
    "\"printable_area\": [\n    \"0x0\",\n    \"220x0\",\n    \"220x220\",\n    \"0x220\"\n  ]",
    r#""printable_area": [
    "220.000000x110.000000",
    "217.886381x131.459935",
    "211.626749x152.095178",
    "201.461657x171.112726",
    "187.781746x187.781746",
    "171.112726x201.461657",
    "152.095178x211.626749",
    "131.459935x217.886381",
    "110.000000x220.000000",
    "88.540065x217.886381",
    "67.904822x211.626749",
    "48.887274x201.461657",
    "32.218254x187.781746",
    "18.538343x171.112726",
    "8.373251x152.095178",
    "2.113619x131.459935",
    "0.000000x110.000000",
    "2.113619x88.540065",
    "8.373251x67.904822",
    "18.538343x48.887274",
    "32.218254x32.218254",
    "48.887274x18.538343",
    "67.904822x8.373251",
    "88.540065x2.113619",
    "110.000000x0.000000",
    "131.459935x2.113619",
    "152.095178x8.373251",
    "171.112726x18.538343",
    "187.781746x32.218254",
    "201.461657x48.887274",
    "211.626749x67.904822",
    "217.886381x88.540065"
  ]"#,
);

#[test]
fn plater_arrange_fits_a_round_bed_like_native() {
    // Four 70 mm instances: a square bed holds them, the round one only two per plate.
    let edits = [
        NAMED[0],
        NAMED[1],
        BY_LAYER,
        ROUND_BED,
        (
            MODEL,
            "1 0 0 0 1 0 0 0 1 50 110 2\"",
            "7 0 0 0 7 0 0 0 1 50 110 2\"",
        ),
        (
            MODEL,
            "1 0 0 0 1 0 0 0 1 125 110 2\"",
            "7 0 0 0 7 0 0 0 1 125 110 2\"",
        ),
        (
            MODEL,
            "1 0 0 0 1 0 0 0 1 200 110 2\"",
            "7 0 0 0 7 0 0 0 1 200 110 2\"",
        ),
        (
            MODEL,
            "</build>",
            "<item objectid=\"2\" transform=\"7 0 0 0 7 0 0 0 1 60 40 2\" printable=\"1\"/></build>",
        ),
    ];
    assert_arranged_like_native("round-bed", CUBES, &edits, settings(false, false));
}

#[test]
fn plater_arrange_moves_an_unprintable_instance_past_the_last_plate_like_native() {
    // The unprintable instance sits at the bed centre, where the CLI's own arrange of it
    // (`OrcaSlicer.cpp:6012`; the GUI job does not arrange it) leaves it.
    let edits = [
        NAMED[0],
        NAMED[1],
        BY_LAYER,
        (
            MODEL,
            "1 0 0 0 1 0 0 0 1 50 110 2\"",
            "1 0 0 0 1 0 0 0 1 60 40 2\"",
        ),
        (
            MODEL,
            "transform=\"1 0 0 0 1 0 0 0 1 125 110 2\" printable=\"1\"",
            "transform=\"1 0 0 0 1 0 0 0 1 110 110 2\" printable=\"0\"",
        ),
    ];
    assert_arranged_like_native("unprintable", CUBES, &edits, settings(false, false));
}
