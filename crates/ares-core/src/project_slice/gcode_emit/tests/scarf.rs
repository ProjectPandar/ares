use super::super::motion::MotionOptions;
use crate::{FloatOrPercent, Percent, ProcessSeamScarfType};

#[test]
fn ksr_scarf_options_reach_active_motion_projection() {
    let prepared = crate::project_slice::perimeters::prepare_post_classic_traversal(
        crate::project_slice::tests::support::ksr_project(),
    )
    .unwrap();

    let options = MotionOptions::from_traversal(&prepared);

    assert_eq!(options.scarf.seam_slope_type, ProcessSeamScarfType::None);
    assert!(!options.scarf.conditional);
    assert_eq!(
        options.scarf.start_height,
        Some(FloatOrPercent::Percent(Percent(10.0)))
    );
    assert!(!options.scarf.entire_loop);
    assert_eq!(options.scarf.min_length, 10.0);
    assert_eq!(options.scarf.steps, 10);
    assert!(!options.scarf.inner_walls);
    assert_eq!(
        options.scarf.speed,
        Some(FloatOrPercent::Percent(Percent(100.0)))
    );
    assert_eq!(options.scarf.flow_ratio, 1.0);
}
