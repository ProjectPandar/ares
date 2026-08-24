use super::{MotionBlock, planned_times, planner};

#[test]
fn planner_refreshes_only_after_threshold_is_exceeded() {
    let blocks = (0..257).map(block).collect::<Vec<_>>();
    let (source_window_times, _) = planner::planned_times_with_initial(&blocks, None);

    let actual = planned_times(&blocks);

    assert_eq!(actual[192].to_bits(), source_window_times[192].to_bits());

    let (_, early_entries) = planner::planned_times_with_initial(&blocks[..256], None);
    let (early_next_times, _) =
        planner::planned_times_with_initial(&blocks[192..], Some(early_entries[192]));
    assert_ne!(
        early_next_times[0].to_bits(),
        source_window_times[192].to_bits(),
        "the fixture must distinguish a >= 256 refresh from source's > 256 refresh",
    );
}

fn block(index: usize) -> MotionBlock {
    MotionBlock {
        index,
        distance: 0.000_1,
        speed: 100.0,
        acceleration: 1.0,
        centripetal_acceleration: 1.0,
        jerk: [9.0, 9.0, 3.0, 2.5],
        direction: [1.0, 0.0, 0.0, 0.0],
    }
}
