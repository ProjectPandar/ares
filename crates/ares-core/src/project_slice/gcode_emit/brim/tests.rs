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
    assert!(!brim.covered_hull().is_empty());
    let path_min_x = brim
        .paths
        .iter()
        .flatten()
        .map(|point| point.x())
        .min()
        .unwrap();
    let covered_min_x = brim
        .covered_hull()
        .iter()
        .map(|point| point.x())
        .min()
        .unwrap();
    assert!(covered_min_x < path_min_x);
}
