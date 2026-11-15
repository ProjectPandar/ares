use super::{
    Case, CaseInputs, OrcaRunner, ParityCase, VendorProfiles, application, json, repo_root,
};

const FIXED: [(&str, &str); 24] = [
    ("layer_height", "0.2"),
    ("initial_layer_print_height", "0.2"),
    ("wall_generator", "classic"),
    ("wall_loops", "2"),
    ("bottom_surface_pattern", "rectilinear"),
    ("top_surface_pattern", "rectilinear"),
    ("internal_solid_infill_pattern", "rectilinear"),
    ("bottom_shell_layers", "3"),
    ("top_shell_layers", "3"),
    ("enable_support", "0"),
    ("raft_layers", "0"),
    ("brim_type", "no_brim"),
    ("skirt_loops", "0"),
    ("enable_prime_tower", "0"),
    ("enable_arc_fitting", "0"),
    ("spiral_mode", "0"),
    ("seam_slope_type", "none"),
    ("fuzzy_skin", "none"),
    ("small_area_infill_flow_compensation", "0"),
    ("xy_contour_compensation", "0"),
    ("xy_hole_compensation", "0"),
    ("elefant_foot_compensation", "0"),
    ("infill_anchor", "400%"),
    ("infill_anchor_max", "20"),
];

pub(super) fn reference(runner: &OrcaRunner, label: &str, case: Case) -> ParityCase {
    let root = std::path::PathBuf::from(
        std::env::var_os("ARES_PARITY_PROFILE_RESOURCES_DIR")
            .expect("ARES_PARITY_PROFILE_RESOURCES_DIR is required"),
    )
    .join("profiles");
    let profiles = VendorProfiles::load(&root, "Creality").unwrap();
    let selection =
        crate::select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    let machine = profiles.machine(&selection.printer).unwrap();
    let mut process = profiles.process(&selection.process).unwrap();
    crate::normalize_process_defaults(&machine, &mut process);
    let mut filaments = selection
        .filaments
        .iter()
        .map(|name| profiles.filament(name).unwrap())
        .collect::<Vec<_>>();
    crate::normalize_filament_defaults(&mut filaments);
    let variable = [
        ("sparse_infill_pattern", case.pattern),
        ("fill_multiline", "1"),
        ("infill_direction", case.angle),
        ("sparse_infill_density", "30%"),
        ("sparse_infill_line_width", "0.45"),
        ("infill_combination", "0"),
    ];
    for (key, value) in FIXED.into_iter().chain(variable) {
        process.insert(key.into(), json!(value));
    }
    let path = repo_root().join("tests/parity/multiline-union/prism.stl");
    let expected = "1d50f639d940e2a95c324b8f7ef1a33ba9bcf7e83706ba3cf0669f67f6feae7e";
    assert_eq!(
        super::runner::cache_identity::raw_digest(&std::fs::read(&path).unwrap()),
        expected
    );
    let exported = runner
        .export_case(
            &CaseInputs {
                label: &format!("tpms-d-rotation/{label}"),
                machine: &machine,
                process: &process,
                filaments: &filaments,
            },
            &crate::smoke_overrides(),
            &path,
        )
        .unwrap();
    for (key, value) in FIXED.into_iter().chain(variable) {
        application::verify_value(&exported.project, key, &json!(value)).unwrap();
    }
    runner.slice_case(&exported).unwrap()
}
