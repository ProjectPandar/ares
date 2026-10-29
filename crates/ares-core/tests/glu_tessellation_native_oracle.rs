//! Admission proof for the test-time oracle, independent of the missing Rust operation.
#[path = "../../../tests/support/native_environment.rs"]
mod native_environment;
#[path = "../../../tests/support/glu_native_oracle.rs"]
mod native_oracle;
use native_oracle::{NativeCase, NativeOracle};

#[test]
fn original_glu_compiles_and_accepts_all_authored_inputs_at_test_time() {
    let Some(mut oracle) = NativeOracle::from_env().unwrap() else {
        return;
    };
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/parity/glu-tessellation");
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("manifest.json")).unwrap()).unwrap();
    for entry in manifest["cases"].as_array().unwrap() {
        let input =
            std::fs::read(root.join(format!("inputs/{}.json", entry["name"].as_str().unwrap())))
                .unwrap();
        assert_eq!(
            native_oracle::sha256(&input),
            entry["input_sha256"].as_str().unwrap()
        );
        let case: NativeCase = serde_json::from_slice(&input).unwrap();
        let bits = oracle.run(&case).unwrap();
        // This only exercises artifact serialization; it makes no Rust parity claim.
        oracle
            .record_bits(&case, "native-admission", &bits)
            .unwrap();
    }
}
