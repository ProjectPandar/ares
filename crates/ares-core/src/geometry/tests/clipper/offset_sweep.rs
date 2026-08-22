//! Offset engine parity sweep against the GCC-built clipper.cpp reference.
//! Expected vertex counts were captured from the real ClipperLib::ClipperOffset
//! (OrcaSlicer deps_src/clipper) over the configuration matrix below.

use crate::geometry::clipper::{ClipperOffset, JoinType};
use crate::geometry::{Point, Polygon};

fn quad_q0() -> Polygon {
    Polygon::new(vec![
        Point::new(35501153, -34999943),
        Point::new(35601153, -34999943),
        Point::new(35601153, -34899943),
        Point::new(35501153, -34899943),
    ])
}

fn quad_q1() -> Polygon {
    Polygon::new(vec![
        Point::new(0, 0),
        Point::new(100000, 0),
        Point::new(100000, 100000),
        Point::new(0, 100000),
    ])
}

const EXPECTED: &[&str] = &[
    "Q0 d50000 jt1 atol0.2500 ml2.0 -> [988]",
    "Q0 d50000 jt1 atol0.2500 ml3.0 -> [988]",
    "Q0 d50000 jt1 atol0.2500 ml5.0 -> [988]",
    "Q0 d50000 jt1 atol625.0000 ml2.0 -> [24]",
    "Q0 d50000 jt1 atol625.0000 ml3.0 -> [24]",
    "Q0 d50000 jt1 atol625.0000 ml5.0 -> [24]",
    "Q0 d50000 jt1 atol1250.0000 ml2.0 -> [20]",
    "Q0 d50000 jt1 atol1250.0000 ml3.0 -> [20]",
    "Q0 d50000 jt1 atol1250.0000 ml5.0 -> [20]",
    "Q0 d50000 jt1 atol2500.0000 ml2.0 -> [12]",
    "Q0 d50000 jt1 atol2500.0000 ml3.0 -> [12]",
    "Q0 d50000 jt1 atol2500.0000 ml5.0 -> [12]",
    "Q0 d50000 jt1 atol5000.0000 ml2.0 -> [12]",
    "Q0 d50000 jt1 atol5000.0000 ml3.0 -> [12]",
    "Q0 d50000 jt1 atol5000.0000 ml5.0 -> [12]",
    "Q0 d50000 jt1 atol10000.0000 ml2.0 -> [8]",
    "Q0 d50000 jt1 atol10000.0000 ml3.0 -> [8]",
    "Q0 d50000 jt1 atol10000.0000 ml5.0 -> [8]",
    "Q0 d50000 jt2 atol0.2500 ml2.0 -> [4]",
    "Q0 d50000 jt2 atol0.2500 ml3.0 -> [4]",
    "Q0 d50000 jt2 atol0.2500 ml5.0 -> [4]",
    "Q0 d50000 jt2 atol625.0000 ml2.0 -> [4]",
    "Q0 d50000 jt2 atol625.0000 ml3.0 -> [4]",
    "Q0 d50000 jt2 atol625.0000 ml5.0 -> [4]",
    "Q0 d50000 jt2 atol1250.0000 ml2.0 -> [4]",
    "Q0 d50000 jt2 atol1250.0000 ml3.0 -> [4]",
    "Q0 d50000 jt2 atol1250.0000 ml5.0 -> [4]",
    "Q0 d50000 jt2 atol2500.0000 ml2.0 -> [4]",
    "Q0 d50000 jt2 atol2500.0000 ml3.0 -> [4]",
    "Q0 d50000 jt2 atol2500.0000 ml5.0 -> [4]",
    "Q0 d50000 jt2 atol5000.0000 ml2.0 -> [4]",
    "Q0 d50000 jt2 atol5000.0000 ml3.0 -> [4]",
    "Q0 d50000 jt2 atol5000.0000 ml5.0 -> [4]",
    "Q0 d50000 jt2 atol10000.0000 ml2.0 -> [4]",
    "Q0 d50000 jt2 atol10000.0000 ml3.0 -> [4]",
    "Q0 d50000 jt2 atol10000.0000 ml5.0 -> [4]",
    "Q0 d50000 jt0 atol0.2500 ml2.0 -> [8]",
    "Q0 d50000 jt0 atol0.2500 ml3.0 -> [8]",
    "Q0 d50000 jt0 atol0.2500 ml5.0 -> [8]",
    "Q0 d50000 jt0 atol625.0000 ml2.0 -> [8]",
    "Q0 d50000 jt0 atol625.0000 ml3.0 -> [8]",
    "Q0 d50000 jt0 atol625.0000 ml5.0 -> [8]",
    "Q0 d50000 jt0 atol1250.0000 ml2.0 -> [8]",
    "Q0 d50000 jt0 atol1250.0000 ml3.0 -> [8]",
    "Q0 d50000 jt0 atol1250.0000 ml5.0 -> [8]",
    "Q0 d50000 jt0 atol2500.0000 ml2.0 -> [8]",
    "Q0 d50000 jt0 atol2500.0000 ml3.0 -> [8]",
    "Q0 d50000 jt0 atol2500.0000 ml5.0 -> [8]",
    "Q0 d50000 jt0 atol5000.0000 ml2.0 -> [8]",
    "Q0 d50000 jt0 atol5000.0000 ml3.0 -> [8]",
    "Q0 d50000 jt0 atol5000.0000 ml5.0 -> [8]",
    "Q0 d50000 jt0 atol10000.0000 ml2.0 -> [8]",
    "Q0 d50000 jt0 atol10000.0000 ml3.0 -> [8]",
    "Q0 d50000 jt0 atol10000.0000 ml5.0 -> [8]",
    "Q0 d49000 jt1 atol0.2500 ml2.0 -> [984]",
    "Q0 d49000 jt1 atol0.2500 ml3.0 -> [984]",
    "Q0 d49000 jt1 atol0.2500 ml5.0 -> [984]",
    "Q0 d49000 jt1 atol625.0000 ml2.0 -> [24]",
    "Q0 d49000 jt1 atol625.0000 ml3.0 -> [24]",
    "Q0 d49000 jt1 atol625.0000 ml5.0 -> [24]",
    "Q0 d49000 jt1 atol1250.0000 ml2.0 -> [16]",
    "Q0 d49000 jt1 atol1250.0000 ml3.0 -> [16]",
    "Q0 d49000 jt1 atol1250.0000 ml5.0 -> [16]",
    "Q0 d49000 jt1 atol2500.0000 ml2.0 -> [12]",
    "Q0 d49000 jt1 atol2500.0000 ml3.0 -> [12]",
    "Q0 d49000 jt1 atol2500.0000 ml5.0 -> [12]",
    "Q0 d49000 jt1 atol5000.0000 ml2.0 -> [12]",
    "Q0 d49000 jt1 atol5000.0000 ml3.0 -> [12]",
    "Q0 d49000 jt1 atol5000.0000 ml5.0 -> [12]",
    "Q0 d49000 jt1 atol10000.0000 ml2.0 -> [8]",
    "Q0 d49000 jt1 atol10000.0000 ml3.0 -> [8]",
    "Q0 d49000 jt1 atol10000.0000 ml5.0 -> [8]",
    "Q0 d49000 jt2 atol0.2500 ml2.0 -> [4]",
    "Q0 d49000 jt2 atol0.2500 ml3.0 -> [4]",
    "Q0 d49000 jt2 atol0.2500 ml5.0 -> [4]",
    "Q0 d49000 jt2 atol625.0000 ml2.0 -> [4]",
    "Q0 d49000 jt2 atol625.0000 ml3.0 -> [4]",
    "Q0 d49000 jt2 atol625.0000 ml5.0 -> [4]",
    "Q0 d49000 jt2 atol1250.0000 ml2.0 -> [4]",
    "Q0 d49000 jt2 atol1250.0000 ml3.0 -> [4]",
    "Q0 d49000 jt2 atol1250.0000 ml5.0 -> [4]",
    "Q0 d49000 jt2 atol2500.0000 ml2.0 -> [4]",
    "Q0 d49000 jt2 atol2500.0000 ml3.0 -> [4]",
    "Q0 d49000 jt2 atol2500.0000 ml5.0 -> [4]",
    "Q0 d49000 jt2 atol5000.0000 ml2.0 -> [4]",
    "Q0 d49000 jt2 atol5000.0000 ml3.0 -> [4]",
    "Q0 d49000 jt2 atol5000.0000 ml5.0 -> [4]",
    "Q0 d49000 jt2 atol10000.0000 ml2.0 -> [4]",
    "Q0 d49000 jt2 atol10000.0000 ml3.0 -> [4]",
    "Q0 d49000 jt2 atol10000.0000 ml5.0 -> [4]",
    "Q0 d49000 jt0 atol0.2500 ml2.0 -> [8]",
    "Q0 d49000 jt0 atol0.2500 ml3.0 -> [8]",
    "Q0 d49000 jt0 atol0.2500 ml5.0 -> [8]",
    "Q0 d49000 jt0 atol625.0000 ml2.0 -> [8]",
    "Q0 d49000 jt0 atol625.0000 ml3.0 -> [8]",
    "Q0 d49000 jt0 atol625.0000 ml5.0 -> [8]",
    "Q0 d49000 jt0 atol1250.0000 ml2.0 -> [8]",
    "Q0 d49000 jt0 atol1250.0000 ml3.0 -> [8]",
    "Q0 d49000 jt0 atol1250.0000 ml5.0 -> [8]",
    "Q0 d49000 jt0 atol2500.0000 ml2.0 -> [8]",
    "Q0 d49000 jt0 atol2500.0000 ml3.0 -> [8]",
    "Q0 d49000 jt0 atol2500.0000 ml5.0 -> [8]",
    "Q0 d49000 jt0 atol5000.0000 ml2.0 -> [8]",
    "Q0 d49000 jt0 atol5000.0000 ml3.0 -> [8]",
    "Q0 d49000 jt0 atol5000.0000 ml5.0 -> [8]",
    "Q0 d49000 jt0 atol10000.0000 ml2.0 -> [8]",
    "Q0 d49000 jt0 atol10000.0000 ml3.0 -> [8]",
    "Q0 d49000 jt0 atol10000.0000 ml5.0 -> [8]",
    "Q1 d50000 jt1 atol0.2500 ml2.0 -> [988]",
    "Q1 d50000 jt1 atol0.2500 ml3.0 -> [988]",
    "Q1 d50000 jt1 atol0.2500 ml5.0 -> [988]",
    "Q1 d50000 jt1 atol625.0000 ml2.0 -> [24]",
    "Q1 d50000 jt1 atol625.0000 ml3.0 -> [24]",
    "Q1 d50000 jt1 atol625.0000 ml5.0 -> [24]",
    "Q1 d50000 jt1 atol1250.0000 ml2.0 -> [20]",
    "Q1 d50000 jt1 atol1250.0000 ml3.0 -> [20]",
    "Q1 d50000 jt1 atol1250.0000 ml5.0 -> [20]",
    "Q1 d50000 jt1 atol2500.0000 ml2.0 -> [12]",
    "Q1 d50000 jt1 atol2500.0000 ml3.0 -> [12]",
    "Q1 d50000 jt1 atol2500.0000 ml5.0 -> [12]",
    "Q1 d50000 jt1 atol5000.0000 ml2.0 -> [12]",
    "Q1 d50000 jt1 atol5000.0000 ml3.0 -> [12]",
    "Q1 d50000 jt1 atol5000.0000 ml5.0 -> [12]",
    "Q1 d50000 jt1 atol10000.0000 ml2.0 -> [8]",
    "Q1 d50000 jt1 atol10000.0000 ml3.0 -> [8]",
    "Q1 d50000 jt1 atol10000.0000 ml5.0 -> [8]",
    "Q1 d50000 jt2 atol0.2500 ml2.0 -> [4]",
    "Q1 d50000 jt2 atol0.2500 ml3.0 -> [4]",
    "Q1 d50000 jt2 atol0.2500 ml5.0 -> [4]",
    "Q1 d50000 jt2 atol625.0000 ml2.0 -> [4]",
    "Q1 d50000 jt2 atol625.0000 ml3.0 -> [4]",
    "Q1 d50000 jt2 atol625.0000 ml5.0 -> [4]",
    "Q1 d50000 jt2 atol1250.0000 ml2.0 -> [4]",
    "Q1 d50000 jt2 atol1250.0000 ml3.0 -> [4]",
    "Q1 d50000 jt2 atol1250.0000 ml5.0 -> [4]",
    "Q1 d50000 jt2 atol2500.0000 ml2.0 -> [4]",
    "Q1 d50000 jt2 atol2500.0000 ml3.0 -> [4]",
    "Q1 d50000 jt2 atol2500.0000 ml5.0 -> [4]",
    "Q1 d50000 jt2 atol5000.0000 ml2.0 -> [4]",
    "Q1 d50000 jt2 atol5000.0000 ml3.0 -> [4]",
    "Q1 d50000 jt2 atol5000.0000 ml5.0 -> [4]",
    "Q1 d50000 jt2 atol10000.0000 ml2.0 -> [4]",
    "Q1 d50000 jt2 atol10000.0000 ml3.0 -> [4]",
    "Q1 d50000 jt2 atol10000.0000 ml5.0 -> [4]",
    "Q1 d50000 jt0 atol0.2500 ml2.0 -> [8]",
    "Q1 d50000 jt0 atol0.2500 ml3.0 -> [8]",
    "Q1 d50000 jt0 atol0.2500 ml5.0 -> [8]",
    "Q1 d50000 jt0 atol625.0000 ml2.0 -> [8]",
    "Q1 d50000 jt0 atol625.0000 ml3.0 -> [8]",
    "Q1 d50000 jt0 atol625.0000 ml5.0 -> [8]",
    "Q1 d50000 jt0 atol1250.0000 ml2.0 -> [8]",
    "Q1 d50000 jt0 atol1250.0000 ml3.0 -> [8]",
    "Q1 d50000 jt0 atol1250.0000 ml5.0 -> [8]",
    "Q1 d50000 jt0 atol2500.0000 ml2.0 -> [8]",
    "Q1 d50000 jt0 atol2500.0000 ml3.0 -> [8]",
    "Q1 d50000 jt0 atol2500.0000 ml5.0 -> [8]",
    "Q1 d50000 jt0 atol5000.0000 ml2.0 -> [8]",
    "Q1 d50000 jt0 atol5000.0000 ml3.0 -> [8]",
    "Q1 d50000 jt0 atol5000.0000 ml5.0 -> [8]",
    "Q1 d50000 jt0 atol10000.0000 ml2.0 -> [8]",
    "Q1 d50000 jt0 atol10000.0000 ml3.0 -> [8]",
    "Q1 d50000 jt0 atol10000.0000 ml5.0 -> [8]",
    "Q1 d49000 jt1 atol0.2500 ml2.0 -> [984]",
    "Q1 d49000 jt1 atol0.2500 ml3.0 -> [984]",
    "Q1 d49000 jt1 atol0.2500 ml5.0 -> [984]",
    "Q1 d49000 jt1 atol625.0000 ml2.0 -> [24]",
    "Q1 d49000 jt1 atol625.0000 ml3.0 -> [24]",
    "Q1 d49000 jt1 atol625.0000 ml5.0 -> [24]",
    "Q1 d49000 jt1 atol1250.0000 ml2.0 -> [16]",
    "Q1 d49000 jt1 atol1250.0000 ml3.0 -> [16]",
    "Q1 d49000 jt1 atol1250.0000 ml5.0 -> [16]",
    "Q1 d49000 jt1 atol2500.0000 ml2.0 -> [12]",
    "Q1 d49000 jt1 atol2500.0000 ml3.0 -> [12]",
    "Q1 d49000 jt1 atol2500.0000 ml5.0 -> [12]",
    "Q1 d49000 jt1 atol5000.0000 ml2.0 -> [12]",
    "Q1 d49000 jt1 atol5000.0000 ml3.0 -> [12]",
    "Q1 d49000 jt1 atol5000.0000 ml5.0 -> [12]",
    "Q1 d49000 jt1 atol10000.0000 ml2.0 -> [8]",
    "Q1 d49000 jt1 atol10000.0000 ml3.0 -> [8]",
    "Q1 d49000 jt1 atol10000.0000 ml5.0 -> [8]",
    "Q1 d49000 jt2 atol0.2500 ml2.0 -> [4]",
    "Q1 d49000 jt2 atol0.2500 ml3.0 -> [4]",
    "Q1 d49000 jt2 atol0.2500 ml5.0 -> [4]",
    "Q1 d49000 jt2 atol625.0000 ml2.0 -> [4]",
    "Q1 d49000 jt2 atol625.0000 ml3.0 -> [4]",
    "Q1 d49000 jt2 atol625.0000 ml5.0 -> [4]",
    "Q1 d49000 jt2 atol1250.0000 ml2.0 -> [4]",
    "Q1 d49000 jt2 atol1250.0000 ml3.0 -> [4]",
    "Q1 d49000 jt2 atol1250.0000 ml5.0 -> [4]",
    "Q1 d49000 jt2 atol2500.0000 ml2.0 -> [4]",
    "Q1 d49000 jt2 atol2500.0000 ml3.0 -> [4]",
    "Q1 d49000 jt2 atol2500.0000 ml5.0 -> [4]",
    "Q1 d49000 jt2 atol5000.0000 ml2.0 -> [4]",
    "Q1 d49000 jt2 atol5000.0000 ml3.0 -> [4]",
    "Q1 d49000 jt2 atol5000.0000 ml5.0 -> [4]",
    "Q1 d49000 jt2 atol10000.0000 ml2.0 -> [4]",
    "Q1 d49000 jt2 atol10000.0000 ml3.0 -> [4]",
    "Q1 d49000 jt2 atol10000.0000 ml5.0 -> [4]",
    "Q1 d49000 jt0 atol0.2500 ml2.0 -> [8]",
    "Q1 d49000 jt0 atol0.2500 ml3.0 -> [8]",
    "Q1 d49000 jt0 atol0.2500 ml5.0 -> [8]",
    "Q1 d49000 jt0 atol625.0000 ml2.0 -> [8]",
    "Q1 d49000 jt0 atol625.0000 ml3.0 -> [8]",
    "Q1 d49000 jt0 atol625.0000 ml5.0 -> [8]",
    "Q1 d49000 jt0 atol1250.0000 ml2.0 -> [8]",
    "Q1 d49000 jt0 atol1250.0000 ml3.0 -> [8]",
    "Q1 d49000 jt0 atol1250.0000 ml5.0 -> [8]",
    "Q1 d49000 jt0 atol2500.0000 ml2.0 -> [8]",
    "Q1 d49000 jt0 atol2500.0000 ml3.0 -> [8]",
    "Q1 d49000 jt0 atol2500.0000 ml5.0 -> [8]",
    "Q1 d49000 jt0 atol5000.0000 ml2.0 -> [8]",
    "Q1 d49000 jt0 atol5000.0000 ml3.0 -> [8]",
    "Q1 d49000 jt0 atol5000.0000 ml5.0 -> [8]",
    "Q1 d49000 jt0 atol10000.0000 ml2.0 -> [8]",
    "Q1 d49000 jt0 atol10000.0000 ml3.0 -> [8]",
    "Q1 d49000 jt0 atol10000.0000 ml5.0 -> [8]",
];

#[test]
fn task22f_offset_sweep_matches_oracle_counts() {
    let quads = [quad_q0(), quad_q1()];
    let joins = [
        (JoinType::Round, 1),
        (JoinType::Miter, 2),
        (JoinType::Square, 0),
    ];
    let deltas = [50000.0f64, 49000.0];
    let arc_tols = [0.25f64, 625.0, 1250.0, 2500.0, 5000.0, 10000.0];
    let miter_limits = [2.0f64, 3.0, 5.0];
    let configurations: Vec<(f64, f64)> = arc_tols
        .iter()
        .flat_map(|&arc_tol| miter_limits.iter().map(move |&ml| (arc_tol, ml)))
        .collect();
    let mut lines = Vec::new();
    for quad in &quads {
        for &delta in &deltas {
            for &(join, jn) in &joins {
                sweep_configurations(quad, delta, (join, jn), &configurations, &mut lines);
            }
        }
    }
    assert_eq!(lines, EXPECTED, "offset sweep diverges from oracle");
}

fn sweep_configurations(
    quad: &Polygon,
    delta: f64,
    (join, jn): (JoinType, u8),
    configurations: &[(f64, f64)],
    lines: &mut Vec<String>,
) {
    for &(arc_tol, ml) in configurations {
        let mut co = ClipperOffset::default();
        if join == JoinType::Round {
            co.set_arc_tolerance(arc_tol);
        } else {
            co.set_miter_limit(ml);
        }
        co.set_shortest_edge_length((delta * 0.005).abs());
        co.add_closed_path(quad, join);
        let out = co.execute_paths(delta).unwrap();
        let sizes: Vec<String> = out
            .iter()
            .map(|p| format!("[{}]", p.points().len()))
            .collect();
        lines.push(format!(
            "Q{} d{delta:.0} jt{jn} atol{arc_tol:.4} ml{ml:.1} -> {}",
            if quad.points()[0] == Point::new(35501153, -34999943) {
                0
            } else {
                1
            },
            sizes.join(" ")
        ));
    }
}
