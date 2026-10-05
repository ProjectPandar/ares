//! Boundary failures: missing/unknown fields, invalid JSON, wrong types and
//! out-of-range integers must fail before model or catalog parsing.

#[tokio::test]
async fn profile_slice_options_contract() {
    let rejected = [
        ("{}", "missing field `seed`"),
        (r#"{"seed":0,"extra":1}"#, "unknown field `extra`"),
        (r#"{"seed":null}"#, "invalid type"),
        (r#"{"seed":"0"}"#, "invalid type"),
        (r#"{"seed":-1}"#, "invalid value"),
        (r#"{"seed":4294967296}"#, "invalid value"),
        (r#"{"seed":0.5}"#, "invalid type"),
        (r#"{"seed":0,"plate":-1}"#, "invalid value"),
        (r#"{"seed":0,"plate":"1"}"#, "invalid type"),
        (r#"{"seed":0,"plate":4294967296}"#, "invalid value"),
        ("not JSON", "expected"),
    ];
    let mut observations = Vec::new();
    for (input, expected) in rejected {
        let error = ares_wasm::slice_with_profiles_bytes(Vec::new(), "", "", "", input)
            .await
            .unwrap_err();
        assert!(
            error.starts_with("invalid profile slice options JSON:"),
            "{error}"
        );
        assert!(error.contains(expected), "{input}: {error}");
        observations.push((input, error));
    }
    for input in [
        r#"{"seed":0}"#,
        r#"{"seed":4294967295,"plate":3}"#,
        r#"{"seed":1,"plate":null}"#,
    ] {
        let error = ares_wasm::slice_with_profiles_bytes(Vec::new(), "", "", "", input)
            .await
            .unwrap_err();
        assert!(
            error.contains("model filename is required"),
            "{input}: {error}"
        );
        observations.push((input, error));
    }
    let root = std::env::var_os("ARES_PARITY_ARTIFACT_ROOT").map_or_else(
        || std::env::temp_dir().join("ares-wasm-profile-options"),
        std::path::PathBuf::from,
    );
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
        root.join("profile-options-contract.json"),
        serde_json::to_vec_pretty(&observations).unwrap(),
    )
    .unwrap();
}
