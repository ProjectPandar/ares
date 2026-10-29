//! Native2 then complete public streams via the canonical main NativeCase engine.

#[test]
fn min50_off_requires_native_seam_cache() {
    check(
        "min50-off-target",
        include_bytes!("../../../../tests/parity/processor-sloped-seam/off-target.3mf"),
        "6282599cada020faa3984a9442879b8c4b6c13f6e41b8ebeaea930546723dacc",
    );
}

#[test]
fn min50_on_sole_key_control() {
    check(
        "min50-on-control",
        include_bytes!("../../../../tests/parity/processor-sloped-seam/on-control.3mf"),
        "0047faf8b3c4c2f500d101aa2acc4df9ebebb5a075971598efcf5f984e16799b",
    );
}

#[test]
fn part_external_requires_native_scarf_flag() {
    check(
        "part-external-target",
        include_bytes!("../../../../tests/parity/processor-sloped-seam/part-external-target.3mf"),
        "19aedecce54f64e70b9cd02752df43a254e300064abb7923b825b5ea480034ca",
    );
}

#[test]
fn part_none_sole_attribute_control() {
    check(
        "part-none-control",
        include_bytes!("../../../../tests/parity/processor-sloped-seam/part-none-control.3mf"),
        "44e347dcf43536d940d203a8035eca8ff6e10313f6fe6ed2da67a080e728e797",
    );
}

fn check(label: &str, project: &[u8], sha: &str) {
    crate::runner::native_stream_case::check(
        (
            "processor-sloped-seam",
            &[
                "PrintApply.cpp:1148-1158",
                "GCode/GCodeProcessor.cpp:2141-2147,2471-2477",
                "GCode/GCodeProcessor.cpp:4199-4204,4210-4221,484-488,574",
            ],
        ),
        label,
        project,
        sha,
        (false, |_, _| {}),
    );
}
