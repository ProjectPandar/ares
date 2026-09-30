pub(super) fn prism() -> String {
    use std::fmt::Write;

    let points = [
        [-12283794, -10814857],
        [-12296642, -10391155],
        [-9136032, -10391163],
        [-9086722, -10254782],
        [-9120996, -10226271],
        [-9149056, -10178366],
        [-9169661, -10066925],
        [-9285236, -9970759],
        [-9307895, -9970759],
        [-9388198, -9916555],
        [-9418295, -9841921],
        [-9523716, -9770764],
        [-9554444, -9770764],
        [-9628500, -9727470],
        [-9684725, -9627600],
        [-9781946, -9570762],
        [-19771699, -9570760],
        [-19867276, -9625116],
        [-19929285, -9729354],
        [-20002093, -9770760],
        [-20009645, -9770760],
        [-20105222, -9825116],
        [-20167231, -9929354],
        [-20240039, -9970760],
        [-20247592, -9970760],
        [-20343169, -10025115],
        [-20517955, -10318937],
        [-20476885, -10391156],
        [-14156151, -10391160],
        [-14156151, -11104075],
    ]
    .map(|[x, y]| [f64::from(x) / 1_000_000.0, f64::from(y) / 1_000_000.0]);
    let vertex = |index: usize, z| [points[index][0], points[index][1], z];
    let mut stl = String::from("solid transition_snap\n");
    let mut triangle = |vertices: [[f64; 3]; 3]| {
        stl.push_str("facet normal 0 0 0\nouter loop\n");
        for [x, y, z] in vertices {
            writeln!(stl, "vertex {x} {y} {z}").unwrap();
        }
        stl.push_str("endloop\nendfacet\n");
    };
    for [a, b, c] in std::iter::once([29, 0, 1]).chain((2..29).map(|index| [1, index, index + 1])) {
        triangle([vertex(c, 0.0), vertex(b, 0.0), vertex(a, 0.0)]);
        triangle([vertex(a, 0.6), vertex(b, 0.6), vertex(c, 0.6)]);
    }
    for index in 0..30 {
        let next = (index + 1) % 30;
        triangle([vertex(index, 0.0), vertex(next, 0.0), vertex(next, 0.6)]);
        triangle([vertex(index, 0.0), vertex(next, 0.6), vertex(index, 0.6)]);
    }
    stl.push_str("endsolid transition_snap\n");
    stl
}
