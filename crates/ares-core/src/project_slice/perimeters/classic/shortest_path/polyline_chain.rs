use super::chain::chain_multiple;
use crate::geometry::Polyline;

/// `chain_polylines` from OrcaSlicer 2.4.2 `ShortestPath.cpp:1968-1994`,
/// using the existing all-paths-reversible shortest-path seam.
pub(crate) fn chain_polylines(polylines: &mut Vec<Polyline>) {
    if polylines.len() < 2 {
        return;
    }
    let positions = polylines
        .iter()
        .flat_map(|polyline| {
            [
                coordinates(polyline.front().expect("plane path is valid")),
                coordinates(polyline.back().expect("plane path is valid")),
            ]
        })
        .collect::<Vec<_>>();
    let chain = chain_multiple(&positions, None);
    let mut source = std::mem::take(polylines)
        .into_iter()
        .map(Some)
        .collect::<Vec<_>>();
    polylines.reserve(chain.len());
    for (index, reverse) in chain {
        let mut polyline = source[index].take().expect("chain indices are unique");
        if reverse {
            polyline.reverse();
        }
        polylines.push(polyline);
    }
}

fn coordinates(point: crate::geometry::Point) -> [f64; 2] {
    [point.x() as f64, point.y() as f64]
}
