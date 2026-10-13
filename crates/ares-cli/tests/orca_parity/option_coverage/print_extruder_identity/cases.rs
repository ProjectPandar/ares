// PrintConfig.cpp:9004-9055,9634-9683; PrintApply.cpp:1169.
pub(super) struct Case {
    pub(super) label: &'static str,
    pub(super) printer: &'static str,
    pub(super) ids: &'static [&'static str],
    pub(super) variants: &'static [&'static str],
    pub(super) expected_ids: &'static str,
    pub(super) expected_variants: &'static str,
}
const PAIR: &[&str] = &["Direct Drive Standard", "Direct Drive Standard"];
const EXPECTED_PAIR: &str = "\"Direct Drive Standard\";\"Direct Drive Standard\"";
const H2D: &str = "Bambu Lab H2D 0.4 nozzle";

pub(super) const CASES: [Case; 6] = [
    Case {
        label: "reversed",
        printer: H2D,
        ids: &["2", "1"],
        variants: PAIR,
        expected_ids: "1,2",
        expected_variants: EXPECTED_PAIR,
    },
    Case {
        label: "ordered",
        printer: H2D,
        ids: &["1", "2"],
        variants: PAIR,
        expected_ids: "1,2",
        expected_variants: EXPECTED_PAIR,
    },
    Case {
        label: "default",
        printer: "Bambu Lab X1 Carbon 0.4 nozzle",
        ids: &["1"],
        variants: &["Direct Drive Standard"],
        expected_ids: "1",
        expected_variants: "\"Direct Drive Standard\"",
    },
    Case {
        label: "repeated",
        printer: H2D,
        ids: &["1", "1", "2"],
        variants: &[
            "Direct Drive High Flow",
            "Direct Drive Standard",
            "Direct Drive Standard",
        ],
        expected_ids: "1,2",
        expected_variants: EXPECTED_PAIR,
    },
    Case {
        label: "short",
        printer: H2D,
        ids: &["7"],
        variants: &[
            "Direct Drive High Flow",
            "Direct Drive Standard",
            "Direct Drive High Flow",
            "Direct Drive Standard",
        ],
        expected_ids: "7,7",
        expected_variants: EXPECTED_PAIR,
    },
    Case {
        label: "unmatched",
        printer: H2D,
        ids: &["1", "2"],
        variants: &["Bowden Standard", "Bowden Standard"],
        expected_ids: "1,1",
        expected_variants: "\"Bowden Standard\";\"Bowden Standard\"",
    },
];
