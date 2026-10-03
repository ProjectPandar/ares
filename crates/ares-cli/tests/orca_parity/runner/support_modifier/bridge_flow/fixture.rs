use std::fmt::Write;

pub(super) fn landed_overhang() -> Vec<u8> {
    let cells = [
        (-15.0, -5.0, 0.0, 3.75),
        (-5.0, 5.0, 0.0, 3.75),
        (5.0, 15.0, 0.0, 3.75),
        (-5.0, 5.0, 3.75, 10.0),
        (-15.0, -5.0, 10.0, 15.0),
        (-5.0, 5.0, 10.0, 15.0),
        (5.0, 15.0, 10.0, 15.0),
    ];
    let mut stl = String::from("solid landed_overhang\n");
    for (x0, x1, z0, z1) in cells {
        for (y, sign) in [(-5.0, -1.0), (5.0, 1.0)] {
            let corners = [[x0, y, z0], [x1, y, z0], [x1, y, z1], [x0, y, z1]];
            let triangles = if sign > 0.0 {
                [[0, 2, 1], [0, 3, 2]]
            } else {
                [[0, 1, 2], [0, 2, 3]]
            };
            for indices in triangles {
                triangle(
                    &mut stl,
                    [0.0, sign, 0.0],
                    indices.map(|index| corners[index]),
                );
            }
        }
    }
    let outline: [(f64, f64); 16] = [
        (-15.0, 0.0),
        (-5.0, 0.0),
        (5.0, 0.0),
        (15.0, 0.0),
        (15.0, 3.75),
        (5.0, 3.75),
        (5.0, 10.0),
        (15.0, 10.0),
        (15.0, 15.0),
        (5.0, 15.0),
        (-5.0, 15.0),
        (-15.0, 15.0),
        (-15.0, 10.0),
        (-5.0, 10.0),
        (-5.0, 3.75),
        (-15.0, 3.75),
    ];
    for (index, &(x0, z0)) in outline.iter().enumerate() {
        let (x1, z1) = outline[(index + 1) % outline.len()];
        let corners = [[x0, -5.0, z0], [x0, 5.0, z0], [x1, 5.0, z1], [x1, -5.0, z1]];
        let normal = [(z1 - z0).signum(), 0.0, -(x1 - x0).signum()];
        for indices in [[0, 1, 2], [0, 2, 3]] {
            triangle(&mut stl, normal, indices.map(|index| corners[index]));
        }
    }
    stl.push_str("endsolid landed_overhang\n");
    stl.into_bytes()
}

fn triangle(stl: &mut String, [nx, ny, nz]: [f64; 3], points: [[f64; 3]; 3]) {
    writeln!(stl, " facet normal {nx} {ny} {nz}\n  outer loop").unwrap();
    for [x, y, z] in points {
        writeln!(stl, "   vertex {x} {y} {z}").unwrap();
    }
    stl.push_str("  endloop\n endfacet\n");
}
