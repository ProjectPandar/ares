//! Exact ordered test serialization; never a production metadata adapter.
use boostvoronoi::prelude::Diagram;
use serde_json::{Value, json};

fn bits(value: f64) -> String {
    value
        .to_le_bytes()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

pub fn serialize(vd: &Diagram) -> Value {
    let cells: Vec<_> = vd
        .cells()
        .iter()
        .map(|cell| {
            // The public enum collapses initial/reverse source-category bits.
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
