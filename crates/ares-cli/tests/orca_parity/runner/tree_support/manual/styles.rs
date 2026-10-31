use super::{
    OrcaRunner, has_support_extrusion, overhang_case, paint_canopy, parity, slice_painted,
};

#[test]
fn manual_default_painted_overhang_preserves_complete_output() {
    compare_manual_style("tree-support/manual/default/painted-overhang", "default");
}

#[test]
fn manual_organic_painted_overhang_preserves_complete_output() {
    compare_manual_style("tree-support/manual/organic/painted-overhang", "organic");
}

#[test]
fn manual_strong_painted_overhang_preserves_complete_output() {
    compare_manual_style("tree-support/manual/strong/painted-overhang", "tree_strong");
}

#[test]
fn manual_hybrid_painted_overhang_preserves_complete_output() {
    compare_manual_style("tree-support/manual/hybrid/painted-overhang", "tree_hybrid");
}

#[test]
fn manual_grid_style_painted_overhang_preserves_complete_output() {
    compare_manual_style("tree-support/manual/grid/painted-overhang", "grid");
}

#[test]
fn manual_snug_style_painted_overhang_preserves_complete_output() {
    compare_manual_style("tree-support/manual/snug/painted-overhang", "snug");
}

fn compare_manual_style(label: &str, style: &str) {
    let Some(baseline) = overhang_case(label, style, &[("support_type", "tree(manual)")]) else {
        return;
    };
    assert!(!has_support_extrusion(
        std::str::from_utf8(&baseline.reference).unwrap()
    ));
    let runner = OrcaRunner::from_env().unwrap().unwrap();
    let painted = slice_painted(
        &runner,
        paint_canopy(&baseline.project, &["4", "4"]),
        "painted-manual-style",
    );
    assert!(has_support_extrusion(
        std::str::from_utf8(&painted.reference).unwrap()
    ));
    let outcome = parity::compare_case(&painted);
    assert_eq!(outcome.status, "PASS", "{}", outcome.detail);
}
