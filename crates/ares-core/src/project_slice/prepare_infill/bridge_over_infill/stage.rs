use crate::{
    ProcessInfillPattern, SliceError,
    geometry::ClipperError,
    project_slice::{
        perimeters::flow::{FillFlowContext, FillFlowRole, resolve_fill_flow},
        prepare_infill::external_surfaces::{self, PreparedPostExternalSurfaces},
    },
};

use super::{
    candidates::{CandidateLayer, CandidateRegion, gather_candidates},
    types::BridgeCandidateObject,
};

pub(in crate::project_slice) struct PreparedPostBridgeCandidates {
    pub(in crate::project_slice) predecessor: PreparedPostExternalSurfaces,
    pub(in crate::project_slice) objects: Vec<BridgeCandidateObject>,
}

pub(in crate::project_slice) fn prepare(
    predecessor: PreparedPostExternalSurfaces,
) -> Result<PreparedPostBridgeCandidates, SliceError> {
    match gather_objects(&predecessor) {
        Ok(objects) => Ok(PreparedPostBridgeCandidates {
            predecessor,
            objects,
        }),
        Err(error) => {
            external_surfaces::dispose(predecessor);
            Err(error)
        }
    }
}

fn gather_objects(
    predecessor: &PreparedPostExternalSurfaces,
) -> Result<Vec<BridgeCandidateObject>, SliceError> {
    let horizontal = &predecessor.predecessor;
    let traversal = &horizontal.predecessor;
    let scale = traversal.scale;
    horizontal
        .objects
        .iter()
        .zip(&traversal.objects)
        .map(|(object, traversal_object)| {
            let prelude = &traversal_object
                .predecessor
                .predecessor
                .predecessor
                .predecessor;
            let input_object = &prelude.object;
            let source_index = input_object.identity().0;
            let resolved = traversal
                .resolved
                .objects
                .iter()
                .find(|resolved| resolved.source_object_index == source_index)
                .expect("O43 object retains its resolved source");
            let filter = resolved.object.dont_filter_internal_bridges;
            let (compensated, inputs) = input_object.as_parts();
            let (post_regions, _) = compensated.as_parts();
            let (plan, _, regions) = post_regions.as_parts();
            let has_lightning_infill = regions.iter().any(|region| {
                region.as_parts().1.sparse_infill_pattern == ProcessInfillPattern::Lightning
            });
            let layer_count = plan.layers.len();
            assert_eq!(object.records.len(), layer_count * regions.len());
            assert_eq!(inputs.len(), object.records.len());
            let layers = plan
                .layers
                .iter()
                .enumerate()
                .map(|(layer_index, layer)| {
                    let solid_infill_spacing = if layer_index == 0 {
                        // The source skips this layer before calling its Flow getter.
                        0
                    } else {
                        let solid_flow = match &inputs[layer_index] {
                            Some(input) => input.solid_infill_flow,
                            None => resolve_fill_flow(
                                FillFlowContext::new(
                                    layer,
                                    layer.height,
                                    traversal
                                        .resolved
                                        .views
                                        .full
                                        .process
                                        .print
                                        .initial_layer_line_width,
                                    regions[0].as_parts().1,
                                    &resolved.object,
                                    &traversal.resolved.views.full.project.print.nozzle_diameter,
                                ),
                                FillFlowRole::Solid,
                            )?,
                        };
                        solid_flow.scaled_spacing(scale)
                    };
                    let regions = regions
                        .iter()
                        .enumerate()
                        .map(|(region_index, region)| {
                            let slot = region_index * layer_count + layer_index;
                            let (fill_expolygons, fill_surfaces) =
                                match (&object.records[slot], &inputs[slot]) {
                                    (Some(record), Some(_)) => (
                                        record.fill_expolygons.as_slice(),
                                        record.fill_surfaces.as_slice(),
                                    ),
                                    (None, None) => (&[][..], &[][..]),
                                    _ => unreachable!("validated O42 record slots remain aligned"),
                                };
                            CandidateRegion {
                                region_index,
                                fill_expolygons,
                                fill_surfaces,
                                sparse_infill_density_percent: region
                                    .as_parts()
                                    .1
                                    .sparse_infill_density
                                    .0,
                            }
                        })
                        .collect();
                    Ok(Some(CandidateLayer {
                        lower_layer_index: layer_index.checked_sub(1),
                        solid_infill_spacing,
                        regions,
                    }))
                })
                .collect::<Result<Vec<_>, SliceError>>()?;
            gather_candidates(&layers, has_lightning_infill, filter, scale).map_err(geometry_error)
        })
        .collect()
}

fn geometry_error(error: ClipperError) -> SliceError {
    match error {
        ClipperError::CoordinateOutOfRange => SliceError::InvalidInput(
            "internal-bridge candidate coordinate is outside the supported Clipper range"
                .to_owned(),
        ),
        ClipperError::OpenPathMustBeSubject | ClipperError::OpenPathsRequirePolyTree => {
            unreachable!("internal-bridge candidate geometry uses closed paths only")
        }
    }
}
