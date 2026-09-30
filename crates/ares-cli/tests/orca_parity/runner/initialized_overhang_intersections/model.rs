pub(super) fn loft() -> String {
    use std::fmt::Write;

    let previous = [
        [37_500_000, -35_000_000],
        [37_500_000, -3_487_053],
        [35_500_000, -3_487_053],
        [35_500_000, -23_000_000],
        [26_914_216, -23_000_000],
        [4_208_360, -294_145],
        [2_794_144, -1_708_360],
        [25_500_000, -24_414_216],
        [25_500_000, -33_000_000],
        [5_987_053, -33_000_000],
        [5_987_053, -35_000_000],
    ];
    let current = [
        [37_500_000, -35_000_000],
        [37_500_000, -3_140_641],
        [35_500_000, -3_140_641],
        [35_500_000, -23_000_000],
        [26_914_216, -23_000_000],
        [3_963_410, -49_195],
        [2_549_195, -1_463_410],
        [25_500_000, -24_414_216],
        [25_500_000, -33_000_000],
        [5_640_641, -33_000_000],
        [5_640_641, -35_000_000],
    ];
    let layers = [
        (previous, 0.0),
        (previous, 0.1),
        (current, 0.3),
        (current, 0.4),
    ];
    let caps = [
        [0, 1, 2],
        [0, 2, 3],
        [0, 3, 4],
        [0, 4, 5],
        [0, 5, 6],
        [0, 6, 7],
        [0, 7, 8],
        [10, 0, 8],
        [8, 9, 10],
    ];
    let mut stl = String::from("solid floating_intersections\n");
    for sign in [1.0, -1.0] {
        let vertex = |layer: usize, index: usize| {
            let (points, z) = layers[layer];
            [
                sign * f64::from(points[index][0]) / 1_000_000.0,
                sign * f64::from(points[index][1]) / 1_000_000.0,
                z,
            ]
        };
        let mut triangle = |vertices: [[f64; 3]; 3]| {
            stl.push_str("facet normal 0 0 0\nouter loop\n");
            for [x, y, z] in vertices {
                writeln!(stl, "vertex {x} {y} {z}").unwrap();
            }
            stl.push_str("endloop\nendfacet\n");
        };
        for [a, b, c] in caps {
            triangle([vertex(0, c), vertex(0, b), vertex(0, a)]);
            triangle([vertex(3, a), vertex(3, b), vertex(3, c)]);
        }
        for layer in 0..3 {
            for index in 0..11 {
                let next = (index + 1) % 11;
                triangle([
                    vertex(layer, index),
                    vertex(layer, next),
                    vertex(layer + 1, next),
                ]);
                triangle([
                    vertex(layer, index),
                    vertex(layer + 1, next),
                    vertex(layer + 1, index),
                ]);
            }
        }
    }
    stl.push_str("endsolid floating_intersections\n");
    stl
}
