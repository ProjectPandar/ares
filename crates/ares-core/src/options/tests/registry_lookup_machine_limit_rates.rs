#[test]
fn exposes_machine_limit_rate_option_definition_lookup() {
    for (key, kind, default_value, source_fragments) in [
        (
            "machine_max_acceleration_extruding",
            crate::OptionValueKind::Floats,
            "1500,1250",
            &["PrintConfig.hpp:1260", "PrintConfig.cpp:4480-4491"][..],
        ),
        (
            "machine_max_acceleration_retracting",
            crate::OptionValueKind::Floats,
            "1500,1250",
            &["PrintConfig.hpp:1261", "PrintConfig.cpp:4494-4503"][..],
        ),
        (
            "machine_max_acceleration_travel",
            crate::OptionValueKind::Floats,
            "0,0",
            &["PrintConfig.hpp:1262", "PrintConfig.cpp:4505-4514"][..],
        ),
        (
            "machine_max_jerk_e",
            crate::OptionValueKind::Floats,
            "2.5,2.5",
            &[
                "PrintConfig.hpp:1268",
                "PrintConfig.cpp:4389",
                "PrintConfig.cpp:4429-4446",
            ][..],
        ),
        (
            "machine_max_jerk_x",
            crate::OptionValueKind::Floats,
            "10,10",
            &[
                "PrintConfig.hpp:1265",
                "PrintConfig.cpp:4386",
                "PrintConfig.cpp:4429-4446",
            ][..],
        ),
        (
            "machine_max_jerk_y",
            crate::OptionValueKind::Floats,
            "10,10",
            &[
                "PrintConfig.hpp:1266",
                "PrintConfig.cpp:4387",
                "PrintConfig.cpp:4429-4446",
            ][..],
        ),
        (
            "machine_max_jerk_z",
            crate::OptionValueKind::Floats,
            "0.2,0.4",
            &[
                "PrintConfig.hpp:1267",
                "PrintConfig.cpp:4388",
                "PrintConfig.cpp:4429-4446",
            ][..],
        ),
        (
            "machine_max_junction_deviation",
            crate::OptionValueKind::Floats,
            "0.01",
            &["PrintConfig.hpp:1270", "PrintConfig.cpp:4449-4458"][..],
        ),
        (
            "machine_min_extruding_rate",
            crate::OptionValueKind::Floats,
            "0,0",
            &["PrintConfig.hpp:1274", "PrintConfig.cpp:4460-4468"][..],
        ),
        (
            "machine_min_travel_rate",
            crate::OptionValueKind::Floats,
            "0,0",
            &["PrintConfig.hpp:1272", "PrintConfig.cpp:4470-4478"][..],
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
        for fragment in source_fragments {
            assert!(definition.source.contains(fragment));
        }
    }
}
