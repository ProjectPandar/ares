use super::outputs;

fn footer(bytes: &[u8]) -> String {
    let text = std::str::from_utf8(bytes).unwrap();
    let start = text.find("; filament used [mm] = ").unwrap();
    let end = text[start..].find("; total layers count = ").unwrap() + start;
    text[start..end].to_owned()
}

#[test]
fn two_nozzle_footer_preserves_each_filament_usage_and_totals() {
    if !crate::runner::reference_available() {
        return;
    }
    let (reference, actual) = outputs("", "", false, &[]);
    let expected = footer(&reference);
    let tools = std::str::from_utf8(&reference)
        .unwrap()
        .lines()
        .filter_map(|line| line.strip_prefix('T')?.parse::<u32>().ok())
        .collect::<std::collections::BTreeSet<_>>();
    assert!(tools.len() > 1);
    let usage = expected
        .lines()
        .next()
        .unwrap()
        .strip_prefix("; filament used [mm] = ")
        .unwrap()
        .split(", ")
        .map(|value| value.parse::<f64>().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(usage.len(), tools.len());
    assert!(usage.iter().all(|&value| value > 0.0));
    assert!(expected.contains("; filament used [g] = "));
    let changes = expected
        .lines()
        .find_map(|line| line.strip_prefix("; total filament change = "))
        .unwrap()
        .parse::<usize>()
        .unwrap();
    assert!(changes > 0);
    assert_eq!(footer(&actual), expected);
}

#[test]
fn two_nozzle_footer_uses_each_filaments_properties_and_zero_cost_slot() {
    if !crate::runner::reference_available() {
        return;
    }
    let (reference, actual) = super::fixture::configured(
        "",
        "",
        false,
        super::fixture::Overrides {
            filaments: &[
                ("filament_diameter", ["1.75", "2.85"]),
                ("filament_density", ["1.11", "1.29"]),
                ("filament_cost", ["0", "32"]),
            ],
            ..Default::default()
        },
    );
    let expected = footer(&reference);
    let cost = expected
        .lines()
        .find_map(|line| line.strip_prefix("; filament cost = "))
        .unwrap();
    let (first, second) = cost.split_once(", ").unwrap();
    assert_eq!(first, "0.00");
    assert!(second.parse::<f64>().unwrap() > 0.0);
    assert_eq!(footer(&actual), expected);
}
