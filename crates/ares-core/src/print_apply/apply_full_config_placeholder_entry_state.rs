#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum StagedApplyStatus {
    Unchanged,
    Changed,
    Invalidated,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum StagedFullConfigPlaceholderEntryEvent {
    LogFullConfigDiffChanged,
    InvalidateStep {
        step: &'static str,
        invalidated: bool,
    },
    ClearPlaceholderParserConfig,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct StagedFullConfigPlaceholderEntry {
    pub(super) num_extruders: usize,
    pub(super) num_extruders_changed: bool,
    pub(super) status: StagedApplyStatus,
    pub(super) events: Vec<StagedFullConfigPlaceholderEntryEvent>,
}

fn staged_update_apply_status(status: &mut StagedApplyStatus, invalidated: bool) {
    let next = if invalidated {
        StagedApplyStatus::Invalidated
    } else {
        StagedApplyStatus::Changed
    };
    *status = (*status).max(next);
}

pub(super) fn staged_apply_full_config_placeholder_entry(
    prior_status: StagedApplyStatus,
    num_extruders: usize,
    full_config_diff: &[&'static str],
    gcode_export_invalidated: bool,
) -> StagedFullConfigPlaceholderEntry {
    let mut status = prior_status;
    let mut events = Vec::new();

    if !full_config_diff.is_empty() {
        events.push(StagedFullConfigPlaceholderEntryEvent::LogFullConfigDiffChanged);
        events.push(StagedFullConfigPlaceholderEntryEvent::InvalidateStep {
            step: "psGCodeExport",
            invalidated: gcode_export_invalidated,
        });
        staged_update_apply_status(&mut status, gcode_export_invalidated);
        events.push(StagedFullConfigPlaceholderEntryEvent::ClearPlaceholderParserConfig);
    }

    StagedFullConfigPlaceholderEntry {
        num_extruders,
        num_extruders_changed: false,
        status,
        events,
    }
}
