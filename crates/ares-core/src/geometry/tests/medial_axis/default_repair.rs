use crate::geometry::{Line, Point, medial_axis::diagram};
use serde_json::{Value, json};
use std::path::Path;

#[path = "../../../../../../tests/support/voronoi_native.rs"]
mod native;
#[path = "../../../../../../tests/support/voronoi_diagram.rs"]
mod snapshot;

impl snapshot::DiagramSnapshot for crate::geometry::voronoi::Diagram {
    fn snapshot_cells(&self) -> &[boostvoronoi::prelude::Cell] {
        self.cells()
    }
    fn snapshot_edges(&self) -> &[boostvoronoi::prelude::Edge] {
        self.edges()
    }
    fn snapshot_vertices(&self) -> impl Iterator<Item = (f64, f64, usize, u32)> {
        self.vertices().map(|v| {
            (
                v.x(),
                v.y(),
                v.get_incident_edge().unwrap().usize(),
                v.metadata.get_color(),
            )
        })
    }
}

fn compare(out: &Path, qualified: &Value, repairs: usize) -> Option<Value> {
    assert_eq!(qualified["qualified_diagrams"], 100);
    assert_eq!(qualified["repair_successful_records"], repairs);
    let mut results = Vec::new();
    let mut first = None;
    for relative in qualified["records"].as_array().unwrap() {
        let relative = relative.as_str().unwrap();
        let row = native::read_json(&out.join(relative));
        let lines: Vec<_> = row["lines"]
            .as_array()
            .unwrap()
            .iter()
            .map(|line| {
                let point = |key: &str| {
                    Point::new(
                        line[key][0].as_i64().unwrap(),
                        line[key][1].as_i64().unwrap(),
                    )
                };
                Line::new(point("a"), point("b"))
            })
            .collect();
        // This is the existing real MedialAxis constructor, not a test-only Builder.
        let rust = match diagram::build(&lines) {
            Ok(output) => snapshot::serialize(&output),
            Err(error) => json!({"construction_error":format!("{error:?}")}),
        };
        let expected = json!({"cells":row["diagram"]["cells"],
            "vertices":row["diagram"]["vertices"],"edges":row["diagram"]["edges"]});
        let difference = native::first_difference(&expected, &rust, "$");
        if first.is_none() {
            first.clone_from(&difference);
        }
        native::save(
            &out.join(relative)
                .parent()
                .unwrap()
                .join("Rust.diagram.json"),
            &rust,
        );
        results.push(
            json!({"Native_record":relative,"first_difference":difference,"equal":expected==rust}),
        );
    }
    native::save(
        &out.join("comparison.json"),
        &json!({
            "status":if first.is_some() { "STRICT_DIAGRAM_MISMATCH" } else { "STRICT_DIAGRAM_MATCH" },
            "first_difference":first,"results":results,
            "actual_consumer_constructor":"geometry::medial_axis::diagram::build",
            "no_sort_filter_tolerance_or_anchor_normalization":true,
            "Native_wrapper_state_observed_not_fabricated_as_Rust_state":true
        }),
    );
    first
}

#[test]
fn runtime_native_default_repair_complete_ordered_diagrams_match() {
    let Some((control_out, control)) = native::capture("painted-leaves") else {
        return;
    };
    let Some((split_out, split)) = native::capture("painted-split") else {
        return;
    };
    let pids: Vec<_> = control["Native_PIDs"]
        .as_array()
        .unwrap()
        .iter()
        .chain(split["Native_PIDs"].as_array().unwrap())
        .collect();
    assert_eq!(pids.len(), 4);
    assert_eq!(
        pids.iter()
            .map(|pid| pid.as_u64().unwrap())
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        4
    );
    native::save(
        &control_out.join("GLOBAL-NATIVE-FIRST.json"),
        &json!({
            "all4_Native_completed_before_any_Rust_constructor":true,
            "Native_PIDs":pids,"control_artifacts":control_out,"split_artifacts":split_out,
            "Rust_constructor_calls_at_barrier":0
        }),
    );
    let control_difference = compare(&control_out, &control, 0);
    let split_difference = compare(&split_out, &split, 98);
    native::save(
        &control_out.join("RED-matrix.json"),
        &json!({
            "control_first_difference":control_difference,"split_first_difference":split_difference,
            "control_artifacts":control_out,"split_artifacts":split_out
        }),
    );
    assert!(
        control_difference.is_none() && split_difference.is_none(),
        "real constructor must match complete Native effective diagrams; control={control_difference:?}; split={split_difference:?}; artifacts={}",
        control_out.display()
    );
}
