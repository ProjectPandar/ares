use serde_json::{Map, Value, json};

#[test]
fn native_three_nullable_region_vectors_round_trip_public_options() {
    let Some((runner, native)) = super::definition_defaults::native_defaults() else {
        return;
    };
    let fields = [
        ("enable_overhang_speed", "bools_nullable"),
        ("slowdown_for_curled_perimeters", "bools_nullable"),
        ("outer_wall_speed", "floats_nullable"),
    ];
    let requested: Map<String, Value> = fields
        .iter()
        .map(|(key, _)| {
            assert!(native[key].is_array(), "Native vector qualification: {key}");
            ((*key).to_owned(), native[key].clone())
        })
        .collect();
    let requested = Value::Object(requested);
    let typed = serde_json::from_value::<ares_core::ProcessRegionSourceOptions>(requested.clone())
        .map_err(|error| error.to_string())
        .map(|options| serde_json::to_value(options).unwrap());
    let options = serde_json::from_value::<ares_core::SliceOptions>(requested.clone())
        .map_err(|error| error.to_string())
        .map(|options| options.values().clone());
    let schema: Value = serde_json::from_slice(ares_core::print_config_def_json()).unwrap();
    let mut failures = Vec::new();
    let mut observations = Vec::new();
    for (key, kind) in fields {
        let definition = schema["options"]
            .as_array()
            .unwrap()
            .iter()
            .find(|definition| definition["key"] == key)
            .unwrap();
        let token = native[key]
            .as_array()
            .unwrap()
            .iter()
            .map(|element| element.as_str().unwrap())
            .collect::<Vec<_>>()
            .join(",");
        let decoded = ares_core::decode_config_value_by_key(key, &token);
        let encoded = decoded
            .as_ref()
            .map_err(Clone::clone)
            .and_then(|decoded| ares_core::encode_config_value_by_key(key, decoded));
        let decoded_value = decoded
            .as_ref()
            .ok()
            .map(|decoded| serde_json::from_str::<Value>(decoded).unwrap());
        let typed_matches = typed
            .as_ref()
            .is_ok_and(|exported| exported[key] == native[key]);
        let requested_matches = options
            .as_ref()
            .is_ok_and(|exported| exported[key] == native[key]);
        let codec_matches = decoded_value.as_ref() == Some(&native[key])
            && encoded.as_deref() == Ok(token.as_str());
        let schema_matches = definition["type"] == kind && definition["nullable"] == true;
        if !(typed_matches && requested_matches && codec_matches && schema_matches) {
            failures.push(key);
        }
        observations.push(json!({
            "key": key, "native": native[key], "native_token": token,
            "public_definition": definition, "decoded": decoded, "encoded": encoded,
            "typed_matches": typed_matches, "requested_matches": requested_matches,
            "codec_matches": codec_matches, "schema_matches": schema_matches,
        }));
    }
    let artifact = runner.work.join("three-nullable-vectors.json");
    std::fs::write(
        &artifact,
        serde_json::to_vec_pretty(&json!({
            "native_values": requested, "typed_export": typed,
            "requested_export": options, "observations": observations,
            "failed_keys": failures,
        }))
        .unwrap(),
    )
    .unwrap();
    assert!(
        failures.is_empty(),
        "public nullable vector contracts differ for {failures:?}; see {artifact:?}"
    );
}
