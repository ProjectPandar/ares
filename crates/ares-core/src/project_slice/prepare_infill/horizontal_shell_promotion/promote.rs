use crate::{
    SliceError,
    options::ExtraSolidInfills,
    project_slice::{
        prepare_infill::surface_type_detection::types::PreparedSurfaceTypeRecord,
        region_slices::RegionSurfaceKind,
    },
};

use super::record_commit;
#[cfg(test)]
use super::{PromotionEvent, record_event};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum StagedDecision {
    Noop,
    PromoteInternal,
}

pub(super) fn stage_decision(
    raw: &str,
    planned_layer_index: usize,
) -> Result<StagedDecision, SliceError> {
    #[cfg(test)]
    record_event(PromotionEvent::RawScheduleVisit);
    if raw.is_empty() {
        return Ok(StagedDecision::Noop);
    }

    #[cfg(test)]
    record_event(PromotionEvent::NonemptySchedule);
    #[cfg(test)]
    record_event(PromotionEvent::ParserInvocation);
    let schedule = ExtraSolidInfills::parse_raw(raw)?;
    #[cfg(test)]
    record_event(PromotionEvent::MatcherInvocation);
    Ok(if schedule.matches_layer(planned_layer_index) {
        StagedDecision::PromoteInternal
    } else {
        StagedDecision::Noop
    })
}

pub(super) fn commit(record: &mut PreparedSurfaceTypeRecord, decision: StagedDecision) {
    if decision == StagedDecision::Noop {
        return;
    }

    record_commit();
    for surface in &mut record.fill_surfaces {
        if surface.as_parts().0 == RegionSurfaceKind::Internal {
            surface.retag(RegionSurfaceKind::InternalSolid);
            #[cfg(test)]
            record_event(PromotionEvent::PromotedSurface);
        }
    }
}
