use super::*;

#[test]
fn config_parses_scalars_and_vectors() {
    let config = Config::from_block(b"; one = 2\n; values = 1,2\n; names = PLA;PETG\n");
    assert_eq!(config.get("one").unwrap().as_number(), Some(2.0));
    assert_eq!(
        config.get("values").unwrap().index(1).unwrap().as_number(),
        Some(2.0)
    );
    assert_eq!(
        config.get("names").unwrap().index(1).unwrap().as_string(),
        "PETG"
    );
    assert_eq!(config.get("values_1").unwrap().as_number(), Some(2.0));
    assert_eq!(
        config.get("values").unwrap().index(7).unwrap().as_number(),
        Some(1.0)
    );
    assert_eq!(config.get("names_0").unwrap().as_string(), "PLA");
}

#[test]
fn scalar_numbers_use_six_significant_digits() {
    assert_eq!(Value::number(523.843179).as_string(), "523.843");
    assert_eq!(Value::number(4.3653598).as_string(), "4.36536");
    assert_eq!(Value::number(0.022).as_string(), "0.022");
}
