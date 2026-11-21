//! `FuzzySkin.cpp:477-494` + `PerimeterGenerator.cpp:146-151,240-255,274-278`:
//! `fuzzy_skin=allwalls` must consume the shared seeded stream in upstream
//! generation order — same-level siblings first, then chain-ordered child
//! recursion — while emission stays children-first. The seeded oracle is the
//! comparison authority; official fuzzy output is nondeterministic and is
//! asserted as such instead of being used as parity evidence.

const SOURCES: &[&str] = &[
    "PrintConfig.hpp:50",
    "PrintConfig.cpp:195-200,3540-3559",
    "Feature/FuzzySkin/FuzzySkin.cpp:294-342,477-494,560-578",
    "PerimeterGenerator.cpp:146-151,240-255,274-278",
];

const ALLWALLS_SHA: &str = "d455419d2782f950d4ffab379a973ca7ba16f1cbf7df871796a3d44b9eefe0e7";
const EXTERNAL_SHA: &str = "13533832531bfd43357a6651788bbecc64f0e69494252eeca1274194c4bc932b";
const ALL_SHA: &str = "7b7f1475e6e7cb8c9f60f6f1446708edd03544e358af59147087e322eeb7fe4e";
const STOCK_SHA: &str = "5d4a5f95fd4c12244e87b17e9e64052f56d66fd83df405438fb4c4e34c8c524c";

fn settings(project: &[u8]) -> serde_json::Value {
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(project)).unwrap();
    serde_json::from_reader(archive.by_name("Metadata/project_settings.config").unwrap()).unwrap()
}

fn assert_shared_k1_settings(project: &[u8], fuzzy_skin: &str) {
    let settings = settings(project);
    assert_eq!(settings["printer_model"], "Creality K1");
    assert_eq!(settings["wall_generator"], "classic");
    assert_eq!(settings["fuzzy_skin"], fuzzy_skin);
    assert_eq!(settings["fuzzy_skin_first_layer"], "0");
    assert_eq!(settings["fuzzy_skin_thickness"], "0.3");
    assert_eq!(settings["fuzzy_skin_point_distance"], "0.8");
}

/// Motion lines inside `;TYPE:Inner wall` sections of one `;Z:` layer. The
/// stock external wall path has a handful of straight segments while the
/// allwalls-fuzzified internal wall carries ~140 jitter segments, so this
/// pins both the first-layer gate and the internal-wall fuzzification.
fn inner_wall_motion_lines(gcode: &str, layer_z: &str) -> usize {
    let layer = layer_z.strip_prefix(";Z:").unwrap();
    let mut in_layer = false;
    let mut in_inner = false;
    let mut count = 0;
    for line in gcode.lines() {
        if let Some(marker) = line.strip_prefix(";Z:") {
            in_layer = marker == layer;
            in_inner = false;
        } else if in_layer && line.starts_with(";TYPE:") {
            in_inner = line == ";TYPE:Inner wall";
        } else if in_layer
            && in_inner
            && line.starts_with(['G', '1', '2', '3'])
            && line.contains(" X")
        {
            count += 1;
        }
    }
    count
}

#[test]
fn fuzzy_allwalls_seeded_oracle_ordered_bytes() {
    if !crate::runner::seeded_reference_enabled(None) {
        return;
    }
    let project = include_bytes!("../../../../tests/parity/fuzzy-allwalls-rng-order/allwalls.3mf");
    crate::runner::native_stream_case::check(
        ("fuzzy-allwalls-rng-order", SOURCES),
        "allwalls",
        project,
        ALLWALLS_SHA,
        (false, |reference, _replay| {
            assert_shared_k1_settings(project, "allwalls");
            assert!(
                inner_wall_motion_lines(&reference.native, ";Z:0.2") <= 10,
                "fuzzy_skin_first_layer=0 keeps the first layer unfuzzed"
            );
            assert!(
                inner_wall_motion_lines(&reference.native, ";Z:0.4") >= 50,
                "allwalls must fuzzify the layer-2 internal wall"
            );
        }),
    );
}

#[test]
fn fuzzy_external_seeded_oracle_control() {
    if !crate::runner::seeded_reference_enabled(None) {
        return;
    }
    let project = include_bytes!("../../../../tests/parity/fuzzy-allwalls-rng-order/external.3mf");
    crate::runner::native_stream_case::check(
        ("fuzzy-allwalls-rng-order", SOURCES),
        "external-control",
        project,
        EXTERNAL_SHA,
        (false, |reference, _replay| {
            assert_shared_k1_settings(project, "external");
            assert!(
                inner_wall_motion_lines(&reference.native, ";Z:0.4") <= 10,
                "external fuzzifies loop-0 only; the internal wall stays unfuzzed"
            );
        }),
    );
}

#[test]
fn fuzzy_all_seeded_oracle_control() {
    if !crate::runner::seeded_reference_enabled(None) {
        return;
    }
    let project = include_bytes!("../../../../tests/parity/fuzzy-allwalls-rng-order/all.3mf");
    crate::runner::native_stream_case::check(
        ("fuzzy-allwalls-rng-order", SOURCES),
        "all-control",
        project,
        ALL_SHA,
        (false, |reference, _replay| {
            assert_shared_k1_settings(project, "all");
            assert!(
                inner_wall_motion_lines(&reference.native, ";Z:0.4") <= 10,
                "all fuzzifies loop-0 contours and holes; the depth-1 internal wall stays unfuzzed"
            );
        }),
    );
}

#[test]
fn fuzzy_stock_native_control() {
    let project = include_bytes!("../../../../tests/parity/fuzzy-allwalls-rng-order/stock.3mf");
    crate::runner::native_stream_case::check(
        ("fuzzy-allwalls-rng-order", SOURCES),
        "stock-control",
        project,
        STOCK_SHA,
        (false, |reference, _replay| {
            assert_shared_k1_settings(project, "none");
            assert!(
                inner_wall_motion_lines(&reference.native, ";Z:0.4") <= 10,
                "stock keeps every wall unfuzzed"
            );
        }),
    );
}

#[test]
fn official_fuzzy_allwalls_oracle_is_explicitly_unstable() {
    if !crate::runner::official_reference_enabled() {
        return;
    }
    let Some(runner) = crate::runner::OrcaRunner::from_env().unwrap() else {
        return;
    };
    let project = include_bytes!("../../../../tests/parity/fuzzy-allwalls-rng-order/allwalls.3mf");
    assert_eq!(
        crate::runner::native_api_cli::sha(project),
        ALLWALLS_SHA,
        "frozen input SHA for the official instability witness"
    );
    assert_shared_k1_settings(project, "allwalls");
    let first = crate::runner::native_api_cli::NativeCase::slice(
        &runner,
        project,
        "fuzzy-allwalls-official-1-",
    );
    let second = crate::runner::native_api_cli::NativeCase::slice(
        &runner,
        project,
        "fuzzy-allwalls-official-2-",
    );
    let normalized = |case: &crate::runner::native_api_cli::NativeCase| {
        crate::golden::normalize_one_generator_line(
            case.native.as_bytes(),
            crate::golden::GeneratorKind::Orca,
        )
        .unwrap()
    };
    let instability = crate::golden::first_difference(&normalized(&first), &normalized(&second));
    std::fs::write(
        first.root.join("official-instability.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "case": "fuzzy-allwalls-rng-order/allwalls",
            "input_sha256": ALLWALLS_SHA,
            "oracle_mode": "official",
            "replay_comparison": instability
                .as_ref()
                .map(|difference| difference.to_string()),
            "policy": "official fuzzy output is nondeterministic and is never parity evidence; see docs/architecture/2026-10-25-fuzzy-allwalls-rng-order-spec.md"
        }))
        .unwrap(),
    )
    .unwrap();
    assert!(
        instability.is_some(),
        "official fuzzy oracle was expected nondeterministic for fuzzy_skin=allwalls; \
         if it is now deterministic, replace this assertion with official ordered-bytes \
         parity instead of letting unstably-produced streams gate anything"
    );
}
