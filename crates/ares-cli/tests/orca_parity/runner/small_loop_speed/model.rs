use std::fmt::Write;

#[path = "loft.rs"]
mod loft;

#[derive(Clone, Copy)]
pub(super) enum Shape {
    Square,
    ThinWall,
    OverhangLoft,
}

pub(super) fn stl(shape: Shape) -> String {
    let half_x = match shape {
        Shape::OverhangLoft => return loft::loft(),
        Shape::Square => 4.0,
        Shape::ThinWall => 0.15,
    };
    let corners = [
        [-half_x, -4.0],
        [half_x, -4.0],
        [half_x, 4.0],
        [-half_x, 4.0],
    ];
    let mut output = String::from("solid small_loop_boundary\n");
    quad(&mut output, corners.map(|[x, y]| [x, y, 0.6]));
    let mut bottom = corners.map(|[x, y]| [x, y, 0.0]);
    bottom.reverse();
    quad(&mut output, bottom);
    for edge in 0..4 {
        let [ax, ay] = corners[edge];
        let [bx, by] = corners[(edge + 1) % 4];
        quad(
            &mut output,
            [[ax, ay, 0.0], [bx, by, 0.0], [bx, by, 0.6], [ax, ay, 0.6]],
        );
    }
    output.push_str("endsolid small_loop_boundary\n");
    output
}

fn quad(output: &mut String, points: [[f64; 3]; 4]) {
    for [a, b, c] in [[0, 1, 2], [0, 2, 3]] {
        output.push_str("facet normal 0 0 0\nouter loop\n");
        for [x, y, z] in [points[a], points[b], points[c]] {
            writeln!(output, "vertex {x} {y} {z}").unwrap();
        }
        output.push_str("endloop\nendfacet\n");
    }
}
