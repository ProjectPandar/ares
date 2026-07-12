use super::super::OptionInventoryRow;

pub(super) fn verify_nozzle_type_default(
    print_config: &str,
    config: &str,
    common_defs: &str,
    rows: &[OptionInventoryRow],
) {
    assert!(common_defs.contains("ntUndefine = 0"));
    assert!(config.lines().any(|line| {
        line.contains("static int")
            && line.contains("nil_value()")
            && line.contains("std::numeric_limits<int>::max()")
    }));
    assert!(config.contains("if (v == nil_value())"));
    assert!(config.contains("ss << \"nil\""));

    let artifact = &rows
        .iter()
        .find(|row| row.key == "nozzle_type")
        .unwrap()
        .default_serialized;
    assert_eq!(derive_nozzle_default(print_config).unwrap(), *artifact);

    let map_mutation = replace_once(
        print_config,
        r#"{ "undefine",       int(NozzleType::ntUndefine) }"#,
        r#"{ "WRONG",          int(NozzleType::ntUndefine) }"#,
    );
    assert_eq!(derive_nozzle_default(&map_mutation).unwrap(), "WRONG");

    let nil_mutation = replace_once(
        print_config,
        "ConfigOptionEnumsGenericNullable({ ntUndefine })",
        "ConfigOptionEnumsGenericNullable({ ConfigOptionEnumsGenericNullable::nil_value() })",
    );
    assert_eq!(derive_nozzle_default(&nil_mutation).unwrap(), "nil");
    assert_ne!(derive_nozzle_default(&nil_mutation).unwrap(), *artifact);
}

fn derive_nozzle_default(source: &str) -> Result<String, &'static str> {
    let start = source
        .find(r#"this->add("nozzle_type", coEnums)"#)
        .ok_or("missing nozzle_type definition")?;
    let end = source[start + 1..]
        .find("this->add(")
        .map(|offset| start + 1 + offset)
        .ok_or("missing definition end")?;
    let definition = &source[start..end];
    let expression = definition
        .split("set_default_value(new ")
        .nth(1)
        .and_then(|tail| tail.split(");").next())
        .ok_or("missing nozzle_type default")?;
    if expression.contains("nil_value()") {
        return Ok("nil".to_owned());
    }
    if !expression.contains("ntUndefine") {
        return Err("unexpected nozzle_type default");
    }

    let map_start = source
        .find("s_keys_map_NozzleType")
        .ok_or("missing NozzleType map")?;
    let map_end = source[map_start..]
        .find("CONFIG_OPTION_ENUM_DEFINE_STATIC_MAPS(NozzleType)")
        .map(|offset| map_start + offset)
        .ok_or("missing NozzleType map end")?;
    let line = source[map_start..map_end]
        .lines()
        .find(|line| line.contains("NozzleType::ntUndefine"))
        .ok_or("missing ntUndefine map entry")?;
    let opening = line.find('"').ok_or("missing enum token")? + 1;
    let closing = line[opening..].find('"').ok_or("missing enum token end")? + opening;
    Ok(line[opening..closing].to_owned())
}

fn replace_once(source: &str, needle: &str, replacement: &str) -> String {
    let first = source.find(needle).unwrap();
    assert_eq!(source.rfind(needle), Some(first));
    format!(
        "{}{}{}",
        &source[..first],
        replacement,
        &source[first + needle.len()..]
    )
}
