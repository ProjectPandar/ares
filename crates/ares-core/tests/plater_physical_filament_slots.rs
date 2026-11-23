use std::io::{Cursor, Read, Write};

use ares_core::{FilamentSlotEdit, FilamentTarget, PlaterSession};
use serde_json::{Value, json};
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};

const TWO_TOOLS: &[u8] =
    include_bytes!("../../../tests/parity/type2-tower-entry-z/j1-two-tools.3mf");

fn three_slots(mixed: bool) -> Vec<u8> {
    let mut source = ZipArchive::new(Cursor::new(TWO_TOOLS)).unwrap();
    let mut output = ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..source.len() {
        let mut member = source.by_index(index).unwrap();
        if member.name() != "Metadata/project_settings.config" {
            output.raw_copy_file(member).unwrap();
            continue;
        }
        let mut bytes = Vec::new();
        member.read_to_end(&mut bytes).unwrap();
        let mut settings: Value = serde_json::from_slice(&bytes).unwrap();
        settings["single_extruder_multi_material"] = json!("1");
        settings["filament_colour"] = json!(["#CF3020", "#307CC0", "#123456"]);
        settings["filament_settings_id"] = json!(["", "Snapmaker J1 PLA", "Snapmaker J1 PLA"]);
        settings["filament_is_mixed"] = json!(["0", "0", if mixed { "1" } else { "0" }]);
        if mixed {
            settings["filament_mixed_components"] = json!(["", "", "1,2"]);
            settings["filament_mixed_sublayer_ratios"] = json!(["", "", "1,1"]);
        }
        settings["flush_volumes_vector"] = json!(["120", "140", "160", "180", "200", "220"]);
        settings["flush_volumes_matrix"] = json!([
            "0", "101", "102", "103", "0", "105", "106", "107", "0", "0", "201", "202", "203", "0",
            "205", "206", "207", "0"
        ]);
        output
            .start_file(member.name(), SimpleFileOptions::default())
            .unwrap();
        output
            .write_all(&serde_json::to_vec(&settings).unwrap())
            .unwrap();
    }
    output
        .start_file(
            "Metadata/custom_gcode_per_layer.xml",
            SimpleFileOptions::default(),
        )
        .unwrap();
    output
        .write_all(
            br##"<custom_gcodes_per_layer>
  <plate>
    <plate_info id="1"/>
    <mode value="MultiAsSingle"/>
    <layer top_z="0.4" extruder="1" color="" type="2" extra=""/>
    <layer top_z="0.6" extruder="2" color="" type="2" extra=""/>
    <layer top_z="0.8" extruder="1" color="" type="2" extra=""/>
    <layer top_z="1.0" extruder="3" color="" type="2" extra=""/>
    <layer top_z="1.2" extruder="1" color="" type="1" extra="wait"/>
  </plate>
</custom_gcodes_per_layer>"##,
        )
        .unwrap();
    output.finish().unwrap().into_inner()
}

#[test]
fn merging_an_empty_first_slot_into_a_later_slot_shifts_assignments_and_supports_undo() {
    let mut session = PlaterSession::load(three_slots(false)).unwrap();
    let second = FilamentTarget {
        object: session.project().objects()[1].id(),
        volume: None,
    };
    session.set_filament(&[second], (3, 3)).unwrap();
    let before_merge = session.snapshot();
    let merged = session
        .edit_filament_slots(FilamentSlotEdit::Merge {
            source: 1,
            target: 3,
        })
        .unwrap();
    assert_eq!(
        merged
            .slots
            .iter()
            .map(|slot| slot.colour.as_str())
            .collect::<Vec<_>>(),
        ["#307CC0", "#123456"]
    );
    let assignments = |session: &PlaterSession| {
        session
            .project()
            .objects()
            .iter()
            .map(|object| object.filament())
            .collect::<Vec<_>>()
    };
    assert_eq!(assignments(&session), [Some(2), Some(2)]);
    let after_merge = session.snapshot();
    session.load_snapshot(&before_merge);
    assert_eq!(session.filament_slots().slots[0].preset, "");
    assert_eq!(assignments(&session), [Some(1), Some(3)]);
    session.load_snapshot(&after_merge);
    assert_eq!(session.filament_slots().slots.len(), 2);
    assert_eq!(assignments(&session), [Some(2), Some(2)]);
}

#[test]
fn deleting_a_middle_slot_preserves_each_nozzles_directed_flush_transitions() {
    let mut session = PlaterSession::load(three_slots(false)).unwrap();
    session
        .edit_filament_slots(FilamentSlotEdit::Delete { slot: 2 })
        .unwrap();
    let print = &session.project().settings().project.print;
    assert_eq!(
        print.flush_volumes_matrix.0,
        [0.0, 102.0, 106.0, 0.0, 0.0, 202.0, 206.0, 0.0]
    );
    assert_eq!(session.project().objects()[1].filament(), Some(1));
}

#[test]
fn adding_before_a_virtual_slot_keeps_its_material_and_flush_entries() {
    let mut session = PlaterSession::load(three_slots(true)).unwrap();
    let object = FilamentTarget {
        object: session.project().objects()[1].id(),
        volume: None,
    };
    session.set_filament(&[object], (3, 3)).unwrap();
    let previous = session.filament_slots();
    let added = session.edit_filament_slots(FilamentSlotEdit::Add).unwrap();
    assert_eq!(added.slots.len(), 4);
    assert_eq!(added.slots[3], previous.slots[2]);
    assert_eq!(session.project().objects()[1].filament(), Some(4));
    let matrix = &session
        .project()
        .settings()
        .project
        .print
        .flush_volumes_matrix
        .0;
    assert_eq!(
        [matrix[3], matrix[12], matrix[19], matrix[28]],
        [102.0, 106.0, 202.0, 206.0]
    );
}

#[test]
fn merging_retargets_each_original_tool_change_without_moving_a_pause() {
    let mut session = PlaterSession::load(three_slots(false)).unwrap();
    let before = session.project().plates_custom_gcodes()[&0].gcodes.clone();
    session
        .edit_filament_slots(FilamentSlotEdit::Merge {
            source: 1,
            target: 3,
        })
        .unwrap();
    let after = &session.project().plates_custom_gcodes()[&0].gcodes;
    assert_eq!(
        after
            .iter()
            .map(|tick| (tick.print_z, tick.extruder))
            .collect::<Vec<_>>(),
        [(0.4, 2), (0.6, 1), (0.8, 2), (1.0, 2), (1.2, 1)]
    );
    assert_eq!(after.last(), before.last());
}

#[test]
fn fixed_heads_keep_their_slots_when_a_count_edit_is_requested() {
    let mut session = PlaterSession::load(TWO_TOOLS.to_vec()).unwrap();
    let before = session.filament_slots();
    assert!(!before.can_add && !before.can_remove);
    assert!(session.edit_filament_slots(FilamentSlotEdit::Add).is_err());
    assert!(
        session
            .edit_filament_slots(FilamentSlotEdit::Delete { slot: 1 })
            .is_err()
    );
    assert_eq!(session.filament_slots(), before);
}
