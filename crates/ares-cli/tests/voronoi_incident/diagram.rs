use boostvoronoi::prelude::{Builder, Line};
use serde_json::Value;

#[path = "../../../../tests/support/voronoi_diagram.rs"]
mod snapshot;

pub(super) fn build(input: &Value) -> Value {
    let lines: Vec<_> = input
        .as_array()
        .unwrap()
        .iter()
        .map(|line| {
            let coord = |point: &str, axis: usize| {
                i32::try_from(line[point][axis].as_i64().unwrap()).unwrap()
            };
            Line::from([coord("a", 0), coord("a", 1), coord("b", 0), coord("b", 1)])
        })
        .collect();
    let vd = Builder::<i32>::default()
        .with_segments(&lines)
        .unwrap()
        .build()
        .unwrap();
    snapshot::serialize(&vd)
}
