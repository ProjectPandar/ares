use boostvoronoi::prelude::{Builder, Line};
use serde_json::{Value, json};

fn bits(value: f64) -> String {
    value
        .to_le_bytes()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

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
    let cells: Vec<_> = vd
        .cells()
        .iter()
        .map(|cell| {
            // SourceCategory::Segment collapses initial/reverse bits; preserve raw bits.
            let debug = format!("{cell:?}");
            let color: u32 = debug
                .rsplit_once("col:")
                .unwrap()
                .1
                .strip_suffix(')')
                .unwrap()
                .parse()
                .unwrap();
            json!({"source":cell.source_index().usize(),"category":color & 31,
            "color":color >> 5,"incident":cell.get_incident_edge().map(|e| e.usize())})
        })
        .collect();
    let vertices: Vec<_> = vd
        .vertices()
        .iter()
        .map(|vertex| {
            json!({"x_bits":bits(vertex.x()),"y_bits":bits(vertex.y()),
            "incident":vertex.get_incident_edge().unwrap().usize(),"color":vertex.get_color()})
        })
        .collect();
    let edges: Vec<_> = vd.edges().iter().map(|edge| {
        let v0 = edge.vertex0().map(|v| v.usize());
        let v1 = vd.edge_get_vertex1(edge.id()).unwrap().map(|v| v.usize());
        json!({"cell":edge.cell().unwrap().usize(),"v0":v0,"v1":v1,
            "twin":edge.twin().unwrap().usize(),"next":edge.next().unwrap().usize(),
            "prev":edge.prev().unwrap().usize(),"linear":edge.is_linear(),
            "primary":edge.is_primary(),"finite":v0.is_some() && v1.is_some(),"color":edge.get_color()})
    }).collect();
    json!({"cells":cells,"vertices":vertices,"edges":edges})
}
