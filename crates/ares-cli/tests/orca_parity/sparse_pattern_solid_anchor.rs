use crate::runner::native_stream_case;

const UPSTREAM: (&str, &[&str]) = (
    "sparse-pattern-solid-anchor",
    &["Fill.cpp:1561-1684", "PrintObject.cpp:4036-4096"],
);

#[test]
fn crosszag_preserves_solid_support_below_internal_bridge() {
    native_stream_case::check(
        UPSTREAM,
        "mk4-crosszag",
        include_bytes!("../../../../tests/parity/sparse-pattern-solid-anchor/mk4-crosszag.3mf"),
        "93b88644e6336640af8395e778e70fba6f458a05d3ef5ce5579808c77034ee2c",
        (false, |_, _| {}),
    );
}

#[test]
fn lockedzag_preserves_solid_support_below_internal_bridge() {
    native_stream_case::check(
        UPSTREAM,
        "mk4-lockedzag",
        include_bytes!("../../../../tests/parity/sparse-pattern-solid-anchor/mk4-lockedzag.3mf"),
        "771fdc9fd6de5a094f1b2476b8e27e61e968efaeaa03de8a2ae471e4b7f7f39e",
        (false, |_, _| {}),
    );
}

#[test]
fn stock_mk4_preserves_solid_support_below_internal_bridge() {
    native_stream_case::check(
        UPSTREAM,
        "mk4-stock",
        include_bytes!("../../../../tests/parity/sparse-pattern-solid-anchor/mk4-stock.3mf"),
        "95dca79f8de403d14ccb1ce9230776cd5f33c8dfa2a859a4d5091ca222ad98fe",
        (false, |_, _| {}),
    );
}
