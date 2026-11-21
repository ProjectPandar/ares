use crate::runner::native_stream_case;

#[test]
fn organic_prints_support_from_vertical_enforcer_paint() {
    check(
        "organic",
        include_bytes!("../../../../tests/parity/organic-vertical-enforcer/organic.3mf"),
        "a634e1916c54b0c06791c2b9ea7d82f0cfbc506a53563e443a507269f2bd481f",
    );
}

#[test]
fn default_tree_prints_support_from_vertical_enforcer_paint() {
    check(
        "default",
        include_bytes!("../../../../tests/parity/organic-vertical-enforcer/default.3mf"),
        "44ccafb28eff4effe208d2a186a6f3e338333413dabb161167cbc17a1a283713",
    );
}

#[test]
fn slim_tree_preserves_its_vertical_enforcer_output() {
    check(
        "slim",
        include_bytes!("../../../../tests/parity/organic-vertical-enforcer/slim.3mf"),
        "d47c53dc8cd9d8fbcbdf0dcf915e3df6198f5c15e5f1123115342e95bf410083",
    );
}

fn check(label: &str, project: &[u8], sha: &str) {
    native_stream_case::check(
        (
            "organic-vertical-enforcer",
            &["Support/TreeSupport3D.cpp:3542-3556::generate_support_areas"],
        ),
        label,
        project,
        sha,
        (false, |reference, _| {
            assert!(has_support_extrusion(&reference.native));
        }),
    );
}

fn has_support_extrusion(gcode: &str) -> bool {
    let mut support = false;
    gcode.lines().any(|line| {
        if let Some(role) = line.strip_prefix("; FEATURE: ") {
            support = matches!(role, "Support" | "Support interface");
        }
        let words = line.split(';').next().unwrap().split_ascii_whitespace();
        support
            && line.starts_with("G1 ")
            && words.clone().any(|word| word.starts_with(['X', 'Y']))
            && words
                .filter_map(|word| word.strip_prefix('E'))
                .any(|value| value.parse::<f64>().is_ok_and(|extrusion| extrusion > 0.0))
    })
}
