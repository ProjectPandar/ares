use super::{assert_project_value, assert_rejected, changed_case};

struct Domain {
    key: &'static str,
    tokens: &'static [&'static str],
    nullable: bool,
}

const DOMAINS: &[Domain] = &[
    Domain { key: "default_nozzle_volume_type", tokens: &["Standard", "High Flow"], nullable: false },
    Domain { key: "extruder_type", tokens: &["Direct Drive", "Bowden"], nullable: false },
    Domain {
        key: "filament_retract_lift_enforce",
        tokens: &["All Surfaces", "Top Only", "Bottom Only", "Top and Bottom"],
        nullable: true,
    },
    Domain {
        key: "filament_z_hop_types",
        tokens: &["Auto Lift", "Normal Lift", "Slope Lift", "Spiral Lift"],
        nullable: true,
    },
    Domain {
        key: "nozzle_type",
        tokens: &["undefine", "hardened_steel", "stainless_steel", "tungsten_carbide", "brass"],
        nullable: true,
    },
    Domain { key: "nozzle_volume_type", tokens: &["Standard", "High Flow"], nullable: false },
    Domain { key: "overhang_fan_threshold", tokens: &["0%", "10%", "25%", "50%", "75%", "95%"], nullable: false },
    Domain {
        key: "retract_lift_enforce",
        tokens: &["All Surfaces", "Top Only", "Bottom Only", "Top and Bottom"],
        nullable: false,
    },
    Domain {
        key: "z_hop_types",
        tokens: &["Auto Lift", "Normal Lift", "Slope Lift", "Spiral Lift"],
        nullable: false,
    },
];

#[test]
fn every_typed_enum_vector_obeys_fixed_wire_rules() {
    assert_eq!(DOMAINS.len(), 9);
    assert_eq!(DOMAINS.iter().filter(|domain| domain.nullable).count(), 3);

    for domain in DOMAINS {
        assert_project_value(domain.key, "");
        for token in domain.tokens {
            assert_project_value(domain.key, token);
        }

        let joined = domain.tokens.join(",");
        assert_project_value(domain.key, &joined);
        let padded = domain
            .tokens
            .iter()
            .map(|token| format!(" {token} "))
            .collect::<Vec<_>>()
            .join(",");
        assert_project_value(domain.key, &padded);
        assert_project_value(domain.key, &format!("{joined},"));
        assert_rejected(domain.key, &format!("{},__invalid__", domain.tokens[0]));
        if domain.tokens[0]
            .chars()
            .any(|character| character.is_ascii_alphabetic())
        {
            assert_rejected(domain.key, &changed_case(domain.tokens[0]));
        }

        if domain.nullable {
            assert_project_value(domain.key, "nil");
            assert_project_value(domain.key, &format!("{}, nil", domain.tokens[0]));
        } else {
            assert_rejected(domain.key, "nil");
        }
    }
}
