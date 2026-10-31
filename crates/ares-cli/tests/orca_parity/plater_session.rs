//! Plater session edits (move, rotate, plate changes) slice like official
//! OrcaSlicer slicing the project the GUI exports for them: the build item
//! transforms (`bbs_3mf.cpp:8050-8073`), the plate map (`:7918`) and one
//! `filament_sequence.json` entry per plate. Native runs `--slice <plate>`.

use ares_core::{ConfigOverrides, GenerationMetadata, PlaterSession, PlaterSlice, SlicerContext};

use super::plater_archive::{attribute, rewrite_archive};
use crate::runner::{OrcaRunner, repo_root};

#[path = "plater_rotation_pivot.rs"]
mod rotation_pivot;

const SINGLE: &str = "tests/parity/orca_cli_ender3.3mf";
const TWO_PLATES: &str = "tests/parity/custom-gcode-overview/two-plates.3mf";
/// `plate_stride_x` of the 220 mm Ender-3 bed (`PartPlate.cpp:4836-4842`).
const STRIDE: f64 = 220.0 * 1.2;
const EMPTY_SEQUENCE: &str = r#"{"nozzle_sequence":[],"optimal_assignment":[],"sequence":[]}"#;

fn load(path: &str) -> PlaterSession {
    PlaterSession::load(std::fs::read(repo_root().join(path)).unwrap()).unwrap()
}

/// A 3MF transform: the first three rows, column by column (`Transform3d::parse_3mf`).
fn transform_attribute(session: &PlaterSession, object_id: u32, instance_id: u32) -> String {
    let (object, instance) = session
        .project()
        .objects()
        .iter()
        .enumerate()
        .find_map(|(index, item)| {
            (item.id() == object_id).then(|| {
                let instance = item
                    .instances()
                    .iter()
                    .position(|i| i.instance_id() == instance_id)
                    .unwrap();
                (index, instance)
            })
        })
        .unwrap();
    let matrix = session
        .instance_transform(object, instance)
        .unwrap()
        .to_column_major();
    (0..4)
        .flat_map(|column| (0..3).map(move |row| matrix[column * 4 + row]))
        .map(|value| value.to_string())
        .collect::<Vec<_>>()
        .join(" ")
}

/// `<item objectid="N" ... transform="...">`: each item's transform from the session.
fn rewrite_build(model: &str, session: &PlaterSession) -> String {
    let mut output = String::new();
    let mut rest = model;
    let mut seen = std::collections::BTreeMap::<u32, u32>::new();
    while let Some(start) = rest.find("<item ") {
        let end = start + rest[start..].find("/>").unwrap();
        let item = &rest[start..end];
        let object_id = attribute(item, "objectid").parse::<u32>().unwrap();
        let instance_id = *seen
            .entry(object_id)
            .and_modify(|count| *count += 1)
            .or_insert(0);
        let old = format!("transform=\"{}\"", attribute(item, "transform"));
        let new = format!(
            "transform=\"{}\"",
            transform_attribute(session, object_id, instance_id)
        );
        output.push_str(&rest[..start]);
        output.push_str(&item.replace(&old, &new));
        rest = &rest[end..];
    }
    output.push_str(rest);
    output
}

/// The `<plate>` blocks of `model_settings.config` for the session plates:
/// a loaded plate keeps its metadata, every plate lists its instances.
fn rewrite_plates(settings: &str, session: &PlaterSession) -> String {
    let first = settings.find("  <plate>").unwrap();
    let last = settings.rfind("</plate>").unwrap() + "</plate>\n".len();
    let blocks = settings[first..last]
        .split_inclusive("</plate>\n")
        .collect::<Vec<_>>();
    let mut plates = String::new();
    for (index, plate) in session.plates().iter().enumerate() {
        plates.push_str("  <plate>\n");
        plates.push_str(&format!(
            "    <metadata key=\"plater_id\" value=\"{}\"/>\n",
            index + 1
        ));
        match plate.source {
            Some(source) => {
                for line in blocks[source].lines().filter(|line| {
                    line.starts_with("    <metadata") && !line.contains("\"plater_id\"")
                }) {
                    plates.push_str(line);
                    plates.push('\n');
                }
            }
            None => plates.push_str(
                "    <metadata key=\"plater_name\" value=\"\"/>\n    <metadata key=\"locked\" value=\"false\"/>\n",
            ),
        }
        for (object, instance) in &plate.instances {
            let item = &session.project().objects()[*object].instances()[*instance];
            plates.push_str(&format!(
                "    <model_instance>\n      <metadata key=\"object_id\" value=\"{}\"/>\n      <metadata key=\"instance_id\" value=\"{}\"/>\n      <metadata key=\"identify_id\" value=\"{}\"/>\n    </model_instance>\n",
                item.object_id(),
                item.instance_id(),
                item.loaded_label_id().unwrap()
            ));
        }
        plates.push_str("  </plate>\n");
    }
    format!("{}{}{}", &settings[..first], plates, &settings[last..])
}

/// `filament_sequence.json`: one entry per plate, kept from the source plate.
fn rewrite_sequences(sequences: &str, session: &PlaterSession) -> String {
    let old: serde_json::Map<String, serde_json::Value> = serde_json::from_str(sequences).unwrap();
    let mut new = serde_json::Map::new();
    for (index, plate) in session.plates().iter().enumerate() {
        let value = plate
            .source
            .and_then(|source| old.get(&format!("plate_{}", source + 1)).cloned())
            .unwrap_or_else(|| serde_json::from_str(EMPTY_SEQUENCE).unwrap());
        new.insert(format!("plate_{}", index + 1), value);
    }
    serde_json::to_string(&new).unwrap()
}

/// The project as the GUI exports the session; other entries are unchanged.
fn export(session: &PlaterSession) -> Vec<u8> {
    rewrite_archive(session.input(), |name, text| match name {
        "3D/3dmodel.model" => Some(rewrite_build(text, session)),
        "Metadata/model_settings.config" => Some(rewrite_plates(text, session)),
        "Metadata/filament_sequence.json" => Some(rewrite_sequences(text, session)),
        _ => None,
    })
}

fn assert_plate_like_native(label: &str, session: &PlaterSession, plate: u32) {
    assert_oracle_like_native(label, session, export(session), plate);
}

/// Native slicing `oracle`, a project the GUI exports for the session.
fn assert_oracle_like_native(label: &str, session: &PlaterSession, oracle: Vec<u8>, plate: u32) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let exported = runner
        .fixture_case(&format!("plater/{label}"), oracle)
        .unwrap();
    let case = runner.slice_selected_plate_case(&exported, plate).unwrap();
    let actual = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(ares_core::slice_plater_plate(
            session,
            PlaterSlice {
                plate: Some(plate),
                overrides: &ConfigOverrides::default(),
                validate_print: false,
            },
            GenerationMetadata::default(),
            SlicerContext::new(0),
        ))
        .unwrap()
        .gcode;
    let outcome = crate::golden::compare_ordered_bytes_generator_only(&case.reference, &actual);
    assert!(
        outcome.is_ok(),
        "{label}: plate {plate} diverged from Native: {outcome:?}"
    );
}

fn offset(session: &PlaterSession, object: usize) -> [f64; 3] {
    let matrix = session
        .instance_transform(object, 0)
        .unwrap()
        .to_column_major();
    [matrix[12], matrix[13], matrix[14]]
}

#[test]
fn plater_move_with_drop_slices_like_native() {
    let mut session = load(SINGLE);
    // A raised move drops back onto the bed (GLCanvas3D::do_move).
    session
        .translate_instance(0, 0, [35.0, -20.0, 7.0])
        .unwrap();
    assert_eq!(offset(&session, 0), [145.0, 90.0, 5.0]);
    session.set_instance_position(0, 0, 1, 95.5).unwrap();
    assert_eq!(offset(&session, 0), [145.0, 95.5, 5.0]);
    assert_plate_like_native("move", &session, 1);
}

/// `Selection::rotate` of a lone instance about its bounding-sphere centre
/// in world coordinates is `R · M` around that point; the cube's sphere
/// centre is its offset.
fn rotation_about_offset(rotations: &[[[f64; 3]; 3]], offset: [f64; 3]) -> [f64; 16] {
    let linear = rotations
        .iter()
        .fold(IDENTITY, |acc, rotation| multiply(rotation, &acc));
    let mut matrix = [0.0; 16];
    for column in 0..3 {
        for row in 0..3 {
            matrix[column * 4 + row] = linear[row][column];
        }
    }
    matrix[12..15].copy_from_slice(&offset);
    matrix[15] = 1.0;
    matrix
}

const IDENTITY: [[f64; 3]; 3] = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

fn multiply(a: &[[f64; 3]; 3], b: &[[f64; 3]; 3]) -> [[f64; 3]; 3] {
    std::array::from_fn(|row| {
        std::array::from_fn(|column| (0..3).map(|k| a[row][k] * b[k][column]).sum())
    })
}

fn about_x(degrees: f64) -> [[f64; 3]; 3] {
    let (sin, cos) = degrees.to_radians().sin_cos();
    [[1.0, 0.0, 0.0], [0.0, cos, -sin], [0.0, sin, cos]]
}

fn about_z(degrees: f64) -> [[f64; 3]; 3] {
    let (sin, cos) = degrees.to_radians().sin_cos();
    [[cos, -sin, 0.0], [sin, cos, 0.0], [0.0, 0.0, 1.0]]
}

fn assert_transform(session: &PlaterSession, expected: [f64; 16]) {
    let actual = session.instance_transform(0, 0).unwrap().to_column_major();
    for (actual, expected) in actual.iter().zip(expected) {
        assert!(
            (actual - expected).abs() < 1e-9,
            "{actual:?} != {expected:?}"
        );
    }
}

#[test]
fn plater_rotate_with_drop_slices_like_native() {
    let mut session = load(SINGLE);
    session
        .rotate_instance(0, 0, [0.0, 0.0, std::f64::consts::FRAC_PI_4])
        .unwrap();
    assert_transform(
        &session,
        rotation_about_offset(&[about_z(45.0)], [110.0, 110.0, 5.0]),
    );
    // A quarter turn about X keeps a flat bottom (a corner on the bed slices to an empty first
    // layer, a Native slicing error); the absolute field rotates by the change (X 0 -> 90) and
    // do_rotate drops the turned cube, whose half height is now 5√2, to the bed.
    session.set_absolute_rotation(0, 0, 0, 90.0).unwrap();
    let height = 5.0 * std::f64::consts::SQRT_2;
    assert_transform(
        &session,
        rotation_about_offset(&[about_z(45.0), about_x(90.0)], [110.0, 110.0, height]),
    );
    assert_plate_like_native("rotate", &session, 1);
}

#[test]
fn plater_move_to_second_plate_slices_like_native() {
    let mut session = load(TWO_PLATES);
    session
        .translate_instance(0, 0, [STRIDE + 50.0 - 110.0, 60.0 - 110.0, 0.0])
        .unwrap();
    let plates = session.plates();
    assert!(plates[0].instances.is_empty());
    assert_eq!(plates[1].instances, vec![(0, 0), (1, 0)]);
    assert_plate_like_native("second-plate", &session, 2);
}

#[test]
fn plater_add_and_delete_plates_slices_like_native() {
    let mut session = load(SINGLE);
    session.add_plate();
    session
        .translate_instance(0, 0, [STRIDE, 0.0, 0.0])
        .unwrap();
    assert_eq!(session.plates()[1].instances, vec![(0, 0)]);
    session.add_plate();
    // Deleting plate 1 moves the later plates and their instances up.
    assert!(session.delete_plate(0));
    let plates = session.plates();
    assert_eq!(plates.len(), 2);
    assert_eq!(plates[0].instances, vec![(0, 0)]);
    assert_eq!(offset(&session, 0), [110.0, 110.0, 5.0]);
    assert_plate_like_native("plates", &session, 1);
}

#[test]
fn plater_delete_plate_keeps_custom_gcodes_by_index_like_native() {
    let mut session = load(TWO_PLATES);
    session.add_plate();
    // The second plate's instance moves up; Model::plates_custom_gcodes stays keyed by plate
    // index (PartPlateList::delete_plate does not touch it), so plate 1 keeps plate 1's
    // custom G-codes and the GUI export writes them unchanged (bbs_3mf.cpp:8469).
    assert!(session.delete_plate(0));
    assert_eq!(session.plates()[0].instances, vec![(1, 0)]);
    assert_eq!(offset(&session, 1), [110.0, 110.0, 5.0]);
    assert_plate_like_native("custom-gcodes-by-index", &session, 1);
}

/// The single cube with its offset in the component transform, the part `matrix` and source
/// offsets (model_settings.config), and the build item at `item`: the frame `--export-3mf` writes for a placed STL
/// (runner/profile_entry_3mf.rs). Loading moves the volume offset into the instance.
fn volume_offset_project(item: &str) -> Vec<u8> {
    let input = std::fs::read(repo_root().join(SINGLE)).unwrap();
    rewrite_archive(&input, |name, text| match name {
        "3D/3dmodel.model" => {
            let component = text.find("<component ").unwrap();
            let start = component + text[component..].find("transform=\"").unwrap() + 11;
            let end = start + text[start..].find('"').unwrap();
            let text = format!(
                "{}1 0 0 0 1 0 0 0 1 110 110 5{}",
                &text[..start],
                &text[end..]
            );
            let item_start = text.find("<item ").unwrap();
            Some(format!(
                "{}{}",
                &text[..item_start],
                text[item_start..].replace("1 0 0 0 1 0 0 0 1 110 110 5", item)
            ))
        }
        "Metadata/model_settings.config" => Some(
            [
                (
                    "matrix",
                    "1 0 0 0 0 1 0 0 0 0 1 0 0 0 0 1",
                    "1 0 0 110 0 1 0 110 0 0 1 5 0 0 0 1",
                ),
                ("source_offset_x", "0", "110"),
                ("source_offset_y", "0", "110"),
                ("source_offset_z", "0", "5"),
            ]
            .iter()
            .fold(text.to_owned(), |text, (key, from, to)| {
                text.replace(
                    &format!("\"{key}\" value=\"{from}\""),
                    &format!("\"{key}\" value=\"{to}\""),
                )
            }),
        ),
        _ => None,
    })
}

#[test]
fn plater_volume_offset_project_slices_like_native() {
    let input = volume_offset_project("1 0 0 0 1 0 0 0 1 0 0 0");
    let mut session = PlaterSession::load(input.clone()).unwrap();
    assert_oracle_like_native("volume-offset", &session, input, 1);
    // A world move of the instance moves the build item the same way in the project's frame.
    session.translate_instance(0, 0, [10.0, -5.0, 0.0]).unwrap();
    let moved = volume_offset_project("1 0 0 0 1 0 0 0 1 10 -5 0");
    assert_oracle_like_native("volume-offset-moved", &session, moved, 1);
}
