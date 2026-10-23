// Steep-overhang wall reversal ported from OrcaSlicer v2.4.2
// `PerimeterGenerator.cpp`: `detect_steep_overhang` (:58-99), the
// traverse-time flag updates (:165-176, :219-223) and
// `reorient_perimeters` (:1117-1145).

use crate::geometry::{
    BoundingBox, CoordinateScale, ExPolygon, JoinType, clip_clipper_expolygons_with_subject_bbox,
    diff_pl, offset_paths,
};
use crate::{FloatOrPercent, RegionOptions};

use super::super::{
    chained_loops::ExtrusionLoopRole,
    entity_collections::{ExtrusionEntity, ExtrusionEntityCollection},
    materialize::ExtrusionRole,
    traversal::{ClassicTraversalRecord, PendingPathBranch, TraversalSeed},
};
use super::types::SteepOverhang;

/// Upstream `EPSILON` (`libslic3r.h`): resolved thresholds below this
/// millimetre value take the reverse-on-every-odd-layer special case.
const EPSILON: f64 = 1e-4;
/// `ClipperUtils::offset` default miter limit with `jtMiter`.
const MITER_LIMIT: f64 = 3.0;

pub(super) struct ReorientPlan<'a> {
    region: &'a RegionOptions,
    /// `overhangs_reverse` (`PerimeterGenerator.cpp:111-113`).
    active: bool,
    /// `detect_overhang_wall && layer_id > raft_layers` selects the
    /// per-loop detection branch (`:157-176`); otherwise the flags turn on
    /// wholesale above the raft (`:219-223`).
    detect_branch: bool,
    above_raft: bool,
    has_fuzzy_skin: bool,
    has_fuzzy_hole: bool,
}

impl<'a> ReorientPlan<'a> {
    pub(super) fn new(record: &ClassicTraversalRecord, region: &'a RegionOptions) -> Self {
        let (detect_overhang_wall, layer_id, raft_layers) = match record.branch {
            PendingPathBranch::OverhangClipping {
                detect_overhang_wall,
                layer_id,
                raft_layers,
            }
            | PendingPathBranch::OrdinaryUnsplit {
                detect_overhang_wall,
                layer_id,
                raft_layers,
            } => (detect_overhang_wall, layer_id, raft_layers),
        };
        let above_raft = i32::try_from(layer_id).is_ok_and(|layer| layer > raft_layers);
        Self {
            region,
            active: region.overhang_reverse.0 && layer_id % 2 == 1,
            detect_branch: detect_overhang_wall && above_raft,
            above_raft,
            // `FuzzySkin.cpp:458-464`
            has_fuzzy_skin: record.fuzzy_skin.has_contour,
            has_fuzzy_hole: record.fuzzy_skin.has_hole,
        }
    }

    /// `PerimeterGenerator.cpp:1446-1453`: when `overhang_reverse` is
    /// configured, compute the surface's steep flags and reverse the
    /// affected loops.
    pub(super) fn apply(
        self,
        collection: &mut ExtrusionEntityCollection,
        seeds: &[TraversalSeed],
        lower: Option<&[ExPolygon]>,
        scale: CoordinateScale,
    ) {
        if !self.region.overhang_reverse.0 {
            return;
        }
        let flags = self.steep_flags(seeds, lower, scale);
        reorient_perimeters(
            collection,
            flags.contour,
            flags.hole,
            self.region.overhang_reverse_internal_only.0,
        );
    }

    fn steep_flags(
        &self,
        seeds: &[TraversalSeed],
        lower: Option<&[ExPolygon]>,
        scale: CoordinateScale,
    ) -> SteepOverhang {
        let mut flags = SteepOverhang::default();
        if !self.active {
            return flags;
        }
        if self.detect_branch {
            for seed in seeds {
                self.detect_seed(seed, lower, scale, &mut flags);
            }
        } else if self.above_raft {
            flags.contour = true;
            flags.hole = true;
        }
        flags
    }

    /// Per-loop flag updates inside `traverse_loops`
    /// (`PerimeterGenerator.cpp:165-176`), applied to the seed tree.
    fn detect_seed(
        &self,
        seed: &TraversalSeed,
        lower: Option<&[ExPolygon]>,
        scale: CoordinateScale,
        flags: &mut SteepOverhang,
    ) {
        if self.has_fuzzy_skin {
            if seed.is_contour {
                flags.contour = true;
            } else if self.has_fuzzy_hole {
                flags.hole = true;
            }
        }
        let found = if seed.is_contour {
            flags.contour
        } else {
            flags.hole
        };
        if !found {
            self.detect_steep_overhang(seed, lower, scale, flags);
        }
        for child in &seed.children {
            self.detect_seed(child, lower, scale, flags);
        }
    }

    /// `detect_steep_overhang` (`PerimeterGenerator.cpp:58-99`).
    fn detect_steep_overhang(
        &self,
        seed: &TraversalSeed,
        lower: Option<&[ExPolygon]>,
        scale: CoordinateScale,
        flags: &mut SteepOverhang,
    ) {
        let width = f64::from(seed.width);
        let threshold = match &self.region.overhang_reverse_threshold {
            FloatOrPercent::Float(value) => *value,
            FloatOrPercent::Percent(percent) => percent.0 / 100.0 * width,
        };
        if threshold < EPSILON {
            // Special case: reverse on every odd layer.
            if seed.is_contour {
                flags.contour = true;
            } else {
                flags.hole = true;
            }
            return;
        }
        let Some(lower) = lower else {
            return;
        };
        let Some(bbox) = BoundingBox::from_polygon(&seed.polygon) else {
            return;
        };
        let lower_chopped = clip_clipper_expolygons_with_subject_bbox(lower, bbox);
        let delta = scale.scaled_delta(threshold - 0.5 * width) as f32;
        let Ok(limiton) = offset_paths(&lower_chopped, delta, JoinType::Miter, MITER_LIMIT) else {
            return;
        };
        let Ok(remain) = diff_pl(std::slice::from_ref(&seed.polygon), &limiton) else {
            return;
        };
        if !remain.is_empty() {
            if seed.is_contour {
                flags.contour = true;
            } else {
                flags.hole = true;
            }
        }
    }
}

/// `reorient_perimeters` (`PerimeterGenerator.cpp:1117-1145`).
pub(in crate::project_slice) fn reorient_perimeters(
    collection: &mut ExtrusionEntityCollection,
    contour: bool,
    hole: bool,
    internal_only: bool,
) {
    if !(contour || hole) {
        return;
    }
    for entity in &mut collection.entities {
        let ExtrusionEntity::Loop(loop_) = entity else {
            continue;
        };
        let extrusion_loop = &mut loop_.extrusion_loop;
        let need_reverse = match extrusion_loop.role {
            ExtrusionLoopRole::Hole => hole,
            _ => contour,
        };
        let is_external = internal_only
            && extrusion_loop
                .paths
                .iter()
                .any(|path| path.role == ExtrusionRole::ExternalPerimeter);
        if need_reverse && !is_external {
            extrusion_loop.reverse();
        }
    }
}

#[cfg(test)]
mod tests;
