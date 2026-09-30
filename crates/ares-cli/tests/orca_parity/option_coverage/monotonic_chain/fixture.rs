use std::fmt::Write;

pub(super) fn concave_slab() -> String {
    let points = [
        [3.9672, 0.5561],
        [4.2852, 1.2809],
        [4.7181, 1.9436],
        [5.2542, 2.5259],
        [5.8788, 3.0121],
        [6.5749, 3.3888],
        [7.3235, 3.6458],
        [7.9519, 3.7506],
        [10.2231, 6.0217],
        [10.0345, 6.2103],
        [4.0123, 6.2103],
        [0.9219, 3.1199],
        [1.1212, 3.012],
        [1.6536, 2.5976],
        [1.7458, 2.5259],
        [2.2819, 1.9436],
        [2.7148, 1.2809],
        [3.0328, 0.5561],
        [3.1492, 0.0963],
        [3.8508, 0.0963],
    ];
    let caps = [
        [6, 7, 8],
        [5, 6, 8],
        [4, 5, 8],
        [4, 8, 9],
        [4, 9, 10],
        [3, 4, 10],
        [2, 3, 10],
        [1, 2, 10],
        [0, 1, 10],
        [19, 0, 10],
        [10, 11, 12],
        [10, 12, 13],
        [10, 13, 14],
        [10, 14, 15],
        [19, 10, 15],
        [19, 15, 16],
        [19, 16, 17],
        [17, 18, 19],
    ];
    let mut faces = Vec::new();
    let count = points.len();
    for [a, b, c] in caps {
        faces.push([c, b, a]);
        faces.push([a + count, b + count, c + count]);
    }
    for a in 0..count {
        let b = (a + 1) % count;
        faces.extend([[a, b, b + count], [a, b + count, a + count]]);
    }
    let mut stl = String::from("solid concave_slab\n");
    for face in faces {
        stl.push_str("facet normal 0 0 0\nouter loop\n");
        for index in face {
            let [x, y] = points[index % count];
            let z = if index < count { 0.0 } else { 0.2 };
            writeln!(stl, "vertex {x} {y} {z}").unwrap();
        }
        stl.push_str("endloop\nendfacet\n");
    }
    stl.push_str("endsolid concave_slab\n");
    stl
}
