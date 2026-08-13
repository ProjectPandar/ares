mod arithmetic;
mod output;
mod sections;
mod tracing;

use crate::{
    geometry::{Line, Point, Polygon},
    project_slice::perimeters::types::Flow,
};

fn line(ax: i64, ay: i64, bx: i64, by: i64) -> Line {
    Line::new(Point::new(ax, ay), Point::new(bx, by))
}

fn polygon(points: &[(i64, i64)]) -> Polygon {
    Polygon::new(points.iter().map(|&(x, y)| Point::new(x, y)).collect())
}

fn bridge_flow(diameter: f32) -> Flow {
    flow(diameter, (f64::from(diameter) + 0.05) as f32)
}

fn flow(width: f32, spacing: f32) -> Flow {
    Flow {
        width,
        height: width,
        spacing,
        nozzle_diameter: width,
        bridge: true,
        mm3_per_mm: 1.0,
    }
}
