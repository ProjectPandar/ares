//! F1 single-group whole coverage public RED; SPEC308583e3a approved.
//! Official stochastic execution is diagnostic only, never a strict parity PASS.
use crate::runner::OrcaRunner;
use sha2::{Digest, Sha256};

#[path = "arachne_fuzzy_f1/context.rs"]
mod context;
#[path = "arachne_fuzzy_f1/native.rs"]
mod native;
#[path = "arachne_fuzzy_f1/public.rs"]
mod public;
#[path = "arachne_fuzzy_f1/witness.rs"]
mod witness;

const MANIFEST: &str = include_str!("../../../../tests/parity/arachne-fuzzy-f1/provenance.json");

fn sha(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn compare(label: &str, project: &[u8]) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        crate::native_environment::skipped("F1 Native oracle not configured");
        return;
    };
    let root = tempfile::Builder::new()
        .prefix(&format!("fuzzy-f1-{label}-"))
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    public::compare(native::prepare(&runner, label, project, 0, &root), project);
}

macro_rules! case {
    ($test:ident,$label:literal) => {
        #[test]
        fn $test() {
            compare(
                $label,
                include_bytes!(concat!(
                    "../../../../tests/parity/arachne-fuzzy-f1/",
                    $label,
                    ".3mf"
                )),
            );
        }
    };
}
case!(f1_classic_displacement, "classic-displacement");
case!(f1_classic_extrusion, "classic-extrusion");
case!(f1_classic_combined, "classic-combined");
case!(f1_perlin_displacement, "perlin-displacement");
case!(f1_perlin_extrusion, "perlin-extrusion");
case!(f1_perlin_combined, "perlin-combined");
case!(f1_billow_displacement, "billow-displacement");
case!(f1_billow_extrusion, "billow-extrusion");
case!(f1_billow_combined, "billow-combined");
case!(f1_ridgedmulti_displacement, "ridgedmulti-displacement");
case!(f1_ridgedmulti_extrusion, "ridgedmulti-extrusion");
case!(f1_ridgedmulti_combined, "ridgedmulti-combined");
case!(f1_voronoi_displacement, "voronoi-displacement");
case!(f1_voronoi_extrusion, "voronoi-extrusion");
case!(f1_voronoi_combined, "voronoi-combined");
case!(f1_ripple_displacement, "ripple-displacement");
case!(f1_ripple_extrusion, "ripple-extrusion");
case!(f1_ripple_combined, "ripple-combined");
case!(f1_hole_eligibility, "hole");
case!(f1_all_eligibility, "all");
case!(f1_allwalls_inset_eligibility, "allwalls");
case!(f1_disabled_control, "disabled");
case!(f1_none_control, "none");
case!(f1_first_layer_false, "first-layer-off");
case!(f1_zero_thickness_source_eligibility, "zero-thickness");
case!(f1_sub_threshold_source_eligibility, "sub-threshold");
case!(f1_ripple_phase_layer_groups, "ripple-phase");
case!(f1_open_neck_variable_width, "open-neck");
case!(f1_small_padding, "small-padding");
