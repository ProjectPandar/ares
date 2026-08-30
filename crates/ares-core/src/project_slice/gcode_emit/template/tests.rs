use super::*;
use crate::project_slice::gcode_emit::value::Config;

#[test]
fn renderer_distinguishes_option_and_expression_booleans() {
    let mut config = Config::default();
    config.insert("enabled", super::super::value::Value::option_bool(true));
    config.insert("disabled", super::super::value::Value::option_bool(false));

    assert_eq!(
        render("{enabled} {disabled} {enabled == 1}", &mut config).unwrap(),
        "1 0 true"
    );
}

#[test]
fn renderer_selects_nested_branches_and_replaces_values() {
    let mut config = Config::from_block(b"; enabled = 1\n; n = 2\n");
    let template = "{if enabled}\nA [n]\n{if n > 1}\nB\n{endif}\n{else}\nC\n{endif}\n";
    assert_eq!(render(template, &mut config).unwrap(), "\nA 2\n\nB\n\n\n");
}

#[test]
fn renderer_coalesces_multiline_conditions_and_selects_else() {
    let mut config = Config::from_block(b"; enabled = 0\n; n = 2\n");
    let template = "{if enabled == 1 ||\n n == 3}\nA\n{else}\nB [n]\n{endif}\n";
    assert_eq!(render(template, &mut config).unwrap(), "\nB 2\n\n");
}

#[test]
fn renderer_keeps_closing_blank_only_for_selected_single_branch() {
    let mut config = Config::from_block(b"; enabled = 1\n");
    let template = "{if enabled}\nA\n{endif}\n{if !enabled}\nB\n{endif}\n";
    assert_eq!(render(template, &mut config).unwrap(), "\nA\n\n\n");
}

#[test]
fn renderer_keeps_only_the_closing_newline_when_no_branch_matches() {
    let mut config = Config::from_block(b"; enabled = 0\n");
    let template = "{if enabled}\nA\n{elsif enabled == 2}\nB\n{endif}\n";

    assert_eq!(render(template, &mut config).unwrap(), "\n");
}

#[test]
fn renderer_accepts_spaces_inside_directive_braces() {
    let mut config = Config::from_block(b"; enabled = 1\n");
    let template = "{ if enabled }yes{ else }no{ endif }";

    assert_eq!(render(template, &mut config).unwrap(), "yes");
}

#[test]
fn renderer_supports_inline_conditionals_on_one_line() {
    let mut config = Config::from_block(b"; lift = 1\n; z = 10\n; top = 250\n; off = 0.1\n");
    let template = "{if lift}G1 Z{off+min(z+2, top)} F600 ; lift{endif}\nG1 X5\n";
    assert_eq!(
        render(template, &mut config).unwrap(),
        "G1 Z12.1 F600 ; lift\nG1 X5\n"
    );
}

#[test]
fn renderer_drops_unselected_inline_branch_and_keeps_line_newline() {
    let mut config = Config::from_block(b"; lift = 0\n");
    let template = "{if lift}G1 Z5{endif}\nG1 X5\n";
    assert_eq!(render(template, &mut config).unwrap(), "\nG1 X5\n");
}

#[test]
fn renderer_selects_inline_else_and_trailing_segment() {
    let mut config = Config::from_block(b"; park = 1\n; width = 220\n");
    let template = "G1 X{if park}5{else}7{endif} Y{width*0.8} F600\n";
    let rendered = render(template, &mut config).unwrap();
    assert!(rendered.starts_with("G1 X5 Y"), "{rendered}");
    assert!(rendered.ends_with(" F600\n"), "{rendered}");
}

#[test]
fn assignment_at_nonzero_offset_preserves_following_text() {
    let mut config = Config::from_block(b"; position = 0,0,0\n");
    let template = "G1 E-1\n{position[0] = 2}\nG92 E0\n";

    assert_eq!(render(template, &mut config).unwrap(), "G1 E-1\n\nG92 E0\n");
    assert_eq!(
        config
            .get("position")
            .unwrap()
            .index(0)
            .unwrap()
            .as_number(),
        Some(2.0)
    );
}
