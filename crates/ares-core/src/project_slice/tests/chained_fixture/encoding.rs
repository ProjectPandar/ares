use std::io::Write as _;

use crate::{geometry::Point, mesh_slicer::ChainedLayer};

pub(super) fn encode(layers: &[ChainedLayer], semantic_order: bool) -> Vec<u8> {
    let mut output = Vec::new();
    for (layer_index, layer) in layers.iter().enumerate() {
        writeln!(&mut output, "L{layer_index}").unwrap();
        if semantic_order {
            let mut polygons = layer
                .polygons()
                .iter()
                .map(|polygon| rotate_to_earliest_minimum(point_pairs(polygon.points())))
                .collect::<Vec<_>>();
            polygons.sort_unstable();
            for points in polygons {
                write_polygon(&mut output, &points);
            }
        } else {
            for polygon in layer.polygons() {
                write_polygon(&mut output, &point_pairs(polygon.points()));
            }
        }
    }
    output
}

fn point_pairs(points: &[Point]) -> Vec<(i64, i64)> {
    points.iter().map(|point| (point.x(), point.y())).collect()
}

fn rotate_to_earliest_minimum(points: Vec<(i64, i64)>) -> Vec<(i64, i64)> {
    let minimum = points.iter().min().unwrap();
    let start = points.iter().position(|point| point == minimum).unwrap();
    points[start..]
        .iter()
        .chain(&points[..start])
        .copied()
        .collect()
}

fn write_polygon(output: &mut Vec<u8>, points: &[(i64, i64)]) {
    write!(output, "C;{}", points.len()).unwrap();
    for &(x, y) in points {
        write!(output, ";{x},{y}").unwrap();
    }
    output.push(b'\n');
}
