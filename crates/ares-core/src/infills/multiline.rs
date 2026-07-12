use crate::{InfillOptions, InfillPattern, options::InfillLayerRole};

use super::scanline::{InfillCandidate, ScanlineBounds, Vector2};

pub(super) fn source_spacing(
    role: InfillLayerRole,
    pattern: InfillPattern,
    base_spacing: f64,
    options: &InfillOptions,
) -> f64 {
    if applies(role, pattern, options) {
        base_spacing * options.fill_multiline() as f64
    } else {
        base_spacing
    }
}

pub(super) fn expand_candidates(
    candidates: Vec<InfillCandidate>,
    expansion: Expansion<'_>,
) -> Vec<InfillCandidate> {
    if !applies(expansion.role, expansion.pattern, expansion.options) {
        return candidates;
    }
    let offsets = offsets(
        expansion.options.fill_multiline(),
        expansion.options.line_width(),
    );
    let mut expanded = Vec::with_capacity(candidates.len() * offsets.len());
    for candidate in candidates {
        for &offset in &offsets {
            let translated = candidate.translated(expansion.normal, offset);
            if expansion.bounds.contains(translated.scanline) {
                expanded.push(translated);
            }
        }
    }
    expanded
}

pub(super) struct Expansion<'a> {
    pub(super) normal: Vector2,
    pub(super) bounds: ScanlineBounds,
    pub(super) role: InfillLayerRole,
    pub(super) pattern: InfillPattern,
    pub(super) options: &'a InfillOptions,
}

fn applies(role: InfillLayerRole, pattern: InfillPattern, options: &InfillOptions) -> bool {
    role.is_sparse()
        && options.fill_multiline() > 1
        && matches!(
            pattern,
            InfillPattern::Rectilinear
                | InfillPattern::AlignedRectilinear
                | InfillPattern::Line
                | InfillPattern::Grid
        )
}

fn offsets(fill_multiline: usize, line_width: f64) -> Vec<f64> {
    let rings = fill_multiline / 2;
    let mut offsets = Vec::with_capacity(fill_multiline);
    if fill_multiline % 2 == 1 {
        for index in (1..=rings).rev() {
            offsets.push(-(index as f64) * line_width);
        }
        offsets.push(0.0);
        for index in 1..=rings {
            offsets.push(index as f64 * line_width);
        }
    } else {
        for index in (0..rings).rev() {
            offsets.push(-((index as f64) + 0.5) * line_width);
        }
        for index in 0..rings {
            offsets.push(((index as f64) + 0.5) * line_width);
        }
    }
    offsets
}
