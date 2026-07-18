use super::{edge, line, points, vertex};
use crate::mesh_slicer::{chain_lines_by_triangle_connectivity, make_loops};

#[test]
fn task22d_make_loops_keeps_closed_polygons_and_discards_residual_opens() {
    let chained = chain_lines_by_triangle_connectivity(vec![
        line((0, 0, edge(1)), (4, 0, edge(2))),
        line((4, 0, edge(2)), (1, 3, edge(3))),
        line((1, 3, edge(3)), (0, 0, edge(1))),
        line((10_000_000, 0, vertex(10)), (20_000_000, 0, vertex(11))),
    ]);

    assert_eq!(chained.polygons().len(), 1);
    assert_eq!(chained.open_polylines().len(), 1);

    let looped = make_loops(chained, 2_000_000);

    assert_eq!(looped.polygons().len(), 1);
    assert_eq!(
        looped.polygons()[0].points(),
        points(&[(0, 0), (4, 0), (1, 3)])
    );
}

#[test]
fn task22d_make_loops_runs_four_repair_passes_in_source_order() {
    let chained = chain_lines_by_triangle_connectivity(vec![
        line((0, 0, vertex(40)), (4, 0, edge(0))),
        line((4, 0, vertex(0)), (4, 3, edge(41))),
        line((4, 0, vertex(0)), (0, 0, vertex(40))),
        line((100, 0, vertex(50)), (140, 0, edge(51))),
        line((100, 0, vertex(50)), (120, 30, edge(52))),
        line((120, 30, edge(52)), (140, 0, edge(51))),
        line((200, 0, vertex(60)), (230, 0, edge(61))),
        line((230, 0, edge(61)), (201, 0, vertex(62))),
        line((300, 0, vertex(70)), (300, 100, edge(71))),
        line((300, 100, edge(71)), (400, 100, edge(72))),
        line((300, 1, vertex(73)), (400, 0, edge(74))),
        line((400, 0, edge(74)), (400, 99, vertex(75))),
        line((500, 0, vertex(80)), (600, 0, vertex(81))),
    ]);

    assert!(chained.polygons().is_empty());
    assert_eq!(chained.open_polylines().len(), 9);

    let looped = make_loops(chained, 2);

    let polygons = looped.polygons();
    assert_eq!(polygons.len(), 4);
    assert_eq!(
        polygons[0].points(),
        points(&[(140, 0), (120, 30), (100, 0)])
    );
    assert_eq!(
        polygons[1].points(),
        points(&[(200, 0), (230, 0), (201, 0)])
    );
    assert_eq!(polygons[2].points(), points(&[(0, 0), (4, 0), (4, 3)]));
    assert_eq!(
        polygons[3].points(),
        points(&[
            (300, 1),
            (400, 0),
            (400, 99),
            (400, 100),
            (300, 100),
            (300, 0),
        ])
    );
}
