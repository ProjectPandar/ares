use crate::{
    ProcessInfillPattern, SliceError,
    geometry::ClipperError,
    project_slice::prepare_infill::external_surfaces::{self, PreparedPostExternalSurfaces},
};

use super::{
    candidates::{CandidateLayer, gather_candidates},
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
            Err(geometry_error(error))
        }
    }
}

#[cfg(test)]
pub(in crate::project_slice) fn dispose(prepared: PreparedPostBridgeCandidates) {
    external_surfaces::dispose(prepared.predecessor);
}

fn gather_objects(
    predecessor: &PreparedPostExternalSurfaces,
) -> Result<Vec<BridgeCandidateObject>, ClipperError> {
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
            let filter = traversal
                .resolved
                .objects
                .iter()
                .find(|resolved| resolved.source_object_index == source_index)
                .expect("O43 object retains its resolved source")
                .object
                .dont_filter_internal_bridges;
            let (compensated, inputs) = input_object.as_parts();
            let (post_regions, _) = compensated.as_parts();
            let (_, _, regions) = post_regions.as_parts();
            let has_lightning_infill = regions.iter().any(|region| {
                region.as_parts().1.sparse_infill_pattern == ProcessInfillPattern::Lightning
            });
            let layers = object
                .records
                .iter()
                .zip(&prelude.records)
                .zip(inputs)
                .map(
                    |((record, classic), input)| match (record, classic, input) {
                        (Some(record), Some(classic), Some(input)) => {
                            let options = input_object.region_options(input);
                            Some(CandidateLayer {
                                lower_layer_index: input.lower_layer_index,
                                region_index: input.current.region_index,
                                fill_expolygons: &record.fill_expolygons,
                                fill_surfaces: &record.fill_surfaces,
                                sparse_infill_density_percent: options.sparse_infill_density.0,
                                solid_infill_spacing: classic.solid_infill_spacing,
                            })
                        }
                        (None, None, None) => None,
                        _ => unreachable!("validated O42 record slots remain aligned"),
                    },
                )
                .collect::<Vec<_>>();
            gather_candidates(&layers, has_lightning_infill, filter, scale)
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
