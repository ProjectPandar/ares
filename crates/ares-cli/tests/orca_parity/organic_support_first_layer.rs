use crate::runner::native_stream_case;

const SOURCE: (&str, &[&str]) = (
    "organic-support-first-layer",
    &[
        "Support/TreeModelVolumes.cpp:525,573-587",
        "Support/TreeSupport3D.cpp:2490-2584,3612-3635,3960-3995,4270-4305,3671-3702",
        "Support/SupportCommon.cpp:1497-1543",
    ],
);

fn prints_first_layer_support(gcode: &str) -> bool {
    let mut first_layer = false;
    let mut role = "";
    for line in gcode.lines() {
        if let Some(z) = line
            .strip_prefix("; Z_HEIGHT: ")
            .or_else(|| line.strip_prefix(";Z:"))
        {
            if first_layer {
                break;
            }
            first_layer = z.parse::<f64>() == Ok(0.2);
        }
        if let Some(value) = line
            .strip_prefix("; FEATURE: ")
            .or_else(|| line.strip_prefix(";TYPE:"))
        {
            role = value;
        }
        if first_layer
            && role == "Support"
            && line.starts_with("G1 ")
            && (line.contains(" X") || line.contains(" Y"))
            && line
                .split_ascii_whitespace()
                .filter_map(|word| word.strip_prefix('E'))
                .any(|value| value.parse::<f64>().is_ok_and(|e| e > 0.0))
        {
            return true;
        }
    }
    false
}

macro_rules! organic_case {
    ($test:ident, $file:literal, $sha:literal) => {
        #[test]
        fn $test() {
            native_stream_case::check(
                SOURCE,
                $file,
                include_bytes!(concat!(
                    "../../../../tests/parity/organic-support-first-layer/",
                    $file,
                    ".3mf"
                )),
                $sha,
                (false, |reference, _| {
                    assert!(
                        prints_first_layer_support(&reference.native),
                        "Native must extrude support at 0.2 mm: {:?}",
                        reference.root
                    );
                    assert!(
                        !reference.native.lines().any(|line| matches!(
                            line,
                            "; FEATURE: Internal Bridge" | ";TYPE:Internal Bridge"
                        )),
                        "The ramp fixture must not produce internal bridges: {:?}",
                        reference.root
                    );
                }),
            );
        }
    };
}

organic_case!(
    k1_sloped_overhangs_print_organic_support_on_the_bed,
    "k1-no-raft",
    "0665fb29920b41d43d305fdc79ca361c0def0896148a8c2d6291f94ad099fc56"
);
organic_case!(
    x1c_sloped_overhangs_print_organic_support_on_the_bed,
    "x1c-no-raft",
    "c40768429b03976790bd2ba5b7809afee4db7c77227678a6baa8b4d0a2df9fb1"
);
organic_case!(
    mk4_sloped_overhangs_print_organic_support_on_the_bed,
    "mk4-no-raft",
    "13fa89ea44ad39bba639ce40b8c3b77c4244667f0a80dcaf8f39ca6083b9d6a4"
);
