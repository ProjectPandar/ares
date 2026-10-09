//! PrintConfig.cpp:2215-2248,10436-10447; Print.cpp:613-930,1284-1306.
//! Bounded G2 witnesses; no global physical upper-bound claim.

use super::super::{OptionOutcome, domains, inject_case, profiles_root};
use crate::{
    presets::VendorProfiles,
    runner::{self, CaseInputs, OrcaRunner},
};
use serde_json::json;

#[path = "clearance/geometry.rs"]
mod geometry;

pub(in crate::option_coverage) fn admit(key: &str, cases: &mut Vec<domains::OptionCase>) -> bool {
    if !matches!(
        key,
        "extruder_clearance_height_to_lid"
            | "extruder_clearance_height_to_rod"
            | "extruder_clearance_radius"
            | "nozzle_height"
    ) {
        return false;
    }
    // PrintConfig::validate's open-positive loaded domain plus the named
    // Print::validate consumer witnesses, not the inherited FLT_MAX ceiling.
    cases.push(domains::OptionCase {
        label: "near-zero".to_owned(),
        value: Some(json!("0.00001")),
    });
    true
}

pub(in crate::option_coverage) fn metadata(key: &str) -> serde_json::Value {
    json!({
        "claim":"finite samples and named sequential witnesses; not a global float/geometric domain",
        "source":["PrintConfig.cpp:10436-10447", "Print.cpp:613-930,1284-1306", "Print.hpp:1120-1122"],
        "loaded_lower_bound":{"value":0,"inclusive":false},
        "global_upper_bound":"unproved; neither 250/400 nor FLT_MAX is a physical safety maximum",
        "canonical":{"nozzle_diameter":["0.4"],"print_sequence":"by layer","near_zero_sample":"0.00001",
            "retained_samples":if key == "nozzle_height" { ["1","400"] } else { ["0.5","250"] }},
        "loaded_rejections":["0","-0.00001"],
        "sequential":{"model":"two separate cube10 objects, bottom Z=0","print_sequence":"by object",
            "nozzle_diameter":["0.4"],"skirt_loops":0,"raft_layers":0,"enable_support":false,
            "draft_shield":"disabled","enable_prime_tower":false,"timelapse":"traditional",
            "enable_wrapping_detection":false,"disable_m73":true,
            "witnesses":"lid/rod 9.99999,10,10.00001; radius20.19999,20.2,20.20001; nozzle9.99999,10,10.00001; tall-last/reverse",
            "entry":"slice_project_plate_validated; rejected CLI channels/result/code also compared"},
        "deferred":"global upper bound; arbitrary meshes, nonzero Z, rotations, skirts/draft-shield, mixed nozzles, support/raft, compensation"
    })
}

#[test]
fn orca_parity_g2_inventory_record() {
    let Some(plans) = domains::try_load(&runner::repo_root()) else {
        return;
    };
    if std::env::var_os("ARES_PARITY_ARTIFACT_ROOT").is_none() {
        crate::native_environment::skipped("offline G2 inventory artifact root is unset");
        return;
    }
    let records = plans.iter().map(|plan| json!({
        "key":plan.key,"type":plan.option_type,"scope":plan.raw_scope,"source":plan.source,
        "cases":plan.cases.iter().map(|case|json!({"label":case.label,"value":case.value})).collect::<Vec<_>>(),
        "omission":plan.omission,"provably_legal":plan.provably_legal,
        "width_domain":plan.width_domain.as_ref().map(|domain|json!({"metadata":domain.metadata,"probes":domain.probes}))
    })).collect::<Vec<_>>();
    crate::artifacts::overwrite(
        &crate::artifacts::root_from_env()
            .unwrap()
            .join("g2-all-650-plans.json"),
        &serde_json::to_vec_pretty(&records).unwrap(),
    )
    .unwrap();
}

#[test]
fn orca_parity_g2_bounded_metadata_and_unproved_countercontrol() {
    let Some(plans) = domains::try_load(&runner::repo_root()) else {
        return;
    };
    for key in [
        "extruder_clearance_height_to_lid",
        "extruder_clearance_height_to_rod",
        "extruder_clearance_radius",
        "nozzle_height",
    ] {
        let plan = plans.iter().find(|plan| plan.key == key).unwrap();
        let mut unproved = plan.clone();
        unproved.provably_legal = false;
        unproved.omission =
            Some("matching samples without sequential consumer admission remain unproved");
        let outcome = OptionOutcome::executed(&unproved, unproved.cases.len(), Vec::new(), 0, None);
        assert_eq!(outcome.status, "INCOMPLETE");
        assert!(!outcome.detail.is_empty());
        assert!(
            plan.provably_legal,
            "{key}: finite consumer context not admitted"
        );
        assert!(plan.omission.is_none());
        assert_eq!(
            plan.cases
                .iter()
                .map(|case| case.label.as_str())
                .collect::<Vec<_>>(),
            ["probe-min", "probe-max", "near-zero"]
        );
        assert_eq!(metadata(key)["loaded_lower_bound"]["inclusive"], false);
    }
}

#[test]
fn orca_parity_g2_lid_loaded_and_complete_output() {
    loaded_and_complete(
        "extruder_clearance_height_to_lid",
        &["0.00001", "0.5", "250"],
    );
}

#[test]
fn orca_parity_g2_rod_loaded_and_complete_output() {
    loaded_and_complete(
        "extruder_clearance_height_to_rod",
        &["0.00001", "0.5", "250"],
    );
}

#[test]
fn orca_parity_g2_radius_loaded_and_complete_output() {
    loaded_and_complete("extruder_clearance_radius", &["0.00001", "0.5", "250"]);
}

#[test]
fn orca_parity_g2_nozzle_loaded_and_complete_output() {
    loaded_and_complete("nozzle_height", &["0.00001", "1", "400"]);
}

fn loaded_and_complete(key: &str, values: &[&str]) {
    if crate::native_environment::binary("ARES_ORCA_NATIVE_BIN").is_none()
        || crate::native_environment::value("ARES_ORCA_RESOURCES_DIR").is_none()
    {
        return;
    }
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let plan = domains::load(&runner::repo_root())
        .into_iter()
        .find(|p| p.key == key)
        .unwrap();
    let mut proofs = Vec::new();
    for wire in values {
        let case = domains::OptionCase {
            label: format!("loaded-{wire}"),
            value: Some(json!(wire)),
        };
        let exported = export(&runner, &plan, &case).unwrap();
        let effective = runner::application::settings(&exported.project).unwrap();
        for dependency in [
            "extruder_clearance_height_to_lid",
            "extruder_clearance_height_to_rod",
            "extruder_clearance_radius",
            "nozzle_height",
        ] {
            assert!(
                effective[dependency]
                    .as_str()
                    .unwrap()
                    .parse::<f64>()
                    .unwrap()
                    > 0.0
            );
        }
        let loaded = ares_core::load_project(&exported.project).unwrap();
        assert_eq!(loaded.objects().len(), 1);
        let object = &loaded.objects()[0];
        assert_eq!(object.instances().len(), 1);
        assert_eq!(object.volumes().len(), 1);
        let volume = &object.volumes()[0];
        let matrix = object.instances()[0].transform().then(volume.transform());
        let points = volume
            .mesh()
            .vertices()
            .iter()
            .map(|point| matrix.transform_point(*point))
            .collect::<Vec<_>>();
        let min = [0, 1, 2].map(|axis| {
            points
                .iter()
                .map(|p| [p.x, p.y, p.z][axis])
                .fold(f64::INFINITY, f64::min)
        });
        let max = [0, 1, 2].map(|axis| {
            points
                .iter()
                .map(|p| [p.x, p.y, p.z][axis])
                .fold(f64::NEG_INFINITY, f64::max)
        });
        assert_eq!(
            [max[0] - min[0], max[1] - min[1], min[2], max[2]],
            [10., 10., 0., 10.]
        );
        // Float serialization may use scientific notation; equality is exact, not a tolerance.
        assert_eq!(
            effective[key].as_str().unwrap().parse::<f64>().unwrap(),
            wire.parse::<f64>().unwrap()
        );
        runner::application::verify_value(&exported.project, "print_sequence", &json!("by layer"))
            .unwrap();
        runner::application::verify_value(&exported.project, "nozzle_diameter", &json!(["0.4"]))
            .unwrap();
        let sliced = runner.slice_case(&exported).unwrap();
        let outcome = crate::compare_case(&sliced);
        assert_eq!(outcome.status, "PASS", "{key}={wire}: {}", outcome.detail);
        proofs.push(json!({"requested":wire,"effective":effective[key],"status":outcome.status,"artifacts":outcome.artifacts}));
    }
    for wire in ["0", "-0.00001"] {
        let case = domains::OptionCase {
            label: format!("rejected-{wire}"),
            value: Some(json!(wire)),
        };
        let error = export(&runner, &plan, &case)
            .err()
            .expect("nonpositive clearance must reject");
        assert_eq!(error.stage, runner::stages::Stage::Export);
        assert_eq!(error.kind, runner::stages::FailureKind::Process);
        let captured = error.captured.unwrap();
        let code = if cfg!(unix) { 238 } else { -18 };
        assert_eq!(captured.code, Some(code));
        let options: ares_core::SliceOptions = serde_json::from_value(json!({key:wire})).unwrap();
        let errors = options.validate_extruder_clearance_options().unwrap();
        assert_eq!(errors.len(), 1);
        assert!(
            String::from_utf8_lossy(&captured.stderr).contains(&format!("{key}: {}", errors[key]))
        );
        proofs.push(json!({"rejected":wire,"stage":"Export","code":captured.code,"evidence":error.evidence,"validation":errors}));
    }
    let mut observed_plan = plan.clone();
    observed_plan.cases = values
        .iter()
        .map(|wire| domains::OptionCase {
            label: format!("loaded-{wire}"),
            value: Some(json!(wire)),
        })
        .collect();
    let outcome = OptionOutcome::executed(&observed_plan, values.len(), Vec::new(), 0, None);
    let root = crate::artifacts::root_from_env().unwrap();
    crate::artifacts::overwrite(&root.join(format!("g2-{key}-loaded.json")), &serde_json::to_vec_pretty(&json!({
        "scope":"one object, by layer, 0.4mm nozzle; sequential consumers require separate witnesses",
        "domain":metadata(key),
        "proofs":proofs,"classification":outcome.status,"detail":outcome.detail
    })).unwrap()).unwrap();
    assert_eq!(outcome.status, "PASS", "{key}: {}", outcome.detail);
    let authored = plan
        .cases
        .iter()
        .map(|case| case.value.as_ref().unwrap().as_str().unwrap())
        .collect::<Vec<_>>();
    for wire in values {
        assert!(authored.contains(wire), "{key}: missing canonical {wire}");
    }
}

fn export(
    runner: &OrcaRunner,
    plan: &domains::OptionPlan,
    case: &domains::OptionCase,
) -> Result<runner::ExportedCase, runner::stages::StageError> {
    let profiles = VendorProfiles::load(&profiles_root(), "Creality").unwrap();
    let selection =
        crate::select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    let mut machine = profiles.machine(&selection.printer).unwrap();
    let mut process = profiles.process(&selection.process).unwrap();
    crate::normalize_process_defaults(&machine, &mut process);
    let mut filaments = selection
        .filaments
        .iter()
        .map(|name| profiles.filament(name).unwrap())
        .collect::<Vec<_>>();
    crate::normalize_filament_defaults(&mut filaments);
    inject_case(plan, case, &mut machine, &mut process, &mut filaments);
    runner.export_case(
        &CaseInputs {
            label: &format!("g2/{}/{}", plan.key, case.label),
            machine: &machine,
            process: &process,
            filaments: &filaments,
        },
        &crate::smoke_overrides(),
        &runner::repo_root().join("tests/parity/cube10.stl"),
    )
}
