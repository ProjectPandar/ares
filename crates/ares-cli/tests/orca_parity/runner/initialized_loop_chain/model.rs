use std::fmt::Write;

pub(super) fn plate() -> String {
    let xs = [-20.0, -16.0, -14.0, -1.0, 1.0, 14.0, 16.0, 20.0];
    let ys = [-5.0, -1.0, 1.0, 5.0];
    let mut output = String::from("solid three_posts_on_plate\n");
    for (x, y) in (0..3).flat_map(|y| (0..7).map(move |x| (x, y))) {
        let height = cell_height(x, y);
        let corners = [
            [xs[x as usize], ys[y as usize]],
            [xs[x as usize + 1], ys[y as usize]],
            [xs[x as usize + 1], ys[y as usize + 1]],
            [xs[x as usize], ys[y as usize + 1]],
        ];
        append_quad(&mut output, corners.map(|[x, y]| [x, y, height]));
        let mut bottom = corners.map(|[x, y]| [x, y, 0.0]);
        bottom.reverse();
        append_quad(&mut output, bottom);
        for (edge, (nx, ny)) in [(x, y - 1), (x + 1, y), (x, y + 1), (x - 1, y)]
            .into_iter()
            .enumerate()
        {
            let low = cell_height(nx, ny);
            if low >= height {
                continue;
            }
            let [ax, ay] = corners[edge];
            let [bx, by] = corners[(edge + 1) % 4];
            append_quad(
                &mut output,
                [
                    [ax, ay, low],
                    [bx, by, low],
                    [bx, by, height],
                    [ax, ay, height],
                ],
            );
        }
    }
    output.push_str("endsolid three_posts_on_plate\n");
    output
}

fn cell_height(x: i32, y: i32) -> f64 {
    if !(0..7).contains(&x) || !(0..3).contains(&y) {
        0.0
    } else if x % 2 == 1 && y == 1 {
        0.4
    } else {
        0.2
    }
}

fn append_quad(output: &mut String, points: [[f64; 3]; 4]) {
    for [a, b, c] in [[0, 1, 2], [0, 2, 3]] {
        let u = std::array::from_fn::<_, 3, _>(|i| points[b][i] - points[a][i]);
        let v = std::array::from_fn::<_, 3, _>(|i| points[c][i] - points[a][i]);
        let normal = [
            u[1] * v[2] - u[2] * v[1],
            u[2] * v[0] - u[0] * v[2],
            u[0] * v[1] - u[1] * v[0],
        ];
        let length = normal.iter().map(|value| value * value).sum::<f64>().sqrt();
        let [nx, ny, nz] = normal.map(|value| value / length);
        writeln!(output, "facet normal {nx} {ny} {nz}\nouter loop").unwrap();
        for [x, y, z] in [points[a], points[b], points[c]] {
            writeln!(output, "vertex {x} {y} {z}").unwrap();
        }
        output.push_str("endloop\nendfacet\n");
    }
}
