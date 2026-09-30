pub(super) fn prism() -> String {
    use std::fmt::Write;

    let points = [
        [-3348537, -8522528],
        [-3348537, -7702128],
        [-14357302, -7702128],
        [-14419861, -7673625],
        [-14500245, -7581182],
        [-14969248, -7554130],
        [-15112089, -7681392],
        [-15166540, -7702128],
        [-26651459, -7702128],
        [-26651459, -8522529],
        [-17878943, -8522527],
        [-17592718, -8041368],
        [-14874901, -8041368],
        [-14874901, -8522529],
        [-14659759, -8522529],
        [-14620048, -8179472],
        [-9238182, -8179472],
        [-9238182, -8522531],
    ]
    .map(|[x, y]| [f64::from(x) / 1_000_000.0, f64::from(y) / 1_000_000.0]);
    let caps = [
        [17, 0, 1],
        [3, 4, 5],
        [2, 3, 5],
        [2, 5, 6],
        [2, 6, 7],
        [7, 8, 9],
        [2, 7, 9],
        [9, 10, 11],
        [2, 9, 11],
        [1, 2, 11],
        [1, 11, 12],
        [12, 13, 14],
        [12, 14, 15],
        [1, 12, 15],
        [1, 15, 16],
        [1, 16, 17],
    ];
    let vertex = |index: usize, z| [points[index][0], points[index][1], z];
    let mut stl = String::from("solid stitch_junction\n");
    let mut triangle = |vertices: [[f64; 3]; 3]| {
        stl.push_str("facet normal 0 0 0\nouter loop\n");
        for [x, y, z] in vertices {
            writeln!(stl, "vertex {x} {y} {z}").unwrap();
        }
        stl.push_str("endloop\nendfacet\n");
    };
    for [a, b, c] in caps {
        triangle([vertex(c, 0.0), vertex(b, 0.0), vertex(a, 0.0)]);
        triangle([vertex(a, 0.6), vertex(b, 0.6), vertex(c, 0.6)]);
    }
    for index in 0..18 {
        let next = (index + 1) % 18;
        triangle([vertex(index, 0.0), vertex(next, 0.0), vertex(next, 0.6)]);
        triangle([vertex(index, 0.0), vertex(next, 0.6), vertex(index, 0.6)]);
    }
    stl.push_str("endsolid stitch_junction\n");
    stl
}
