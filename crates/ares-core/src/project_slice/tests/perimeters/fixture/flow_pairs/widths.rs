use super::*;

#[test]
fn task22n_flow_option_pairs_cover_width_sources() {
    run(OptionPair {
        name: "initial",
        setup: &[],
        delta: INITIAL_ZERO,
        key: Key::Initial,
        raw: [absolute(0.5), absolute(0.0)],
        effective: [absolute(0.5), absolute(0.0)],
        changes: &[
            change(
                FIRST_LAYER,
                INTERNAL_PERIMETER_ROLE,
                INITIAL_ABSOLUTE_050_NOZZLE_04,
                ABSOLUTE_045_NOZZLE_04,
            ),
            change(
                FIRST_LAYER,
                EXTERNAL_PERIMETER_ROLE | SOLID_INFILL_ROLE,
                INITIAL_ABSOLUTE_050_NOZZLE_04,
                ABSOLUTE_042_NOZZLE_04,
            ),
        ],
    });
    run(OptionPair {
        name: "outer-percent",
        setup: &[INITIAL_ZERO, NOZZLES_46, OUTER_TWO],
        delta: process(
            r#""outer_wall_line_width": "0.42""#,
            r#""outer_wall_line_width": "125%""#,
        ),
        key: Key::OuterWidth,
        raw: [absolute(0.42), percent(125.0)],
        effective: [absolute(0.42), percent(125.0)],
        changes: &[change(
            BOTH_LAYERS,
            EXTERNAL_PERIMETER_ROLE,
            ABSOLUTE_042_NOZZLE_06,
            PERCENT_125_NOZZLE_06,
        )],
    });
    run(OptionPair {
        name: "inner-percent",
        setup: &[INITIAL_ZERO, NOZZLES_46, INNER_TWO],
        delta: process(
            r#""inner_wall_line_width": "0.45""#,
            r#""inner_wall_line_width": "110%""#,
        ),
        key: Key::InnerWidth,
        raw: [absolute(0.45), percent(110.0)],
        effective: [absolute(0.45), percent(110.0)],
        changes: &[change(
            BOTH_LAYERS,
            INTERNAL_PERIMETER_ROLE,
            ABSOLUTE_045_NOZZLE_06,
            PERCENT_110_NOZZLE_06,
        )],
    });
    run(OptionPair {
        name: "solid-percent",
        setup: &[INITIAL_ZERO, NOZZLES_46, SOLID_TWO],
        delta: process(
            r#""internal_solid_infill_line_width": "0.42""#,
            r#""internal_solid_infill_line_width": "100%""#,
        ),
        key: Key::SolidWidth,
        raw: [absolute(0.42), percent(100.0)],
        effective: [absolute(0.42), percent(100.0)],
        changes: &[change(
            BOTH_LAYERS,
            SOLID_INFILL_ROLE,
            ABSOLUTE_042_NOZZLE_06,
            PERCENT_100_NOZZLE_06,
        )],
    });
    run(OptionPair {
        name: "object-fallback",
        setup: &[
            INITIAL_ZERO,
            process(
                r#""outer_wall_line_width": "0.42""#,
                r#""outer_wall_line_width": "0""#,
            ),
            process(
                r#""inner_wall_line_width": "0.45""#,
                r#""inner_wall_line_width": "0""#,
            ),
            process(
                r#""internal_solid_infill_line_width": "0.42""#,
                r#""internal_solid_infill_line_width": "0""#,
            ),
        ],
        delta: process(r#""line_width": "0.42""#, r#""line_width": "0.52""#),
        key: Key::ObjectWidth,
        raw: [absolute(0.42), absolute(0.52)],
        effective: [absolute(0.42), absolute(0.52)],
        changes: &[change(
            BOTH_LAYERS,
            INTERNAL_PERIMETER_ROLE | EXTERNAL_PERIMETER_ROLE | SOLID_INFILL_ROLE,
            ABSOLUTE_042_NOZZLE_04,
            OBJECT_ABSOLUTE_052_NOZZLE_04,
        )],
    });
}
