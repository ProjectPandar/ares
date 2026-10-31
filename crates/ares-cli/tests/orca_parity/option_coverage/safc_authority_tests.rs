use std::{fs, process::Command};

use serde_json::Value;
use sha2::{Digest, Sha256};

#[test]
fn safc_concentric_cases_use_initialized_authority_in_both_modes() {
    if let Some(request) = std::env::var_os(crate::runner::safc_reference::REQUEST) {
        crate::runner::safc_reference::child(std::path::Path::new(&request));
        return;
    }
    let Some(initialized) = crate::native_environment::binary("ARES_KSR_INITIALIZED_NATIVE_BIN")
    else {
        return;
    };
    if crate::runner::OrcaRunner::from_env().unwrap().is_none() {
        return;
    }
    let root = tempfile::Builder::new()
        .prefix("safc-authority-policy-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let output = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "option_coverage::safc_coverage::orca_parity_safc_coupled_models_preserve_whole_output_and_segment_consumers",
            "--nocapture",
        ])
        .env("ARES_PARITY_ARTIFACT_ROOT", &root)
        .output()
        .unwrap();
    fs::write(root.join("child.stdout"), &output.stdout).unwrap();
    fs::write(root.join("child.stderr"), &output.stderr).unwrap();
    let proofs: Vec<Value> =
        serde_json::from_slice(&fs::read(root.join("safc-cases.json")).unwrap()).unwrap();
    assert_eq!(proofs.len(), 7, "{root:?}");
    let hash = |bytes| {
        Sha256::digest(bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    };
    let initialized_sha = hash(fs::read(initialized).unwrap());
    let caller_sha = hash(fs::read(std::env::var_os("ARES_ORCA_NATIVE_BIN").unwrap()).unwrap());
    for (index, proof) in proofs.iter().enumerate() {
        let concentric = index >= 5;
        assert_eq!(
            proof["native_producer"]["components"]["native"],
            if concentric {
                initialized_sha.as_str()
            } else {
                caller_sha.as_str()
            },
            "case producer missing or incorrect: {proof}; {root:?}"
        );
        if concentric {
            assert_eq!(
                proof["native_producer"]["components"]["policy"]["ORCA_REPLAY_SEED"], "0",
                "{proof}; {root:?}"
            );
            assert_eq!(
                proof["native_producer"]["components"]["policy"]["ARES_PARITY_SEEDED_REFERENCE"],
                "1",
                "{proof}; {root:?}"
            );
        }
        assert_eq!(proof["full_output_status"], "PASS", "{proof}; {root:?}");
    }
    assert!(output.status.success(), "{output:?}; {root:?}");
}
