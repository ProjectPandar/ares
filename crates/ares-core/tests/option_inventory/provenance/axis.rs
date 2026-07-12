use std::collections::BTreeMap;

use super::super::OptionInventoryRow;

pub(super) fn verify_axis_defaults(source: &str, rows: &[OptionInventoryRow]) {
    let derived = derive_axis_defaults(source).unwrap();
    assert_eq!(derived.len(), 12);
    let artifact = rows
        .iter()
        .filter(|row| is_axis_key(&row.key))
        .map(|row| (row.key.clone(), row.default_serialized.clone()))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(artifact.len(), 12);
    assert_eq!(derived, artifact);

    let speed = "ConfigOptionFloats(axis.max_feedrate)";
    let acceleration = "ConfigOptionFloats(axis.max_acceleration)";
    let swapped = replace_once(source, speed, "ConfigOptionFloats(axis.__swap__)");
    let swapped = replace_once(&swapped, acceleration, speed);
    let swapped = replace_once(&swapped, "ConfigOptionFloats(axis.__swap__)", acceleration);
    assert!(derive_axis_defaults(&swapped).is_err());

    let mutated = replace_once(source, "{  0.2,  0.4 }", "{  9.9,  9.9 }");
    let mutated = derive_axis_defaults(&mutated).unwrap();
    assert_ne!(mutated, artifact);
    assert_eq!(mutated["machine_max_jerk_z"], "9.9,9.9");

    let declaration_swapped = replace_once(
        source,
        "std::vector<double> max_feedrate;",
        "std::vector<double> __swap__;",
    );
    let declaration_swapped = replace_once(
        &declaration_swapped,
        "std::vector<double> max_acceleration;",
        "std::vector<double> max_feedrate;",
    );
    let declaration_swapped = replace_once(
        &declaration_swapped,
        "std::vector<double> __swap__;",
        "std::vector<double> max_acceleration;",
    );
    assert!(derive_axis_defaults(&declaration_swapped).is_err());
}

fn derive_axis_defaults(source: &str) -> Result<BTreeMap<String, String>, &'static str> {
    let table_start = source
        .find("std::vector<AxisDefault> axes")
        .ok_or("missing axis table")?;
    let structure_start = source[..table_start]
        .rfind("struct AxisDefault")
        .ok_or("missing AxisDefault structure")?;
    let structure_end = source[structure_start..]
        .find("};")
        .map(|offset| structure_start + offset)
        .ok_or("missing AxisDefault structure end")?;
    let members = source[structure_start..structure_end]
        .lines()
        .filter(|line| line.contains("std::vector<double>"))
        .filter_map(|line| line.split_whitespace().last())
        .map(|member| member.trim_end_matches(';'))
        .collect::<Vec<_>>();
    if members != ["max_feedrate", "max_acceleration", "max_jerk"] {
        return Err("wrong AxisDefault member order");
    }
    let table_end = source[table_start..]
        .find("\n        };")
        .map(|offset| table_start + offset)
        .ok_or("missing axis table end")?;
    let table = &source[table_start..table_end];

    let loop_start = source[table_end..]
        .find("for (const AxisDefault &axis : axes)")
        .map(|offset| table_end + offset)
        .ok_or("missing axis loop")?;
    let loop_end = source[loop_start..]
        .find("\n        }\n")
        .map(|offset| loop_start + offset)
        .ok_or("missing axis loop end")?;
    let loop_body = &source[loop_start..loop_end];
    verify_bindings(loop_body)?;

    let mut defaults = BTreeMap::new();
    for axis in ["x", "y", "z", "e"] {
        let line = table
            .lines()
            .find(|line| line.contains(&format!(r#"{{ "{axis}""#)))
            .ok_or("missing axis row")?;
        let groups = nested_groups(line);
        if groups.len() != 3 {
            return Err("invalid axis row");
        }
        for ((prefix, _), values) in bindings().into_iter().zip(groups) {
            defaults.insert(format!("{prefix}{axis}"), numeric_list(values)?);
        }
    }
    Ok(defaults)
}

fn verify_bindings(loop_body: &str) -> Result<(), &'static str> {
    let bindings = bindings();
    let starts = bindings
        .iter()
        .map(|(prefix, _)| {
            loop_body.find(&format!(r#"this->add("{prefix}" + axis.name, coFloats)"#))
        })
        .collect::<Option<Vec<_>>>()
        .ok_or("missing axis registration")?;
    if !starts.windows(2).all(|pair| pair[0] < pair[1]) {
        return Err("axis registrations out of order");
    }
    for (index, ((_, member), start)) in bindings.iter().zip(&starts).enumerate() {
        let end = starts.get(index + 1).copied().unwrap_or(loop_body.len());
        let block = &loop_body[*start..end];
        if !block.contains(&format!("ConfigOptionFloats(axis.{member})")) {
            return Err("axis registration has wrong default member");
        }
        if bindings.iter().any(|(_, other)| {
            other != member && block.contains(&format!("ConfigOptionFloats(axis.{other})"))
        }) {
            return Err("axis registration crosses default members");
        }
    }
    Ok(())
}

fn bindings() -> [(&'static str, &'static str); 3] {
    [
        ("machine_max_speed_", "max_feedrate"),
        ("machine_max_acceleration_", "max_acceleration"),
        ("machine_max_jerk_", "max_jerk"),
    ]
}

fn nested_groups(line: &str) -> Vec<&str> {
    let mut groups = Vec::new();
    let mut depth = 0;
    let mut start = 0;
    for (index, byte) in line.bytes().enumerate() {
        match byte {
            b'{' => {
                depth += 1;
                if depth == 2 {
                    start = index + 1;
                }
            }
            b'}' => {
                if depth == 2 {
                    groups.push(&line[start..index]);
                }
                depth -= 1;
            }
            _ => {}
        }
    }
    groups
}

fn numeric_list(values: &str) -> Result<String, &'static str> {
    values
        .split(',')
        .map(|value| {
            value
                .trim()
                .trim_end_matches('f')
                .parse::<f64>()
                .map(|value| value.to_string())
                .map_err(|_| "invalid axis default")
        })
        .collect::<Result<Vec<_>, _>>()
        .map(|values| values.join(","))
}

fn is_axis_key(key: &str) -> bool {
    bindings().iter().any(|(prefix, _)| {
        key.strip_prefix(prefix)
            .is_some_and(|axis| matches!(axis, "x" | "y" | "z" | "e"))
    })
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
