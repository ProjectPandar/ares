//! Exact ordered test serialization; never a production metadata adapter.
use boostvoronoi::prelude::{Cell, Diagram, Edge};
use serde_json::{Value, json};

fn bits(value: f64) -> String {
    value
        .to_le_bytes()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

pub trait DiagramSnapshot {
    fn snapshot_cells(&self) -> &[Cell];
    fn snapshot_edges(&self) -> &[Edge];
    fn snapshot_vertices(&self) -> impl Iterator<Item = (f64, f64, usize, u32)>;
}

impl DiagramSnapshot for Diagram {
    fn snapshot_cells(&self) -> &[Cell] {
        self.cells()
    }
    fn snapshot_edges(&self) -> &[Edge] {
        self.edges()
    }
    fn snapshot_vertices(&self) -> impl Iterator<Item = (f64, f64, usize, u32)> {
        self.vertices().iter().map(|v| {
            (
                v.x(),
                v.y(),
                v.get_incident_edge().unwrap().usize(),
                v.get_color(),
            )
        })
    }
}

pub fn serialize(vd: &impl DiagramSnapshot) -> Value {
    serialize_parts(
        vd.snapshot_cells(),
        vd.snapshot_vertices(),
        vd.snapshot_edges(),
    )
}

fn serialize_parts(
    cells: &[Cell],
    vertices: impl Iterator<Item = (f64, f64, usize, u32)>,
    edges: &[Edge],
) -> Value {
    let cells: Vec<_> = cells
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
    let vertices: Vec<_> = vertices
        .map(|(x, y, incident, color)| {
            json!({"x_bits":bits(x),"y_bits":bits(y),
            "incident":incident,"color":color})
        })
        .collect();
    let edges: Vec<_> = edges.iter().map(|edge| {
        let v0 = edge.vertex0().map(|v| v.usize());
        let v1 = edges[edge.twin().unwrap().usize()].vertex0().map(|v| v.usize());
        json!({"cell":edge.cell().unwrap().usize(),"v0":v0,"v1":v1,
            "twin":edge.twin().unwrap().usize(),"next":edge.next().unwrap().usize(),
            "prev":edge.prev().unwrap().usize(),"linear":edge.is_linear(),
            "primary":edge.is_primary(),"finite":v0.is_some() && v1.is_some(),"color":edge.get_color()})
    }).collect();
    json!({"cells":cells,"vertices":vertices,"edges":edges})
}
