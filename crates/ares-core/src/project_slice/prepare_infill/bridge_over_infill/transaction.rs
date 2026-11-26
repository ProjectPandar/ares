mod anchor_projection;
mod candidate_expansion;
mod extra_bridge;
mod surface_rewrite;

use crate::{
    SliceError, geometry::ClipperError,
    project_slice::prepare_infill::external_surfaces::PreparedPostExternalSurfaces,
};

use super::{PreparedPostBridgeCandidates, types::BridgeCandidateObject};

pub(in crate::project_slice) struct PreparedPostBridgeOverInfill {
    pub(in crate::project_slice) predecessor: PreparedPostExternalSurfaces,
    /// `prepare_lightning_infill_data` per object — the generator built on the
    /// expanded surfaces, shared between anchoring and the `fill_entities`
    /// emit (`PrintObject.cpp:2711` builds it once and `make_fills` reuses it).
    pub(in crate::project_slice) lightning_generators:
        Vec<Option<std::rc::Rc<crate::fill::lightning::generator::Generator>>>,
    /// `prepare_adaptive_infill_fields` per object (`PrintObject.cpp:3365`).
    pub(in crate::project_slice) adaptive_infill_fields:
        Vec<crate::fill::adaptive_infill_field::AdaptiveInfillFields>,
}

pub(in crate::project_slice) fn prepare(
    predecessor: PreparedPostBridgeCandidates,
) -> Result<PreparedPostBridgeOverInfill, SliceError> {
    let PreparedPostBridgeCandidates {
        mut predecessor,
        mut objects,
    } = predecessor;
    validate_capabilities(&predecessor, &objects)?;
    candidate_expansion::prepare(&predecessor, &mut objects)?;
    surface_rewrite::prepare(&mut predecessor, &objects)?;
    extra_bridge::prepare(&mut predecessor)?;
    let lightning_generators = objects
        .iter()
        .map(|object| object.lightning_generator.clone())
        .collect();
    let adaptive_infill_fields = objects
        .into_iter()
        .map(|object| object.adaptive_infill_fields)
        .collect();
    Ok(PreparedPostBridgeOverInfill {
        predecessor,
        lightning_generators,
        adaptive_infill_fields,
    })
}

fn validate_capabilities(
    predecessor: &PreparedPostExternalSurfaces,
    objects: &[BridgeCandidateObject],
) -> Result<(), SliceError> {
    let _ = (predecessor, objects);
    // `sparse_infill_pattern=lightning` is now ported — the
    // `has_lightning_infill` gate upstream only selects the generator build
    // path, it does not reject bridge-over-infill (`PrintObject.cpp:2593`).
    Ok(())
}

pub(super) fn geometry_error(error: ClipperError) -> SliceError {
    match error {
        ClipperError::CoordinateOutOfRange => SliceError::InvalidInput(
            "bridge-over-infill coordinate is outside the supported Clipper range".to_owned(),
        ),
    }
}
