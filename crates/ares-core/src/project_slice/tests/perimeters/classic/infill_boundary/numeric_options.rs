use crate::{
    SliceError,
    project_slice::perimeters::{
        classic::{gap_extrusion::PreparedPostClassicGapExtrusion, infill_boundary},
        prepare_post_classic_gap_extrusion,
    },
};

use super::super::super::super::support::KsrArchive;
use super::precedence::assert_numeric_precedence;

const CONFIG: &str = "Metadata/project_settings.config";
const RANGE_ERROR: &str =
    "Classic infill-boundary overlap is outside the supported coordinate range";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum NumericFailure {
    Conversion,
    PostSubtraction,
    NoOverlapDelta,
}

#[derive(Clone, Copy)]
struct NumericContext {
    pre_inset: i64,
    basis: i64,
    min_half: i64,
}

#[test]
fn task22o15_typed_3mf_reaches_post_subtraction_overflow() {
    let source = prepare_post_classic_gap_extrusion(&KsrArchive::new().bytes()).unwrap();
    let contexts = ordinary_contexts(&source);
    let max_basis = contexts.iter().map(|context| context.basis).max().unwrap();
    let percent = i64::MIN as f64 / max_basis as f64 * 100.0;
    assert_eq!(
        first_failure(&contexts, percent),
        Some(NumericFailure::PostSubtraction)
    );

    let mut archive = KsrArchive::new();
    replace_ordinary_overlap(&mut archive, percent);
    assert_preflight_range_error(archive);
}

#[test]
fn task22o15_typed_3mf_reaches_no_overlap_delta_overflow() {
    let mut wide = KsrArchive::new();
    replace_solid_width(&mut wide);
    let source = prepare_post_classic_gap_extrusion(&wide.bytes()).unwrap();
    let contexts = ordinary_contexts(&source);
    let context = contexts.iter().max_by_key(|context| context.basis).unwrap();
    let post_target = i64::MAX as f64 - (1_u64 << 20) as f64;
    let target_overlap = context.pre_inset as f64 - post_target;
    let percent = target_overlap / context.basis as f64 * 100.0;
    assert_eq!(
        first_failure(&contexts, percent),
        Some(NumericFailure::NoOverlapDelta)
    );

    let mut archive = KsrArchive::new();
    replace_solid_width(&mut archive);
    replace_ordinary_overlap(&mut archive, percent);
    assert_preflight_range_error(archive);
}

fn ordinary_contexts(source: &PreparedPostClassicGapExtrusion) -> Vec<NumericContext> {
    source
        .predecessor
        .objects
        .iter()
        .flat_map(|traversal| {
            let onion = &traversal.predecessor.predecessor;
            let top_split = &onion.predecessor;
            let prelude = &top_split.predecessor;
            onion
                .records
                .iter()
                .zip(&prelude.records)
                .zip(&prelude.object.records)
                .filter_map(|((onion, prelude), input)| {
                    let (onion, prelude, input) =
                        (onion.as_ref()?, prelude.as_ref()?, input.as_ref()?);
                    (input.layer_id > 0 && input.upper_layer_index.is_some())
                        .then_some((onion, prelude))
                })
                .flat_map(|(onion, prelude)| {
                    onion.surfaces.iter().filter_map(move |surface| {
                        let pre_inset = match surface.effective_loop_number {
                            value if value < 0 => 0,
                            0 => prelude.external_spacing / 2,
                            _ => prelude.perimeter_spacing / 2,
                        };
                        (pre_inset > 0).then_some(NumericContext {
                            pre_inset,
                            basis: pre_inset + prelude.solid_infill_spacing / 2,
                            min_half: ((prelude.solid_infill_spacing as f64 * 0.6) as i64) / 2,
                        })
                    })
                })
        })
        .collect()
}

fn first_failure(contexts: &[NumericContext], percent: f64) -> Option<NumericFailure> {
    contexts.iter().find_map(|context| {
        let overlap = context.basis as f64 * 0.000_001 * percent / 100.0 / 0.000_001;
        let Some(overlap) = checked_trunc(overlap) else {
            return Some(NumericFailure::Conversion);
        };
        let Some(inset) = context.pre_inset.checked_sub(overlap) else {
            return Some(NumericFailure::PostSubtraction);
        };
        let _ = inset.checked_neg()?;
        if context.min_half > overlap && context.min_half.checked_sub(overlap).is_none() {
            Some(NumericFailure::NoOverlapDelta)
        } else {
            None
        }
    })
}

fn checked_trunc(value: f64) -> Option<i64> {
    (value.is_finite() && value >= i64::MIN as f64 && value < -(i64::MIN as f64))
        .then_some(value as i64)
}

fn replace_ordinary_overlap(archive: &mut KsrArchive, percent: f64) {
    archive.replace_unique(
        CONFIG,
        "\"infill_wall_overlap\": \"15%\"",
        &format!("\"infill_wall_overlap\": \"{percent:.17e}%\""),
    );
}

fn replace_solid_width(archive: &mut KsrArchive) {
    archive.replace_unique(
        CONFIG,
        "\"internal_solid_infill_line_width\": \"0.42\"",
        "\"internal_solid_infill_line_width\": \"1000\"",
    );
}

fn assert_preflight_range_error(archive: KsrArchive) {
    let source = prepare_post_classic_gap_extrusion(&archive.bytes()).unwrap();
    assert_numeric_precedence(&source);
    assert!(matches!(
        infill_boundary::finish(source),
        Err(SliceError::InvalidInput(message)) if message == RANGE_ERROR
    ));
}
