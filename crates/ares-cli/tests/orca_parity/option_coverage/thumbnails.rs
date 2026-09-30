use super::*;
use crate::runner::CaseInputs;
use serde_json::json;

#[test]
fn thumbnail_probes_use_explicit_formats_and_cover_empty_and_multiple_definitions() {
    let Some(plans) = domains::try_load(&runner::repo_root()) else {
        return;
    };
    let plan = plans.iter().find(|plan| plan.key == "thumbnails").unwrap();
    assert_eq!(
        plan.cases
            .iter()
            .map(|case| case.value.clone())
            .collect::<Vec<_>>(),
        [
            "300x300/PNG",
            "300x300/JPG",
            "300x300/QOI",
            "300x300/BTT_TFT",
            "300x300/COLPIC",
            "",
            "48x48/PNG, 300x300/JPG"
        ]
        .map(|value| Some(json!(value)))
    );
}

#[test]
fn orca_parity_thumbnail_definitions_and_default_formats_match_complete_output() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let plan = domains::load(&runner::repo_root())
        .into_iter()
        .find(|plan| plan.key == "thumbnails")
        .unwrap();
    let profiles = VendorProfiles::load(&profiles_root(), "Creality").unwrap();
    let selection =
        parity::select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    let mut cases: Vec<_> = plan
        .cases
        .iter()
        .map(|case| {
            let value = case.value.as_ref().unwrap().as_str().unwrap();
            (case.label.as_str(), value, "PNG", value)
        })
        .collect();
    cases.extend([
        ("short-png", "123x45", "PNG", "123x45/PNG"),
        ("short-jpg", "123x45", "JPG", "123x45/JPG"),
        ("short-qoi", "123x45", "QOI", "123x45/QOI"),
        ("short-btt", "123x45", "BTT_TFT", "123x45/BTT_TFT"),
        ("short-colpic", "123x45", "COLPIC", "123x45/COLPIC"),
    ]);
    for (label, requested, format, expected) in cases {
        let mut machine = profiles.machine(&selection.printer).unwrap();
        let mut process = profiles.process(&selection.process).unwrap();
        parity::normalize_process_defaults(&machine, &mut process);
        let mut filaments = selection
            .filaments
            .iter()
            .map(|name| profiles.filament(name).unwrap())
            .collect::<Vec<_>>();
        parity::normalize_filament_defaults(&mut filaments);
        inject_case(
            &plan,
            &domains::OptionCase {
                label: label.to_owned(),
                value: Some(json!(requested)),
            },
            &mut machine,
            &mut process,
            &mut filaments,
        );
        machine.insert("thumbnails_format".into(), json!(format));
        process.insert("thumbnails_format".into(), json!(format));
        let exported = runner
            .export_case(
                &CaseInputs {
                    label: &format!("thumbnail-definitions/{label}"),
                    machine: &machine,
                    process: &process,
                    filaments: &filaments,
                },
                &parity::smoke_overrides(),
                &runner::repo_root().join("tests/parity/cube10.stl"),
            )
            .unwrap();
        let proof =
            runner::application::verify_value(&exported.project, "thumbnails", &json!(expected))
                .unwrap();
        assert_eq!(proof["effective"], expected);
        if requested != expected {
            assert!(
                runner::application::verify_value(
                    &exported.project,
                    "thumbnails",
                    &json!(requested),
                )
                .is_err(),
                "shorthand must not silently pass raw equality"
            );
        }
        let sliced = runner.slice_case(&exported).unwrap();
        let reference = std::str::from_utf8(&sliced.reference).unwrap();
        assert!(
            reference
                .lines()
                .any(|line| line == format!("; thumbnails = {expected}"))
        );
        let outcome = parity::compare_case(&sliced);
        assert_eq!(outcome.status, "PASS", "{label}: {}", outcome.detail);
    }
}
