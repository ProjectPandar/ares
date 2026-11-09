//! GCode.cpp:4645-4675,5145-5202: real timelapse template branch witnesses.
use super::*;
use serde_json::json;

#[path = "timelapse_coverage/input.rs"]
pub(super) mod input;

#[test]
fn timelapse_type_conditional_template_and_empty_controls() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = parity::artifacts::root_from_env().unwrap();
    let evidence = root.join("timelapse-type-bounded.json");
    let mut proofs = Vec::new();
    for active in [true, false] {
        for mode in ["0", "1"] {
            let label = format!("timelapse/template/{active}/{mode}");
            let exported = input::export(&runner, &json!(mode), active, &label).unwrap();
            let sliced = runner.slice_case(&exported).unwrap();
            let native = std::str::from_utf8(&sliced.reference).unwrap();
            let markers = branch_lines(native, mode, active);
            let outcome = parity::compare_case(&sliced);
            proofs.push(json!({"label":label,"mode":mode,"template_active":active,
                "marker_lines":markers,"full_output_status":outcome.status,
                "detail":outcome.detail,"artifacts":outcome.artifacts}));
            parity::artifacts::overwrite(&evidence, &serde_json::to_vec_pretty(&proofs).unwrap())
                .unwrap();
            assert_eq!(
                outcome.status, "PASS",
                "{label}: {}; {root:?}",
                outcome.detail
            );
        }
    }
}

fn branch_lines<'a>(native: &'a str, mode: &str, active: bool) -> Vec<(usize, &'a str)> {
    let expected = if mode == "0" {
        ";W6_TL_TRAD "
    } else {
        ";W6_TL_SMOOTH "
    };
    let other = if mode == "0" {
        ";W6_TL_SMOOTH "
    } else {
        ";W6_TL_TRAD "
    };
    let markers = native
        .lines()
        .enumerate()
        .filter(|(_, line)| line.starts_with(expected))
        .collect::<Vec<_>>();
    assert_eq!(
        !markers.is_empty(),
        active,
        "timelapse mode={mode}, active={active}"
    );
    assert!(!native.lines().any(|line| line.starts_with(other)));
    for (_, line) in &markers {
        assert!(
            !line.contains('{') && !line.contains('}'),
            "unexpanded {line}"
        );
        let fields = line.split_whitespace().collect::<Vec<_>>();
        assert_eq!(fields.len(), 4, "{line}");
        for (field, prefix) in fields[1..].iter().zip(["layer=", "z=", "max="]) {
            assert!(
                field
                    .strip_prefix(prefix)
                    .unwrap()
                    .parse::<f64>()
                    .unwrap()
                    .is_finite(),
                "{line}"
            );
        }
    }
    markers
}
