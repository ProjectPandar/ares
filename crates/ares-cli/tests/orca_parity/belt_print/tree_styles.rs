use super::organic_support::{
    CANTILEVER, matches_reference, native_slice, project, reference, support_moves,
};
use std::path::Path;

const ALIGNED_CANTILEVER: &[u8] =
    include_bytes!("../../../../../tests/parity/belt-tree-support/hybrid-cantilever.3mf");

#[tokio::test]
async fn prints_slim_branches_on_a_translated_belt() {
    let Some(runner) = reference() else { return };
    let root = runner.work_dir();
    let work = root.join("slim");
    let slim = style(CANTILEVER, "tree_slim");
    let default = style(CANTILEVER, "default");
    let native = native_slice(&runner, &work, &slim);
    let control = native_slice(&runner, &root.join("default"), &default);
    assert_distinct_support(&native, &control, &work);
    matches_reference(&native, &work, &slim).await;
}

#[tokio::test]
async fn strong_branches_print_distinct_support_motion_on_a_belt() {
    let Some(runner) = reference() else { return };
    let root = runner.work_dir();
    let work = root.join("strong");
    let strong = style(CANTILEVER, "tree_strong");
    let native = native_slice(&runner, &work, &strong);
    for key in ["default", "tree_slim"] {
        let control = native_slice(&runner, &root.join(key), &style(CANTILEVER, key));
        assert_distinct_support(&native, &control, &work);
    }
    matches_reference(&native, &work, &strong).await;
}

#[tokio::test]
async fn hybrid_supports_a_slice_aligned_overhang_differently_from_slim() {
    let Some(runner) = reference() else { return };
    let root = runner.work_dir();
    let work = root.join("hybrid");
    let hybrid = style(ALIGNED_CANTILEVER, "tree_hybrid");
    let slim = style(ALIGNED_CANTILEVER, "tree_slim");
    let native = native_slice(&runner, &work, &hybrid);
    let control = native_slice(&runner, &root.join("slim"), &slim);
    assert_distinct_support(&native, &control, &work);
    matches_reference(&native, &work, &hybrid).await;
}

#[tokio::test]
async fn raising_the_belt_floor_changes_hybrid_support_motion() {
    let Some(runner) = reference() else { return };
    let root = runner.work_dir();
    let work = root.join("hybrid-raised-floor");
    let raised = project(
        ALIGNED_CANTILEVER,
        &[("belt_support_floor_offset", "1")],
        false,
    );
    let native = native_slice(&runner, &work, &raised);
    let control = native_slice(&runner, &root.join("hybrid-floor-zero"), ALIGNED_CANTILEVER);
    assert_distinct_support(&native, &control, &work);
    matches_reference(&native, &work, &raised).await;
}

fn style(fixture: &[u8], value: &str) -> Vec<u8> {
    project(fixture, &[("support_style", value)], false)
}

fn assert_distinct_support(left: &str, right: &str, work: &Path) {
    let (left, right) = (support_moves(left), support_moves(right));
    assert!(
        !left.is_empty() && !right.is_empty(),
        "missing support at {work:?}"
    );
    assert!(
        left != right,
        "identical positive-E support motion at {work:?}"
    );
}
