use super::BrimPlan;
use crate::project_slice::{perimeters, tests::support::KsrArchive};

#[test]
fn explicit_outer_brim_generates_outside_in_paths() {
    let mut archive = KsrArchive::new();
    archive.replace_unique(
        "Metadata/project_settings.config",
        "\"brim_type\": \"auto_brim\"",
        "\"brim_type\": \"outer_only\"",
    );
    let traversal = perimeters::prepare_post_classic_traversal(&archive.bytes()).unwrap();

    let brim = BrimPlan::generate(&traversal).unwrap().unwrap();

    assert!(brim.paths.len() > 2);
    assert!(brim.paths.iter().all(|path| path.first() == path.last()));
}
