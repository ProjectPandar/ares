use super::numeric_boundaries;
use serde_json::json;

#[test]
fn unbounded_sides_are_reported_separately_from_declared_endpoints() {
    if crate::artifacts::root_from_env().is_ok() {
        super::super::summary::write_summary(&[]);
    }
    let fixture = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(fixture.path().join("OrcaSlicer/src/libslic3r")).unwrap();
    std::fs::create_dir_all(fixture.path().join("tests/ksr_fdmtest_v4")).unwrap();
    std::fs::write(
        fixture
            .path()
            .join("OrcaSlicer/src/libslic3r/PrintConfig.cpp"),
        r#"
def = this->add("bridge_speed", coFloat);
def->min = 1;
def = this->add("resolution", coFloat);
def->min = 0;
def = this->add("min_bead_width", coPercent);
def->min = 0;
def = this->add("enable_long_retraction_when_cut", coInt);
def = this->add("retraction_distances_when_cut", coFloats);
def->min = 10;
def->max = 18;
for (auto& opt_key : filament_extruder_override_keys) {
def = this->add_nullable(opt_key, it_opt->second.type);
def->min = it_opt->second.min;
def->max = it_opt->second.max;
}
"#,
    )
    .unwrap();
    let entries = [("bridge_speed", "coFloat"), ("resolution", "coFloat"), ("min_bead_width", "coPercent"), ("enable_long_retraction_when_cut", "coInt"), ("filament_retraction_distances_when_cut", "coFloats")].map(|(key, kind)| json!({"key":key,"option_type":kind,"upstream_definition":{"path":"src/libslic3r/PrintConfig.cpp","line":1}}));
    std::fs::write(
        fixture
            .path()
            .join("tests/ksr_fdmtest_v4/options-v242.json"),
        serde_json::to_vec(&entries).unwrap(),
    )
    .unwrap();
    let rows = numeric_boundaries::inventory(fixture.path());
    for (key, minimum) in [
        ("bridge_speed", 1.0),
        ("resolution", 0.0),
        ("min_bead_width", 0.0),
    ] {
        let row = rows.iter().find(|row| row["key"] == key).unwrap();
        assert_eq!(row["minimum"], json!({"kind":"DECLARED", "value":minimum}));
        assert_eq!(row["maximum"], json!({"kind":"UNBOUNDED"}));
    }
    let retract = rows
        .iter()
        .find(|row| row["key"] == "enable_long_retraction_when_cut")
        .unwrap();
    assert_eq!(retract["minimum"], json!({"kind":"UNBOUNDED"}));
    assert_eq!(retract["maximum"], json!({"kind":"UNBOUNDED"}));
    let cut = rows
        .iter()
        .find(|row| row["key"] == "filament_retraction_distances_when_cut")
        .unwrap();
    assert_eq!(cut["minimum"], json!({"kind":"DECLARED", "value":10.0}));
    assert_eq!(cut["maximum"], json!({"kind":"DECLARED", "value":18.0}));
    assert!(rows.iter().all(|row| {
        row["source"]
            .as_str()
            .unwrap()
            .starts_with("src/libslic3r/PrintConfig.cpp:")
    }));
}
