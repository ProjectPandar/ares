//! Cooperative cancellation (`PrintBase::throw_if_canceled` / `CanceledException`,
//! `PrintBase.hpp:47-51,498-514,550-555`) and deterministic multi-threaded slicing E2E tests.
//!
//! Failure inventory:
//! - Pre-canceled `CancelToken` fails to short-circuit with `Err(SliceError::Canceled)`.
//! - `CancelToken` is not `Clone + Send + Sync` or fails to propagate across threads.
//! - Mid-slice cancellation triggered from `SlicerContext::set_status_callback` at mesh slicing,
//!   wall generation, infill, support, or G-code layer emission fails to stop with
//!   `Err(SliceError::Canceled)` or continues emitting later stages.
//! - `SliceError::Canceled` display text diverges from `Slic3r::CanceledException::what()`.
//! - Repeated slices of classic, Arachne, fuzzy-skin, or organic tree-support projects diverge
//!   across invocations when parallel loops are active.

#[path = "../../../tests/support/slice_project_plate.rs"]
mod direct_project;

use std::sync::mpsc;

use ares_core::{CancelToken, ConfigOverrides, GenerationMetadata, SliceError, SlicerContext};

const OVERHANG: &[u8] =
    include_bytes!("../../ares-cli/tests/organic_merge_affinity/overhang-organic.3mf");
const TWO_CUBES: &[u8] =
    include_bytes!("../../../tests/parity/aligned-seam-rotated-object/rotated-two-cubes.3mf");
const ARACHNE: &[u8] = include_bytes!("../../../tests/parity/arachne/default-prisms.3mf");
const FUZZY: &[u8] =
    include_bytes!("../../../tests/parity/arachne-fuzzy-f1/classic-displacement.3mf");

async fn slice_with_context(
    project: &[u8],
    context: SlicerContext<'_>,
) -> Result<Vec<u8>, SliceError> {
    direct_project::slice_project_plate(
        project,
        GenerationMetadata::default(),
        None,
        &ConfigOverrides::default(),
        context,
    )
    .await
}

fn assert_send_sync_clone<T: Clone + Send + Sync>() {}

#[test]
fn cancel_token_contract_and_error_display() {
    assert_send_sync_clone::<CancelToken>();

    let token = CancelToken::new();
    let cloned = token.clone();
    assert!(!token.is_canceled());
    assert!(token.check().is_ok());

    let context = SlicerContext::new(0).with_cancel_token(token.clone());
    assert!(context.check_canceled().is_ok());
    let from_context = context.cancel_token();
    assert!(!from_context.is_canceled());

    cloned.cancel();
    assert!(token.is_canceled());
    assert!(from_context.is_canceled());
    assert_eq!(context.check_canceled(), Err(SliceError::Canceled));
    assert_eq!(
        SliceError::Canceled.to_string(),
        "Background processing has been canceled"
    );
}

#[tokio::test]
async fn pre_canceled_token_aborts_immediately_without_status_updates() {
    let token = CancelToken::new();
    token.cancel();

    let mut seen_statuses = Vec::new();
    let mut record = |status: &ares_core::SlicingStatus<'_>| {
        seen_statuses.push(status.text.to_owned());
    };
    let mut context = SlicerContext::new(0).with_cancel_token(token);
    context.set_status_callback(&mut record);

    let result = slice_with_context(OVERHANG, context).await;
    assert_eq!(result, Err(SliceError::Canceled));
    assert!(
        seen_statuses.is_empty(),
        "pre-canceled slice must stop before emitting stages, got {seen_statuses:?}"
    );
}

async fn assert_cancels_at_stage(trigger: &str, forbidden_later: &str) {
    let token = CancelToken::new();
    let trigger_token = token.clone();
    let mut seen = Vec::new();
    let mut callback = |status: &ares_core::SlicingStatus<'_>| {
        seen.push(status.text.to_owned());
        if status.text == trigger {
            trigger_token.cancel();
        }
    };
    let mut context = SlicerContext::new(0).with_cancel_token(token);
    context.set_status_callback(&mut callback);

    let result = slice_with_context(OVERHANG, context).await;
    assert_eq!(
        result,
        Err(SliceError::Canceled),
        "trigger `{trigger}` should cancel slice"
    );
    assert!(
        seen.iter().any(|text| text == trigger),
        "trigger `{trigger}` was never observed in {seen:?}"
    );
    assert!(
        !seen.iter().any(|text| text == forbidden_later),
        "slice kept running to `{forbidden_later}` after canceling at `{trigger}`: {seen:?}"
    );
}

#[tokio::test]
async fn status_callback_cancellation_stops_each_major_stage() {
    let stage_triggers = [
        ("Slicing mesh", "Generating walls"),
        ("Generating walls", "Generating infill regions"),
        ("Generating infill regions", "Generating support"),
        ("Generating support", "Generating G-code"),
        ("Generating G-code: layer 1", "Generating G-code: layer 5"),
    ];

    for (trigger, forbidden_later) in stage_triggers {
        assert_cancels_at_stage(trigger, forbidden_later).await;
    }
}

#[tokio::test]
async fn concurrent_thread_cancellation_stops_slice_cleanly() {
    let token = CancelToken::new();
    let thread_token = token.clone();
    let (entered_tx, entered_rx) = mpsc::channel::<()>();
    let (canceled_tx, canceled_rx) = mpsc::channel::<()>();

    let canceller = std::thread::spawn(move || {
        entered_rx
            .recv()
            .expect("slicing stage notifies cancel thread");
        thread_token.cancel();
        canceled_tx
            .send(())
            .expect("cancel thread acknowledges cancellation");
    });

    let mut notified = false;
    let mut callback = |status: &ares_core::SlicingStatus<'_>| {
        if !notified && status.text == "Generating walls" {
            notified = true;
            entered_tx.send(()).expect("signal cancel thread");
            canceled_rx.recv().expect("wait for cross-thread cancel");
        }
    };
    let mut context = SlicerContext::new(0).with_cancel_token(token);
    context.set_status_callback(&mut callback);

    let result = slice_with_context(OVERHANG, context).await;
    drop(entered_tx);
    canceller.join().expect("cancel thread exits cleanly");
    assert_eq!(result, Err(SliceError::Canceled));
}

#[tokio::test]
async fn repeated_slices_produce_byte_identical_gcode() {
    for (label, project) in [
        ("two_cubes", TWO_CUBES),
        ("arachne", ARACHNE),
        ("fuzzy", FUZZY),
        ("overhang_organic", OVERHANG),
    ] {
        let first = slice_with_context(project, SlicerContext::new(0))
            .await
            .unwrap_or_else(|error| panic!("{label} first slice failed: {error}"));
        let second = slice_with_context(project, SlicerContext::new(0))
            .await
            .unwrap_or_else(|error| panic!("{label} second slice failed: {error}"));
        assert!(
            !first.is_empty(),
            "{label} slice should produce non-empty G-code"
        );
        assert!(
            first == second,
            "{label} repeated slices diverged in G-code output"
        );
    }
}
