//! `FillHoneycomb` port (`src/libslic3r/Fill/FillHoneycomb.cpp`):
//! hexagonal-lattice sparse infill. Generates a vertical hex-cell lattice in
//! pattern space, rotates it by the infill direction about the cell center,
//! expands by multiline, clips to the surface, and chains the segments.
//!
//! Used both for the emitted sparse fill and the bridge-over-infill anchor
//! projection (upstream `Fill::new_from_type` generates real polylines for
//! every pattern; see `Fill.cpp:1394`).

use crate::geometry::{
    BoundingBox, ClipperError, CoordinateScale, ExPolygon, Point, Polygon, Polyline,
    douglas_peucker, intersection_open_polylines,
};

/// Parameters carried into `fill_honeycomb` — subset of the surface-fill
/// params the upstream `FillParams` provides.
#[derive(Clone, Copy)]
pub(crate) struct HoneycombFillParams {
    /// Regional runtime flag from Fill.cpp:1235-1242 / FillBase.cpp:311-313.
    pub(crate) dont_alternate_fill_direction: bool,
    /// Line spacing in mm (unscaled).
    pub(crate) spacing: f64,
    /// Infill density fraction (0..1) — upstream `params.density` is a percent
    /// handled by the caller; here it is already a 0..1 fraction.
    pub(crate) density: f32,
    /// Multiline overlap count.
    pub(crate) multiline: i32,
    /// Infill direction angle (radians) BEFORE `Fill::_infill_direction`
    /// adjustments (`+pi/2`, layer alternation).
    pub(crate) angle: f32,
    /// `layer_id` — drives the `+_layer_angle` alternation.
    pub(crate) layer_index: usize,
    /// `thickness_layers` — divisor for the alternation index.
    pub(crate) thickness_layers: u16,
    /// `fixed_angle` — skips the layer alternation when true.
    pub(crate) fixed_angle: bool,
    /// Bridge angle override (`surface->bridge_angle >= 0`).
    pub(crate) bridge_angle: Option<f32>,
    /// Anchor lengths forwarded to `connect_infill`.
    pub(crate) anchor_length: f32,
    pub(crate) anchor_length_max: f32,
}

// `Point::rotate(angle, center)` (`Point.cpp:50`): rotate about an arbitrary
// center using `fast_round_up` (floor(x+0.5)), NOT std::round.
fn rotate_about(point: Point, angle: f64, center: Point) -> Point {
    let s = angle.sin();
    let c = angle.cos();
    let dx = (point.x() - center.x()) as f64;
    let dy = (point.y() - center.y()) as f64;
    Point::new(
        (center.x() as f64 + c * dx - s * dy + 0.5).floor() as i64,
        (center.y() as f64 + s * dx + c * dy + 0.5).floor() as i64,
    )
}

fn rotate_polygon(polygon: &Polygon, angle: f64, center: Point) -> Polygon {
    Polygon::new(
        polygon
            .points()
            .iter()
            .map(|p| rotate_about(*p, angle, center))
            .collect(),
    )
}

fn rotate_polyline(polyline: &Polyline, angle: f64, center: Point) -> Polyline {
    Polyline::new(
        polyline
            .points()
            .iter()
            .map(|p| rotate_about(*p, angle, center))
            .collect(),
    )
}

// `align_to_grid(coord, spacing)` (`Point.hpp:709`) — round DOWN for negatives.
fn align_scalar(coord: i64, spacing: i64) -> i64 {
    if coord < 0 {
        ((coord - spacing + 1) / spacing) * spacing
    } else {
        (coord / spacing) * spacing
    }
}

fn bounding_box_polygon(bb: BoundingBox) -> Polygon {
    let (min, max) = (bb.min(), bb.max());
    Polygon::new(vec![
        min,
        Point::new(max.x(), min.y()),
        max,
        Point::new(min.x(), max.y()),
    ])
}

fn polygon_bbox(polygon: &Polygon) -> BoundingBox {
    let mut it = polygon.points().iter();
    let first = *it.next().unwrap_or(&Point::new(0, 0));
    it.fold(BoundingBox::new(first, first), |bb, p| {
        BoundingBox::new(
            Point::new(bb.min().x().min(p.x()), bb.min().y().min(p.y())),
            Point::new(bb.max().x().max(p.x()), bb.max().y().max(p.y())),
        )
    })
}

fn append_hexagon_column(
    pts: &mut Vec<Point>,
    ax: [i64; 2],
    bb: BoundingBox,
    (y_short, hex_side, y_offset): (i64, i64, i64),
) {
    let mut y = bb.min().y();
    while y <= bb.max().y() {
        pts.push(Point::new(ax[1], y + y_offset));
        pts.push(Point::new(ax[0], y + y_short - y_offset));
        pts.push(Point::new(ax[0], y + y_short + hex_side + y_offset));
        pts.push(Point::new(
            ax[1],
            y + y_short + hex_side + y_short - y_offset,
        ));
        pts.push(Point::new(
            ax[1],
            y + y_short + hex_side + y_short + hex_side + y_offset,
        ));
        y += y_short + hex_side + y_short + hex_side;
    }
}

/// `FillHoneycomb::_fill_surface_single` (`FillHoneycomb.cpp:9`). Upstream
/// rotates the bounding-box polygon and output polylines about `hex_center`
/// (the pattern cell anchor), NOT the object center.
pub(crate) fn fill_honeycomb(
    expolygon: &ExPolygon,
    params: HoneycombFillParams,
    scale: CoordinateScale,
) -> Result<Vec<Polyline>, ClipperError> {
    // Cache math (`CacheData`, FillHoneycomb.cpp:21-32):
    let min_spacing = (params.spacing / scale.factor()) as i64 * i64::from(params.multiline);
    let distance = (min_spacing as f32 / params.density) as i64;
    let hex_side = (distance as f64 / (3.0_f64.sqrt() / 2.0)) as i64;
    let hex_width = distance * 2;
    let hex_height = hex_side * 2;
    let pattern_height = hex_height + hex_side;
    let y_short = (distance as f64 * 3.0_f64.sqrt() / 3.0) as i64;
    let x_offset = min_spacing / 2;
    let y_offset = (x_offset as f64 * 3.0_f64.sqrt() / 3.0) as i64;
    let hex_center = Point::new(hex_width / 2, hex_side);

    // `Fill::_infill_direction` (`FillBase.cpp:275-323`): `out_angle =
    // bridge_angle.unwrap_or(angle)`, `+ _layer_angle` when
    // `!fixed_angle && !dont_alternate_fill_direction`,
    // then `+ pi/2`.
    let mut direction_f = params.bridge_angle.unwrap_or(params.angle);
    if params.bridge_angle.is_none() && !params.fixed_angle && !params.dont_alternate_fill_direction
    {
        // FillHoneycomb::_layer_angle overrides the base two-layer alternation.
        let layer = params.layer_index / usize::from(params.thickness_layers);
        direction_f += (std::f64::consts::PI / 3.0) as f32 * (layer % 3) as f32;
    }
    let direction = f64::from(direction_f + std::f32::consts::FRAC_PI_2);
    let mut all_polylines: Vec<Polyline> = Vec::new();
    {
        // Rotate the contour bbox by `direction` about hex_center, then align
        // the bbox to the hex grid (world-aligned, FillHoneycomb.cpp:39-52).
        let bb0 = polygon_bbox(&Polygon::new(expolygon.contour().points().to_vec()));
        let rotated = rotate_polygon(&bounding_box_polygon(bb0), direction, hex_center);
        let rbb = polygon_bbox(&rotated);
        // `bounding_box.merge(align_to_grid(bounding_box.min, Point(hex_width, pattern_height)))`
        let aligned = Point::new(
            align_scalar(rbb.min().x(), hex_width),
            align_scalar(rbb.min().y(), pattern_height),
        );
        let bb = BoundingBox::new(
            Point::new(
                rbb.min().x().min(aligned.x()),
                rbb.min().y().min(aligned.y()),
            ),
            Point::new(
                rbb.max().x().max(aligned.x()),
                rbb.max().y().max(aligned.y()),
            ),
        );

        let mut x = bb.min().x();
        while x <= bb.max().x() {
            let mut ax = [x + x_offset, x + distance - x_offset];
            let mut pts: Vec<Point> = Vec::new();
            for _ in 0..2 {
                pts.reverse(); // turn first half upside down
                append_hexagon_column(&mut pts, ax, bb, (y_short, hex_side, y_offset));
                ax[0] += distance;
                ax[1] += distance;
                ax.swap(0, 1);
                x += distance;
            }
            let p = rotate_polyline(&Polyline::new(pts), -direction, hex_center);
            // Upstream passes unscaled spacing even though points are scaled.
            let tol = 5.0 * params.spacing;
            all_polylines.push(Polyline::new(douglas_peucker(p.points(), tol)));
        }
    }
    // `multiline_fill` — expand by the multiline overlap count.
    let all_polylines = if params.multiline > 1 {
        crate::fill::multiline_offset::apply(
            all_polylines,
            params.multiline,
            params.spacing,
            scale,
        )?
    } else {
        all_polylines
    };
    // `intersection_pl` + `chain_or_connect_infill` (FillHoneycomb.cpp:80-81).
    let (contour, holes) = (expolygon.contour(), expolygon.holes());
    let mut clip = Vec::with_capacity(holes.len() + 1);
    clip.push(contour.clone());
    clip.extend(holes.iter().cloned());
    let clipped = intersection_open_polylines(&all_polylines, &clip)?;
    if clipped.is_empty() {
        return Ok(Vec::new());
    }
    crate::fill::connect::connect_infill(
        clipped,
        expolygon,
        params.spacing,
        crate::fill::connect::FillConnectionParams {
            anchor_length: params.anchor_length,
            anchor_length_max: params.anchor_length_max,
            multiline: params.multiline,
            dont_sort: false,
        },
        scale,
    )
}
