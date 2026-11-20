use crate::runner::native_stream_case;

const UPSTREAM: (&str, &[&str]) = (
    "tree-style-raft-footprint",
    &[
        "PrintObject.cpp:998-1002,5088-5093",
        "Support/TreeSupport.cpp:1333-1378,1557-1604,1901-1950",
    ],
);

fn first_layer_has_linear_support(gcode: &str) -> bool {
    let mut first_layer = false;
    let mut support = false;
    for line in gcode.lines() {
        if matches!(line, "; CHANGE_LAYER" | ";LAYER_CHANGE") {
            if first_layer {
                break;
            }
            first_layer = true;
        }
        if let Some(role) = line
            .strip_prefix("; FEATURE: ")
            .or_else(|| line.strip_prefix(";TYPE:"))
        {
            support = role == "Support";
        }
        if first_layer && support && line.starts_with("G1 ") {
            let mut fields = line.split(';').next().unwrap().split_ascii_whitespace();
            if (line.contains(" X") || line.contains(" Y"))
                && fields.any(|field| {
                    field
                        .strip_prefix('E')
                        .and_then(|value| value.parse::<f64>().ok())
                        .is_some_and(|value| value > 0.0)
                })
            {
                return true;
            }
        }
    }
    false
}

macro_rules! raft_case {
    ($name:ident, $fixture:literal, $sha:literal) => {
        #[test]
        fn $name() {
            native_stream_case::check(
                UPSTREAM,
                $fixture,
                include_bytes!(concat!(
                    "../../../../tests/parity/tree-style-raft-footprint/",
                    $fixture,
                    ".3mf"
                )),
                $sha,
                (false, |reference, _| {
                    assert!(
                        first_layer_has_linear_support(&reference.native),
                        "expected a deposited Native raft base: {:?}",
                        reference.root
                    );
                }),
            );
        }
    };
}

raft_case!(
    x1c_slim_tree_raft_keeps_its_footprint_without_object_support,
    "x1c-tree_slim",
    "f10b2f3bd9e6eee9ec9590f2dc7365452141d6d0268bf3bc8787557fbc789714"
);
raft_case!(
    x1c_strong_tree_raft_keeps_its_footprint_without_object_support,
    "x1c-tree_strong",
    "b42e5784d86f159cb880aee611aea1b918f24acfed96dd89bd2b342308eefaa3"
);
raft_case!(
    x1c_hybrid_tree_raft_keeps_its_footprint_without_object_support,
    "x1c-tree_hybrid",
    "cfa252074c88ac55e9a5644f853f7b80516f8ac5a51c2e900df1cd5f4a24f1fa"
);
raft_case!(
    mk4_slim_tree_raft_keeps_its_footprint_without_object_support,
    "mk4-tree_slim",
    "0e0e9e3b79cfe2bda0563adf3eaf222a6fd3b7852167ff1a13cb92ceb032c295"
);
raft_case!(
    mk4_strong_tree_raft_keeps_its_footprint_without_object_support,
    "mk4-tree_strong",
    "d0cde5897901d6b03115f9763b63e4fc26676e2dd8494f5f7ab6678023e24bec"
);
raft_case!(
    mk4_hybrid_tree_raft_keeps_its_footprint_without_object_support,
    "mk4-tree_hybrid",
    "b588e3a2f2994355b5f9655d4266da0766e717f78197c84794253dce383faeda"
);
