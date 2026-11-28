use std::{fmt::Write as _, process::Command};

use ares_core::{
    CancelToken, ConfigOverrides, DirectProjectSliceRequest, GenerationMetadata, PrintWarning,
    SliceError, SlicerContext, SlicingNotificationType, WarningLevel, slice_project_plate_to_sink,
};
use sha2::{Digest, Sha256};

const KSR: &[u8] = include_bytes!("../../../tests/ksr_fdmtest_v4/ksr_fdmtest_v4.project.3mf");
const ARACHNE: &[u8] = include_bytes!("../../../tests/parity/arachne/default-prisms.3mf");
const TOOLCHANGE: &[u8] = include_bytes!(
    "../../../tests/parity/validation/skirt_wipe_tower_flush/toolchange-wipe-control.3mf"
);
const TWO_REGIONS: &[u8] =
    include_bytes!("../../../tests/parity/interface-shells/two-region-prism.3mf");
const BRIDGE: &[u8] =
    include_bytes!("../../../tests/parity/grid-surface-collections/k1-bridge.3mf");
const VOLUME_RANGES: &[u8] =
    include_bytes!("../../../tests/parity/object-layer-ranges/volume-precedence.3mf");
const HILBERT_BRIDGE: &[u8] =
    include_bytes!("../../../tests/parity/turning-bridge-boundary-sampling/mk4-hilbert.3mf");
const LIGHTNING: &[u8] =
    include_bytes!("../../../tests/parity/lightning-initial-overhang-join/positive.3mf");
const CURVED_WALLS: &[u8] =
    include_bytes!("../../../tests/parity/gyroid-standard-boundary-waves/paired-cylinders.3mf");
const SUPPORT_NECESSITY: &[u8] =
    include_bytes!("../../../tests/parity/spiral-vase-admission/two-object-by-object-apart.3mf");
const TREE_OVERHANG: &[u8] =
    include_bytes!("../../ares-cli/tests/organic_merge_affinity/overhang-organic.3mf");
const CHILD_OUTPUT: &str = "ARES_PARALLEL_SLICING_HASH_FILE";

async fn slice(
    project: &[u8],
    overrides: &ConfigOverrides,
    context: SlicerContext<'_>,
    output: &mut Vec<u8>,
) -> Result<Vec<PrintWarning>, SliceError> {
    slice_project_plate_to_sink(
        project,
        DirectProjectSliceRequest {
            plate: Some(1),
            overrides,
            system_catalog: None,
        },
        GenerationMetadata::default(),
        context,
        output,
    )
    .await
    .map(|result| result.warnings)
}

fn write_hash(hashes: &mut String, label: &str, bytes: impl AsRef<[u8]>) {
    write!(hashes, "{label}: ").unwrap();
    for byte in Sha256::digest(bytes) {
        write!(hashes, "{byte:02x}").unwrap();
    }
    hashes.push('\n');
}

fn is_support_warning(warning: &PrintWarning) -> bool {
    warning.message_type == SlicingNotificationType::NeedSupportOn
}

async fn fixture_hashes() -> String {
    let mut hashes = String::new();
    for (label, project, options) in [
        ("ksr", KSR, "{}"),
        ("arachne", ARACHNE, "{}"),
        ("toolchange", TOOLCHANGE, "{}"),
        ("shared_shells", TWO_REGIONS, r#"{"interface_shells":"0"}"#),
        (
            "regional_shells",
            TWO_REGIONS,
            r#"{"interface_shells":"1"}"#,
        ),
        (
            "external_extra_bridge",
            BRIDGE,
            r#"{"enable_extra_bridge_layer":"external_bridge_only"}"#,
        ),
        ("volume_ranges", VOLUME_RANGES, "{}"),
        ("hilbert_bridge", HILBERT_BRIDGE, "{}"),
        (
            "random_template_bridge",
            HILBERT_BRIDGE,
            r#"{"sparse_infill_rotate_template":"+30~2","solid_infill_rotate_template":"+30^2"}"#,
        ),
        ("lightning", LIGHTNING, "{}"),
        (
            "arc_fitting_enabled",
            CURVED_WALLS,
            r#"{"enable_arc_fitting":"enabled"}"#,
        ),
        (
            "arc_fitting_disabled",
            CURVED_WALLS,
            r#"{"enable_arc_fitting":"disabled"}"#,
        ),
    ] {
        let mut output = Vec::new();
        let overrides = ConfigOverrides::from_json(options.as_bytes()).unwrap();
        slice(project, &overrides, SlicerContext::new(0), &mut output)
            .await
            .unwrap_or_else(|error| panic!("{label}: {error}"));
        assert!(!output.is_empty(), "{label}: no G-code");
        write_hash(&mut hashes, label, output);
    }
    for (label, project, options, enabled) in [
        ("support_necessity", SUPPORT_NECESSITY, "{}", false),
        ("organic_support", TREE_OVERHANG, "{}", true),
        (
            "hybrid_support",
            TREE_OVERHANG,
            r#"{"support_style":"tree_hybrid"}"#,
            true,
        ),
    ] {
        let overrides = ConfigOverrides::from_json(options.as_bytes()).unwrap();
        let mut output = Vec::new();
        let warnings = slice(project, &overrides, SlicerContext::new(0), &mut output)
            .await
            .unwrap_or_else(|error| panic!("{label}: {error}"));
        let gcode = std::str::from_utf8(&output).unwrap();
        assert!(!gcode.is_empty(), "{label}: no G-code");
        assert_eq!(gcode.contains(";TYPE:Support\n"), enabled, "{label}");
        assert_eq!(warnings.iter().any(is_support_warning), !enabled, "{label}");
        write_hash(&mut hashes, label, output);
        write_hash(
            &mut hashes,
            &format!("{label}_warnings"),
            format!("{warnings:?}"),
        );
    }
    hashes
}

#[tokio::test]
async fn changing_thread_count_preserves_complete_gcode_bytes() {
    if let Some(file) = std::env::var_os(CHILD_OUTPUT) {
        std::fs::write(file, fixture_hashes().await).unwrap();
        return;
    }
    // The default Rayon pool reads its thread count at initialization. Each
    // run must start a process before any slicing initializes that pool.
    let directory = tempfile::tempdir().unwrap();
    let mut reference = None;
    let thread_counts: &[usize] = if cfg!(feature = "parallel") {
        &[1, 2, 7, 16]
    } else {
        &[1]
    };
    for threads in thread_counts {
        let file = directory.path().join(format!("{threads}.sha256"));
        let child = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "changing_thread_count_preserves_complete_gcode_bytes",
            ])
            .env("RAYON_NUM_THREADS", threads.to_string())
            .env(CHILD_OUTPUT, &file)
            .output()
            .unwrap();
        assert!(
            child.status.success(),
            "{threads} threads: {}{}",
            String::from_utf8_lossy(&child.stdout),
            String::from_utf8_lossy(&child.stderr)
        );
        let hashes = std::fs::read_to_string(file).unwrap();
        if let Some(reference) = &reference {
            assert_eq!(
                &hashes, reference,
                "{threads} threads changed G-code bytes or warnings"
            );
        } else {
            reference = Some(hashes);
        }
    }
    print!("{}", reference.unwrap());
}

async fn assert_cancel_before_phase(project: &[u8], phase: &str, overrides: &ConfigOverrides) {
    let token = CancelToken::new();
    let mut reached_phase = false;
    let mut cancel_at_phase = |status: &ares_core::SlicingStatus<'_>| {
        if status.text == phase {
            reached_phase = true;
            token.cancel();
        }
    };
    let mut context = SlicerContext::new(0).with_cancel_token(token.clone());
    context.set_status_callback(&mut cancel_at_phase);
    let mut canceled = Vec::new();
    let result = slice(project, overrides, context, &mut canceled).await;
    assert!(reached_phase, "did not reach {phase}");
    assert_eq!(result, Err(SliceError::Canceled), "cancel at {phase}");
    assert!(canceled.is_empty(), "cancel at {phase} wrote G-code");
}

#[tokio::test]
async fn canceling_before_mesh_or_infill_work_leaves_sink_empty_and_allows_a_fresh_slice() {
    let overrides = ConfigOverrides::default();
    let mut original = Vec::new();
    slice(ARACHNE, &overrides, SlicerContext::new(0), &mut original)
        .await
        .unwrap();
    assert!(!original.is_empty());

    for phase in ["Slicing mesh", "Generating infill regions"] {
        assert_cancel_before_phase(ARACHNE, phase, &overrides).await;
    }

    let mut recovered = Vec::new();
    let fresh = SlicerContext::new(0).with_cancel_token(CancelToken::new());
    slice(ARACHNE, &overrides, fresh, &mut recovered)
        .await
        .unwrap();
    assert!(
        recovered == original,
        "cancel changed the next slice's bytes"
    );
}

#[tokio::test]
async fn canceling_support_necessity_leaves_sink_empty_and_preserves_fresh_warnings() {
    let overrides = ConfigOverrides::default();
    let mut original = Vec::new();
    let expected = slice(
        SUPPORT_NECESSITY,
        &overrides,
        SlicerContext::new(0),
        &mut original,
    )
    .await
    .unwrap();
    let warning = expected
        .iter()
        .find(|warning| is_support_warning(warning))
        .unwrap();
    assert_eq!(warning.level, WarningLevel::NonCritical);
    assert!(warning.message.contains("a floating cantilever"));

    assert_cancel_before_phase(SUPPORT_NECESSITY, "Checking support necessity", &overrides).await;

    let fresh = SlicerContext::new(0).with_cancel_token(CancelToken::new());
    let mut recovered = Vec::new();
    let actual = slice(SUPPORT_NECESSITY, &overrides, fresh, &mut recovered)
        .await
        .unwrap();
    assert!(
        recovered == original,
        "cancel changed the next slice's bytes"
    );
    assert_eq!(actual, expected, "cancel changed the next slice's warnings");
}
