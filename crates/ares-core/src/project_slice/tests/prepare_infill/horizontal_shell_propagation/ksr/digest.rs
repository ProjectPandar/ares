use super::super::super::horizontal_shell_promotion::ksr::digest::{mix, surfaces_digest};
use crate::project_slice::prepare_infill::{
    horizontal_shell_propagation::{self, PropagationEvent},
    surface_type_detection::PreparedSurfaceTypeObject,
};

pub(in crate::project_slice::tests::prepare_infill::horizontal_shell_propagation) fn surface_sequence_digest(
    objects: &[PreparedSurfaceTypeObject],
) -> i128 {
    let mut digest = 0x4f25_5355_5246_4143_455f_4449_4745_5354_i128;
    surfaces_digest(&mut digest, objects);
    digest
}

pub(in crate::project_slice::tests::prepare_infill::horizontal_shell_propagation) fn event_sequence_digest(
    events: &[PropagationEvent],
) -> i128 {
    let mut digest = 0x4f26_4556_454e_545f_5345_5155_454e_4345_i128;
    mix(&mut digest, events.len() as i128);
    for (index, event) in events.iter().enumerate() {
        mix(&mut digest, index as i128);
        match *event {
            PropagationEvent::FillClone { object, layer } => {
                mix(&mut digest, 1);
                mix(&mut digest, object as i128);
                mix(&mut digest, layer as i128);
            }
            PropagationEvent::RecordVisit { object, layer } => {
                mix(&mut digest, 2);
                mix(&mut digest, object as i128);
                mix(&mut digest, layer as i128);
            }
            PropagationEvent::EnsureAllSkip { object, layer } => {
                mix(&mut digest, 3);
                mix(&mut digest, object as i128);
                mix(&mut digest, layer as i128);
            }
            PropagationEvent::SourceKindVisit {
                object,
                layer,
                kind,
            } => {
                mix(&mut digest, 4);
                mix(&mut digest, object as i128);
                mix(&mut digest, layer as i128);
                mix(&mut digest, source_kind_tag(kind));
            }
            PropagationEvent::NeighborVisit {
                object,
                source,
                neighbor,
                kind,
            } => {
                mix(&mut digest, 5);
                mix(&mut digest, object as i128);
                mix(&mut digest, source as i128);
                mix(&mut digest, neighbor as i128);
                mix(&mut digest, source_kind_tag(kind));
            }
            PropagationEvent::Rebuild {
                object,
                source,
                neighbor,
                kind,
            } => {
                mix(&mut digest, 6);
                mix(&mut digest, object as i128);
                mix(&mut digest, source as i128);
                mix(&mut digest, neighbor as i128);
                mix(&mut digest, source_kind_tag(kind));
            }
            PropagationEvent::DirtyCommit { object, layer } => {
                mix(&mut digest, 7);
                mix(&mut digest, object as i128);
                mix(&mut digest, layer as i128);
            }
        }
    }
    digest
}

pub(in crate::project_slice::tests::prepare_infill::horizontal_shell_propagation) fn propagation_event_counts(
    events: &[PropagationEvent],
) -> [usize; 7] {
    let mut counts = [0; 7];
    for event in events {
        let index = match event {
            PropagationEvent::FillClone { .. } => 0,
            PropagationEvent::RecordVisit { .. } => 1,
            PropagationEvent::EnsureAllSkip { .. } => 2,
            PropagationEvent::SourceKindVisit { .. } => 3,
            PropagationEvent::NeighborVisit { .. } => 4,
            PropagationEvent::Rebuild { .. } => 5,
            PropagationEvent::DirtyCommit { .. } => 6,
        };
        counts[index] += 1;
    }
    counts
}

fn source_kind_tag(kind: horizontal_shell_propagation::SourceKind) -> i128 {
    match kind {
        horizontal_shell_propagation::SourceKind::Top => 10,
        horizontal_shell_propagation::SourceKind::Bottom => 11,
        horizontal_shell_propagation::SourceKind::BottomBridge => 12,
    }
}
