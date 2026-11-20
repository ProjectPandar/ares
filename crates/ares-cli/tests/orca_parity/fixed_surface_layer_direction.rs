use crate::runner::native_stream_case;

const UPSTREAM: (&str, &[&str]) = (
    "fixed-surface-layer-direction",
    &["Fill.cpp:1046-1064", "FillBase.cpp:447-466"],
);

fn check(label: &str, project: &[u8], sha: &str) {
    native_stream_case::check(UPSTREAM, label, project, sha, (false, |_, _| {}));
}

#[test]
fn mk4_fixed_top_surface_direction() {
    check(
        "mk4-fixed-top",
        include_bytes!("../../../../tests/parity/fixed-surface-layer-direction/mk4-fixed-top.3mf"),
        "0d9a0acedcc1603e1eca8f4c0eea9eb888f82c53aa8b3503c6e7972fe58795f8",
    );
}

#[test]
fn mk4_fixed_bottom_surface_direction() {
    check(
        "mk4-fixed-bottom",
        include_bytes!(
            "../../../../tests/parity/fixed-surface-layer-direction/mk4-fixed-bottom.3mf"
        ),
        "e92405ee1cc187c236064673955ef1199b8b0f62706603a0d80111436e128272",
    );
}

#[test]
fn mk4_stock_surface_direction() {
    check(
        "mk4-stock",
        include_bytes!("../../../../tests/parity/fixed-surface-layer-direction/mk4-stock.3mf"),
        "95dca79f8de403d14ccb1ce9230776cd5f33c8dfa2a859a4d5091ca222ad98fe",
    );
}

#[test]
fn k1_fixed_bottom_surface_direction() {
    check(
        "k1-fixed-bottom",
        include_bytes!(
            "../../../../tests/parity/fixed-surface-layer-direction/k1-fixed-bottom.3mf"
        ),
        "aea582b061c94f2387ef6fbaa4e7ef3e79bd5a3b36a0198516c92357446df319",
    );
}

#[test]
fn k1_stock_surface_direction() {
    check(
        "k1-stock",
        include_bytes!("../../../../tests/parity/fixed-surface-layer-direction/k1-stock.3mf"),
        "f7488a9b1530e2649282249f2506d58db7e1b08609bd573f4f94a8cf7e7dfe35",
    );
}
