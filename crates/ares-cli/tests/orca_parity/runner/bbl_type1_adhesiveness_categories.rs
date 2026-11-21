//! The pre-slice footprint witness covers Type1 category sizing before tower
//! placement: WipeTower.cpp:1760–1819 and WipeTowerEstimate.cpp:86–245.

#[path = "type1_nozzle_change_width.rs"]
mod type1_nozzle_change_width;
#[path = "../type1_tower_category_gap.rs"]
mod type1_tower_category_gap;
#[test]
fn bbl_type1_prime_tower_waypoints_two_categories() {
    crate::runner::native_stream_case::check(
        (
            "bbl-type1-two-categories",
            &[
                "GCode/WipeTower.cpp:1760-1819",
                "GCode/WipeTowerEstimate.cpp:86-245",
                "PartPlate.cpp:4065-4155",
            ],
        ),
        "pla-support",
        include_bytes!("../../../../../tests/parity/bbl-type1-tower/two-categories.3mf"),
        "09d8037066427023093b02b068e7c29dd36acb27d9680d173f82925e6eb42f3a",
        (false, |_, _| {}),
    );
}
