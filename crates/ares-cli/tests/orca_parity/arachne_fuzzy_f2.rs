//! F2 group/mask/source-index public RED, before either production slice.
use crate::runner::OrcaRunner;
use sha2::{Digest, Sha256};

#[path = "arachne_fuzzy_f2/native.rs"]
mod native;
#[path = "arachne_fuzzy_f2/public.rs"]
mod public;
#[path = "arachne_fuzzy_f2/witness.rs"]
mod witness;

const MANIFEST: &str = include_str!("../../../../tests/parity/arachne-fuzzy-f2/provenance.json");

fn sha(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn compare(label: &str, project: &[u8]) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        crate::native_environment::skipped("F2 Native oracle not configured");
        return;
    };
    let root = tempfile::Builder::new()
        .prefix(&format!("fuzzy-f2-{label}-"))
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
                    "../../../../tests/parity/arachne-fuzzy-f2/",
                    $label,
                    ".3mf"
                )),
            );
        }
    };
}
case!(f2_none_external_equal_walls, "none-external");
case!(f2_external_none_equal_walls, "external-none");
case!(f2_two_distinct_active_effects, "two-active");
case!(f2_same_effect_type_merge, "same-effect-types");
case!(f2_inner_eligibility, "inner-eligibility");
case!(f2_later_overlap_priority, "overlap-priority");
case!(f2_open_width_source_crossing, "open-width-crossing");
