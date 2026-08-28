use serde::{Serialize, de::DeserializeOwned};

use super::super::super::{
    ProcessBrimType, ProcessExtraBridgeLayer, ProcessGapFillTarget, ProcessInfillPattern,
    ProcessInternalBridgeFilter, ProcessPerimeterGenerator, ProcessSeamPosition,
    ProcessSlicingMode, ProcessSupportBasePattern, ProcessSupportInterfacePattern,
    ProcessSupportStyle, ProcessSupportType,
};

#[test]
fn process_object_source_enum_domains_are_exact() {
    assert_domain::<ProcessBrimType>(&[
        "auto_brim", "brim_ears", "painted", "outer_only", "inner_only", "outer_and_inner",
        "no_brim",
    ]);
    assert_domain::<ProcessInternalBridgeFilter>(&["disabled", "limited", "nofilter"]);
    assert_domain::<ProcessExtraBridgeLayer>(&[
        "disabled", "external_bridge_only", "internal_bridge_only", "apply_to_all",
    ]);
    assert_domain::<ProcessGapFillTarget>(&["everywhere", "topbottom", "nowhere"]);
    assert_domain::<ProcessPerimeterGenerator>(&["classic", "arachne"]);
    assert_domain::<ProcessSeamPosition>(&[
        "nearest", "aligned", "aligned_back", "back", "random",
    ]);
    assert_domain::<ProcessSlicingMode>(&["regular", "even_odd", "close_holes"]);
    assert_domain::<ProcessSupportBasePattern>(&[
        "rectilinear", "rectilinear-grid", "honeycomb", "lightning", "default", "hollow",
    ]);
    assert_domain::<ProcessSupportInterfacePattern>(&[
        "auto", "rectilinear", "concentric", "rectilinear_interlaced", "grid",
    ]);
    assert_domain::<ProcessSupportStyle>(&[
        "default", "grid", "snug", "organic", "tree_slim", "tree_strong", "tree_hybrid",
    ]);
    assert_domain::<ProcessSupportType>(&[
        "normal(auto)", "tree(auto)", "normal(manual)", "tree(manual)",
    ]);
    assert_domain::<ProcessInfillPattern>(&[
        "monotonic", "monotonicline", "rectilinear", "alignedrectilinear", "zig-zag",
        "crosszag", "lockedzag", "line", "grid", "triangles", "tri-hexagon", "cubic",
        "adaptivecubic", "quartercubic", "supportcubic", "lightning", "honeycomb",
        "3dhoneycomb", "lateral-honeycomb", "lateral-lattice", "crosshatch", "tpmsd",
        "tpmsfk", "gyroid", "concentric", "hilbertcurve", "archimedeanchords",
        "octagramspiral",
    ]);
}

fn assert_domain<T>(tokens: &[&str])
where
    T: DeserializeOwned + Serialize,
{
    for token in tokens {
        let json = serde_json::to_string(token).unwrap();
        let value: T = serde_json::from_str(&json).unwrap();
        assert_eq!(serde_json::to_string(&value).unwrap(), json);
    }
    for invalid in ["", "unknown", "Default", "normal"] {
        assert!(
            serde_json::from_str::<T>(&serde_json::to_string(invalid).unwrap()).is_err(),
            "{invalid}"
        );
    }
}
