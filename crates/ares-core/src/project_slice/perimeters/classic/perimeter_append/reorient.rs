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
            has_fuzzy_skin: record.fuzzy_skin.should_fuzzify(layer_id, 0, true),
            has_fuzzy_hole: record.fuzzy_skin.should_fuzzify(layer_id, 0, false),
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
            flags,
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
fn reorient_perimeters(
    collection: &mut ExtrusionEntityCollection,
    flags: SteepOverhang,
    internal_only: bool,
) {
    if !(flags.contour || flags.hole) {
        return;
    }
    for entity in &mut collection.entities {
        let ExtrusionEntity::Loop(loop_) = entity else {
            continue;
        };
        let extrusion_loop = &mut loop_.extrusion_loop;
        let need_reverse = match extrusion_loop.role {
            ExtrusionLoopRole::Hole => flags.hole,
            _ => flags.contour,
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
mod tests {
    use super::ReorientPlan;
    use crate::geometry::{CoordinateScale, Point, Polygon};
    use crate::{
        FloatOrPercent, OrcaBool, Percent, ProjectSettings, RegionOptions,
        project_slice::perimeters::types::Flow,
    };

    use super::super::super::{
        chained_loops::{ExtrusionLoop, ExtrusionLoopRole},
        entity_collections::{ExtrusionEntity, ExtrusionEntityCollection, OrderedExtrusionLoop},
        materialize::{ExtrusionPath, ExtrusionRole, Point3, Polyline3},
        traversal::{
            ClassicTraversalRecord, LowerFlowRoute, PendingExtrusionRole, PendingLoopRole,
            PendingPathBranch, TraversalSeed,
        },
    };

    fn region(threshold: f64) -> RegionOptions {
        let mut region = RegionOptions::from_base(&ProjectSettings::default().process.region);
        region.overhang_reverse = OrcaBool(true);
        region.overhang_reverse_internal_only = OrcaBool(true);
        region.overhang_reverse_threshold = FloatOrPercent::Percent(Percent(threshold));
        region
    }

    fn record(layer_id: usize, raft_layers: i32, detect: bool) -> ClassicTraversalRecord {
        ClassicTraversalRecord {
            surfaces: Vec::new(),
            layer_id,
            layer_height: 0.2,
            slice_z: layer_id as f64 * 0.2 + 0.1,
            fuzzy_skin: crate::perimeters::FuzzySkinConfig::disabled(),
            simplification_tolerance: 0.012,
            overhang_flow: Flow {
                width: 0.4,
                height: 0.2,
                spacing: 0.36,
                nozzle_diameter: 0.4,
                bridge: false,
                mm3_per_mm: 0.08,
            },
            branch: if detect {
                PendingPathBranch::OverhangClipping {
                    detect_overhang_wall: true,
                    layer_id,
                    raft_layers,
                }
            } else {
                PendingPathBranch::OrdinaryUnsplit {
                    detect_overhang_wall: false,
                    layer_id,
                    raft_layers,
                }
            },
        }
    }

    fn seed(is_contour: bool) -> TraversalSeed {
        TraversalSeed {
            polygon: Polygon::new(vec![
                Point::new(0, 0),
                Point::new(1_000_000, 0),
                Point::new(1_000_000, 1_000_000),
                Point::new(0, 1_000_000),
            ]),
            depth: 0,
            is_contour,
            is_smaller_width_perimeter: false,
            extrusion_role: if is_contour {
                PendingExtrusionRole::ExternalPerimeter
            } else {
                PendingExtrusionRole::Perimeter
            },
            loop_role: if is_contour {
                PendingLoopRole::Default
            } else {
                PendingLoopRole::Hole
            },
            route: LowerFlowRoute::External,
            width: 0.42,
            mm3_per_mm: 0.08,
            children: Vec::new(),
        }
    }

    fn loop_entity(role: ExtrusionLoopRole, path_role: ExtrusionRole, x: i64) -> ExtrusionEntity {
        ExtrusionEntity::Loop(OrderedExtrusionLoop {
            extrusion_loop: ExtrusionLoop {
                paths: vec![ExtrusionPath {
                    polyline: Polyline3 {
                        points: vec![
                            Point3 { x, y: 0, z: 0 },
                            Point3 {
                                x: x + 4,
                                y: 0,
                                z: 0,
                            },
                            Point3 {
                                x: x + 9,
                                y: 0,
                                z: 0,
                            },
                        ],
                        fitting: Vec::new(),
                    },
                    role: path_role,
                    can_reverse: true,
                    mm3_per_mm: 0.08,
                    width: 0.4,
                    height: 0.2,
                }],
                role,
            },
            inset_idx: 0,
        })
    }

    fn first_points(collection: &ExtrusionEntityCollection, index: usize) -> Vec<i64> {
        match &collection.entities[index] {
            ExtrusionEntity::Loop(ordered) => ordered
                .extrusion_loop
                .paths
                .iter()
                .flat_map(|path| path.polyline.points.iter().map(|point| point.x))
                .collect(),
            ExtrusionEntity::MultiPath(_) => panic!("classic append keeps loop entities"),
        }
    }

    #[test]
    fn zero_threshold_odd_layer_reverses_inner_and_exempts_external() {
        let seeds = vec![seed(true)];
        let mut collection = ExtrusionEntityCollection {
            entities: vec![
                loop_entity(
                    ExtrusionLoopRole::Default,
                    ExtrusionRole::ExternalPerimeter,
                    30,
                ),
                loop_entity(ExtrusionLoopRole::Default, ExtrusionRole::Perimeter, 10),
            ],
            source_order: 0,
        };
        ReorientPlan::new(&record(1, 0, true), &region(0.0)).apply(
            &mut collection,
            &seeds,
            None,
            CoordinateScale::Normal,
        );
        // `reorient_perimeters` exempts the external loop
        // (`PerimeterGenerator.cpp:1129-1135`) and reverses the internal one.
        assert_eq!(first_points(&collection, 0), vec![30, 34, 39]);
        assert_eq!(first_points(&collection, 1), vec![19, 14, 10]);
    }

    #[test]
    fn even_layer_keeps_orientation() {
        let seeds = vec![seed(true)];
        let mut collection = ExtrusionEntityCollection {
            entities: vec![loop_entity(
                ExtrusionLoopRole::Default,
                ExtrusionRole::Perimeter,
                10,
            )],
            source_order: 0,
        };
        ReorientPlan::new(&record(0, 0, true), &region(0.0)).apply(
            &mut collection,
            &seeds,
            None,
            CoordinateScale::Normal,
        );
        assert_eq!(first_points(&collection, 0), vec![10, 14, 19]);
    }

    #[test]
    fn ordinary_unsplit_above_raft_reverses_internal_only() {
        let seeds = vec![seed(true)];
        let mut collection = ExtrusionEntityCollection {
            entities: vec![
                loop_entity(
                    ExtrusionLoopRole::Default,
                    ExtrusionRole::ExternalPerimeter,
                    30,
                ),
                loop_entity(ExtrusionLoopRole::Default, ExtrusionRole::Perimeter, 10),
            ],
            source_order: 0,
        };
        ReorientPlan::new(&record(3, 0, false), &region(50.0)).apply(
            &mut collection,
            &seeds,
            None,
            CoordinateScale::Normal,
        );
        assert_eq!(first_points(&collection, 0), vec![30, 34, 39]);
        assert_eq!(first_points(&collection, 1), vec![19, 14, 10]);
    }

    #[test]
    fn hole_loop_follows_hole_flag() {
        let seeds = vec![seed(false)];
        let mut collection = ExtrusionEntityCollection {
            entities: vec![
                loop_entity(ExtrusionLoopRole::Hole, ExtrusionRole::Perimeter, 10),
                loop_entity(ExtrusionLoopRole::Default, ExtrusionRole::Perimeter, 20),
            ],
            source_order: 0,
        };
        ReorientPlan::new(&record(1, 0, true), &region(0.0)).apply(
            &mut collection,
            &seeds,
            None,
            CoordinateScale::Normal,
        );
        // Only the steep-hole flag is set, so the default-role loop stays.
        assert_eq!(first_points(&collection, 0), vec![19, 14, 10]);
        assert_eq!(first_points(&collection, 1), vec![20, 24, 29]);
    }
}
