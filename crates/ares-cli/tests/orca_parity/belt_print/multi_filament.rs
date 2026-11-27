use serde_json::{Value, json};
use std::{
    fmt::Write as _,
    io::{Cursor, Write as _},
};
use zip::{ZipWriter, write::SimpleFileOptions};

pub(super) const SWITCHES: &[(u32, u32)] = &[(80, 2), (82, 1)];

#[test]
fn unused_slots_do_not_schedule_tools_or_towers() {
    only_first_slot_prints("two-slots-unused", 2, "1");
}

#[test]
fn accepts_three_configured_slots_with_only_one_used() {
    only_first_slot_prints("three-slots-unused", 3, "0");
}

fn only_first_slot_prints(name: &str, slots: usize, enabled: &str) {
    compare(
        name,
        (slots, 1, false),
        &[],
        &[
            ("enable_belt_purge_tower", json!(enabled)),
            ("enable_prime_tower", json!(enabled)),
        ],
        |native| {
            assert_eq!(native.selections(), [0]);
            assert_eq!(native.tools("Outer wall"), [0]);
            assert_eq!(native.tools("Brim"), [0]);
        },
    );
}

#[test]
fn selects_nonzero_slot_for_object_and_both_kinds_of_brim() {
    compare("slot-2-only", (2, 2, false), &[], &[], |native| {
        assert_eq!(native.selections(), [1]);
        assert_eq!(native.tools("Outer wall"), [1]);
        assert_eq!(native.tools("Brim"), [1]);
        assert!(native.prints("77.9675", 1, "Outer wall"));
        assert!(native.prints("77.9675", 1, "Brim"));
        assert!(native.first_z("Brim") < native.first_z("Outer wall"));
    });
}

#[test]
fn inactive_marked_mesh_prints_without_brim_with_two_slots() {
    compare("idle-marked-off", (2, 1, true), &[], &[], |native| {
        assert_eq!(native.selections(), [0]);
        assert_eq!(native.tools("Outer wall"), [0]);
        assert!(native.tools("Brim").is_empty());
    });
}

#[test]
fn plate_events_switch_the_extruding_object_without_brim() {
    compare(
        "events-no-brim",
        (2, 1, false),
        SWITCHES,
        &[("brim_type", json!("no_brim"))],
        |native| {
            assert_eq!(native.selections(), [0, 1, 0]);
            assert!(native.tools("Brim").is_empty());
            for (z, tool) in [("80.1675", 1), ("82.1675", 0)] {
                for role in ["Outer wall", "Internal solid infill"] {
                    assert!(native.prints(z, tool, role), "{z} T{tool} {role}");
                }
            }
        },
    );
}

#[test]
fn plate_events_do_not_override_the_brim_filament() {
    switches_with_brim("events-with-brim", SWITCHES, "80.1675", "1");
}

#[test]
fn an_early_plate_event_starts_after_the_apron() {
    switches_with_brim("events-early-brim", &[(1, 2), (80, 1)], "77.9675", "0");
}

fn switches_with_brim(name: &str, events: &[(u32, u32)], z: &str, enabled: &str) {
    compare(
        name,
        (2, 1, false),
        events,
        &[
            ("enable_belt_purge_tower", json!(enabled)),
            ("enable_prime_tower", json!(enabled)),
        ],
        |native| {
            assert_eq!(native.selections()[..3], [0, 1, 0]);
            assert_eq!(native.tools("Brim"), [0]);
            assert!(native.first_z("Brim") < native.first_z("Outer wall"));
            let wall = native.position(z, 1, "Outer wall").unwrap();
            let brim = native.position(z, 0, "Brim").unwrap();
            assert!(
                wall < brim,
                "coincident band must print after the changed wall"
            );
        },
    );
}

#[test]
fn role_split_changes_tools_with_purge_disabled() {
    role_split("split-off", "0");
}

#[test]
fn role_split_changes_tools_when_purge_is_enabled_without_a_prism() {
    role_split("split-unmarked-on", "1");
}

fn role_split(name: &str, enabled: &str) {
    compare(
        name,
        (2, 1, false),
        &[],
        &[
            ("internal_solid_filament_id", json!("2")),
            ("enable_belt_purge_tower", json!(enabled)),
        ],
        |native| {
            assert_eq!(native.selections()[..3], [0, 1, 0]);
            assert_eq!(native.tools("Outer wall"), [0]);
            assert_eq!(native.tools("Brim"), [0]);
            assert_eq!(native.tools("Internal solid infill"), [1]);
            assert!(native.prints("79.1675", 1, "Internal solid infill"));
        },
    );
}

fn compare(
    name: &str,
    (slots, object_slot, marker): (usize, u32, bool),
    switches: &[(u32, u32)],
    edits: &[(&str, Value)],
    check: impl FnOnce(Witness<'_>),
) {
    super::compare_project(
        name,
        project(slots, object_slot, marker, switches, edits),
        |gcode| {
            for tag in [
                ";TYPE:Prime tower\n",
                "; WIPE_TOWER_START\n",
                "; PURGING FINISHED\n",
            ] {
                assert!(!gcode.contains(tag), "{name}: unexpected {tag}");
            }
            check(Witness::of(gcode));
        },
    );
}

pub(super) fn project(
    slots: usize,
    object_slot: u32,
    marker: bool,
    switches: &[(u32, u32)],
    edits: &[(&str, Value)],
) -> Vec<u8> {
    let input = super::project_with(true, false, &[], false, |member, bytes| match member {
        "Metadata/project_settings.config" => {
            let mut config: serde_json::Map<String, Value> =
                serde_json::from_slice(&bytes).unwrap();
            for (key, value) in &mut config {
                if key.starts_with("filament_")
                    && let Some([only]) = value.as_array().map(Vec::as_slice)
                {
                    *value = json!(vec![only.clone(); slots]);
                }
            }
            let matrix = (0..slots)
                .flat_map(|i| (0..slots).map(move |j| if i == j { "0" } else { "280" }))
                .collect::<Vec<_>>();
            let starts = (0..slots)
                .map(|i| format!("; filament start {i}"))
                .collect::<Vec<_>>();
            for (key, value) in [
                ("belt_slice_rotation_angle", json!("45")),
                ("brim_type", json!("outer_only")),
                ("brim_width", json!("5")),
                (
                    "change_filament_gcode",
                    json!("; swap [previous_extruder] -> [next_extruder]"),
                ),
                (
                    "filament_colour",
                    json!(["#ff0000", "#0000ff", "#00ff00"][..slots]),
                ),
                ("filament_start_gcode", json!(starts)),
                ("flush_volumes_matrix", json!(matrix)),
                ("flush_volumes_vector", json!(vec!["140"; slots * 2])),
                ("inherits_group", json!(vec![""; slots + 2])),
            ] {
                config.insert(key.to_owned(), value);
            }
            config.extend(
                edits
                    .iter()
                    .map(|(key, value)| ((*key).to_owned(), value.clone())),
            );
            serde_json::to_vec(&config).unwrap()
        }
        "Metadata/model_settings.config" => {
            let object_setting = "<metadata key=\"extruder\" value=\"1\"/>";
            let config = std::str::from_utf8(&bytes).unwrap();
            assert_eq!(config.matches(object_setting).count(), 1);
            config.replace(object_setting, &format!(
                "<metadata key=\"extruder\" value=\"{object_slot}\"/>\n    <metadata key=\"belt_purge_tower_object\" value=\"{}\"/>",
                u8::from(marker)
            )).into_bytes()
        }
        _ => bytes,
    });
    if switches.is_empty() {
        return input;
    }
    let mut xml = String::from("<custom_gcodes_per_layer><plate><plate_info id=\"1\"/>");
    for (z, slot) in switches {
        write!(
            xml,
            "<layer top_z=\"{z}\" type=\"2\" extruder=\"{slot}\" color=\"\" extra=\"\"/>"
        )
        .unwrap();
    }
    xml.push_str("<mode value=\"MultiAsSingle\"/></plate></custom_gcodes_per_layer>");
    let mut zip = ZipWriter::new_append(Cursor::new(input)).unwrap();
    zip.start_file(
        "Metadata/custom_gcode_per_layer.xml",
        SimpleFileOptions::default(),
    )
    .unwrap();
    zip.write_all(xml.as_bytes()).unwrap();
    zip.finish().unwrap().into_inner()
}

fn extrudes(line: &str) -> bool {
    let Some(parameters) = line.strip_prefix("G1 ") else {
        return false;
    };
    let words = parameters
        .split_whitespace()
        .take_while(|word| !word.starts_with(';'));
    words.clone().any(|word| word.starts_with(['X', 'Y', 'Z']))
        && words
            .filter_map(|word| word.strip_prefix('E'))
            .any(|e| e.parse::<f64>().is_ok_and(|e| e > 0.0))
}

struct Witness<'a> {
    selections: Vec<u32>,
    extrusion: Vec<(&'a str, u32, &'a str)>,
}

impl<'a> Witness<'a> {
    fn of(gcode: &'a str) -> Self {
        let (mut z, mut tool, mut role) = ("", 0, "");
        let (mut selections, mut extrusion) = (Vec::new(), Vec::new());
        for line in gcode
            .lines()
            .take_while(|line| *line != "; CONFIG_BLOCK_START")
        {
            if let Some(height) = line.strip_prefix(";Z:") {
                z = height;
            } else if let Some(kind) = line.strip_prefix(";TYPE:") {
                role = kind;
            } else if let Some(id) = line
                .split_whitespace()
                .next()
                .and_then(|word| word.strip_prefix('T'))
                .and_then(|id| id.parse().ok())
            {
                tool = id;
                selections.push(tool);
            } else if !z.is_empty() && extrudes(line) {
                extrusion.push((z, tool, role));
            }
        }
        Self {
            selections,
            extrusion,
        }
    }

    fn selections(&self) -> &[u32] {
        &self.selections
    }

    fn tools(&self, role: &str) -> Vec<u32> {
        let mut tools = self
            .extrusion
            .iter()
            .filter_map(|&(_, tool, kind)| (kind == role).then_some(tool))
            .collect::<Vec<_>>();
        tools.sort_unstable();
        tools.dedup();
        tools
    }

    fn first_z(&self, role: &str) -> f64 {
        self.extrusion
            .iter()
            .find(|&&(_, _, kind)| kind == role)
            .unwrap()
            .0
            .parse()
            .unwrap()
    }

    fn prints(&self, z: &str, tool: u32, role: &str) -> bool {
        self.position(z, tool, role).is_some()
    }

    fn position(&self, z: &str, tool: u32, role: &str) -> Option<usize> {
        self.extrusion
            .iter()
            .position(|&path| path == (z, tool, role))
    }
}
