use super::*;
use crate::SliceOptions;

#[test]
fn empty_first_layer_paths_fall_back_to_configured_mesh_bounds() {
    let placeholders =
        placeholders(&SliceOptions::default(), GCodeFlavor::MarlinLegacy, &[]).unwrap();

    assert_eq!(placeholders.min_list(), "-99999,-99999");
    assert_eq!(placeholders.max_list(), "99999,99999");
    assert_eq!(placeholders.probe_count_list(), "4001,4001");
    assert_eq!(placeholders.algorithm(), "bicubic");
}
