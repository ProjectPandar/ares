//! Diagnostic probes for `connect_base_support` phase behavior
//! (moved from `connect.rs`; they exercise the public entry).

use super::connect::connect_base_support;
use crate::geometry::Polyline;

#[cfg(test)]
mod vprobe {
    #[test]
    fn probe_vertical_counts() {
        use super::*;
        let polygon = crate::geometry::Polygon::new(vec![
            crate::geometry::Point::new(-7_650_601, -7_650_601),
            crate::geometry::Point::new(7_650_601, -7_650_601),
            crate::geometry::Point::new(7_650_601, 7_650_601),
            crate::geometry::Point::new(-7_650_601, 7_650_601),
        ]);
        // 13 vertical lines at pitch 537000 spanning ±7650601.
        let lines: Vec<Polyline> = (-14..=14)
            .map(|i| {
                let x = i * 537_000;
                Polyline::new(vec![
                    crate::geometry::Point::new(x, -7_000_000),
                    crate::geometry::Point::new(x, 7_000_000),
                ])
            })
            .collect();
        let bbox = crate::geometry::BoundingBox::from_polygon(&polygon).unwrap();
        let out = connect_base_support(
            lines,
            &[polygon],
            bbox,
            0.407,
            0.67,
            crate::geometry::CoordinateScale::Normal,
        )
        .unwrap();
        eprintln!(
            "VPROBE out={} lens={:?}",
            out.len(),
            out.iter().map(|p| p.points().len()).collect::<Vec<_>>()
        );
    }
}

#[cfg(test)]
mod chain_probe {
    #[test]
    fn probe_intersection_chain() {
        use super::*;
        // Square boundary at ±7650601, 27 vertical lines whose endpoints
        // sit EXACTLY on the top/bottom edges.
        let polygon = crate::geometry::Polygon::new(vec![
            crate::geometry::Point::new(-7_650_601, -7_650_601),
            crate::geometry::Point::new(7_650_601, -7_650_601),
            crate::geometry::Point::new(7_650_601, 7_650_601),
            crate::geometry::Point::new(-7_650_601, 7_650_601),
        ]);
        let lines: Vec<Polyline> = (-13..=13)
            .map(|i| {
                let x = i * 537_000;
                Polyline::new(vec![
                    crate::geometry::Point::new(x, -7_650_601),
                    crate::geometry::Point::new(x, 7_650_601),
                ])
            })
            .collect();
        let bbox = crate::geometry::BoundingBox::from_polygon(&polygon).unwrap();
        let graph = crate::fill::connect::graph::build_working_graph(
            lines.clone(),
            &[polygon.clone()],
            bbox,
            0.407,
            crate::geometry::CoordinateScale::Normal,
        )
        .unwrap();
        let connected = graph
            .intersections
            .iter()
            .filter(|i| i.contour_index.is_some())
            .count();
        let with_next = graph
            .intersections
            .iter()
            .filter(|i| i.next.is_some())
            .count();
        eprintln!(
            "CHAIN hits={connected} with_next={with_next} total={}",
            graph.intersections.len()
        );
        let out = connect_base_support(
            lines,
            &[polygon],
            bbox,
            0.407,
            0.67,
            crate::geometry::CoordinateScale::Normal,
        )
        .unwrap();
        eprintln!(
            "CHAIN out={} lens={:?}",
            out.len(),
            out.iter().map(|p| p.points().len()).collect::<Vec<_>>()
        );
    }
}

#[cfg(test)]
mod arch_probe {
    #[test]
    fn probe_every_other_arch() {
        use super::*;
        let polygon = crate::geometry::Polygon::new(vec![
            crate::geometry::Point::new(-7_650_601, -7_650_601),
            crate::geometry::Point::new(7_650_601, -7_650_601),
            crate::geometry::Point::new(7_650_601, 7_650_601),
            crate::geometry::Point::new(-7_650_601, 7_650_601),
        ]);
        let lines: Vec<Polyline> = (-13..=13)
            .map(|i| {
                let x = i * 537_000;
                Polyline::new(vec![
                    crate::geometry::Point::new(x, -7_650_601),
                    crate::geometry::Point::new(x, 7_650_601),
                ])
            })
            .collect();
        let bbox = crate::geometry::BoundingBox::from_polygon(&polygon).unwrap();
        let graph = crate::fill::connect::graph::build_working_graph(
            lines.clone(),
            &[polygon.clone()],
            bbox,
            0.407,
            crate::geometry::CoordinateScale::Normal,
        )
        .unwrap();
        // Dump the vertical-consumption decisions: for each unconsumed
        // intersection, print prev/next candidates and vertical flags.
        for index in 0..graph.intersections.len() {
            let i = &graph.intersections[index];
            let Some(ci) = i.contour_index else { continue };
            let pt = graph.boundary[ci].points[i.point_index];
            let flag = |other: Option<usize>| {
                other.map(|o| {
                    let j = &graph.intersections[o];
                    let p2 = graph.boundary[j.contour_index.unwrap()].points[j.point_index];
                    format!("{}(dy={})", o, p2.y() == pt.y())
                })
            };
            eprintln!(
                "ARCH idx={index} pt=({},{}) prev={:?} next={:?}",
                pt.x(),
                pt.y(),
                flag(i.prev),
                flag(i.next),
            );
        }
    }
}

#[cfg(test)]
mod merge_probe {
    #[test]
    fn probe_same_chain_append() {
        use super::*;
        let polygon = crate::geometry::Polygon::new(vec![
            crate::geometry::Point::new(-7_650_601, -7_650_601),
            crate::geometry::Point::new(7_650_601, -7_650_601),
            crate::geometry::Point::new(7_650_601, 7_650_601),
            crate::geometry::Point::new(-7_650_601, 7_650_601),
        ]);
        // 3 lines only: chain growth over idx 0..5.
        let lines: Vec<Polyline> = (-1..=1)
            .map(|i| {
                let x = i * 537_000;
                Polyline::new(vec![
                    crate::geometry::Point::new(x, -7_650_601),
                    crate::geometry::Point::new(x, 7_650_601),
                ])
            })
            .collect();
        let bbox = crate::geometry::BoundingBox::from_polygon(&polygon).unwrap();
        let out = connect_base_support(
            lines,
            &[polygon],
            bbox,
            0.407,
            0.67,
            crate::geometry::CoordinateScale::Normal,
        )
        .unwrap();
        for (n, p) in out.iter().enumerate() {
            eprintln!(
                "MERGE #{n} pts={}",
                p.points()
                    .iter()
                    .map(|pt| format!("({},{})", pt.x(), pt.y()))
                    .collect::<Vec<_>>()
                    .join(" ")
            );
        }
    }
}

#[cfg(test)]
mod order_probe {
    #[test]
    fn probe_27_line_order() {
        use super::*;
        let polygon = crate::geometry::Polygon::new(vec![
            crate::geometry::Point::new(-7_650_601, -7_650_601),
            crate::geometry::Point::new(7_650_601, -7_650_601),
            crate::geometry::Point::new(7_650_601, 7_650_601),
            crate::geometry::Point::new(-7_650_601, 7_650_601),
        ]);
        let lines: Vec<Polyline> = (-13..=13)
            .map(|i| {
                let x = i * 537_000;
                Polyline::new(vec![
                    crate::geometry::Point::new(x, -7_650_601),
                    crate::geometry::Point::new(x, 7_650_601),
                ])
            })
            .collect();
        let bbox = crate::geometry::BoundingBox::from_polygon(&polygon).unwrap();
        let out = connect_base_support(
            lines,
            &[polygon],
            bbox,
            0.407,
            0.67,
            crate::geometry::CoordinateScale::Normal,
        )
        .unwrap();
        eprintln!(
            "ORDER27 out={} lens={:?}",
            out.len(),
            out.iter().map(|p| p.points().len()).collect::<Vec<_>>()
        );
    }
}

#[cfg(test)]
mod orient_probe {
    #[test]
    fn probe_contour_orientation() {
        use crate::fill::connect::graph::build_working_graph;
        use crate::geometry::{CoordinateScale, Polyline};

        let polygon = crate::geometry::Polygon::new(vec![
            crate::geometry::Point::new(-7_650_601, -7_650_601),
            crate::geometry::Point::new(7_650_601, -7_650_601),
            crate::geometry::Point::new(7_650_601, 7_650_601),
            crate::geometry::Point::new(-7_650_601, 7_650_601),
        ]);
        let lines: Vec<Polyline> = (-1..=1)
            .map(|i| {
                let x = i * 537_000;
                Polyline::new(vec![
                    crate::geometry::Point::new(x, -7_650_601),
                    crate::geometry::Point::new(x, 7_650_601),
                ])
            })
            .collect();
        let bbox = crate::geometry::BoundingBox::from_polygon(&polygon).unwrap();
        let graph =
            build_working_graph(lines, &[polygon], bbox, 0.407, CoordinateScale::Normal).unwrap();
        let pts = &graph.boundary[0].points;
        eprintln!(
            "ORIENT contour first5={:?}",
            pts.iter()
                .take(5)
                .map(|p| (p.x(), p.y()))
                .collect::<Vec<_>>()
        );
        for idx in 0..6 {
            let i = &graph.intersections[idx];
            if let Some(ci) = i.contour_index {
                let p = graph.boundary[ci].points[i.point_index];
                eprintln!(
                    "ORIENT idx={idx} pt=({},{}) prev={} prev_pt={:?} next={} next_pt={:?}",
                    p.x(),
                    p.y(),
                    i.prev.unwrap_or(usize::MAX),
                    i.prev.map(|o| {
                        let j = &graph.intersections[o];
                        let q = graph.boundary[j.contour_index.unwrap()].points[j.point_index];
                        (q.x(), q.y())
                    }),
                    i.next.unwrap_or(usize::MAX),
                    i.next.map(|o| {
                        let j = &graph.intersections[o];
                        let q = graph.boundary[j.contour_index.unwrap()].points[j.point_index];
                        (q.x(), q.y())
                    }),
                );
            }
        }
    }
}
