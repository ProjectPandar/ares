use super::*;
use crate::geometry::Point;

fn square(min: Coord, max: Coord) -> ExPolygon {
    ExPolygon::new(
        Polygon::new(vec![
            Point::new(min, min),
            Point::new(max, min),
            Point::new(max, max),
            Point::new(min, max),
        ]),
        Vec::new(),
    )
}

/// Containment envelope of a polygon set: (min_x, max_x) across all
/// points, used to compare against oracle fill spans.
fn x_span(polygons: &[Polygon]) -> (Coord, Coord) {
    let mut min = Coord::MAX;
    let mut max = Coord::MIN;
    for point in polygons.iter().flat_map(|polygon| polygon.points()) {
        min = min.min(point.x());
        max = max.max(point.x());
    }
    (min, max)
}

fn params(raft_layers: usize, raft_expansion_microns: Coord) -> RaftPolygonParams {
    RaftPolygonParams {
        raft_expansion: raft_expansion_microns,
        first_layer_expansion: 0,
        raft_layers,
        grid: SupportGridParams::new(2440, 420),
    }
}

/// The oracle chain on the KSR cube (lslices 105..115mm,
/// raft_expansion 1.5, grid resolution ≈ 2.44mm+):
/// interface = contact + 0.5 (jtSquare), base = interface,
/// first layer = base (first_layer_expansion 0 → max(0, 0 − 0.5) = 0).
#[test]
fn chain_grows_contact_to_interface_by_half_millimeter() {
    let lslices = [square(105_000, 115_000)];
    let params = params(2, 1_500);

    let polygons = raft_polygons(&lslices, &params).unwrap();

    let (contact_min, contact_max) = x_span(&polygons.contact);
    let (interface_min, interface_max) = x_span(&polygons.interface);
    // Interface = contact + 0.5mm on each side (square join).
    assert_eq!(interface_min, contact_min - 500);
    assert_eq!(interface_max, contact_max + 500);
    // Contact contains the raft_expansion-expanded silhouette: at
    // least lslices + 1.5mm on each side (grid growth only adds).
    assert!(contact_min <= 105_000 - 1_500);
    assert!(contact_max >= 115_000 + 1_500);
    // Base equals interface in the raft-only path; first layer equals
    // base when first_layer_expansion is 0.
    assert_eq!(x_span(&polygons.base), (interface_min, interface_max));
    assert_eq!(
        x_span(&polygons.first_layer),
        (interface_min, interface_max)
    );
}

/// `first_layer_expansion > fine` grows only the 1st layer
/// (`SupportCommon.cpp:279-281`).
#[test]
fn first_layer_expansion_grows_only_first_layer() {
    let lslices = [square(105_000, 115_000)];
    let mut params = params(2, 1_500);
    params.first_layer_expansion = 2_000;

    let polygons = raft_polygons(&lslices, &params).unwrap();

    let (base_min, base_max) = x_span(&polygons.base);
    let (first_min, first_max) = x_span(&polygons.first_layer);
    // first = base + (2.0 − 0.5)mm on each side.
    assert_eq!(first_min, base_min - 1_500);
    assert_eq!(first_max, base_max + 1_500);
}

/// `raft_layers == 1`: fine = 0, so interface == contact == base ==
/// first layer (all degenerate to the single contact layer).
#[test]
fn single_layer_raft_has_no_fine_expansion() {
    let lslices = [square(105_000, 115_000)];
    let params = params(1, 1_500);

    let polygons = raft_polygons(&lslices, &params).unwrap();

    assert_eq!(x_span(&polygons.contact), x_span(&polygons.interface));
    assert_eq!(x_span(&polygons.interface), x_span(&polygons.base));
    assert_eq!(x_span(&polygons.base), x_span(&polygons.first_layer));
}

/// Holes in the first layer lslices are filled by the grid stretch
/// (`fill_holes=true`, `SupportMaterial.cpp:1861-1887`).
#[test]
fn holes_do_not_reach_the_contact_silhouette() {
    let hole = {
        let mut hole = Polygon::new(vec![
            Point::new(108_000, 108_000),
            Point::new(112_000, 108_000),
            Point::new(112_000, 112_000),
            Point::new(108_000, 112_000),
        ]);
        hole.reverse();
        hole
    };
    let lslices = [ExPolygon::new(
        square(105_000, 115_000).into_parts().0,
        vec![hole],
    )];
    let params = params(2, 1_500);

    let polygons = raft_polygons(&lslices, &params).unwrap();

    // Exactly one outer contact contour (the 4mm hole is grid-filled).
    assert_eq!(polygons.contact.len(), 1);
}
