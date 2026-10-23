//! `print_order = as_obj_list` object-instance ordering (ByLayer), GCode.cpp consumers.

#[test]
fn class_default_control() {
    check(
        "class-default",
        include_bytes!("../../../../tests/parity/print-order-as-obj-list/class-default.3mf"),
        "02847e15291f553f69582f44682f85d946f1af39f4eaf414e836b2bd7a8f78a3",
    );
}

#[test]
fn class_as_obj_list_target() {
    check(
        "class-as-obj-list",
        include_bytes!("../../../../tests/parity/print-order-as-obj-list/class-as-obj-list.3mf"),
        "3909269439ed746f4416cfec83e97dfe5b47880482f38ea69d9fd93fda87282d",
    );
}

#[test]
fn three_instance_default_control() {
    check(
        "three-instance-default",
        include_bytes!(
            "../../../../tests/parity/print-order-as-obj-list/three-instance-default.3mf"
        ),
        "4e240a273acd99e27a024e1957540a32d5d4ab4f8388b933e56c34b23a287971",
    );
}

#[test]
fn three_instance_as_obj_list_target() {
    check(
        "three-instance-as-obj-list",
        include_bytes!(
            "../../../../tests/parity/print-order-as-obj-list/three-instance-as-obj-list.3mf"
        ),
        "6c0f60d3becca6f731504a16ba22fe3209d07f746848c50e389c68d61b2b69c2",
    );
}

fn check(label: &str, project: &[u8], expected_sha: &str) {
    crate::runner::native_stream_case::check(
        (
            "print-order",
            &[
                "GCode.cpp:2392-2430::sort_object_instances_by_model_order",
                "GCode.cpp:2791-2796::print_object_instances_ordering",
                "GCode.cpp:5129-5133::ordering_for_filament",
                "GCode.cpp:4118-4167::sort_print_object_instances",
            ],
        ),
        label,
        project,
        expected_sha,
        (false, |_, _| {}),
    );
}
