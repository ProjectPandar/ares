//! PerimeterGenerator.cpp:2521 Arachne application; Classic1672 is a control.
use crate::{
    OrcaRunner,
    runner::native_api_cli::{NativeCase, sha},
};

#[test]
fn arachne_extra_overhang_call_positive() {
    run(
        "arachne-positive",
        include_bytes!("../../../../tests/parity/arachne-extra-overhang/arachne-positive.3mf"),
        "77968f1ac5b7497c7f7207bf43153a954d1328a32faf89eff1c8d2268fcb037f",
    );
}

#[test]
fn arachne_extra_overhang_call_extra_off_control() {
    run(
        "arachne-control",
        include_bytes!("../../../../tests/parity/arachne-extra-overhang/arachne-control.3mf"),
        "0f6cac9cd2e66be4b28ca80edfeab5bcc713609d99ddebd08dbedb364ed6d8cc",
    );
}

#[test]
fn arachne_extra_overhang_call_detect_off_extra_off_control() {
    run(
        "arachne-undetected-control",
        include_bytes!(
            "../../../../tests/parity/arachne-extra-overhang/arachne-undetected-control.3mf"
        ),
        "1c0b73fabffd108411557558eadd34db539bcb9cd1db6a3a265460579f44a545",
    );
}

#[test]
fn arachne_extra_overhang_call_detect_off_extra_on_control() {
    run(
        "arachne-undetected-extra",
        include_bytes!(
            "../../../../tests/parity/arachne-extra-overhang/arachne-undetected-extra.3mf"
        ),
        "550e19473adf26e61e0861b2736784179808a7035fdac58168b521bd79ccbdf5",
    );
}

#[test]
fn arachne_extra_overhang_call_classic_control() {
    run(
        "classic-detected-extra-control",
        include_bytes!(
            "../../../../tests/parity/arachne-extra-overhang/classic-detected-extra-control.3mf"
        ),
        "85e93cb93363cd43ce7d6007db39162871d6e9d6539679bb974926617c414fce",
    );
}

fn run(label: &str, project: &[u8], expected_sha: &str) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    assert_eq!(sha(project), expected_sha);
    let case = NativeCase::slice(&runner, project, &format!("extra-overhang-{label}-"));
    let (generator, detect, extra) = match label {
        "arachne-control" => ("arachne", "1", "0"),
        "arachne-positive" => ("arachne", "1", "1"),
        "arachne-undetected-control" => ("arachne", "0", "0"),
        "arachne-undetected-extra" => ("arachne", "0", "1"),
        "classic-detected-extra-control" => ("classic", "1", "1"),
        _ => unreachable!("fixture label is fixed by the tests"),
    };
    for (key, value) in [
        ("wall_generator", generator),
        ("detect_overhang_wall", detect),
        ("extra_perimeters_on_overhangs", extra),
        ("staggered_inner_seams", "0"),
        ("seam_slope_type", "none"),
        ("wipe", "0"),
        ("sparse_infill_density", "0%"),
    ] {
        assert!(case.native.contains(&format!("; {key} = {value}\n")));
    }
    if label == "arachne-positive" {
        assert!(
            case.native
                .lines()
                .any(|line| line == "G1 X117.223 Y117.223 Z1")
        );
    }
    case.compare(ares_core::GenerationMetadata::deterministic(
        2026, 10, 20, 0, 0, 0,
    ));
}
