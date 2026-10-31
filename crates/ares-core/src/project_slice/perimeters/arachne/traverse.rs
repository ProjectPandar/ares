// `traverse_extrusions` port from OrcaSlicer v2.4.2
// `PerimeterGenerator.cpp:370-574` (called from `process_arachne` at
// `PerimeterGenerator.cpp:2471`): converts ordered arachne wall lines into
// the typed `ExtrusionEntityCollection` vocabulary — closed lines become
// `ExtrusionLoop` entities (`:529-545`) and open lines become
// `ExtrusionMultiPath` entities that split on discontinuities (`:546-566`).
// Ordinary steep-overhang detection and start-point rechaining are consumed
// here. Clipped-width interpolation is owned by geometry::clip_open_path_widths;
// junction fuzzification is owned by the source-cited fuzzy module.

use crate::{
    FloatOrPercent, ProcessWallDirection, ProcessWallSequence, SliceError,
    arachne::ExtrusionLine,
    geometry::{CoordinateScale, ExPolygon, Polygon},
    project_slice::perimeters::{
        classic::{
            chained_loops::{ExtrusionLoop, ExtrusionLoopRole},
            entity_collections::{
                ExtrusionEntity, ExtrusionEntityCollection, ExtrusionMultiPath,
                OrderedExtrusionLoop, orient_loop,
            },
            materialize::{ExtrusionPath, ExtrusionRole},
        },
        types::Flow,
    },
};

mod overhang;
use super::variable_width;
use overhang::append_overhang_clipped_paths;

/// `PerimeterGeneratorArachneExtrusion` (`PerimeterGenerator.cpp:363-367`):
/// one ordered wall line plus the contour flag computed by the candidate
/// traversal (`is_contour()` at `:2363`).
#[derive(Clone, Debug, PartialEq)]
pub(in crate::project_slice) struct PerimeterGeneratorArachneExtrusion {
    pub(in crate::project_slice) extrusion: ExtrusionLine,
    pub(in crate::project_slice) is_contour: bool,
}

/// The `PerimeterGenerator` fields `traverse_extrusions` reads
/// (`PerimeterGenerator.cpp:370-574`): `lower_slices_polygons` is the
/// nozzle-half-grown lower slice set prepared by `process_arachne`
/// (`PerimeterGenerator.cpp:2107-2111`).
#[derive(Clone, Copy)]
pub(in crate::project_slice) struct TraverseExtrusionsContext<'a> {
    pub(in crate::project_slice) layer_id: usize,
    pub(in crate::project_slice) raft_layers: i32,
    pub(in crate::project_slice) detect_overhang_wall: bool,
    pub(in crate::project_slice) overhang_reverse: bool,
    pub(in crate::project_slice) overhang_reverse_threshold: FloatOrPercent,
    pub(in crate::project_slice) wall_direction: ProcessWallDirection,
    pub(in crate::project_slice) wall_sequence: ProcessWallSequence,
    pub(in crate::project_slice) outer_walls_first_for_brim: bool,
    pub(in crate::project_slice) fuzzy_skin: &'a super::fuzzy::Regions,
    pub(in crate::project_slice) slice_z: f64,
    pub(in crate::project_slice) perimeter_flow: Flow,
    pub(in crate::project_slice) ext_perimeter_flow: Flow,
    pub(in crate::project_slice) overhang_flow: Flow,
    pub(in crate::project_slice) lower_slices_polygons: &'a [Polygon],
    pub(in crate::project_slice) lower_slices: &'a [ExPolygon],
    pub(in crate::project_slice) scale: CoordinateScale,
}

/// `traverse_extrusions` returns the collection plus the steep-overhang flags
/// its caller feeds to `reorient_perimeters` (`PerimeterGenerator.cpp:2472`).
#[derive(Debug)]
pub(in crate::project_slice) struct TraverseExtrusionsOutcome {
    pub(in crate::project_slice) collection: ExtrusionEntityCollection,
    // Consumed by `process_arachne::reorient_perimeters` (:2472–2474).
    pub(in crate::project_slice) steep_overhang_contour: bool,
    pub(in crate::project_slice) steep_overhang_hole: bool,
}

pub(in crate::project_slice) fn traverse_extrusions(
    pg_extrusions: Vec<PerimeterGeneratorArachneExtrusion>,
    context: &TraverseExtrusionsContext,
    slice_context: &mut crate::SlicerContext,
) -> Result<TraverseExtrusionsOutcome, SliceError> {
    // Detect steep overhangs. Only calculate overhang degree on even (from
    // GUI POV) layers (`PerimeterGenerator.cpp:375-377`).
    let overhangs_reverse = context.overhang_reverse && context.layer_id % 2 == 1;
    let above_raft = i32::try_from(context.layer_id).is_ok_and(|layer| layer > context.raft_layers);
    let thin_wall_context = pg_extrusions.len() == 2;

    let mut collection = ExtrusionEntityCollection::default();
    let mut steep_overhang_contour = false;
    let mut steep_overhang_hole = false;

    for pg_extrusion in pg_extrusions {
        let mut extrusion = pg_extrusion.extrusion;
        if extrusion.junctions.is_empty() {
            continue;
        }

        let is_external = extrusion.inset_index == 0;
        let role = if is_external {
            ExtrusionRole::ExternalPerimeter
        } else {
            ExtrusionRole::Perimeter
        };

        let is_contour = !extrusion.is_closed || pg_extrusion.is_contour;
        context.fuzzy_skin.apply(
            &mut extrusion,
            is_contour,
            super::fuzzy::Layer {
                id: context.layer_id,
                slice_z: context.slice_z,
                scale: context.scale,
            },
            slice_context,
        )?;

        let mut paths: Vec<ExtrusionPath> = Vec::new();
        if context.detect_overhang_wall && above_raft {
            // `traverse_extrusions:422-428`: fuzzy walls force reversal
            // before the ordinary steep-overhang detection.
            let fuzzy_reverse = overhangs_reverse && context.fuzzy_skin.has_contour;
            steep_overhang_contour |= fuzzy_reverse && pg_extrusion.is_contour;
            steep_overhang_hole |=
                fuzzy_reverse && !pg_extrusion.is_contour && context.fuzzy_skin.has_hole;
            let found = if pg_extrusion.is_contour {
                steep_overhang_contour
            } else {
                steep_overhang_hole
            };
            let detected = append_overhang_clipped_paths(
                &mut paths,
                &extrusion,
                role,
                context,
                overhangs_reverse && !found,
            )?;
            steep_overhang_contour |= detected && pg_extrusion.is_contour;
            steep_overhang_hole |= detected && !pg_extrusion.is_contour;
        } else {
            if overhangs_reverse && above_raft {
                // Always reverse if detect overhang wall is not enabled
                // (`PerimeterGenerator.cpp:520-524`).
                steep_overhang_contour = true;
                steep_overhang_hole = true;
            }
            variable_width::append_extrusion_paths(
                &mut paths,
                &extrusion,
                role,
                if is_external {
                    context.ext_perimeter_flow
                } else {
                    context.perimeter_flow
                },
                context.scale,
            )?;
        }

        // Append paths to collection (`PerimeterGenerator.cpp:528-566`).
        if paths.is_empty() {
            continue;
        }
        if extrusion.is_closed {
            let mut extrusion_loop = ExtrusionLoop {
                paths,
                role: if pg_extrusion.is_contour {
                    ExtrusionLoopRole::Default
                } else {
                    ExtrusionLoopRole::Hole
                },
            };
            // ExtrusionLoops occasionally have significantly disconnected
            // paths, triggering these asserts upstream too
            // (`PerimeterGenerator.cpp:537-543`).
            debug_assert!(
                extrusion_loop
                    .paths
                    .iter()
                    .skip(1)
                    .all(|path| path.polyline.points.len() >= 2)
            );
            debug_assert!(
                extrusion_loop
                    .paths
                    .windows(2)
                    .all(|pair| pair[0].polyline.points.last() == pair[1].polyline.points.first())
            );
            // Native release (NDEBUG) admits Ripple's open sampled loop.
            // `PerimeterGenerator.cpp:534-540` documents disconnected paths;
            // closure-width-audit.json confirms no closure repair is applied.
            orient_loop(
                &mut extrusion_loop,
                context.wall_direction,
                pg_extrusion.is_contour,
                thin_wall_context,
            );
            collection
                .entities
                .push(ExtrusionEntity::Loop(OrderedExtrusionLoop {
                    unsupported_distance: None,
                    extrusion_loop,
                    // v2.5.0-beta PerimeterGenerator.cpp:565-566.
                    inset_idx: extrusion.inset_index as i32,
                }));
            // Orca: Reverse the order of paths for thin wall holes. We define
            // thin wall hole as a hole with only one perimeter
            // (v2.5.0-beta PerimeterGenerator.cpp:581-584).
            let thin_wall_hole = !pg_extrusion.is_contour && thin_wall_context;
            if thin_wall_hole
                && context.wall_sequence != ProcessWallSequence::OuterInner
                && !context.outer_walls_first_for_brim
            {
                collection.entities.reverse();
            }
        } else {
            debug_assert!(
                paths
                    .iter()
                    .skip(1)
                    .all(|path| path.polyline.points.len() >= 2)
            );
            debug_assert!(
                paths
                    .windows(2)
                    .all(|pair| pair[0].polyline.points.last() == pair[1].polyline.points.first())
            );
            append_open_multi_path(&mut collection, paths);
        }
    }

    Ok(TraverseExtrusionsOutcome {
        collection,
        steep_overhang_contour,
        steep_overhang_hole,
    })
}

/// Open wall lines append `ExtrusionMultiPath` entities, splitting into a new
/// entity when consecutive sub-paths do not chain end-to-start
/// (`PerimeterGenerator.cpp:546-566`).
fn append_open_multi_path(collection: &mut ExtrusionEntityCollection, paths: Vec<ExtrusionPath>) {
    let mut paths = paths.into_iter();
    let mut current = vec![paths.next().expect("an open wall line has a path")];
    for path in paths {
        let chains = current
            .last()
            .expect("a multi-path keeps its last sub-path")
            .polyline
            .points
            .last()
            == path.polyline.points.first();
        if !chains {
            collection
                .entities
                .push(ExtrusionEntity::MultiPath(ExtrusionMultiPath {
                    paths: std::mem::take(&mut current),
                }));
        }
        current.push(path);
    }
    collection
        .entities
        .push(ExtrusionEntity::MultiPath(ExtrusionMultiPath {
            paths: current,
        }));
}

fn unsupported(key: &str) -> SliceError {
    SliceError::UnsupportedProjectFeature(key.to_owned())
}
