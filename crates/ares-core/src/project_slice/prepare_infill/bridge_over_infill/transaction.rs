mod anchor_projection;
mod candidate_expansion;
mod extra_bridge;
mod surface_rewrite;

use crate::{
    SliceError,
    geometry::ClipperError,
    project_slice::prepare_infill::external_surfaces::{self, PreparedPostExternalSurfaces},
};

use super::{PreparedPostBridgeCandidates, types::BridgeCandidateObject};

#[cfg(test)]
thread_local! {
    static INVOCATIONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static DISPOSALS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

pub(in crate::project_slice) struct PreparedPostBridgeOverInfill {
    pub(in crate::project_slice) predecessor: PreparedPostExternalSurfaces,
}

pub(in crate::project_slice) fn prepare(
    predecessor: PreparedPostBridgeCandidates,
) -> Result<PreparedPostBridgeOverInfill, SliceError> {
    #[cfg(test)]
    INVOCATIONS.with(|count| count.set(count.get() + 1));
    let PreparedPostBridgeCandidates {
        mut predecessor,
        mut objects,
    } = predecessor;
    if let Err(error) = validate_capabilities(&predecessor, &objects)
        .and_then(|()| candidate_expansion::prepare(&predecessor, &mut objects))
        .and_then(|()| surface_rewrite::prepare(&mut predecessor, &objects))
        .and_then(|()| extra_bridge::prepare(&mut predecessor))
    {
        external_surfaces::dispose(predecessor);
        return Err(error);
    }
    Ok(PreparedPostBridgeOverInfill { predecessor })
}

pub(in crate::project_slice) fn dispose(prepared: PreparedPostBridgeOverInfill) {
    #[cfg(test)]
    DISPOSALS.with(|count| count.set(count.get() + 1));
    external_surfaces::dispose(prepared.predecessor);
}

#[cfg(test)]
pub(in crate::project_slice) fn reset_hooks() {
    INVOCATIONS.with(|count| count.set(0));
    DISPOSALS.with(|count| count.set(0));
}

#[cfg(test)]
pub(in crate::project_slice) fn invocations() -> usize {
    INVOCATIONS.with(std::cell::Cell::get)
}

#[cfg(test)]
pub(in crate::project_slice) fn disposals() -> usize {
    DISPOSALS.with(std::cell::Cell::get)
}

fn validate_capabilities(
    predecessor: &PreparedPostExternalSurfaces,
    objects: &[BridgeCandidateObject],
) -> Result<(), SliceError> {
    let traversal = &predecessor.predecessor.predecessor;

    if objects.iter().any(|object| object.has_lightning_infill) {
        return Err(unsupported("sparse_infill_pattern"));
    }
    let horizontal = &predecessor.predecessor;
    if traversal
        .objects
        .iter()
        .zip(&horizontal.objects)
        .any(|(object, horizontal)| {
            let prelude = &object
                .predecessor
                .predecessor
                .predecessor
                .predecessor
                .object;
            let (compensated, _) = prelude.as_parts();
            let (post_regions, _) = compensated.as_parts();
            let (_, _, regions) = post_regions.as_parts();
            let needs_adaptive_octree = regions.iter().any(|region| {
                let options = region.as_parts().1;
                matches!(
                    options.sparse_infill_pattern,
                    crate::ProcessInfillPattern::AdaptiveCubic
                        | crate::ProcessInfillPattern::SupportCubic
                ) && options.sparse_infill_density.0 > 0.0
            });
            needs_adaptive_octree
                && horizontal
                    .records
                    .iter()
                    .flatten()
                    .any(|record| !record.fill_surfaces.is_empty())
        })
    {
        return Err(unsupported("sparse_infill_pattern"));
    }
    Ok(())
}

pub(super) fn geometry_error(error: ClipperError) -> SliceError {
    match error {
        ClipperError::CoordinateOutOfRange => SliceError::InvalidInput(
            "bridge-over-infill coordinate is outside the supported Clipper range".to_owned(),
        ),
        ClipperError::OpenPathMustBeSubject | ClipperError::OpenPathsRequirePolyTree => {
            unreachable!("bridge-over-infill open paths use subject input and PolyTree output")
        }
    }
}

fn unsupported(key: &str) -> SliceError {
    SliceError::UnsupportedProjectFeature(key.to_owned())
}
