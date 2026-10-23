//! WipeTower.cpp:3614–3703: stock X1C distinct adhesiveness-category blocks.
#[test]
fn bbl_type1_tower_two_categories() {
    crate::runner::native_stream_case::check(
        (
            "bbl-type1-two-categories",
            &["Print.cpp:3444-3537", "GCode/WipeTower.cpp:3614-3806,3864"],
        ),
        "pla-support",
        include_bytes!("../../../../../tests/parity/bbl-type1-tower/two-categories.3mf"),
        "09d8037066427023093b02b068e7c29dd36acb27d9680d173f82925e6eb42f3a",
        (false, |_, _| {}),
    );
}
