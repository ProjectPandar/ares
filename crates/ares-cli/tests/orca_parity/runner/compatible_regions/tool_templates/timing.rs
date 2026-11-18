use serde_json::json;

#[test]
fn mapped_tool_load_and_change_delays_match_native_total_deltas() {
    if !crate::runner::reference_available() {
        return;
    }
    let (native_base, ares_base) = super::outputs("", "", false, &[]);
    let (native, ares) = super::outputs(
        "",
        "",
        false,
        &[
            ("machine_load_filament_time", json!("31")),
            ("machine_unload_filament_time", json!("17")),
            ("machine_tool_change_time", json!("23")),
        ],
    );
    let expected = total_seconds(&native) - total_seconds(&native_base);
    assert!(
        expected > 0,
        "native delays must affect the complete project"
    );
    assert_eq!(total_seconds(&ares) - total_seconds(&ares_base), expected);
}

#[test]
fn shared_hotend_unload_and_load_delays_match_native_total_deltas() {
    if !crate::runner::reference_available() {
        return;
    }
    for end in ["", "M702 MACRO", "M702 C", "M702\u{b}C"] {
        let mapping = ("filament_map", json!(["1", "1"]));
        let (native_base, ares_base) =
            super::outputs("", end, false, std::slice::from_ref(&mapping));
        let (native, ares) = super::outputs(
            "",
            end,
            false,
            &[
                mapping,
                ("machine_load_filament_time", json!("31")),
                ("machine_unload_filament_time", json!("17")),
                ("machine_tool_change_time", json!("23")),
            ],
        );
        let expected = total_seconds(&native) - total_seconds(&native_base);
        assert!(
            expected > 0,
            "native delays must affect the complete project"
        );
        assert_eq!(
            total_seconds(&ares) - total_seconds(&ares_base),
            expected,
            "{end:?}"
        );
    }
}

fn total_seconds(bytes: &[u8]) -> i64 {
    let text = std::str::from_utf8(bytes).unwrap();
    let time = text
        .lines()
        .find_map(|line| line.strip_prefix("; estimated printing time (normal mode) = "))
        .unwrap();
    time.split_ascii_whitespace()
        .map(|part| {
            let (number, unit) = part.split_at(part.len() - 1);
            let multiplier = match unit {
                "d" => 86_400,
                "h" => 3_600,
                "m" => 60,
                "s" => 1,
                _ => panic!("unknown duration unit: {unit}"),
            };
            number.parse::<i64>().unwrap() * multiplier
        })
        .sum()
}
