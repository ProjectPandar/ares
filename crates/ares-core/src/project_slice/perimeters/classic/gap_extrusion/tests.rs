mod coverage;
mod filter;
mod preflight;
mod variable_width;

use crate::{
    geometry::{Point, ThickPolyline},
    project_slice::perimeters::types::Flow,
};

fn flow() -> Flow {
    Flow {
        width: 0.4,
        height: 0.2,
        spacing: 0.35,
        nozzle_diameter: 0.4,
        bridge: false,
        mm3_per_mm: 0.08,
    }
}

fn thick(points: &[(i64, i64)], widths: &[f64]) -> ThickPolyline {
    ThickPolyline {
        points: points.iter().map(|&(x, y)| Point::new(x, y)).collect(),
        width: widths.to_vec(),
        endpoints: (false, false),
    }
}
