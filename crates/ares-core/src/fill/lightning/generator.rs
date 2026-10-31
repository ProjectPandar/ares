//! `FillLightning::Generator` (`src/libslic3r/Fill/Lightning/Generator.cpp`):
//! builds the lightning tree across ALL layers top-to-bottom, then
//! `getTreesForLayer(layer_id)` returns the per-layer `Layer` whose
//! `convertToLines` emits the infill. `generateInitialInternalOverhangs`
//! computes the per-layer overhang (internal surfaces minus walls via
//! `m_wall_supporting_radius`); `generateTrees` propagates trees downward.

use crate::geometry::{
    BoundingBox, ClipperError, Coord, CoordinateScale, EdgeGrid, ExPolygon, JoinType, Point,
    Polygon, Polyline, difference_polygons_ex, offset_paths,
};

use super::layer::Layer;
use super::layer_geom::GroundingContext;
use super::tree_node::{Node, PropagationSettings};

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
    locator_cell_size: Coord,
    /// The shared `rand()` stream (TreeNode.cpp:365 `rand() % m_children.size()`)
    /// — upstream consumes the process-global C `rand()` continuously across ALL
    /// `convertToPolylines` calls and ALL layers, not a per-layer reseed.
    rng: std::cell::RefCell<super::glibc_rand::GlibcRand>,
}

/// Infill constructor inputs from `Generator.cpp:66-99`.
pub(crate) struct InfillSettings {
    pub(crate) layer_height: f64,
    pub(crate) infill_line_width: f64,
    pub(crate) line_width: f64,
    pub(crate) nozzle_diameter: f64,
    pub(crate) multiline: i32,
    pub(crate) density_percent: f64,
    pub(crate) overhang_angle_deg: f64,
    pub(crate) prune_angle_deg: f64,
    pub(crate) straightening_angle_deg: f64,
    pub(crate) scale: CoordinateScale,
}

pub(crate) struct SupportSettings {
    pub(crate) layer_height: f64,
    pub(crate) infill_line_width: f64,
    pub(crate) line_width: f64,
    pub(crate) max_nozzle_diameter: f64,
    pub(crate) scale: CoordinateScale,
}

impl Generator {
    /// Support constructor and `generateTreesforSupport` (`Generator.cpp:104-145,236-285`).
    pub(crate) fn for_support(
        contours: &[Vec<Polygon>],
        overhangs: Vec<Vec<Polygon>>,
        density: f32,
        settings: &SupportSettings,
    ) -> Result<Self, ClipperError> {
        let scale = settings.scale;
        let mut width = scale.scaled_delta(settings.infill_line_width) as f32;
        if f64::from(width) < 0.0001 {
            let fallback = if settings.line_width < 0.0001 {
                f64::from(1.125_f32 * settings.max_nozzle_diameter as f32)
            } else {
                settings.line_width
            };
            width = scale.scaled_delta(fallback) as f32;
        }
        let density = 0.15_f32.max(density);
        let supporting_radius = (width as Coord as f32 / density) as Coord;
        let growth_radius = (scale.scaled_delta(settings.layer_height)
            * std::f64::consts::FRAC_PI_4.tan()) as Coord;
        let locator_cell_size = super::layer::locator_cell_size(scale);
        let mut generator = Self {
            supporting_radius,
            wall_supporting_radius: growth_radius,
            prune_length: growth_radius,
            straightening_max_distance: growth_radius,
            overhang_per_layer: overhangs,
            lightning_layers: (0..contours.len())
                .map(|_| Layer::new(locator_cell_size))
                .collect(),
            locator_cell_size,
            rng: std::cell::RefCell::new(super::glibc_rand::GlibcRand::new()),
        };
        generator.generate_trees(contours)?;
        Ok(generator)
    }

    /// `Generator::Generator` params (Generator.cpp:66-99): supporting radius
    /// `width*100*multiline/density`, and the overhang/prune/straighten radii
    /// from `layer_height * tan(angle)`.
    pub(crate) fn new(
        layer_internal_outlines: Vec<Vec<Polygon>>,
        InfillSettings {
            layer_height,
            infill_line_width,
            line_width,
            nozzle_diameter,
            multiline,
            density_percent,
            overhang_angle_deg,
            prune_angle_deg,
            straightening_angle_deg,
            scale,
        }: InfillSettings,
    ) -> Result<Self, ClipperError> {
        // `m_infill_extrusion_width = scaled<float>(sparse.get_abs_value(nozzle))`
        // (Generator.cpp:79) — scale-domain FLOAT. `float(mm*1e6)` rounds
        // 0.42*1e6=419999.9999 up to 420000.0; a coord_t truncation would give
        // 419999. EPSILON fallback (Generator.cpp:83-88) to `line_width` or
        // `Flow::auto_extrusion_width` = 1.125*nozzle when the sparse width ~0.
        let scaled_float = |mm: f64| (mm / scale.factor()) as f32;
        // `EPSILON` is the libslic3r.h:52 DOUBLE epsilon `1e-4` — the
        // comparison promotes the f32 width to f64 (a scaled `1e-4_f32` is
        // ~0.00009999999747 < 0.0001, so upstream takes the fallback).
        const EPSILON: f64 = 1e-4;
        let mut infill_extrusion_width = scaled_float(infill_line_width);
        if f64::from(infill_extrusion_width) < EPSILON {
            // `Flow::auto_extrusion_width` multiplies in FLOAT (Flow.cpp:21-34):
            // `1.125f * nozzle_f` — keep the f32 product, not a double one.
            let fallback_mm = if line_width < EPSILON {
                f64::from(1.125_f32 * nozzle_diameter as f32)
            } else {
                line_width
            };
            infill_extrusion_width = scaled_float(fallback_mm);
        }
        let layer_thickness = scale.scaled_delta(layer_height);
        let locator_cell_size = super::layer::locator_cell_size(scale);
        // `m_supporting_radius = coord_t(m_infill_extrusion_width) * 100 * n_multiline /
        // sparse_infill_density` (Generator.cpp:90): `coord_t(float)` truncates the
        // scale-domain width FIRST (420000.5 → 420000), then the integer product
        // promotes to double for the division and truncates back — NOT f64-f64.
        let supporting_radius =
            (infill_extrusion_width as Coord as f64 * 100.0 * f64::from(multiline)
                / density_percent.max(f64::EPSILON)) as Coord;
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
            lightning_layers: (0..n).map(|_| Layer::new(locator_cell_size)).collect(),
            locator_cell_size,
            rng: std::cell::RefCell::new(super::glibc_rand::GlibcRand::new()),
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
            let shrunk = offset_paths(
                infill_area_here,
                -self.wall_supporting_radius as f32,
                JoinType::Miter,
                3.0,
            )?;
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
            self.locator_cell_size,
        )?;

        for layer_id in (0..=top).rev() {
            let current_outlines = &layer_outlines[layer_id];
            let current_bbox = BoundingBox::from_polygons(current_outlines)
                .unwrap_or(BoundingBox::new(Point::new(0, 0), Point::new(0, 0)));
            // `to_be_reconnected_tree_roots = tree_roots` — a shared_ptr vector
            // COPY (`Generator.cpp:210`), NOT a move: the roots stay in
            // `tree_roots` so `std::find` inside `reconnectRoots` locates them.
            let to_be_reconnected = self.lightning_layers[layer_id].tree_roots.clone();
            let overhang = self.overhang_per_layer[layer_id].clone();
            let context = GroundingContext {
                outlines: current_outlines,
                bbox: current_bbox,
                locator: &outlines_locator,
                supporting_radius: self.supporting_radius,
                wall_supporting_radius: self.wall_supporting_radius,
            };
            self.lightning_layers[layer_id].generate_new_trees(&overhang, &context)?;
            self.lightning_layers[layer_id].reconnect_roots(to_be_reconnected, &context);

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
                self.locator_cell_size,
            )?;

            let trees = self.lightning_layers[layer_id].tree_roots.clone();
            let mut lower_trees = Vec::new();
            let settings = PropagationSettings {
                prune_distance: self.prune_length,
                smooth_magnitude: self.straightening_max_distance,
                max_remove_colinear_dist: self.locator_cell_size / 2,
            };
            for tree in &trees {
                Node::propagate_to_next_layer(
                    tree,
                    &mut lower_trees,
                    below_outlines,
                    &outlines_locator,
                    &settings,
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
        let lines = self.lightning_layers[layer_id].convert_to_lines(
            limit_to_outline,
            line_overlap,
            &mut self.rng.borrow_mut(),
        )?;
        Ok(lines)
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
