//! Runtime Orca MMU VD -> unchanged workspace backend; no Native goldens.

#[path = "../../../tests/support/native_environment.rs"]
mod native_environment;

#[path = "../../../tests/support/voronoi_native.rs"]
mod native;

#[path = "voronoi_incident/diagram.rs"]
mod diagram;

use native::{first_difference, read_json, save};
use serde_json::json;

#[test]
fn runtime_native_rectangle_complete_ordered_diagram_matches() {
    let Some((out, qualified)) = native::capture("painted-leaves") else {
        return;
    };
    assert_eq!(qualified["qualified_diagrams"], 100);
    let mut results = Vec::new();
    let mut first = None;
    for relative in qualified["records"].as_array().unwrap() {
        let relative = relative.as_str().unwrap();
        let row = read_json(&out.join(relative));
        let rust = diagram::build(&row["lines"]);
        let native = json!({
            "cells":row["diagram"]["cells"],
            "vertices":row["diagram"]["vertices"],
            "edges":row["diagram"]["edges"]
        });
        let difference = first_difference(&native, &rust, "$");
        if first.is_none() {
            first.clone_from(&difference);
        }
        let folder = out.join(relative).parent().unwrap().to_path_buf();
        save(&folder.join("Rust.diagram.json"), &rust);
        results.push(
            json!({"Native_record":relative,"first_difference":difference,"equal":native==rust}),
        );
    }
    assert_eq!(results.len(), 100);
    save(
        &out.join("comparison.json"),
        &json!({
            "status":if first.is_some() { "STRICT_DIAGRAM_MISMATCH" } else { "STRICT_DIAGRAM_MATCH" },
            "first_difference":first,"results":results,
            "no_sort_filter_tolerance_or_anchor_normalization":true,
            "scope":"one actual rectangle geometry, not full repaired VD equivalence"
        }),
    );
    assert!(
        first.is_none(),
        "strict complete ordered diagram mismatch: {first:?}; artifacts={}",
        out.display()
    );
}
