use super::*;

#[test]
fn disabled_line_numbering_returns_input_unchanged() {
    let input = "G90\n;comment\nM2\n".to_owned();

    assert_eq!(add_line_numbers_if_enabled(input.clone(), false), input);
}

#[test]
fn enabled_line_numbering_prefixes_every_line() {
    let input = "G90\n;comment\nM2\n".to_owned();

    assert_eq!(
        add_line_numbers_if_enabled(input, true),
        "N1 G90\nN2 ;comment\nN3 M2\n"
    );
}
