//! `FillLightning::Generator` (`src/libslic3r/Fill/Lightning/Generator.cpp`):
//! builds the lightning tree across ALL layers top-to-bottom, then
//! `getTreesForLayer(layer_id)` returns the per-layer `Layer` whose
//! `convertToLines` emits the infill. `generateInitialInternalOverhangs`
//! computes the per-layer overhang (internal surfaces minus walls via
//! `m_wall_supporting_radius`); `generateTrees` propagates trees downward.

use crate::geometry::{
    BoundingBox, ClipperError, Coord, CoordinateScale, EdgeGrid, ExPolygon, FillRule, JoinType,
    Point, Polygon, Polyline, difference_polygons_ex, offset_expolygons,
};

use super::layer::{LOCATOR_CELL_SIZE, Layer};
use super::tree_node::Node;

/// `to_polygons(ExPolygons)` — flatten each expolygon to contour+holes.
fn to_polygons(expolys: &[ExPolygon]) -> Vec<Polygon> {
    expolys
        .iter()
        .flat_map(|e| {
            let mut v = vec![e.contour().clone()];
            v.extend(e.holes().iter().cloned());
            v
        })
        .collect()
}

/// `FillLightning::Generator`.
pub(crate) struct Generator {
    supporting_radius: Coord,
    wall_supporting_radius: Coord,
    prune_length: Coord,
    straightening_max_distance: Coord,
    overhang_per_layer: Vec<Vec<Polygon>>,
    lightning_layers: Vec<Layer>,
}

impl Generator {
    /// `Generator::Generator` params (Generator.cpp:66-99): supporting radius
    /// `width*100*multiline/density`, and the overhang/prune/straighten radii
    /// from `layer_height * tan(angle)`.
    pub(crate) fn new(
        layer_internal_outlines: Vec<Vec<Polygon>>,
        layer_height: f64,
        infill_line_width: f64,
        multiline: i32,
        density_percent: f64,
        overhang_angle_deg: f64,
        prune_angle_deg: f64,
        straightening_angle_deg: f64,
        scale: CoordinateScale,
    ) -> Result<Self, ClipperError> {
        let infill_extrusion_width = scale.checked_scale(infill_line_width).unwrap_or(1);
        let layer_thickness = scale.checked_scale(layer_height).unwrap_or(1) as f64;
        // `m_infill_extrusion_width * 100 * n_multiline / sparse_infill_density`.
        let supporting_radius =
            infill_extrusion_width * 100 * (multiline as Coord) / (density_percent as Coord).max(1);
        let wall_supporting_radius =
            (layer_thickness * overhang_angle_deg.to_radians().tan()) as Coord;
        let prune_length = (layer_thickness * prune_angle_deg.to_radians().tan()) as Coord;
        let straightening_max_distance =
            (layer_thickness * straightening_angle_deg.to_radians().tan()) as Coord;

        let n = layer_internal_outlines.len();
        let mut generator = Self {
            supporting_radius,
            wall_supporting_radius,
            prune_length,
            straightening_max_distance,
            overhang_per_layer: vec![Vec::new(); n],
            lightning_layers: (0..n).map(|_| Layer::new()).collect(),
        };
        generator.generate_initial_internal_overhangs(&layer_internal_outlines)?;
        generator.generate_trees(&layer_internal_outlines)?;
        Ok(generator)
    }

    /// `generateInitialInternalOverhangs` (Generator.cpp:151): top-to-bottom,
    /// `diff(offset(infill_here, -wall_radius), infill_above)`.
    fn generate_initial_internal_overhangs(
        &mut self,
        layer_outlines: &[Vec<Polygon>],
    ) -> Result<(), ClipperError> {
        let mut infill_area_above: Vec<Polygon> = Vec::new();
        for layer_nr in (0..layer_outlines.len()).rev() {
            let infill_area_here = &layer_outlines[layer_nr];
            let shrunk = to_polygons(&offset_expolygons(
                &infill_area_here
                    .iter()
                    .map(|p| ExPolygon::new(p.clone(), Vec::new()))
                    .collect::<Vec<_>>(),
                -self.wall_supporting_radius as f32,
                JoinType::Square,
                3.0,
            )?);
            let overhang = to_polygons(&difference_polygons_ex(&shrunk, &infill_area_above)?);
            self.overhang_per_layer[layer_nr] = overhang;
            infill_area_above = infill_area_here.clone();
        }
        Ok(())
    }

    /// `generateTrees` (Generator.cpp:179) — top-to-bottom grow/reconnect/propagate.
    fn generate_trees(&mut self, layer_outlines: &[Vec<Polygon>]) -> Result<(), ClipperError> {
        let n = layer_outlines.len();
        if n == 0 {
            return Ok(());
        }
        let eps = 100; // SCALED_EPSILON at the 1e6 scale
        let top = n - 1;
        let top_bbox = BoundingBox::from_polygons(&layer_outlines[top])
            .unwrap_or(BoundingBox::new(Point::new(0, 0), Point::new(0, 0)))
            .inflated(eps);
        let mut outlines_locator = EdgeGrid::new_from_contours(
            layer_outlines[top].iter().map(|p| p.points()),
            top_bbox.min(),
            top_bbox.max(),
            LOCATOR_CELL_SIZE,
        )?;

        for layer_id in (0..=top).rev() {
            let current_outlines = &layer_outlines[layer_id];
            let current_bbox = BoundingBox::from_polygons(current_outlines)
                .unwrap_or(BoundingBox::new(Point::new(0, 0), Point::new(0, 0)));
            let to_be_reconnected = std::mem::take(&mut self.lightning_layers[layer_id].tree_roots);
            let overhang = self.overhang_per_layer[layer_id].clone();
            self.lightning_layers[layer_id].generate_new_trees(
                &overhang,
                current_outlines,
                current_bbox,
                &outlines_locator,
                self.supporting_radius,
                self.wall_supporting_radius,
            )?;
            self.lightning_layers[layer_id].reconnect_roots(
                to_be_reconnected,
                current_outlines,
                current_bbox,
                &outlines_locator,
                self.supporting_radius,
                self.wall_supporting_radius,
            );

            if layer_id == 0 {
                return Ok(());
            }
            let below_outlines = &layer_outlines[layer_id - 1];
            let mut below_bbox = BoundingBox::from_polygons(below_outlines)
                .unwrap_or(BoundingBox::new(Point::new(0, 0), Point::new(0, 0)))
                .inflated(eps);
            below_bbox = merge_locator_and_tree_bbox(
                &outlines_locator,
                &self.lightning_layers[layer_id],
                below_bbox,
                eps,
            );
            outlines_locator = EdgeGrid::new_from_contours(
                below_outlines.iter().map(|p| p.points()),
                below_bbox.min(),
                below_bbox.max(),
                LOCATOR_CELL_SIZE,
            )?;

            let trees = self.lightning_layers[layer_id].tree_roots.clone();
            let mut lower_trees = Vec::new();
            for tree in &trees {
                Node::propagate_to_next_layer(
                    tree,
                    &mut lower_trees,
                    below_outlines,
                    &outlines_locator,
                    self.prune_length,
                    self.straightening_max_distance,
                    LOCATOR_CELL_SIZE / 2,
                );
            }
            self.lightning_layers[layer_id - 1].tree_roots = lower_trees;
        }
        Ok(())
    }

    /// `getTreesForLayer` + `convertToLines`.
    pub(crate) fn convert_to_lines(
        &self,
        layer_id: usize,
        limit_to_outline: &[Polygon],
        line_overlap: Coord,
    ) -> Result<Vec<Polyline>, ClipperError> {
        self.lightning_layers[layer_id].convert_to_lines(limit_to_outline, line_overlap)
    }
}

/// `below_outlines_bbox.merge(locator.bbox) ; .merge(tree_roots extents)`
/// (Generator.cpp:230-232).
fn merge_locator_and_tree_bbox(
    locator: &EdgeGrid,
    layer: &Layer,
    mut bbox: BoundingBox,
    eps: Coord,
) -> BoundingBox {
    let (lmin, lmax) = locator.bounds();
    bbox = BoundingBox::new(
        Point::new(bbox.min().x().min(lmin.x()), bbox.min().y().min(lmin.y())),
        Point::new(bbox.max().x().max(lmax.x()), bbox.max().y().max(lmax.y())),
    );
    if !layer.tree_roots.is_empty() {
        let mut it = layer.tree_roots.iter();
        if let Some(first) = it.next() {
            let fp = first.borrow().p;
            let mut tb = BoundingBox::new(fp, fp);
            for t in it {
                let p = t.borrow().p;
                tb.merge_point(p);
            }
            let tb = tb.inflated(eps);
            bbox = BoundingBox::new(
                Point::new(
                    bbox.min().x().min(tb.min().x()),
                    bbox.min().y().min(tb.min().y()),
                ),
                Point::new(
                    bbox.max().x().max(tb.max().x()),
                    bbox.max().y().max(tb.max().y()),
                ),
            );
        }
    }
    bbox
}
