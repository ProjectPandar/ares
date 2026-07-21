use super::*;

#[test]
fn task22n_flow_option_pairs_cover_selectors_nozzles_and_anti_map() {
    selector_pair(
        "outer-selector",
        Key::OuterSelector,
        [
            r#""outer_wall_filament_id": "0""#,
            r#""outer_wall_filament_id": "1""#,
            r#""outer_wall_filament_id": "2""#,
        ],
        &[change(
            BOTH_LAYERS,
            EXTERNAL_PERIMETER_ROLE,
            ABSOLUTE_042_NOZZLE_04,
            ABSOLUTE_042_NOZZLE_06,
        )],
    );
    selector_pair(
        "inner-selector",
        Key::InnerSelector,
        [
            r#""inner_wall_filament_id": "0""#,
            r#""inner_wall_filament_id": "1""#,
            r#""inner_wall_filament_id": "2""#,
        ],
        &[
            change(
                BOTH_LAYERS,
                INTERNAL_PERIMETER_ROLE,
                ABSOLUTE_045_NOZZLE_04,
                ABSOLUTE_045_NOZZLE_06,
            ),
            change(
                BOTH_LAYERS,
                OVERHANG_ROLE,
                PERCENT_100_NOZZLE_04,
                PERCENT_100_NOZZLE_06,
            ),
        ],
    );
    selector_pair(
        "solid-selector",
        Key::SolidSelector,
        [
            r#""internal_solid_filament_id": "0""#,
            r#""internal_solid_filament_id": "1""#,
            r#""internal_solid_filament_id": "2""#,
        ],
        &[change(
            BOTH_LAYERS,
            SOLID_INFILL_ROLE,
            ABSOLUTE_042_NOZZLE_04,
            ABSOLUTE_042_NOZZLE_06,
        )],
    );
    run(OptionPair {
        name: "raw-zero-one",
        setup: &[],
        delta: process(
            r#""outer_wall_filament_id": "0""#,
            r#""outer_wall_filament_id": "1""#,
        ),
        key: Key::OuterSelector,
        raw: [Value::Int(0), Value::Int(1)],
        effective: [Value::Int(1), Value::Int(1)],
        changes: &[],
    });
    run(OptionPair {
        name: "scoped-fallback",
        setup: &[
            NOZZLES_46,
            Edit {
                path: MODEL,
                from: r#"<object id="2"><part id="1" subtype="normal_part"/>"#,
                to: r#"<object id="2"><metadata key="extruder" value="2"/><part id="1" subtype="normal_part"><metadata key="outer_wall_filament_id" value="0"/></part>"#,
            },
        ],
        delta: Edit {
            path: MODEL,
            from: r#"<metadata key="outer_wall_filament_id" value="0"/>"#,
            to: r#"<metadata key="outer_wall_filament_id" value="2"/>"#,
        },
        key: Key::ScopedOuter,
        raw: [Value::Int(0), Value::Int(2)],
        effective: [Value::Int(2), Value::Int(2)],
        changes: &[],
    });
    run(OptionPair {
        name: "nozzle-list",
        setup: &[
            INITIAL_ZERO,
            OUTER_TWO,
            INNER_TWO,
            SOLID_TWO,
            process(
                r#""outer_wall_line_width": "0.42""#,
                r#""outer_wall_line_width": "125%""#,
            ),
            process(
                r#""inner_wall_line_width": "0.45""#,
                r#""inner_wall_line_width": "110%""#,
            ),
            process(
                r#""internal_solid_infill_line_width": "0.42""#,
                r#""internal_solid_infill_line_width": "100%""#,
            ),
            process(
                r#""bridge_line_width": "100%""#,
                r#""bridge_line_width": "80%""#,
            ),
        ],
        delta: NOZZLES_46,
        key: Key::Nozzles,
        raw: [pair(0.4, 0.4), pair(0.4, 0.6)],
        effective: [pair(0.4, 0.4), pair(0.4, 0.6)],
        changes: &[
            change(
                BOTH_LAYERS,
                INTERNAL_PERIMETER_ROLE,
                PERCENT_110_NOZZLE_04,
                PERCENT_110_NOZZLE_06,
            ),
            change(
                BOTH_LAYERS,
                EXTERNAL_PERIMETER_ROLE,
                INITIAL_ABSOLUTE_050_NOZZLE_04,
                PERCENT_125_NOZZLE_06,
            ),
            change(
                BOTH_LAYERS,
                OVERHANG_ROLE,
                PERCENT_080_NOZZLE_04,
                PERCENT_080_NOZZLE_06,
            ),
            change(
                BOTH_LAYERS,
                SOLID_INFILL_ROLE,
                PERCENT_100_NOZZLE_04,
                PERCENT_100_NOZZLE_06,
            ),
        ],
    });
    run(OptionPair {
        name: "anti-map",
        setup: &[
            INITIAL_ZERO,
            NOZZLES_46,
            process(
                r#""outer_wall_filament_id": "0""#,
                r#""outer_wall_filament_id": "1""#,
            ),
            process(
                r#""inner_wall_filament_id": "0""#,
                r#""inner_wall_filament_id": "1""#,
            ),
            process(
                r#""internal_solid_filament_id": "0""#,
                r#""internal_solid_filament_id": "1""#,
            ),
            process(
                "\"filament_map\": [\r\n\t\t\"1\",\r\n\t\t\"1\"\r\n\t]",
                "\"filament_map\": [\r\n\t\t\"1\",\r\n\t\t\"2\"\r\n\t]",
            ),
        ],
        delta: process(
            "\"filament_map\": [\r\n\t\t\"1\",\r\n\t\t\"2\"\r\n\t]",
            "\"filament_map\": [\r\n\t\t\"2\",\r\n\t\t\"1\"\r\n\t]",
        ),
        key: Key::FilamentMap,
        raw: [Value::IntPair([1, 2]), Value::IntPair([2, 1])],
        effective: [Value::IntPair([1, 2]), Value::IntPair([2, 1])],
        changes: &[],
    });
}

fn selector_pair(name: &str, key: Key, [zero, one, two]: [&'static str; 3], changes: &[Change]) {
    run(OptionPair {
        name,
        setup: &[INITIAL_ZERO, NOZZLES_46, process(zero, one)],
        delta: process(one, two),
        key,
        raw: [Value::Int(1), Value::Int(2)],
        effective: [Value::Int(1), Value::Int(2)],
        changes,
    });
}
